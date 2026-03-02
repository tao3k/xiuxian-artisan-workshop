type ToolResultsMap = std::collections::HashMap<String, skill::ToolSearchResult>;
type SearchDynArrayRef<'a> = Option<&'a std::sync::Arc<dyn lance::deps::arrow_array::Array>>;

struct SearchBatchColumns<'a> {
    values: &'a lance::deps::arrow_array::Float32Array,
    content: &'a lance::deps::arrow_array::StringArray,
    ids: &'a lance::deps::arrow_array::StringArray,
    metadata: Option<&'a lance::deps::arrow_array::StringArray>,
    skill_name: SearchDynArrayRef<'a>,
    category: SearchDynArrayRef<'a>,
    tool_name: SearchDynArrayRef<'a>,
    file_path: SearchDynArrayRef<'a>,
    routing_keywords: SearchDynArrayRef<'a>,
    intents: SearchDynArrayRef<'a>,
    row_count: usize,
}

struct ResolvedSearchRow {
    canonical_tool_name: String,
    skill_name: String,
    file_path: String,
    routing_keywords: Vec<String>,
    intents: Vec<String>,
    category: String,
    input_schema: serde_json::Value,
}

fn search_project_columns(has_metadata: bool) -> Vec<&'static str> {
    if has_metadata {
        vec![
            VECTOR_COLUMN,
            METADATA_COLUMN,
            CONTENT_COLUMN,
            "id",
            crate::SKILL_NAME_COLUMN,
            crate::CATEGORY_COLUMN,
            crate::TOOL_NAME_COLUMN,
            crate::FILE_PATH_COLUMN,
            crate::ROUTING_KEYWORDS_COLUMN,
            crate::INTENTS_COLUMN,
        ]
    } else {
        vec![
            VECTOR_COLUMN,
            CONTENT_COLUMN,
            "id",
            crate::SKILL_NAME_COLUMN,
            crate::CATEGORY_COLUMN,
            crate::TOOL_NAME_COLUMN,
            crate::FILE_PATH_COLUMN,
            crate::ROUTING_KEYWORDS_COLUMN,
            crate::INTENTS_COLUMN,
        ]
    }
}

fn finalize_tool_results(
    results_map: ToolResultsMap,
    threshold: f32,
    limit: usize,
) -> Vec<skill::ToolSearchResult> {
    let mut results: Vec<_> = results_map.into_values().collect();
    if threshold > 0.0 {
        results.retain(|result| result.score >= threshold);
    }
    results.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.tool_name.cmp(&b.tool_name))
    });
    results.truncate(limit);
    results
}
