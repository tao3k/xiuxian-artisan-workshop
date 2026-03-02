impl VectorStore {
    /// Infer (`skill_name`, `tool_name`) from canonical id (e.g. "`knowledge.ingest_document`" -> ("knowledge", "`ingest_document`")).
    /// Ensures `list_all_tools` output always has valid `skill_name/tool_name` for discovery.
    fn infer_skill_tool_from_id(id: &str) -> (String, String) {
        let id = id.trim();
        if id.is_empty() {
            return (String::from("unknown"), String::from("unknown"));
        }
        if let Some(dot) = id.find('.') {
            let (skill, tool) = id.split_at(dot);
            let tool = tool.trim_start_matches('.');
            (
                skill.to_string(),
                if tool.is_empty() {
                    String::from("unknown")
                } else {
                    tool.to_string()
                },
            )
        } else {
            (id.to_string(), String::from("unknown"))
        }
    }

    /// List all tools in a specific table.
    /// When `source_filter` is set (e.g. "2601.03192.pdf"), applies predicate pushdown:
    /// `metadata LIKE '%{source}%'` so only matching rows are scanned (reduces I/O ~98% for `full_document`).
    /// Multiple source filters can be provided by joining terms with `||`.
    /// When `row_limit` is set, applies scanner limit to cap returned rows.
    ///
    /// # Errors
    ///
    /// Returns an error if the table cannot be opened/scanned, projection/filter/limit
    /// setup fails, stream iteration fails, or JSON serialization fails.
    pub async fn list_all_tools(
        &self,
        table_name: &str,
        source_filter: Option<&str>,
        row_limit: Option<usize>,
    ) -> Result<String, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Ok("[]".to_string());
        }
        let dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let mut scanner = dataset.scan();
        let arrow_schema = lance::deps::arrow_schema::Schema::from(dataset.schema());
        let has_metadata = arrow_schema.field_with_name("metadata").is_ok();
        let has_file_path = arrow_schema.field_with_name("file_path").is_ok();
        let source_filters = parse_source_filters(source_filter);
        let source_filter_active = !source_filters.is_empty();

        // Predicate pushdown for document-targeted fetch.
        // NOTE:
        // - For many document rows (e.g. knowledge_chunks), `file_path` column can be empty
        //   while metadata JSON still contains `source`/`file_path`.
        // - Therefore when both columns exist, use OR so metadata-backed rows are not dropped.
        if let Some(filter_expr) =
            build_source_filter_expr(&source_filters, has_file_path, has_metadata)
            && let Err(e) = scanner.filter(&filter_expr)
        {
            log::debug!("list_all_tools source_filter failed (fallback to full scan): {e}");
        }

        // For source-filtered document fetches we only need minimal columns.
        // Tool-index workflows (no source filter) keep the richer projection.
        let project_cols = list_all_tools_projection(source_filter_active, has_metadata);
        scanner.project(&project_cols)?;
        if let Some(limit) = normalize_row_limit(row_limit) {
            scanner.limit(Some(limit), None)?;
        }

        let mut stream = scanner.try_into_stream().await?;
        let mut tools = Vec::new();
        while let Some(batch) = stream.try_next().await? {
            append_list_all_tools_rows_from_batch(&batch, &mut tools);
        }
        // Deduplicate by (source, chunk_index) when metadata has chunk_index (e.g. knowledge_chunks)
        let tools = dedup_by_source_chunk_index(tools);
        serde_json::to_string(&tools).map_err(|e| VectorStoreError::General(e.to_string()))
    }
}

fn parse_source_filters(source_filter: Option<&str>) -> Vec<String> {
    source_filter
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|raw| {
            raw.split("||")
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn build_like_filter_expr(column: &str, source_filters: &[String]) -> String {
    let predicates = source_filters
        .iter()
        .map(|raw| {
            let escaped = raw.replace('\'', "''");
            format!("{column} LIKE '%{escaped}%'")
        })
        .collect::<Vec<_>>();
    if predicates.len() <= 1 {
        predicates.join("")
    } else {
        format!("({})", predicates.join(" OR "))
    }
}

fn build_source_filter_expr(
    source_filters: &[String],
    has_file_path: bool,
    has_metadata: bool,
) -> Option<String> {
    if source_filters.is_empty() {
        return None;
    }
    let filter_expr = match (has_file_path, has_metadata) {
        (true, true) => {
            let file_path_expr = build_like_filter_expr("file_path", source_filters);
            let metadata_expr = build_like_filter_expr("metadata", source_filters);
            format!("({file_path_expr} OR {metadata_expr})")
        }
        (true, false) => build_like_filter_expr("file_path", source_filters),
        (false, true) => build_like_filter_expr("metadata", source_filters),
        (false, false) => String::new(),
    };
    if filter_expr.is_empty() {
        None
    } else {
        Some(filter_expr)
    }
}

fn list_all_tools_projection(source_filter_active: bool, has_metadata: bool) -> Vec<&'static str> {
    if source_filter_active {
        if has_metadata {
            vec!["id", "content", "metadata"]
        } else {
            vec!["id", "content"]
        }
    } else if has_metadata {
        vec![
            "id",
            "content",
            "skill_name",
            "category",
            "tool_name",
            "file_path",
            "metadata",
        ]
    } else {
        vec![
            "id",
            "content",
            "skill_name",
            "category",
            "tool_name",
            "file_path",
        ]
    }
}

fn normalize_row_limit(row_limit: Option<usize>) -> Option<i64> {
    let limit = row_limit.filter(|value| *value > 0)?;
    if let Ok(value) = i64::try_from(limit) {
        Some(value)
    } else {
        Some(i64::MAX)
    }
}

struct ListAllToolsRowContext {
    id: String,
    content: String,
    skill_name: String,
    category: String,
    tool_name: String,
    file_path: String,
}

fn list_all_tools_metadata_fallback(row: &ListAllToolsRowContext) -> serde_json::Value {
    serde_json::json!({
        "id": row.id,
        "content": row.content,
        "skill_name": row.skill_name,
        "category": row.category,
        "tool_name": row.tool_name,
        "file_path": row.file_path,
    })
}

fn parse_list_all_tools_metadata(
    metadata_col: Option<&std::sync::Arc<dyn lance::deps::arrow_array::Array>>,
    row_index: usize,
    row: &ListAllToolsRowContext,
) -> serde_json::Value {
    use crate::ops::column_read::get_utf8_at;

    if let Some(meta_col) = metadata_col {
        let raw = get_utf8_at(meta_col.as_ref(), row_index);
        if raw.is_empty() {
            list_all_tools_metadata_fallback(row)
        } else {
            serde_json::from_str(&raw)
                .unwrap_or_else(|_| serde_json::json!({ "id": row.id, "content": row.content }))
        }
    } else {
        list_all_tools_metadata_fallback(row)
    }
}

fn merge_non_empty_tool_columns(
    metadata: &mut serde_json::Value,
    skill_name: &str,
    tool_name: &str,
    file_path: &str,
) {
    if let Some(obj) = metadata.as_object_mut() {
        if !skill_name.is_empty() {
            obj.insert("skill_name".to_string(), serde_json::json!(skill_name));
        }
        if !tool_name.is_empty() {
            obj.insert("tool_name".to_string(), serde_json::json!(tool_name));
        }
        if !file_path.is_empty() {
            obj.insert("file_path".to_string(), serde_json::json!(file_path));
        }
    }
}

fn ensure_skill_and_tool_in_metadata(metadata: &mut serde_json::Value, id: &str) {
    let Some(obj) = metadata.as_object_mut() else {
        return;
    };
    let skill_name = obj
        .get("skill_name")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();
    let tool_name = obj
        .get("tool_name")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();

    if skill_name.is_empty() || tool_name.is_empty() {
        let (inferred_skill, inferred_tool) = VectorStore::infer_skill_tool_from_id(id);
        if skill_name.is_empty() {
            obj.insert("skill_name".to_string(), serde_json::json!(inferred_skill));
        }
        if tool_name.is_empty() {
            obj.insert("tool_name".to_string(), serde_json::json!(inferred_tool));
        }
    }
}

fn append_list_all_tools_rows_from_batch(
    batch: &lance::deps::arrow_array::RecordBatch,
    tools: &mut Vec<serde_json::Value>,
) {
    use crate::ops::column_read::get_utf8_at;
    use lance::deps::arrow_array::Array;
    use lance::deps::arrow_array::StringArray;

    let id_col = batch.column_by_name("id");
    let content_col = batch.column_by_name("content");
    let skill_name_col = batch.column_by_name("skill_name");
    let category_col = batch.column_by_name("category");
    let tool_name_col = batch.column_by_name("tool_name");
    let file_path_col = batch.column_by_name("file_path");
    let metadata_col = batch.column_by_name("metadata");

    let (Some(ids), Some(contents)) = (id_col, content_col) else {
        return;
    };
    let id_arr = ids.as_any().downcast_ref::<StringArray>();
    let content_arr = contents.as_any().downcast_ref::<StringArray>();

    for row_index in 0..batch.num_rows() {
        let row = ListAllToolsRowContext {
            id: id_arr.map_or(String::new(), |arr| arr.value(row_index).to_string()),
            content: content_arr.map_or(String::new(), |arr| arr.value(row_index).to_string()),
            skill_name: skill_name_col.map_or(String::new(), |col| get_utf8_at(col, row_index)),
            category: category_col.map_or(String::new(), |col| get_utf8_at(col, row_index)),
            tool_name: tool_name_col.map_or(String::new(), |col| get_utf8_at(col, row_index)),
            file_path: file_path_col.map_or(String::new(), |col| get_utf8_at(col, row_index)),
        };

        let mut metadata = parse_list_all_tools_metadata(metadata_col, row_index, &row);
        merge_non_empty_tool_columns(
            &mut metadata,
            &row.skill_name,
            &row.tool_name,
            &row.file_path,
        );
        ensure_skill_and_tool_in_metadata(&mut metadata, &row.id);

        tools.push(serde_json::json!({
            "id": row.id,
            "content": row.content,
            "metadata": metadata
        }));
    }
}

/// Deduplicate `list_all_tools` rows by (`metadata.source`, `metadata.chunk_index`).
/// Only applied when at least one row has numeric `metadata.chunk_index` (e.g. `knowledge_chunks`).
/// Keeps first occurrence per (`source`, `chunk_index`) and sorts by (`source`, `chunk_index`).
fn dedup_by_source_chunk_index(tools: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
    use std::collections::HashSet;
    let has_chunk_index = tools.iter().any(|t| {
        t.get("metadata")
            .and_then(|m| m.get("chunk_index"))
            .and_then(serde_json::Value::as_i64)
            .is_some()
    });
    if !has_chunk_index {
        return tools;
    }
    let mut seen: HashSet<(String, i64)> = HashSet::new();
    let mut out: Vec<serde_json::Value> = Vec::new();
    for t in tools {
        let meta = t.get("metadata");
        let source = meta
            .and_then(|m| m.get("source"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        let idx = meta
            .and_then(|m| m.get("chunk_index"))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        if seen.insert((source.clone(), idx)) {
            out.push(t);
        }
    }
    out.sort_by(|a, b| {
        let (sa, ia) = (
            a.get("metadata")
                .and_then(|m| m.get("source"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            a.get("metadata")
                .and_then(|m| m.get("chunk_index"))
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0),
        );
        let (sb, ib) = (
            b.get("metadata")
                .and_then(|m| m.get("source"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            b.get("metadata")
                .and_then(|m| m.get("chunk_index"))
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0),
        );
        (sa, ia).cmp(&(sb, ib))
    });
    out
}
