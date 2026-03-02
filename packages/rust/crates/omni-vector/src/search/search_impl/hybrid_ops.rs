use super::{
    HybridSearchResult, KEYWORD_WEIGHT, KeywordSearchBackend, RRF_K, SEMANTIC_WEIGHT,
    SearchOptions, VectorStore, VectorStoreError, apply_weighted_rrf, f64_to_f32_saturating,
};

impl VectorStore {
    /// Unified keyword search entrypoint for configured backend.
    ///
    /// # Errors
    ///
    /// Returns an error if keyword backend is unavailable or backend query fails.
    pub async fn keyword_search(
        &self,
        table_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::skill::ToolSearchResult>, VectorStoreError> {
        match self.keyword_backend {
            KeywordSearchBackend::Tantivy => {
                let index = self.keyword_index.as_ref().ok_or_else(|| {
                    VectorStoreError::General("Keyword index not enabled.".to_string())
                })?;
                index.search(query, limit)
            }
            KeywordSearchBackend::LanceFts => self.search_fts(table_name, query, limit, None).await,
        }
    }

    /// Hybrid search combining vector similarity and keyword (`BM25`) search.
    /// Vector and keyword queries run in parallel via `try_join!` to reduce latency;
    /// vector failure fails fast, keyword failure falls back to empty.
    ///
    /// # Errors
    ///
    /// Returns an error if vector search fails.
    pub async fn hybrid_search(
        &self,
        table_name: &str,
        query: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<HybridSearchResult>, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Err(VectorStoreError::TableNotFound(table_name.to_string()));
        }

        let vector_fut = self.search_optimized(
            table_name,
            query_vector,
            limit * 2,
            SearchOptions::default(),
        );
        let kw_fut = async {
            match self.keyword_search(table_name, query, limit * 2).await {
                Ok(v) => Ok(v),
                Err(e) => {
                    log::debug!("Keyword search failed, falling back to vector-only: {e}");
                    Ok(Vec::new())
                }
            }
        };
        let (vector_results, kw_results) = tokio::try_join!(vector_fut, kw_fut)?;

        let vector_scores: Vec<(String, f32)> = vector_results
            .iter()
            .map(|r| (r.id.clone(), f64_to_f32_saturating(1.0 - r.distance)))
            .collect();

        let fused_results = apply_weighted_rrf(
            vector_scores,
            kw_results,
            RRF_K,
            SEMANTIC_WEIGHT,
            KEYWORD_WEIGHT,
            query,
        );

        Ok(fused_results.into_iter().take(limit).collect())
    }

    /// Index a document in the keyword index.
    ///
    /// # Errors
    ///
    /// Returns an error if keyword backend upsert fails.
    pub fn index_keyword(
        &self,
        name: &str,
        description: &str,
        category: &str,
        keywords: &[String],
        intents: &[String],
    ) -> Result<(), VectorStoreError> {
        if self.keyword_backend != KeywordSearchBackend::Tantivy {
            return Ok(());
        }
        if let Some(index) = &self.keyword_index {
            index.upsert_document(name, description, category, keywords, intents)?;
        }
        Ok(())
    }

    /// Bulk index documents in the keyword index.
    ///
    /// # Errors
    ///
    /// Returns an error if bulk keyword indexing fails.
    pub fn bulk_index_keywords<I>(&self, docs: I) -> Result<(), VectorStoreError>
    where
        I: IntoIterator<Item = (String, String, String, Vec<String>, Vec<String>)>,
    {
        if self.keyword_backend != KeywordSearchBackend::Tantivy {
            return Ok(());
        }
        if let Some(index) = &self.keyword_index {
            index.bulk_upsert(docs)?;
        }
        Ok(())
    }
}
