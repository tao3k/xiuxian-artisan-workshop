use std::io::Cursor;

use arrow::array::{Array, Float32Array, Float64Array, ListBuilder, StringArray, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow_ipc::writer::StreamWriter;
use omni_types::VectorSearchResult;

use super::confidence::{
    ConfidenceProfile, build_ranking_reason, calibrate_confidence_with_attributes,
    input_schema_digest,
};

/// Allowed column names for IPC projection (vector search result batch).
const IPC_VECTOR_COLUMNS: &[&str] = &[
    "id",
    "content",
    "tool_name",
    "file_path",
    "routing_keywords",
    "intents",
    "_distance",
    "metadata",
];

struct VectorIpcData {
    ids: Vec<String>,
    contents: Vec<String>,
    tool_names: Vec<String>,
    file_paths: Vec<String>,
    distances: Vec<f64>,
    metadata_json: Vec<String>,
    routing_keywords_array: std::sync::Arc<dyn Array>,
    intents_array: std::sync::Arc<dyn Array>,
}

fn collect_vector_ipc_data(results: &[VectorSearchResult]) -> VectorIpcData {
    let ids = results.iter().map(|r| r.id.clone()).collect();
    let contents = results.iter().map(|r| r.content.clone()).collect();
    let tool_names = results.iter().map(|r| r.tool_name.clone()).collect();
    let file_paths = results.iter().map(|r| r.file_path.clone()).collect();
    let distances = results.iter().map(|r| r.distance).collect();
    let metadata_json = results
        .iter()
        .map(|r| serde_json::to_string(&r.metadata).unwrap_or_else(|_| "null".to_string()))
        .collect();

    let mut rk_builder = ListBuilder::new(StringBuilder::new());
    for result in results {
        for keyword in result
            .routing_keywords
            .split_whitespace()
            .filter(|token| !token.is_empty())
        {
            rk_builder.values().append_value(keyword);
        }
        rk_builder.append(true);
    }
    let routing_keywords_array: std::sync::Arc<dyn Array> =
        std::sync::Arc::new(rk_builder.finish());

    let mut intents_builder = ListBuilder::new(StringBuilder::new());
    for result in results {
        for intent in result
            .intents
            .split(" | ")
            .map(str::trim)
            .filter(|token| !token.is_empty())
        {
            intents_builder.values().append_value(intent);
        }
        intents_builder.append(true);
    }
    let intents_array: std::sync::Arc<dyn Array> = std::sync::Arc::new(intents_builder.finish());

    VectorIpcData {
        ids,
        contents,
        tool_names,
        file_paths,
        distances,
        metadata_json,
        routing_keywords_array,
        intents_array,
    }
}

fn resolve_vector_ipc_projection(projection: Option<&[String]>) -> Result<Vec<&str>, String> {
    match projection {
        Some(columns) if !columns.is_empty() => {
            for name in columns {
                if !IPC_VECTOR_COLUMNS.contains(&name.as_str()) {
                    return Err(format!("invalid ipc_projection column: {name}"));
                }
            }
            Ok(columns.iter().map(String::as_str).collect())
        }
        _ => Ok(IPC_VECTOR_COLUMNS.to_vec()),
    }
}

fn append_vector_ipc_column(
    col: &str,
    data: &VectorIpcData,
    schema_fields: &mut Vec<Field>,
    arrays: &mut Vec<std::sync::Arc<dyn Array>>,
) {
    match col {
        "id" => {
            schema_fields.push(Field::new("id", DataType::Utf8, true));
            arrays.push(std::sync::Arc::new(StringArray::from(data.ids.clone())));
        }
        "content" => {
            schema_fields.push(Field::new("content", DataType::Utf8, true));
            arrays.push(std::sync::Arc::new(StringArray::from(
                data.contents.clone(),
            )));
        }
        "tool_name" => {
            schema_fields.push(Field::new("tool_name", DataType::Utf8, true));
            arrays.push(std::sync::Arc::new(StringArray::from(
                data.tool_names.clone(),
            )));
        }
        "file_path" => {
            schema_fields.push(Field::new("file_path", DataType::Utf8, true));
            arrays.push(std::sync::Arc::new(StringArray::from(
                data.file_paths.clone(),
            )));
        }
        "routing_keywords" => {
            schema_fields.push(Field::new(
                "routing_keywords",
                DataType::List(std::sync::Arc::new(Field::new(
                    "item",
                    DataType::Utf8,
                    true,
                ))),
                true,
            ));
            arrays.push(data.routing_keywords_array.clone());
        }
        "intents" => {
            schema_fields.push(Field::new(
                "intents",
                DataType::List(std::sync::Arc::new(Field::new(
                    "item",
                    DataType::Utf8,
                    true,
                ))),
                true,
            ));
            arrays.push(data.intents_array.clone());
        }
        "_distance" => {
            schema_fields.push(Field::new("_distance", DataType::Float64, true));
            arrays.push(std::sync::Arc::new(Float64Array::from(
                data.distances.clone(),
            )));
        }
        "metadata" => {
            schema_fields.push(Field::new("metadata", DataType::Utf8, true));
            arrays.push(std::sync::Arc::new(StringArray::from(
                data.metadata_json.clone(),
            )));
        }
        _ => {}
    }
}

fn record_batch_to_ipc_bytes(batch: &arrow::record_batch::RecordBatch) -> Result<Vec<u8>, String> {
    let mut buf = Cursor::new(Vec::new());
    let mut writer =
        StreamWriter::try_new(&mut buf, batch.schema().as_ref()).map_err(|e| e.to_string())?;
    writer.write(batch).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

/// Encode search results as Arrow IPC stream bytes (single `RecordBatch`).
/// If `projection` is Some and non-empty, only those columns are included (smaller payload).
/// Schema (full): id, content, `tool_name`, `file_path`, `routing_keywords` (List<Utf8>),
/// intents (List<Utf8>), _distance, metadata (Utf8).
pub(super) fn search_results_to_ipc(
    results: &[VectorSearchResult],
    projection: Option<&[String]>,
) -> Result<Vec<u8>, String> {
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;

    let cols = resolve_vector_ipc_projection(projection)?;
    let data = collect_vector_ipc_data(results);

    let mut schema_fields = Vec::with_capacity(cols.len());
    let mut arrays: Vec<Arc<dyn Array>> = Vec::with_capacity(cols.len());
    for col in &cols {
        append_vector_ipc_column(col, &data, &mut schema_fields, &mut arrays);
    }

    let schema = Schema::new(schema_fields);
    let batch = RecordBatch::try_new(Arc::new(schema), arrays).map_err(|e| e.to_string())?;
    record_batch_to_ipc_bytes(&batch)
}

struct ToolIpcData {
    names: Vec<String>,
    descriptions: Vec<String>,
    scores: Vec<f32>,
    skill_names: Vec<String>,
    tool_names: Vec<String>,
    file_paths: Vec<String>,
    categories: Vec<String>,
    metadata_json: Vec<String>,
    vector_scores: Vec<Option<f32>>,
    keyword_scores: Vec<Option<f32>>,
    final_scores: Vec<f32>,
    confidences: Vec<String>,
    ranking_reasons: Vec<String>,
    input_schema_digests: Vec<String>,
    routing_keywords_array: std::sync::Arc<dyn Array>,
    intents_array: std::sync::Arc<dyn Array>,
}

fn empty_tool_search_ipc_batch() -> Result<arrow::record_batch::RecordBatch, String> {
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;

    let schema = Schema::new(vec![
        Field::new("name", DataType::Utf8, true),
        Field::new("description", DataType::Utf8, true),
        Field::new("score", DataType::Float32, true),
        Field::new("final_score", DataType::Float32, true),
        Field::new("confidence", DataType::Utf8, true),
        Field::new("ranking_reason", DataType::Utf8, true),
        Field::new("input_schema_digest", DataType::Utf8, true),
    ]);
    RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(StringArray::from(Vec::<String>::new())),
            Arc::new(StringArray::from(Vec::<String>::new())),
            Arc::new(Float32Array::from(Vec::<f32>::new())),
            Arc::new(Float32Array::from(Vec::<f32>::new())),
            Arc::new(StringArray::from(Vec::<String>::new())),
            Arc::new(StringArray::from(Vec::<String>::new())),
            Arc::new(StringArray::from(Vec::<String>::new())),
        ],
    )
    .map_err(|e| e.to_string())
}

fn collect_tool_ipc_data(results: &[crate::skill::ToolSearchResult]) -> ToolIpcData {
    let names = results.iter().map(|r| r.name.clone()).collect();
    let descriptions = results.iter().map(|r| r.description.clone()).collect();
    let scores = results.iter().map(|r| r.score).collect();
    let skill_names = results.iter().map(|r| r.skill_name.clone()).collect();
    let tool_names = results.iter().map(|r| r.tool_name.clone()).collect();
    let file_paths = results.iter().map(|r| r.file_path.clone()).collect();
    let categories = results.iter().map(|r| r.category.clone()).collect();
    let metadata_json = results
        .iter()
        .map(|r| serde_json::to_string(&r.input_schema).unwrap_or_else(|_| "{}".to_string()))
        .collect();
    let vector_scores = results.iter().map(|r| r.vector_score).collect();
    let keyword_scores = results.iter().map(|r| r.keyword_score).collect();

    let profile = ConfidenceProfile::default();
    let mut final_scores: Vec<f32> = Vec::with_capacity(results.len());
    let mut confidences: Vec<String> = Vec::with_capacity(results.len());
    let mut ranking_reasons: Vec<String> = Vec::with_capacity(results.len());
    let mut input_schema_digests: Vec<String> = Vec::with_capacity(results.len());
    for (index, result) in results.iter().enumerate() {
        let second_score = results.get(index + 1).map(|s| s.score);
        let (confidence, final_score) = calibrate_confidence_with_attributes(
            result.score,
            second_score,
            result.vector_score,
            result.keyword_score,
            &profile,
        );
        final_scores.push(final_score);
        confidences.push(confidence.to_string());
        ranking_reasons.push(build_ranking_reason(
            result,
            result.score,
            final_score,
            confidence,
        ));
        input_schema_digests.push(input_schema_digest(&result.input_schema));
    }

    let mut routing_keywords_builder = ListBuilder::new(StringBuilder::new());
    for result in results {
        for keyword in &result.routing_keywords {
            routing_keywords_builder
                .values()
                .append_value(keyword.as_str());
        }
        routing_keywords_builder.append(true);
    }
    let routing_keywords_array: std::sync::Arc<dyn Array> =
        std::sync::Arc::new(routing_keywords_builder.finish());

    let mut intents_builder = ListBuilder::new(StringBuilder::new());
    for result in results {
        for intent in &result.intents {
            intents_builder.values().append_value(intent.as_str());
        }
        intents_builder.append(true);
    }
    let intents_array: std::sync::Arc<dyn Array> = std::sync::Arc::new(intents_builder.finish());

    ToolIpcData {
        names,
        descriptions,
        scores,
        skill_names,
        tool_names,
        file_paths,
        categories,
        metadata_json,
        vector_scores,
        keyword_scores,
        final_scores,
        confidences,
        ranking_reasons,
        input_schema_digests,
        routing_keywords_array,
        intents_array,
    }
}

fn build_tool_search_ipc_batch(
    data: &ToolIpcData,
) -> Result<arrow::record_batch::RecordBatch, String> {
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;

    let vector_score_array = data
        .vector_scores
        .clone()
        .into_iter()
        .collect::<Float32Array>();
    let keyword_score_array = data
        .keyword_scores
        .clone()
        .into_iter()
        .collect::<Float32Array>();

    let schema = Schema::new(vec![
        Field::new("name", DataType::Utf8, true),
        Field::new("description", DataType::Utf8, true),
        Field::new("score", DataType::Float32, true),
        Field::new("skill_name", DataType::Utf8, true),
        Field::new("tool_name", DataType::Utf8, true),
        Field::new("file_path", DataType::Utf8, true),
        Field::new(
            "routing_keywords",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            true,
        ),
        Field::new(
            "intents",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            true,
        ),
        Field::new("category", DataType::Utf8, true),
        Field::new("metadata", DataType::Utf8, true),
        Field::new("vector_score", DataType::Float32, true),
        Field::new("keyword_score", DataType::Float32, true),
        Field::new("final_score", DataType::Float32, true),
        Field::new("confidence", DataType::Utf8, true),
        Field::new("ranking_reason", DataType::Utf8, true),
        Field::new("input_schema_digest", DataType::Utf8, true),
    ]);

    RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(StringArray::from(data.names.clone())),
            Arc::new(StringArray::from(data.descriptions.clone())),
            Arc::new(Float32Array::from(data.scores.clone())),
            Arc::new(StringArray::from(data.skill_names.clone())),
            Arc::new(StringArray::from(data.tool_names.clone())),
            Arc::new(StringArray::from(data.file_paths.clone())),
            data.routing_keywords_array.clone(),
            data.intents_array.clone(),
            Arc::new(StringArray::from(data.categories.clone())),
            Arc::new(StringArray::from(data.metadata_json.clone())),
            Arc::new(vector_score_array),
            Arc::new(keyword_score_array),
            Arc::new(Float32Array::from(data.final_scores.clone())),
            Arc::new(StringArray::from(data.confidences.clone())),
            Arc::new(StringArray::from(data.ranking_reasons.clone())),
            Arc::new(StringArray::from(data.input_schema_digests.clone())),
        ],
    )
    .map_err(|e| e.to_string())
}

/// Encode tool search results as Arrow IPC stream bytes (single `RecordBatch`).
/// Schema: name, description, score, `skill_name`, `tool_name`, `file_path`,
/// `routing_keywords` (List<Utf8>), intents (List<Utf8>), category, metadata (Utf8 JSON),
/// `vector_score`, `keyword_score`, `final_score`, `confidence`, `ranking_reason`,
/// `input_schema_digest`.
/// Python `ToolSearchPayload.from_arrow_table` consumes this canonical contract directly.
pub(super) fn tool_search_results_to_ipc(
    results: &[crate::skill::ToolSearchResult],
) -> Result<Vec<u8>, String> {
    if results.is_empty() {
        let batch = empty_tool_search_ipc_batch()?;
        return record_batch_to_ipc_bytes(&batch);
    }
    let data = collect_tool_ipc_data(results);
    let batch = build_tool_search_ipc_batch(&data)?;
    record_batch_to_ipc_bytes(&batch)
}
