impl VectorStore {
    /// Add tool records to the vector store.
    ///
    /// # Errors
    ///
    /// Returns an error when `LanceDB` batch construction or write paths fail.
    pub async fn add(
        &self,
        table_name: &str,
        tools: Vec<OmniToolRecord>,
    ) -> Result<(), VectorStoreError> {
        if tools.is_empty() {
            return Ok(());
        }

        let mut prepared_tools = Vec::with_capacity(tools.len());
        for tool in tools {
            let command_name = tool
                .tool_name
                .split('.')
                .skip(1)
                .collect::<Vec<_>>()
                .join(".");
            let full_name = format!("{}.{}", tool.skill_name, command_name);
            let routing_keywords = Self::derive_routing_keywords(&tool);
            prepared_tools.push((tool, full_name, routing_keywords, command_name));
        }

        // 1. Write to Keyword Index if enabled
        if let Some(kw_index) = &self.keyword_index {
            let search_results: Vec<skill::ToolSearchResult> = prepared_tools
                .iter()
                .map(|(tool, full_name, _routing_keywords, _command_name)| {
                    skill::ToolSearchResult {
                        name: full_name.clone(),
                        description: tool.description.clone(),
                        input_schema: serde_json::json!({}),
                        score: 1.0,
                        vector_score: None,
                        keyword_score: Some(1.0),
                        skill_name: tool.skill_name.clone(),
                        tool_name: tool.tool_name.clone(),
                        file_path: tool.file_path.clone(),
                        routing_keywords: tool.keywords.clone(),
                        intents: tool.intents.clone(),
                        category: tool.category.clone(),
                        parameters: tool.parameters.clone(),
                    }
                })
                .collect();
            if let Err(e) = kw_index.index_batch(&search_results) {
                log::error!(
                    "Keyword index batch failed for {} tools: {e}",
                    search_results.len()
                );
            } else {
                log::info!("Keyword index: indexed {} tools", search_results.len());
            }
        }

        // 2. Prepare metadata and IDs for `LanceDB`
        let mut ids = Vec::with_capacity(prepared_tools.len());
        let mut contents = Vec::with_capacity(prepared_tools.len());
        let mut metadatas = Vec::with_capacity(prepared_tools.len());
        for (tool, full_name, routing_keywords, command_name) in prepared_tools {
            ids.push(full_name);
            contents.push(tool.description.clone());
            metadatas.push(
                serde_json::json!({
                    "type": "command", "skill_name": tool.skill_name, "command": command_name, "tool_name": tool.tool_name,
                    "file_path": tool.file_path, "function_name": tool.function_name, "intents": tool.intents,
                    "routing_keywords": routing_keywords,
                    "file_hash": tool.file_hash, "input_schema": tool.input_schema, "docstring": tool.docstring,
                    "category": tool.category, "annotations": tool.annotations, "parameters": tool.parameters,
                    "skill_tools_refers": tool.skill_tools_refers,
                    "resource_uri": tool.resource_uri,
                })
                .to_string(),
            );
        }

        // Standard vectors (dummy for now as `SkillIndexer` provides actual
        // vectors via `add_documents`).
        let vectors: Vec<Vec<f32>> = (0..ids.len()).map(|_| vec![0.0; self.dimension]).collect();

        self.add_documents(table_name, ids, vectors, contents, metadatas)
            .await?;
        Ok(())
    }

    /// Batch add documents with vectors to a table.
    ///
    /// # Errors
    ///
    /// Returns an error when input validation fails, dataset create/append fails,
    /// or `Arrow` batch construction fails.
    pub async fn add_documents(
        &self,
        table_name: &str,
        ids: Vec<String>,
        vectors: Vec<Vec<f32>>,
        contents: Vec<String>,
        metadatas: Vec<String>,
    ) -> Result<(), VectorStoreError> {
        use lance::deps::arrow_array::RecordBatchIterator;

        if ids.is_empty() {
            return Ok(());
        }

        let contents_for_keyword = contents.clone();
        let metadatas_for_keyword = metadatas.clone();
        let (schema, batch) = self.build_document_batch(ids, vectors, contents, metadatas)?;

        let (mut dataset, created) = self
            .get_or_create_dataset(table_name, false, Some((schema.clone(), batch.clone())))
            .await?;
        if !created {
            dataset
                .append(
                    Box::new(RecordBatchIterator::new(vec![Ok(batch)], schema)),
                    Some(default_write_params()),
                )
                .await?;
        }

        // DUAL WRITE: Also write to Keyword Index if enabled
        if let Some(ref kw_index) = self.keyword_index {
            let mut keyword_docs = Vec::new();
            for (i, meta_str) in metadatas_for_keyword.iter().enumerate() {
                if let Some(meta) = parse_metadata_value(meta_str) {
                    if meta.get("type").and_then(|s| s.as_str()) != Some("command") {
                        continue;
                    }
                    let Some(name) = Self::canonical_tool_name_from_metadata(&meta) else {
                        continue;
                    };
                    let category = meta
                        .get("category")
                        .and_then(|s| s.as_str())
                        .or_else(|| meta.get("skill_name").and_then(|s| s.as_str()))
                        .unwrap_or("unknown")
                        .to_string();
                    let kws = crate::skill::resolve_routing_keywords(&meta);
                    let intents = crate::skill::resolve_intents(&meta);
                    keyword_docs.push((
                        name,
                        contents_for_keyword[i].clone(),
                        category,
                        kws,
                        intents,
                    ));
                }
            }
            if !keyword_docs.is_empty() {
                let _ = kw_index.bulk_upsert(keyword_docs);
            }
        }
        Ok(())
    }

    /// Add documents with rows grouped by a partition column so fragments align by partition
    /// (enables partition pruning at read). Partition value is read from each row's metadata JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when input validation fails, partitioned append fails,
    /// or `Arrow` batch construction fails.
    pub async fn add_documents_partitioned(
        &self,
        table_name: &str,
        partition_by: &str,
        ids: Vec<String>,
        vectors: Vec<Vec<f32>>,
        contents: Vec<String>,
        metadatas: Vec<String>,
    ) -> Result<(), VectorStoreError> {
        use lance::deps::arrow_array::RecordBatchIterator;
        use std::collections::BTreeMap;

        if ids.is_empty() {
            return Ok(());
        }
        if ids.len() != vectors.len() || ids.len() != contents.len() || ids.len() != metadatas.len()
        {
            return Err(VectorStoreError::General(
                "Mismatched input lengths for ids/vectors/contents/metadatas".to_string(),
            ));
        }

        let partition_values: Vec<String> = metadatas
            .iter()
            .map(|s| {
                parse_metadata_value(s)
                    .and_then(|v| {
                        v.get(partition_by)
                            .and_then(|x| x.as_str())
                            .map(String::from)
                    })
                    .unwrap_or_else(|| "_unknown".to_string())
            })
            .collect();

        let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, pv) in partition_values.into_iter().enumerate() {
            groups.entry(pv).or_default().push(i);
        }

        let contents_for_keyword = contents.clone();
        let metadatas_for_keyword = metadatas.clone();

        let (mut dataset, _) = self.get_or_create_dataset(table_name, false, None).await?;
        let schema = self.create_schema();

        for (_partition_value, indices) in groups {
            let part_ids: Vec<String> = indices.iter().map(|&i| ids[i].clone()).collect();
            let part_vectors: Vec<Vec<f32>> = indices.iter().map(|&i| vectors[i].clone()).collect();
            let part_contents: Vec<String> = indices.iter().map(|&i| contents[i].clone()).collect();
            let part_metadatas: Vec<String> =
                indices.iter().map(|&i| metadatas[i].clone()).collect();

            let (_, batch) =
                self.build_document_batch(part_ids, part_vectors, part_contents, part_metadatas)?;
            let batches: Vec<Result<_, crate::error::ArrowError>> = vec![Ok(batch)];
            dataset
                .append(
                    Box::new(RecordBatchIterator::new(batches, schema.clone())),
                    Some(default_write_params()),
                )
                .await?;
        }

        if let Some(ref kw_index) = self.keyword_index {
            let mut keyword_docs = Vec::new();
            for (i, meta_str) in metadatas_for_keyword.iter().enumerate() {
                if let Some(meta) = parse_metadata_value(meta_str) {
                    if meta.get("type").and_then(|s| s.as_str()) != Some("command") {
                        continue;
                    }
                    let Some(name) = Self::canonical_tool_name_from_metadata(&meta) else {
                        continue;
                    };
                    let category = meta
                        .get("category")
                        .and_then(|s| s.as_str())
                        .or_else(|| meta.get("skill_name").and_then(|s| s.as_str()))
                        .unwrap_or("unknown")
                        .to_string();
                    let kws = crate::skill::resolve_routing_keywords(&meta);
                    let intents = crate::skill::resolve_intents(&meta);
                    keyword_docs.push((
                        name,
                        contents_for_keyword[i].clone(),
                        category,
                        kws,
                        intents,
                    ));
                }
            }
            if !keyword_docs.is_empty() {
                let _ = kw_index.bulk_upsert(keyword_docs);
            }
        }
        Ok(())
    }
}
