impl KeywordIndex {
    /// Retrieve a full `ToolSearchResult` from the index by tool name.
    ///
    /// # Errors
    ///
    /// Returns an error when term search or document fetch fails.
    pub fn get_tool(&self, name: &str) -> Result<Option<ToolSearchResult>, VectorStoreError> {
        let searcher = self.reader.searcher();
        let term = Term::from_field_text(self.tool_name, name);
        let term_query = tantivy::query::TermQuery::new(term, IndexRecordOption::Basic);

        let top_docs = searcher
            .search(&term_query, &TopDocs::with_limit(1))
            .map_err(VectorStoreError::Tantivy)?;

        if let Some((_score, doc_address)) = top_docs.first() {
            let doc: TantivyDocument = searcher
                .doc(*doc_address)
                .map_err(VectorStoreError::Tantivy)?;

            let tool_name = doc
                .get_first(self.tool_name)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if !crate::skill::is_routable_tool_name(&tool_name) {
                return Ok(None);
            }
            let description = doc
                .get_first(self.description)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let keywords_str = doc
                .get_first(self.keywords)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let intents_str = doc
                .get_first(self.intents)
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let keywords = keywords_str
                .split_whitespace()
                .map(ToString::to_string)
                .collect();
            let intents = intents_str.split(" | ").map(ToString::to_string).collect();

            Ok(Some(ToolSearchResult {
                name: tool_name.clone(),
                description,
                input_schema: serde_json::json!({}),
                score: 1.0,
                vector_score: None,
                keyword_score: Some(1.0),
                skill_name: tool_name.split('.').next().unwrap_or("").to_string(),
                tool_name,
                file_path: String::new(),
                routing_keywords: keywords,
                intents,
                parameters: vec![],
                category: doc
                    .get_first(self.category)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }))
        } else {
            Ok(None)
        }
    }
}
