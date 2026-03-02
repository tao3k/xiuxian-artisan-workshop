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

mod boost_ops;
mod confidence;
mod filter;
mod hybrid_ops;
mod ipc;
mod rows;
mod vector_ops;

use confidence::KEYWORD_BOOST;
use ipc::{search_results_to_ipc, tool_search_results_to_ipc};
use rows::{
    FtsRowColumns, build_fts_result_row, build_search_result_row, extract_vector_row_columns,
    required_lance_string_column,
};

pub use filter::json_to_lance_where;

fn f64_to_f32_saturating(value: f64) -> f32 {
    use num_traits::ToPrimitive;

    if !value.is_finite() {
        return 0.0;
    }

    match value.to_f32() {
        Some(v) => v,
        None if value.is_sign_negative() => f32::MIN,
        None => f32::MAX,
    }
}
