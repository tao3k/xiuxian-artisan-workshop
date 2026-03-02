fn append_vector_results_from_batch(
    batch: &lance::deps::arrow_array::RecordBatch,
    query_vector: &[f32],
    skill_filter: Option<&str>,
    results_map: &mut ToolResultsMap,
) {
    let Some(columns) = extract_search_batch_columns(batch) else {
        return;
    };
    for row_index in 0..columns.row_count {
        let Some((canonical_name, result)) =
            build_vector_result_row(row_index, query_vector, skill_filter, &columns)
        else {
            continue;
        };
        results_map.insert(canonical_name, result);
    }
}

fn extract_search_batch_columns(
    batch: &lance::deps::arrow_array::RecordBatch,
) -> Option<SearchBatchColumns<'_>> {
    use lance::deps::arrow_array::Array;

    let vector_col = batch.column_by_name(VECTOR_COLUMN)?;
    let content_col = batch.column_by_name(CONTENT_COLUMN)?;
    let id_col = batch.column_by_name("id")?;
    let vector_arr = vector_col
        .as_any()
        .downcast_ref::<lance::deps::arrow_array::FixedSizeListArray>()?;
    let content = content_col
        .as_any()
        .downcast_ref::<lance::deps::arrow_array::StringArray>()?;
    let ids = id_col
        .as_any()
        .downcast_ref::<lance::deps::arrow_array::StringArray>()?;
    let values = vector_arr
        .values()
        .as_any()
        .downcast_ref::<lance::deps::arrow_array::Float32Array>()?;

    Some(SearchBatchColumns {
        values,
        content,
        ids,
        metadata: batch.column_by_name(METADATA_COLUMN).and_then(|column| {
            column
                .as_any()
                .downcast_ref::<lance::deps::arrow_array::StringArray>()
        }),
        skill_name: batch.column_by_name(crate::SKILL_NAME_COLUMN),
        category: batch.column_by_name(crate::CATEGORY_COLUMN),
        tool_name: batch.column_by_name(crate::TOOL_NAME_COLUMN),
        file_path: batch.column_by_name(crate::FILE_PATH_COLUMN),
        routing_keywords: batch.column_by_name(crate::ROUTING_KEYWORDS_COLUMN),
        intents: batch.column_by_name(crate::INTENTS_COLUMN),
        row_count: batch.num_rows(),
    })
}

fn build_vector_result_row(
    row_index: usize,
    query_vector: &[f32],
    skill_filter: Option<&str>,
    columns: &SearchBatchColumns<'_>,
) -> Option<(String, skill::ToolSearchResult)> {
    let skill_name = search_utf8_at(columns.skill_name, row_index);
    if skill_filter.is_some_and(|skill| skill_name != skill) {
        return None;
    }
    let category = search_utf8_at(columns.category, row_index);
    let score = vector_score_for_row(columns.values, columns.row_count, row_index, query_vector);
    let row_id = columns.ids.value(row_index).to_string();
    let resolved = resolve_search_row_fields(row_index, &row_id, &skill_name, &category, columns)?;
    let tool_name = if row_id.contains('.') {
        row_id
    } else {
        resolved.canonical_tool_name.clone()
    };
    if !skill::is_routable_tool_name(&tool_name) {
        return None;
    }

    Some((
        resolved.canonical_tool_name,
        skill::ToolSearchResult {
            name: tool_name.clone(),
            description: columns.content.value(row_index).to_string(),
            input_schema: resolved.input_schema,
            score,
            vector_score: Some(score),
            keyword_score: None,
            skill_name: resolved.skill_name,
            tool_name,
            file_path: resolved.file_path,
            routing_keywords: resolved.routing_keywords,
            intents: resolved.intents,
            category: resolved.category,
            parameters: vec![],
        },
    ))
}

fn vector_score_for_row(
    values: &lance::deps::arrow_array::Float32Array,
    row_count: usize,
    row_index: usize,
    query_vector: &[f32],
) -> f32 {
    if row_count == 0 {
        return 0.0;
    }
    let vector_len = values.len() / row_count;
    let mut dist_sq = 0.0f32;
    for (vector_index, query_value) in query_vector.iter().copied().enumerate() {
        let db_value = if vector_index < vector_len {
            values.value(row_index * vector_len + vector_index)
        } else {
            0.0
        };
        let diff = db_value - query_value;
        dist_sq += diff * diff;
    }
    1.0 / (1.0 + dist_sq.sqrt())
}

fn resolve_search_row_fields(
    row_index: usize,
    row_id: &str,
    skill_name: &str,
    category: &str,
    columns: &SearchBatchColumns<'_>,
) -> Option<ResolvedSearchRow> {
    if let Some(metadata_arr) = columns.metadata {
        use lance::deps::arrow_array::Array;

        if metadata_arr.is_null(row_index) {
            return Some(resolve_search_row_from_columns(
                row_index, row_id, skill_name, category, columns,
            ));
        }
        let metadata =
            serde_json::from_str::<serde_json::Value>(metadata_arr.value(row_index)).ok()?;
        return resolve_search_row_from_metadata(&metadata, row_id);
    }

    Some(resolve_search_row_from_columns(
        row_index, row_id, skill_name, category, columns,
    ))
}

fn resolve_search_row_from_columns(
    row_index: usize,
    row_id: &str,
    skill_name: &str,
    category: &str,
    columns: &SearchBatchColumns<'_>,
) -> ResolvedSearchRow {
    let tool_name = search_utf8_at(columns.tool_name, row_index);
    let canonical_tool_name = if tool_name.is_empty() {
        row_id.to_string()
    } else {
        tool_name
    };
    let resolved_skill_name = if skill_name.is_empty() {
        canonical_tool_name
            .split('.')
            .next()
            .unwrap_or("")
            .to_string()
    } else {
        skill_name.to_string()
    };
    let routing_keywords_raw = search_routing_keywords_at(columns.routing_keywords, row_index);
    let intents_raw = search_intents_at(columns.intents, row_index);
    let metadata = search_metadata_from_arrays(&routing_keywords_raw, &intents_raw);

    ResolvedSearchRow {
        canonical_tool_name,
        skill_name: resolved_skill_name.clone(),
        file_path: search_utf8_at(columns.file_path, row_index),
        routing_keywords: skill::resolve_routing_keywords(&metadata),
        intents: skill::resolve_intents(&metadata),
        category: if category.is_empty() {
            resolved_skill_name
        } else {
            category.to_string()
        },
        input_schema: serde_json::json!({}),
    }
}

fn resolve_search_row_from_metadata(
    metadata: &serde_json::Value,
    row_id: &str,
) -> Option<ResolvedSearchRow> {
    if metadata.get("type").and_then(|kind| kind.as_str()) != Some("command") {
        return None;
    }
    let canonical_tool_name = canonical_tool_name_from_result_meta(metadata, row_id)?;
    let skill_name = metadata
        .get("skill_name")
        .and_then(|value| value.as_str())
        .map_or_else(
            || {
                canonical_tool_name
                    .split('.')
                    .next()
                    .unwrap_or("")
                    .to_string()
            },
            String::from,
        );
    let file_path = metadata
        .get("file_path")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();
    let category = metadata
        .get("category")
        .and_then(|value| value.as_str())
        .or_else(|| metadata.get("skill_name").and_then(|value| value.as_str()))
        .unwrap_or("")
        .to_string();
    let input_schema = metadata.get("input_schema").map_or_else(
        || serde_json::json!({}),
        skill::normalize_input_schema_value,
    );

    Some(ResolvedSearchRow {
        canonical_tool_name,
        skill_name,
        file_path,
        routing_keywords: skill::resolve_routing_keywords(metadata),
        intents: skill::resolve_intents(metadata),
        category,
        input_schema,
    })
}

fn search_utf8_at(col: SearchDynArrayRef<'_>, row_index: usize) -> String {
    col.map(|column| crate::ops::get_utf8_at(column.as_ref(), row_index))
        .unwrap_or_default()
}

fn search_routing_keywords_at(col: SearchDynArrayRef<'_>, row_index: usize) -> Vec<String> {
    col.map(|column| crate::ops::get_routing_keywords_at(column.as_ref(), row_index))
        .unwrap_or_default()
}

fn search_intents_at(col: SearchDynArrayRef<'_>, row_index: usize) -> Vec<String> {
    col.map(|column| crate::ops::get_intents_at(column.as_ref(), row_index))
        .unwrap_or_default()
}

fn search_metadata_from_arrays(
    routing_keywords: &[String],
    intents: &[String],
) -> serde_json::Value {
    serde_json::json!({
        "routing_keywords": routing_keywords
            .iter()
            .map(|value| serde_json::Value::String(value.clone()))
            .collect::<Vec<_>>(),
        "intents": intents
            .iter()
            .map(|value| serde_json::Value::String(value.clone()))
            .collect::<Vec<_>>(),
    })
}
