impl KeywordIndex {
    /// Add or update one document in the index.
    ///
    /// # Errors
    ///
    /// Returns an error when writer creation, document write/commit, or reader reload fails.
    pub fn upsert_document(
        &self,
        name: &str,
        description: &str,
        category: &str,
        keywords: &[String],
        intents: &[String],
    ) -> Result<(), VectorStoreError> {
        if !crate::skill::is_routable_tool_name(name) {
            return Ok(());
        }
        let mut cache = self.writer_cache.borrow_mut();
        if cache.is_none() {
            *cache = Some(
                self.index
                    .writer(100_000_000)
                    .map_err(VectorStoreError::Tantivy)?,
            );
        }
        let writer = cache
            .as_mut()
            .ok_or_else(|| VectorStoreError::General("writer cache unavailable".to_string()))?;
        let term = Term::from_field_text(self.tool_name, name);
        writer.delete_term(term);
        writer
            .add_document(doc!(
                self.tool_name => name,
                self.description => description,
                self.category => category,
                self.keywords => keywords.join(" "),
                self.intents => intents.join(" | ")
            ))
            .map_err(VectorStoreError::Tantivy)?;
        writer.commit().map_err(VectorStoreError::Tantivy)?;
        drop(cache);
        self.reader.reload().map_err(VectorStoreError::Tantivy)?;
        Ok(())
    }

    /// Bulk upsert documents. Reuses a cached `IndexWriter` when possible.
    ///
    /// # Errors
    ///
    /// Returns an error when writer creation, document write/commit, or reader reload fails.
    pub fn bulk_upsert<I>(&self, docs: I) -> Result<(), VectorStoreError>
    where
        I: IntoIterator<Item = (String, String, String, Vec<String>, Vec<String>)>,
    {
        let mut cache = self.writer_cache.borrow_mut();
        if cache.is_none() {
            *cache = Some(
                self.index
                    .writer(100_000_000)
                    .map_err(VectorStoreError::Tantivy)?,
            );
        }
        let writer = cache
            .as_mut()
            .ok_or_else(|| VectorStoreError::General("writer cache unavailable".to_string()))?;
        for (name, description, category, kw_list, intent_list) in docs {
            if !crate::skill::is_routable_tool_name(&name) {
                continue;
            }
            let term = Term::from_field_text(self.tool_name, &name);
            writer.delete_term(term);
            writer
                .add_document(doc!(
                    self.tool_name => name,
                    self.description => description,
                    self.category => category,
                    self.keywords => kw_list.join(" "),
                    self.intents => intent_list.join(" | ")
                ))
                .map_err(VectorStoreError::Tantivy)?;
        }
        writer.commit().map_err(VectorStoreError::Tantivy)?;
        drop(cache);
        self.reader.reload().map_err(VectorStoreError::Tantivy)?;
        Ok(())
    }

    /// Batch index tool records. Reuses a cached `IndexWriter` when possible.
    ///
    /// # Errors
    ///
    /// Returns an error when writer creation, document write/commit, or reader reload fails.
    pub fn index_batch(&self, tools: &[ToolSearchResult]) -> Result<(), TantivyError> {
        let mut cache = self.writer_cache.borrow_mut();
        if cache.is_none() {
            *cache = Some(self.index.writer(100_000_000)?);
        }
        let writer = cache
            .as_mut()
            .ok_or_else(|| TantivyError::InvalidArgument("writer cache unavailable".to_string()))?;
        for tool in tools {
            if !crate::skill::is_routable_tool_name(&tool.name) {
                continue;
            }
            let term = Term::from_field_text(self.tool_name, &tool.name);
            writer.delete_term(term);
            writer.add_document(doc!(
                self.tool_name => tool.name.as_str(),
                self.description => tool.description.as_str(),
                self.category => tool.skill_name.as_str(),
                self.keywords => tool.routing_keywords.join(" "),
                self.intents => tool.intents.join(" | ")
            ))?;
        }
        writer.commit()?;
        drop(cache);
        self.reader.reload()?;
        Ok(())
    }
}
