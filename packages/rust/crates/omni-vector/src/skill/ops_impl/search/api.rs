impl VectorStore {
    /// Search for tools using hybrid search (vector + keyword).
    ///
    /// # Errors
    ///
    /// Returns an error if table scanning fails or if an invalid `where_filter`
    /// is provided for Lance filter parsing.
    pub async fn search_tools(
        &self,
        table_name: &str,
        query_vector: &[f32],
        query_text: Option<&str>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<skill::ToolSearchResult>, VectorStoreError> {
        self.search_tools_with_options(skill::ToolSearchRequest {
            table_name,
            query_vector,
            query_text,
            limit,
            threshold,
            options: skill::ToolSearchOptions::default(),
            where_filter: None,
        })
        .await
    }

    /// Search for tools with explicit ranking options.
    /// When `where_filter` is set (for example `skill_name = 'git'`), only rows
    /// matching the predicate are scanned.
    ///
    /// # Errors
    ///
    /// Returns an error if projection/filter configuration fails, if stream scans
    /// fail at the table boundary, or if `where_filter` is invalid.
    pub async fn search_tools_with_options(
        &self,
        request: skill::ToolSearchRequest<'_>,
    ) -> Result<Vec<skill::ToolSearchResult>, VectorStoreError> {
        let mut results_map = self
            .collect_vector_tool_results(
                request.table_name,
                request.query_vector,
                request.where_filter,
            )
            .await?;
        if let Some(text) = request.query_text {
            results_map = self
                .fuse_tool_results_with_keyword(
                    request.table_name,
                    text,
                    request.limit,
                    request.options,
                    results_map,
                )
                .await;
        }
        Ok(finalize_tool_results(
            results_map,
            request.threshold,
            request.limit,
        ))
    }

    async fn collect_vector_tool_results(
        &self,
        table_name: &str,
        query_vector: &[f32],
        where_filter: Option<&str>,
    ) -> Result<ToolResultsMap, VectorStoreError> {
        let mut results_map = ToolResultsMap::new();
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Ok(results_map);
        }

        let Ok(dataset) = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await
        else {
            return Ok(results_map);
        };
        let schema = dataset.schema();
        let has_metadata = schema.field(METADATA_COLUMN).is_some();
        let project_cols = search_project_columns(has_metadata);
        let mut scanner = dataset.scan();
        scanner.project(&project_cols).ok();

        let skill_filter_from_where = where_filter.and_then(parse_skill_name_from_where_filter);
        if let Some(filter) = where_filter
            && skill_filter_from_where.is_none()
        {
            scanner
                .filter(filter)
                .map_err(|e| VectorStoreError::General(format!("Invalid where_filter: {e}")))?;
        }

        let Ok(mut stream) = scanner.try_into_stream().await else {
            return Ok(results_map);
        };
        while let Ok(Some(batch)) = stream.try_next().await {
            append_vector_results_from_batch(
                &batch,
                query_vector,
                skill_filter_from_where.as_deref(),
                &mut results_map,
            );
        }
        Ok(results_map)
    }

    async fn fuse_tool_results_with_keyword(
        &self,
        table_name: &str,
        query_text: &str,
        limit: usize,
        options: skill::ToolSearchOptions,
        vector_results: ToolResultsMap,
    ) -> ToolResultsMap {
        let mut vector_scores: Vec<(String, f32)> = vector_results
            .iter()
            .map(|(name, result)| (name.clone(), result.score))
            .collect();
        vector_scores.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let kw_hits = self
            .keyword_search(table_name, query_text, limit * 2)
            .await
            .unwrap_or_default();
        let fused = apply_weighted_rrf(
            vector_scores,
            kw_hits.clone(),
            keyword::RRF_K,
            options.semantic_weight.unwrap_or(keyword::SEMANTIC_WEIGHT),
            options.keyword_weight.unwrap_or(keyword::KEYWORD_WEIGHT),
            query_text,
        );
        let kw_lookup: ToolResultsMap = kw_hits
            .into_iter()
            .map(|result| (result.tool_name.clone(), result))
            .collect();
        let query_parts = normalize_query_terms(query_text);
        let file_discovery_intent = query_has_file_discovery_intent(&query_parts);
        let mut merged_results = ToolResultsMap::new();

        for fused_item in fused {
            let Some(mut tool) = vector_results
                .get(&fused_item.tool_name)
                .cloned()
                .or_else(|| kw_lookup.get(&fused_item.tool_name).cloned())
            else {
                continue;
            };
            tool.score = fused_item.rrf_score;
            if options.rerank {
                let mut rerank_bonus = tool_metadata_alignment_boost(&tool, &query_parts);
                if file_discovery_intent {
                    if tool.tool_name == "advanced_tools.smart_find" {
                        rerank_bonus += 0.70;
                    } else if tool_file_discovery_match(&tool) {
                        rerank_bonus += 0.30;
                    }
                }
                tool.score += rerank_bonus;
            }
            tool.vector_score = Some(fused_item.vector_score);
            tool.keyword_score = Some(fused_item.keyword_score);
            merged_results.insert(fused_item.tool_name, tool);
        }
        merged_results
    }
}
