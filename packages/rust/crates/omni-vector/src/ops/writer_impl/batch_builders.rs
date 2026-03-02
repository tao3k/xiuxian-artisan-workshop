impl VectorStore {
    fn derive_routing_keywords(tool: &OmniToolRecord) -> Vec<String> {
        let skill_token = tool.skill_name.trim();
        let tool_token = tool.tool_name.split('.').next_back().map_or("", str::trim);
        let full_tool = tool.tool_name.trim();
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for kw in &tool.keywords {
            let token = kw.trim();
            if token.is_empty() {
                continue;
            }
            if token == skill_token || token == tool_token || token == full_tool {
                continue;
            }
            if seen.insert(token.to_string()) {
                out.push(token.to_string());
            }
        }
        out
    }

    fn canonical_tool_name_from_metadata(meta: &serde_json::Value) -> Option<String> {
        let skill_name = meta
            .get("skill_name")
            .and_then(|s| s.as_str())
            .map_or("", str::trim);
        let tool_name = meta
            .get("tool_name")
            .and_then(|s| s.as_str())
            .map_or("", str::trim);
        if crate::skill::is_routable_tool_name(tool_name) && tool_name.contains('.') {
            return Some(tool_name.to_string());
        }
        if !skill_name.is_empty() && crate::skill::is_routable_tool_name(tool_name) {
            let candidate = format!("{skill_name}.{tool_name}");
            if crate::skill::is_routable_tool_name(&candidate) {
                return Some(candidate);
            }
        }

        let command = meta
            .get("command")
            .and_then(|s| s.as_str())
            .map_or("", str::trim);

        if !skill_name.is_empty() && !command.is_empty() {
            let candidate = format!("{skill_name}.{command}");
            if crate::skill::is_routable_tool_name(&candidate) {
                return Some(candidate);
            }
        }

        if crate::skill::is_routable_tool_name(command) {
            return Some(command.to_string());
        }
        None
    }

    fn build_document_batch(
        &self,
        ids: Vec<String>,
        vectors: Vec<Vec<f32>>,
        contents: Vec<String>,
        metadatas: Vec<String>,
    ) -> Result<
        (
            Arc<lance::deps::arrow_schema::Schema>,
            lance::deps::arrow_array::RecordBatch,
        ),
        VectorStoreError,
    > {
        use lance::deps::arrow_array::StringArray;

        let list_dimension = validate_document_batch_inputs(
            ids.len(),
            &vectors,
            contents.len(),
            metadatas.len(),
            self.dimension,
        )?;
        let metadata_columns = parse_document_metadata_columns(&metadatas, &ids)?;
        let id_array = StringArray::from(ids);
        let content_array = StringArray::from(contents);
        let vector_array = build_vector_list_array(vectors, list_dimension)?;
        let metadata_array = StringArray::from(metadatas);

        let schema = self.create_schema();
        let batch = lance::deps::arrow_array::RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array),
                Arc::new(vector_array),
                Arc::new(content_array),
                Arc::new(metadata_columns.skill_name),
                Arc::new(metadata_columns.category),
                Arc::new(metadata_columns.tool_name),
                Arc::new(metadata_columns.file_path),
                Arc::new(metadata_columns.routing_keywords),
                Arc::new(metadata_columns.intents),
                Arc::new(metadata_array),
            ],
        )
        .map_err(VectorStoreError::Arrow)?;
        Ok((schema, batch))
    }
}
