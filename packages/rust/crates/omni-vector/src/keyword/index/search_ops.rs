impl KeywordIndex {
    /// Search the index with `BM25` scoring.
    ///
    /// # Errors
    ///
    /// Returns an error when query parsing, search execution, or document fetch fails.
    pub fn search(
        &self,
        query_str: &str,
        limit: usize,
    ) -> Result<Vec<ToolSearchResult>, VectorStoreError> {
        let searcher = self.reader.searcher();

        if query_str.trim().is_empty() {
            return Ok(vec![]);
        }

        let mut query_parser = QueryParser::for_index(
            &self.index,
            vec![
                self.tool_name,
                self.keywords,
                self.intents,
                self.description,
            ],
        );

        query_parser.set_field_boost(self.tool_name, 5.0);
        query_parser.set_field_boost(self.intents, 4.0);
        query_parser.set_field_boost(self.keywords, 3.0);
        query_parser.set_field_boost(self.description, 1.0);

        let query = query_parser
            .parse_query(query_str)
            .map_err(|e| VectorStoreError::General(format!("Query parse error: {e}")))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(VectorStoreError::Tantivy)?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(VectorStoreError::Tantivy)?;

            let tool_name = doc
                .get_first(self.tool_name)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if !crate::skill::is_routable_tool_name(&tool_name) {
                continue;
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

            let skill_name = tool_name.split('.').next().unwrap_or("").to_string();
            let category = doc
                .get_first(self.category)
                .and_then(|v| v.as_str())
                .unwrap_or(&skill_name)
                .to_string();

            results.push(ToolSearchResult {
                name: tool_name.clone(),
                description,
                input_schema: serde_json::json!({}),
                score,
                vector_score: None,
                keyword_score: Some(score),
                skill_name,
                tool_name,
                file_path: String::new(),
                routing_keywords: keywords,
                intents,
                category,
                parameters: vec![],
            });
        }

        Ok(results)
    }
}
