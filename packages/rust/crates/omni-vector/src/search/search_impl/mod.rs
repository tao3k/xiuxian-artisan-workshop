use futures::TryStreamExt;
use lance_index::scalar::FullTextSearchQuery;
use omni_types::VectorSearchResult;
use serde_json::Value;

use crate::search::SearchOptions;
use crate::{
    CONTENT_COLUMN, HybridSearchResult, ID_COLUMN, KEYWORD_WEIGHT, KeywordSearchBackend,
    METADATA_COLUMN, RRF_K, SEMANTIC_WEIGHT, VECTOR_COLUMN, VectorStore, VectorStoreError,
    apply_weighted_rrf,
};

mod confidence;
mod filter;
mod ipc;
mod rows;

use confidence::KEYWORD_BOOST;
use ipc::{search_results_to_ipc, tool_search_results_to_ipc};
use rows::{
    FtsRowColumns, build_fts_result_row, build_search_result_row, extract_vector_row_columns,
    required_lance_string_column,
};

pub use filter::json_to_lance_where;

impl VectorStore {
    /// Backward-compatible search wrapper.
    ///
    /// # Errors
    ///
    /// Returns an error when dataset/table access fails or the query cannot be executed.
    pub async fn search(
        &self,
        table_name: &str,
        query: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>, VectorStoreError> {
        self.search_optimized(table_name, query, limit, SearchOptions::default())
            .await
    }

    /// Search with configurable scanner tuning for projection / read-ahead.
    ///
    /// # Errors
    ///
    /// Returns an error when the table does not exist, scanner setup fails, batch decoding
    /// fails, or Lance returns a query execution error.
    pub async fn search_optimized(
        &self,
        table_name: &str,
        query: Vec<f32>,
        limit: usize,
        options: SearchOptions,
    ) -> Result<Vec<VectorSearchResult>, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Err(VectorStoreError::TableNotFound(table_name.to_string()));
        }

        let dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let query_arr = lance::deps::arrow_array::Float32Array::from(query);
        let (pushdown_filter, metadata_filter) =
            Self::build_filter_plan(options.where_filter.as_deref());

        let mut scanner = dataset.scan();
        // When a filter is pushed down, Lance may use scalar indices (e.g. skill_name/category);
        // if filtering happens after ANN we request more candidates so enough pass the filter.
        let fetch_count = if pushdown_filter.is_some() {
            limit.saturating_mul(4).max(limit + 50)
        } else {
            limit.saturating_mul(2).max(limit + 10)
        };
        if !options.projected_columns.is_empty() {
            scanner.project(&options.projected_columns)?;
        }
        scanner.nearest(VECTOR_COLUMN, &query_arr, fetch_count)?;
        if let Some(batch_size) = options.batch_size {
            scanner.batch_size(batch_size);
        }
        if let Some(fragment_readahead) = options.fragment_readahead {
            scanner.fragment_readahead(fragment_readahead);
        }
        if let Some(batch_readahead) = options.batch_readahead {
            scanner.batch_readahead(batch_readahead);
        }
        if let Some(filter) = pushdown_filter {
            scanner.filter(&filter)?;
        }
        let scan_limit = options.scan_limit.unwrap_or(fetch_count);
        scanner.limit(Some(i64::try_from(scan_limit).unwrap_or(i64::MAX)), None)?;

        let mut stream = scanner.try_into_stream().await?;
        let mut results = Vec::with_capacity(limit.min(1024));

        while let Some(batch) = stream.try_next().await? {
            let columns = extract_vector_row_columns(&batch)?;
            for index in 0..batch.num_rows() {
                if let Some(result) =
                    build_search_result_row(index, &columns, metadata_filter.as_ref())
                {
                    results.push(result);
                }
            }
        }

        results.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        Ok(results)
    }

    /// Search with configurable options; returns Arrow IPC stream bytes for zero-copy consumption in Python.
    /// See [search result batch contract](docs/reference/search-result-batch-contract.md).
    ///
    /// # Errors
    ///
    /// Returns an error if underlying vector search fails or IPC encoding fails.
    pub async fn search_optimized_ipc(
        &self,
        table_name: &str,
        query: Vec<f32>,
        limit: usize,
        options: SearchOptions,
    ) -> Result<Vec<u8>, VectorStoreError> {
        let results = self
            .search_optimized(table_name, query, limit, options.clone())
            .await?;
        search_results_to_ipc(&results, options.ipc_projection.as_deref())
            .map_err(VectorStoreError::General)
    }

    /// Tool search; returns Arrow IPC stream bytes for zero-copy consumption in Python.
    /// Schema: name, description, score, `skill_name`, `tool_name`, `file_path`,
    /// `routing_keywords`, intents, category, metadata, `vector_score`, `keyword_score`,
    /// `final_score`, `confidence`, `ranking_reason`, `input_schema_digest`.
    ///
    /// # Errors
    ///
    /// Returns an error if tool search fails or IPC encoding fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn search_tools_ipc(
        &self,
        table_name: &str,
        query_vector: &[f32],
        query_text: Option<&str>,
        limit: usize,
        threshold: f32,
        options: crate::skill::ToolSearchOptions,
        where_filter: Option<&str>,
    ) -> Result<Vec<u8>, VectorStoreError> {
        let results = self
            .search_tools_with_options(
                table_name,
                query_vector,
                query_text,
                limit,
                threshold,
                options,
                where_filter,
            )
            .await?;
        tool_search_results_to_ipc(&results).map_err(VectorStoreError::General)
    }

    /// Run native Lance full-text search over the content column.
    ///
    /// # Errors
    ///
    /// Returns an error if the table is missing or Lance FTS query execution fails.
    #[allow(clippy::cast_possible_truncation)]
    pub async fn search_fts(
        &self,
        table_name: &str,
        query: &str,
        limit: usize,
        where_filter: Option<&str>,
    ) -> Result<Vec<crate::skill::ToolSearchResult>, VectorStoreError> {
        if query.trim().is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Err(VectorStoreError::TableNotFound(table_name.to_string()));
        }

        let dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let mut scanner = dataset.scan();
        scanner.project(&[
            ID_COLUMN,
            CONTENT_COLUMN,
            crate::SKILL_NAME_COLUMN,
            crate::CATEGORY_COLUMN,
            crate::TOOL_NAME_COLUMN,
            crate::FILE_PATH_COLUMN,
            crate::ROUTING_KEYWORDS_COLUMN,
            crate::INTENTS_COLUMN,
        ])?;
        scanner.full_text_search(FullTextSearchQuery::new(query.to_string()))?;
        if let Some(filter) = where_filter.map(str::trim).filter(|f| !f.is_empty()) {
            scanner.filter(filter)?;
        }
        scanner.limit(Some(i64::try_from(limit).unwrap_or(i64::MAX)), None)?;

        let mut stream = scanner.try_into_stream().await?;
        let mut results = Vec::with_capacity(limit.min(1024));

        while let Some(batch) = stream.try_next().await? {
            let columns = FtsRowColumns {
                ids: required_lance_string_column(&batch, ID_COLUMN, "fts")?,
                contents: required_lance_string_column(&batch, CONTENT_COLUMN, "fts")?,
                metadata: batch.column_by_name(METADATA_COLUMN),
                score: batch.column_by_name("_score"),
                skill_name: batch.column_by_name(crate::SKILL_NAME_COLUMN),
                category: batch.column_by_name(crate::CATEGORY_COLUMN),
                tool_name: batch.column_by_name(crate::TOOL_NAME_COLUMN),
                file_path: batch.column_by_name(crate::FILE_PATH_COLUMN),
                routing_keywords: batch.column_by_name(crate::ROUTING_KEYWORDS_COLUMN),
                intents: batch.column_by_name(crate::INTENTS_COLUMN),
            };

            for index in 0..batch.num_rows() {
                results.push(build_fts_result_row(index, &columns));
            }
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        Ok(results)
    }

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

    fn build_filter_plan(where_filter: Option<&str>) -> (Option<String>, Option<Value>) {
        let Some(filter) = where_filter.map(str::trim).filter(|f| !f.is_empty()) else {
            return (None, None);
        };

        if let Ok(json_filter) = serde_json::from_str::<Value>(filter) {
            let pushdown = if Self::is_pushdown_eligible_json(&json_filter) {
                let candidate = json_to_lance_where(&json_filter);
                if candidate.is_empty() {
                    None
                } else {
                    Some(candidate)
                }
            } else {
                None
            };
            return (pushdown, Some(json_filter));
        }

        (Some(filter.to_string()), None)
    }

    fn is_pushdown_eligible_json(expr: &Value) -> bool {
        let Some(obj) = expr.as_object() else {
            return false;
        };
        obj.keys().all(|key| {
            key == ID_COLUMN
                || key == CONTENT_COLUMN
                || key == METADATA_COLUMN
                || key == "_distance"
        })
    }

    /// Hybrid search combining vector similarity and keyword (BM25) search.
    /// Vector and keyword queries run in parallel via `try_join!` to reduce latency;
    /// vector failure fails fast, keyword failure falls back to empty.
    ///
    /// # Errors
    ///
    /// Returns an error if vector search fails.
    #[allow(clippy::cast_possible_truncation)]
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
            .map(|r| (r.id.clone(), 1.0 - r.distance as f32))
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

    /// Apply keyword boosting to search results.
    pub fn apply_keyword_boost(results: &mut [VectorSearchResult], keywords: &[String]) {
        if keywords.is_empty() {
            return;
        }
        let mut query_keywords: Vec<String> = Vec::new();
        for s in keywords {
            let lowered = s.to_lowercase();
            for w in lowered.split_whitespace() {
                query_keywords.push(w.to_string());
            }
        }

        for result in results {
            let mut keyword_score = 0.0;

            // 1. Boost from routing_keywords (Arrow-native or metadata fallback)
            let keywords_to_check: Vec<String> = if !result.routing_keywords.is_empty() {
                result
                    .routing_keywords
                    .split_whitespace()
                    .map(str::to_lowercase)
                    .collect()
            } else if let Some(keywords_arr) = result
                .metadata
                .get("routing_keywords")
                .and_then(|v| v.as_array())
            {
                keywords_arr
                    .iter()
                    .filter_map(|k| k.as_str().map(str::to_lowercase))
                    .collect()
            } else {
                vec![]
            };
            for kw in &query_keywords {
                if keywords_to_check.iter().any(|k| k.contains(kw)) {
                    keyword_score += KEYWORD_BOOST;
                }
            }

            // 2. Boost from intents (Arrow-native or metadata fallback)
            let intents_to_check: Vec<String> = if !result.intents.is_empty() {
                result
                    .intents
                    .split(" | ")
                    .map(|s| s.trim().to_lowercase())
                    .collect()
            } else if let Some(intents_arr) =
                result.metadata.get("intents").and_then(|v| v.as_array())
            {
                intents_arr
                    .iter()
                    .filter_map(|k| k.as_str().map(str::to_lowercase))
                    .collect()
            } else {
                vec![]
            };
            for kw in &query_keywords {
                if intents_to_check.iter().any(|k| k.contains(kw)) {
                    keyword_score += KEYWORD_BOOST * 1.2; // Intents are higher signal
                }
            }

            let tool_name_lower = if result.tool_name.is_empty() {
                result.id.to_lowercase()
            } else {
                result.tool_name.to_lowercase()
            };
            let content_lower = result.content.to_lowercase();
            for kw in &query_keywords {
                if tool_name_lower.contains(kw) {
                    keyword_score += KEYWORD_BOOST * 0.5;
                }
                if content_lower.contains(kw) {
                    keyword_score += KEYWORD_BOOST * 0.3;
                }
            }
            let keyword_bonus = keyword_score * 0.3f32;
            result.distance = (result.distance - f64::from(keyword_bonus)).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests;
