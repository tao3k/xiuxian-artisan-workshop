use std::path::Path;

impl VectorStore {
    fn scan_unique_skill_tools(
        &self,
        base_path: &str,
    ) -> Result<Vec<xiuxian_skills::ToolRecord>, VectorStoreError> {
        let skill_scanner = SkillScanner::new();
        let script_scanner = ToolsScanner::new();
        let resource_scanner = ResourceScanner::new();
        let skills_path = Path::new(base_path);
        log::debug!(
            "scan_unique_skill_tools: scan_path={}, store_base={}",
            skills_path.display(),
            self.base_path.display()
        );
        if !skills_path.exists() {
            log::warn!("Skills path does not exist: {}", skills_path.display());
            return Ok(vec![]);
        }

        let metadatas = skill_scanner
            .scan_all(skills_path, None)
            .map_err(|e| VectorStoreError::General(e.to_string()))?;
        log::info!("Found {} skill manifests", metadatas.len());

        let mut tools_map = std::collections::HashMap::new();
        for metadata in &metadatas {
            let skill_path = skills_path.join(&metadata.skill_name);
            let mut tools = script_scanner
                .scan_scripts(
                    &skill_path.join("scripts"),
                    &metadata.skill_name,
                    &metadata.routing_keywords,
                    &metadata.intents,
                )
                .map_err(|e| VectorStoreError::General(e.to_string()))?;

            // Scan for @skill_resource decorated functions and convert to tools
            let resources: Vec<ResourceRecord> =
                match resource_scanner.scan(&skill_path.join("scripts"), &metadata.skill_name) {
                    Ok(r) => r,
                    Err(e) => return Err(VectorStoreError::General(e.to_string())),
                };

            // Convert resources to tools with resource_uri set
            for resource in resources {
                let resource_tool = ToolRecord {
                    tool_name: format!("{}.{}", resource.skill_name, resource.name),
                    description: resource.description.clone(),
                    skill_name: resource.skill_name.clone(),
                    file_path: resource.file_path.clone(),
                    function_name: resource.function_name.clone(),
                    execution_mode: "resource".to_string(),
                    keywords: vec![resource.skill_name.clone(), resource.name.clone()],
                    intents: metadata.intents.clone(),
                    file_hash: resource.file_hash.clone(),
                    input_schema: "{}".to_string(),
                    docstring: resource.description.clone(),
                    category: "resource".to_string(),
                    annotations: ToolAnnotations::default(),
                    parameters: vec![],
                    skill_tools_refers: vec![],
                    resource_uri: resource.resource_uri,
                };
                tools.push(resource_tool);
            }

            log::debug!(
                "Skill '{}': found {} tools (+ {} resources)",
                metadata.skill_name,
                tools.len(),
                tools.iter().filter(|t| !t.resource_uri.is_empty()).count()
            );

            // Fill skill_tools_refers from markdown front matter (references/*.md for_tools list), not from decorator
            let entry = skill_scanner.build_index_entry(metadata.clone(), &tools, &skill_path);
            for t in &mut tools {
                t.skill_tools_refers = entry
                    .references
                    .iter()
                    .filter(|r| r.applies_to_tool(&t.tool_name))
                    .map(|r| r.ref_name.clone())
                    .collect();
            }
            for tool in tools {
                tools_map.insert(tool.tool_name.clone(), tool);
            }
        }

        Ok(tools_map.into_values().collect())
    }

    /// Index all tools found in a skill directory.
    /// This drops and recreates the table to ensure sync with filesystem.
    ///
    /// # Errors
    ///
    /// Returns an error if filesystem scanning fails, if vector-table writes fail,
    /// or if index/table operations fail in `LanceDB`.
    pub async fn index_skill_tools(
        &mut self,
        base_path: &str,
        table_name: &str,
    ) -> Result<(), VectorStoreError> {
        log::info!("Indexing skills from: {base_path}");

        let tools = self.scan_unique_skill_tools(base_path)?;
        log::info!("Total tools to index: {}", tools.len());
        if tools.is_empty() {
            log::warn!(
                "index_skill_tools: scan returned 0 tools from {base_path}; keeping existing table"
            );
            return Ok(());
        }

        // Drop only when we have tools to write (avoids empty table on scan failure)
        let drop_result = self.drop_table(table_name).await;
        log::debug!("drop_table result: {drop_result:?}");
        if let Err(e) = self.enable_keyword_index() {
            log::warn!("Could not re-enable keyword index after drop: {e}");
        }

        self.add(table_name, tools).await?;
        if let Err(e) = self
            .create_scalar_index(table_name, SKILL_NAME_COLUMN, ScalarIndexType::BTree)
            .await
        {
            log::debug!("Scalar index skill_name skipped: {e}");
        }
        if let Err(e) = self
            .create_scalar_index(table_name, CATEGORY_COLUMN, ScalarIndexType::Bitmap)
            .await
        {
            log::debug!("Scalar index category skipped: {e}");
        }
        log::info!("Successfully indexed tools for table: {table_name}");
        Ok(())
    }

    /// Atomically rebuild two tool tables from a single filesystem scan.
    ///
    /// This guarantees skills/router are indexed from the same snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if skill scanning fails, table writes/counts fail,
    /// or `LanceDB` operations fail while rebuilding either table.
    pub async fn index_skill_tools_dual(
        &mut self,
        base_path: &str,
        skills_table: &str,
        router_table: &str,
    ) -> Result<(usize, usize), VectorStoreError> {
        let tools = self.scan_unique_skill_tools(base_path)?;
        if tools.is_empty() {
            // Do NOT drop: preserve existing data. Empty scan may mean wrong path, transient
            // failure, or no skills yet. Dropping would leave user with empty skill list.
            log::warn!(
                "index_skill_tools_dual: scan returned 0 tools from {base_path}; keeping existing table"
            );
            return Ok((0, 0));
        }

        self.drop_table(skills_table).await.ok();
        // Re-enable keyword index after drop_table cleared it
        if let Err(e) = self.enable_keyword_index() {
            log::warn!("Could not re-enable keyword index after drop: {e}");
        }
        self.add(skills_table, tools.clone()).await?;
        let _ = self
            .create_scalar_index(skills_table, SKILL_NAME_COLUMN, ScalarIndexType::BTree)
            .await;
        let _ = self
            .create_scalar_index(skills_table, CATEGORY_COLUMN, ScalarIndexType::Bitmap)
            .await;
        let skills_count = self.count(skills_table).await? as usize;

        if router_table == skills_table {
            return Ok((skills_count, skills_count));
        }

        self.drop_table(router_table).await.ok();
        // Re-enable keyword index after drop_table cleared it
        if let Err(e) = self.enable_keyword_index() {
            log::warn!("Could not re-enable keyword index after drop: {e}");
        }
        self.add(router_table, tools).await?;
        let _ = self
            .create_scalar_index(router_table, SKILL_NAME_COLUMN, ScalarIndexType::BTree)
            .await;
        let _ = self
            .create_scalar_index(router_table, CATEGORY_COLUMN, ScalarIndexType::Bitmap)
            .await;
        let router_count = self.count(router_table).await? as usize;

        Ok((skills_count, router_count))
    }

    /// Scan skill tools without indexing them.
    ///
    /// # Errors
    ///
    /// Returns an error if skill manifests/scripts cannot be scanned.
    pub fn scan_skill_tools_raw(&self, base_path: &str) -> Result<Vec<String>, VectorStoreError> {
        let skill_scanner = SkillScanner::new();
        let script_scanner = ToolsScanner::new();
        let skills_path = Path::new(base_path);
        log::debug!(
            "scan_skill_tools_raw: scan_path={}, store_base={}",
            skills_path.display(),
            self.base_path.display()
        );
        if !skills_path.exists() {
            return Ok(vec![]);
        }
        let metadatas = skill_scanner
            .scan_all(skills_path, None)
            .map_err(|e| VectorStoreError::General(e.to_string()))?;
        let mut all_tools = Vec::new();
        for metadata in &metadatas {
            let tools = script_scanner
                .scan_scripts(
                    &skills_path.join(&metadata.skill_name).join("scripts"),
                    &metadata.skill_name,
                    &metadata.routing_keywords,
                    &[],
                )
                .map_err(|e| VectorStoreError::General(e.to_string()))?;
            all_tools.extend(tools);
        }
        Ok(all_tools
            .into_iter()
            .map(|t| serde_json::to_string(&t).unwrap_or_default())
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// List all tools that are also MCP resources (have non-empty `resource_uri` in metadata).
    ///
    /// # Errors
    ///
    /// Returns an error if the dataset cannot be opened/scanned, projected stream rows
    /// cannot be read, or the final JSON payload cannot be serialized.
    pub async fn list_all_resources(&self, table_name: &str) -> Result<String, VectorStoreError> {
        use crate::ops::column_read::get_utf8_at;

        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Ok("[]".to_string());
        }
        let dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let schema = dataset.schema();
        if schema.field(METADATA_COLUMN).is_none() {
            return Ok("[]".to_string());
        }
        let mut scanner = dataset.scan();
        scanner.project(&["id", "content", METADATA_COLUMN, "skill_name", "tool_name"])?;
        let mut stream = scanner.try_into_stream().await?;
        let mut resources = Vec::new();
        while let Some(batch) = stream.try_next().await? {
            use lance::deps::arrow_array::Array;
            use lance::deps::arrow_array::StringArray;

            let id_col = batch.column_by_name("id");
            let content_col = batch.column_by_name("content");
            let metadata_col = batch.column_by_name(METADATA_COLUMN);
            let skill_col = batch.column_by_name("skill_name");
            let tool_col = batch.column_by_name("tool_name");

            let m_arr =
                metadata_col.and_then(|c| c.as_any().downcast_ref::<StringArray>().cloned());

            if let (Some(ids), Some(contents)) = (id_col, content_col) {
                let id_arr = ids.as_any().downcast_ref::<StringArray>();
                let content_arr = contents.as_any().downcast_ref::<StringArray>();

                for i in 0..batch.num_rows() {
                    // Only include rows with non-empty resource_uri
                    let resource_uri = m_arr.as_ref().and_then(|ma| {
                        if ma.is_null(i) {
                            return None;
                        }
                        let meta_str = ma.value(i);
                        serde_json::from_str::<serde_json::Value>(meta_str)
                            .ok()
                            .and_then(|v| {
                                v.get("resource_uri")
                                    .and_then(|u| u.as_str())
                                    .filter(|s| !s.is_empty())
                                    .map(String::from)
                            })
                    });

                    let Some(uri) = resource_uri else {
                        continue;
                    };

                    let id = id_arr.map_or(String::new(), |arr| arr.value(i).to_string());
                    let content = content_arr.map_or(String::new(), |arr| arr.value(i).to_string());
                    let skill_name = skill_col.map_or(String::new(), |c| get_utf8_at(c, i));
                    let tool_name = tool_col.map_or(String::new(), |c| get_utf8_at(c, i));

                    resources.push(serde_json::json!({
                        "id": id,
                        "resource_uri": uri,
                        "description": content,
                        "skill_name": skill_name,
                        "tool_name": tool_name,
                    }));
                }
            }
        }
        serde_json::to_string(&resources).map_err(|e| VectorStoreError::General(e.to_string()))
    }
}
