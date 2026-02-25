use omni_types::VectorSearchResult;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    CONTENT_COLUMN, FILE_PATH_COLUMN, ID_COLUMN, INTENTS_COLUMN, METADATA_COLUMN,
    ROUTING_KEYWORDS_COLUMN, TOOL_NAME_COLUMN, VectorStore, VectorStoreError,
};

#[derive(Deserialize, Default)]
struct FtsMetadataRow {
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    skill_name: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    file_path: Option<String>,
    #[serde(default)]
    input_schema: Option<Value>,
    #[serde(default)]
    routing_keywords: Vec<String>,
    #[serde(default)]
    intents: Vec<String>,
    #[serde(default)]
    parameters: Vec<String>,
}

pub(super) type LanceArrayRef<'a> = Option<&'a std::sync::Arc<dyn lance::deps::arrow_array::Array>>;
pub(super) type LanceRecordBatch = lance::deps::arrow_array::RecordBatch;
pub(super) type LanceStringArray = lance::deps::arrow_array::StringArray;

pub(super) struct FtsRowColumns<'a> {
    pub(super) ids: &'a LanceStringArray,
    pub(super) contents: &'a LanceStringArray,
    pub(super) metadata: LanceArrayRef<'a>,
    pub(super) score: LanceArrayRef<'a>,
    pub(super) skill_name: LanceArrayRef<'a>,
    pub(super) category: LanceArrayRef<'a>,
    pub(super) tool_name: LanceArrayRef<'a>,
    pub(super) file_path: LanceArrayRef<'a>,
    pub(super) routing_keywords: LanceArrayRef<'a>,
    pub(super) intents: LanceArrayRef<'a>,
}

pub(super) struct VectorRowColumns<'a> {
    pub(super) ids: &'a LanceStringArray,
    pub(super) contents: &'a LanceStringArray,
    pub(super) distances: &'a lance::deps::arrow_array::Float32Array,
    pub(super) metadata: LanceArrayRef<'a>,
    pub(super) tool_name: LanceArrayRef<'a>,
    pub(super) file_path: LanceArrayRef<'a>,
    pub(super) routing_keywords: LanceArrayRef<'a>,
    pub(super) intents: LanceArrayRef<'a>,
}

fn normalize_string_vec(v: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for s in v {
        let t = s.trim();
        if t.is_empty() {
            continue;
        }
        if seen.insert(t.to_string()) {
            out.push(t.to_string());
        }
    }
    out
}

fn parse_metadata_cell(raw: &str) -> Value {
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::String(inner)) => {
            serde_json::from_str::<Value>(&inner).unwrap_or(Value::String(inner))
        }
        Ok(value) => value,
        Err(_err) => Value::Null,
    }
}

fn utf8_or_default_at(col: LanceArrayRef<'_>, index: usize) -> String {
    col.map(|c| crate::ops::get_utf8_at(c.as_ref(), index))
        .unwrap_or_default()
}

fn non_empty_utf8_at(col: LanceArrayRef<'_>, index: usize) -> Option<String> {
    let value = utf8_or_default_at(col, index);
    if value.is_empty() { None } else { Some(value) }
}

pub(super) fn required_lance_string_column<'a>(
    batch: &'a LanceRecordBatch,
    column: &str,
    context: &str,
) -> Result<&'a LanceStringArray, VectorStoreError> {
    let col = batch
        .column_by_name(column)
        .ok_or_else(|| VectorStoreError::General(format!("{column} column not found")))?;
    col.as_any()
        .downcast_ref::<LanceStringArray>()
        .ok_or_else(|| {
            VectorStoreError::General(format!("{column} column type mismatch for {context}"))
        })
}

fn required_lance_f32_column<'a>(
    batch: &'a LanceRecordBatch,
    column: &str,
    context: &str,
) -> Result<&'a lance::deps::arrow_array::Float32Array, VectorStoreError> {
    let col = batch
        .column_by_name(column)
        .ok_or_else(|| VectorStoreError::General(format!("{column} column not found")))?;
    col.as_any()
        .downcast_ref::<lance::deps::arrow_array::Float32Array>()
        .ok_or_else(|| {
            VectorStoreError::General(format!("{column} column type mismatch for {context}"))
        })
}

fn parse_fts_metadata_row(metadata_col: LanceArrayRef<'_>, index: usize) -> FtsMetadataRow {
    use lance::deps::arrow_array::Array as _;

    let Some(col) = metadata_col else {
        return FtsMetadataRow::default();
    };
    let Some(arr) = col.as_any().downcast_ref::<LanceStringArray>() else {
        return FtsMetadataRow::default();
    };
    if arr.is_null(index) {
        return FtsMetadataRow::default();
    }
    serde_json::from_str(arr.value(index)).unwrap_or_default()
}

fn parse_search_metadata_value(
    metadata_col: LanceArrayRef<'_>,
    index: usize,
    default_object: bool,
) -> Value {
    use lance::deps::arrow_array::Array as _;

    let default = if default_object {
        Value::Object(serde_json::Map::new())
    } else {
        Value::Null
    };
    let Some(col) = metadata_col else {
        return default;
    };
    let Some(arr) = col.as_any().downcast_ref::<LanceStringArray>() else {
        return default;
    };
    if arr.is_null(index) {
        return default;
    }
    parse_metadata_cell(arr.value(index))
}

#[allow(clippy::cast_possible_truncation)]
fn parse_fts_score(score_col: LanceArrayRef<'_>, index: usize) -> f32 {
    let Some(col) = score_col else {
        return 0.0;
    };
    if let Some(arr) = col
        .as_any()
        .downcast_ref::<lance::deps::arrow_array::Float32Array>()
    {
        arr.value(index)
    } else if let Some(arr) = col
        .as_any()
        .downcast_ref::<lance::deps::arrow_array::Float64Array>()
    {
        arr.value(index) as f32
    } else {
        0.0
    }
}

fn metadata_array_to_joined(metadata: &Value, key: &str, separator: &str) -> String {
    metadata
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(separator)
        })
        .unwrap_or_default()
}

fn resolve_vector_row_metadata(
    index: usize,
    metadata_col: LanceArrayRef<'_>,
    tool_name: &str,
    file_path: &str,
    routing_keywords_vec: &[String],
    intents_vec: &[String],
) -> (Value, String, String, String, String) {
    if tool_name.is_empty()
        && file_path.is_empty()
        && routing_keywords_vec.is_empty()
        && intents_vec.is_empty()
    {
        let metadata = parse_search_metadata_value(metadata_col, index, false);
        let tool_name_out = metadata
            .get("tool_name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let file_path_out = metadata
            .get("file_path")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let routing_keywords_out = metadata_array_to_joined(&metadata, "routing_keywords", " ");
        let intents_out = metadata_array_to_joined(&metadata, "intents", " | ");
        return (
            metadata,
            tool_name_out,
            file_path_out,
            routing_keywords_out,
            intents_out,
        );
    }

    let mut metadata = parse_search_metadata_value(metadata_col, index, true);
    if !metadata.is_object() {
        metadata = Value::Object(serde_json::Map::new());
    }
    if let Some(obj) = metadata.as_object_mut() {
        if !tool_name.is_empty() {
            obj.entry("tool_name".to_string())
                .or_insert_with(|| Value::String(tool_name.to_string()));
        }
        if !file_path.is_empty() {
            obj.entry("file_path".to_string())
                .or_insert_with(|| Value::String(file_path.to_string()));
        }
        if !routing_keywords_vec.is_empty() {
            obj.entry("routing_keywords".to_string())
                .or_insert_with(|| {
                    Value::Array(
                        routing_keywords_vec
                            .iter()
                            .map(|value| Value::String(value.clone()))
                            .collect(),
                    )
                });
        }
        if !intents_vec.is_empty() {
            obj.entry("intents".to_string()).or_insert_with(|| {
                Value::Array(
                    intents_vec
                        .iter()
                        .map(|value| Value::String(value.clone()))
                        .collect(),
                )
            });
        }
    }
    (
        metadata,
        tool_name.to_string(),
        file_path.to_string(),
        routing_keywords_vec.join(" "),
        intents_vec.join(" | "),
    )
}

pub(super) fn build_search_result_row(
    index: usize,
    columns: &VectorRowColumns<'_>,
    metadata_filter: Option<&Value>,
) -> Option<VectorSearchResult> {
    let tool_name = utf8_or_default_at(columns.tool_name, index);
    let file_path = utf8_or_default_at(columns.file_path, index);
    let routing_keywords_vec = columns
        .routing_keywords
        .map(|c| crate::ops::get_routing_keywords_at(c.as_ref(), index))
        .unwrap_or_default();
    let intents_vec = columns
        .intents
        .map(|c| crate::ops::get_intents_at(c.as_ref(), index))
        .unwrap_or_default();

    let (metadata, tool_name_out, file_path_out, routing_keywords_out, intents_out) =
        resolve_vector_row_metadata(
            index,
            columns.metadata,
            &tool_name,
            &file_path,
            &routing_keywords_vec,
            &intents_vec,
        );

    if let Some(conditions) = metadata_filter
        && !VectorStore::matches_filter(&metadata, conditions)
    {
        return None;
    }

    let id_val = columns.ids.value(index).to_string();
    let (id, tool_name) = if tool_name_out.is_empty() {
        (id_val.clone(), id_val)
    } else {
        (id_val, tool_name_out)
    };

    Some(VectorSearchResult {
        id,
        content: columns.contents.value(index).to_string(),
        tool_name,
        file_path: file_path_out,
        routing_keywords: routing_keywords_out,
        intents: intents_out,
        metadata,
        distance: f64::from(columns.distances.value(index)),
    })
}

pub(super) fn extract_vector_row_columns(
    batch: &LanceRecordBatch,
) -> Result<VectorRowColumns<'_>, VectorStoreError> {
    Ok(VectorRowColumns {
        ids: required_lance_string_column(batch, ID_COLUMN, "vector search")?,
        contents: required_lance_string_column(batch, CONTENT_COLUMN, "vector search")?,
        distances: required_lance_f32_column(batch, "_distance", "vector search")?,
        metadata: batch.column_by_name(METADATA_COLUMN),
        tool_name: batch.column_by_name(TOOL_NAME_COLUMN),
        file_path: batch.column_by_name(FILE_PATH_COLUMN),
        routing_keywords: batch.column_by_name(ROUTING_KEYWORDS_COLUMN),
        intents: batch.column_by_name(INTENTS_COLUMN),
    })
}

pub(super) fn build_fts_result_row(
    index: usize,
    columns: &FtsRowColumns<'_>,
) -> crate::skill::ToolSearchResult {
    let metadata = parse_fts_metadata_row(columns.metadata, index);
    let score = parse_fts_score(columns.score, index);
    let id_str = columns.ids.value(index).to_string();

    let tool_name = non_empty_utf8_at(columns.tool_name, index)
        .or_else(|| metadata.tool_name.clone().filter(|value| !value.is_empty()))
        .unwrap_or_else(|| id_str.clone());

    let skill_name_raw = utf8_or_default_at(columns.skill_name, index);
    let skill_name = if skill_name_raw.is_empty() {
        metadata
            .skill_name
            .clone()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| tool_name.split('.').next().unwrap_or("").to_string())
    } else {
        skill_name_raw
    };

    let category_raw = utf8_or_default_at(columns.category, index);
    let category = if category_raw.is_empty() {
        metadata
            .category
            .clone()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| skill_name.clone())
    } else {
        category_raw
    };

    let keywords = non_empty_utf8_at(columns.routing_keywords, index).map_or_else(
        || normalize_string_vec(metadata.routing_keywords.clone()),
        |value| normalize_string_vec(value.split_whitespace().map(String::from).collect()),
    );
    let intents = non_empty_utf8_at(columns.intents, index).map_or_else(
        || normalize_string_vec(metadata.intents.clone()),
        |value| normalize_string_vec(value.split(" | ").map(String::from).collect()),
    );
    let file_path = non_empty_utf8_at(columns.file_path, index)
        .unwrap_or_else(|| metadata.file_path.unwrap_or_default());

    let input_schema = metadata.input_schema.as_ref().map_or_else(
        || serde_json::json!({}),
        crate::skill::normalize_input_schema_value,
    );

    crate::skill::ToolSearchResult {
        name: id_str,
        description: columns.contents.value(index).to_string(),
        input_schema,
        score,
        vector_score: Some(score),
        keyword_score: None,
        skill_name,
        tool_name,
        file_path,
        routing_keywords: keywords,
        intents,
        category,
        parameters: metadata.parameters,
    }
}
