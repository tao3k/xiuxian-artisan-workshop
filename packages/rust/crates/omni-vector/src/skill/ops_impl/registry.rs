use arrow::array::Array as _;

impl VectorStore {
    /// Load the tool registry from a table.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the underlying tool table fails.
    pub async fn load_tool_registry(
        &self,
        table_name: &str,
    ) -> Result<Vec<skill::ToolSearchResult>, VectorStoreError> {
        // ... (existing implementation)
        self.get_tools_by_skill_internal(table_name, None).await
    }

    /// Get all tools belonging to a specific skill.
    ///
    /// # Errors
    ///
    /// Returns an error if table scanning/filtering fails.
    pub async fn get_tools_by_skill(
        &self,
        skill_name: &str,
    ) -> Result<Vec<skill::ToolSearchResult>, VectorStoreError> {
        self.get_tools_by_skill_internal("tools", Some(skill_name))
            .await
    }

    async fn get_tools_by_skill_internal(
        &self,
        table_name: &str,
        skill_filter: Option<&str>,
    ) -> Result<Vec<skill::ToolSearchResult>, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Ok(Vec::new());
        }
        let dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let schema = dataset.schema();
        let has_metadata = schema.field(METADATA_COLUMN).is_some();
        let project_cols = get_tools_by_skill_projection(has_metadata);
        let mut scanner = dataset.scan();
        scanner.project(&project_cols)?;

        if let Some(skill) = skill_filter {
            scanner.filter(&format!("skill_name = '{skill}'"))?;
        }

        let mut stream = scanner.try_into_stream().await?;
        let mut tools = Vec::new();
        while let Some(batch) = stream.try_next().await? {
            append_get_tools_by_skill_rows_from_batch(&batch, skill_filter, &mut tools);
        }
        Ok(tools)
    }
}

type SkillDynArrayRef<'a> = Option<&'a std::sync::Arc<dyn lance::deps::arrow_array::Array>>;
type SkillStringArrayRef<'a> = Option<&'a lance::deps::arrow_array::StringArray>;

struct GetToolsBySkillBatchColumns<'a> {
    content: &'a lance::deps::arrow_array::StringArray,
    metadata: SkillStringArrayRef<'a>,
    skill_name: SkillDynArrayRef<'a>,
    tool_name: SkillDynArrayRef<'a>,
    file_path: SkillDynArrayRef<'a>,
    routing_keywords: SkillDynArrayRef<'a>,
    intents: SkillDynArrayRef<'a>,
    category: SkillDynArrayRef<'a>,
}

struct ResolvedToolSkillRow {
    name: String,
    skill_name: String,
    tool_name: String,
    file_path: String,
    routing_keywords: Vec<String>,
    intents: Vec<String>,
    category: String,
    input_schema: serde_json::Value,
}

fn get_tools_by_skill_projection(has_metadata: bool) -> Vec<&'static str> {
    if has_metadata {
        vec![
            METADATA_COLUMN,
            CONTENT_COLUMN,
            crate::SKILL_NAME_COLUMN,
            crate::TOOL_NAME_COLUMN,
            crate::FILE_PATH_COLUMN,
            crate::ROUTING_KEYWORDS_COLUMN,
            crate::INTENTS_COLUMN,
            crate::CATEGORY_COLUMN,
        ]
    } else {
        vec![
            CONTENT_COLUMN,
            crate::SKILL_NAME_COLUMN,
            crate::TOOL_NAME_COLUMN,
            crate::FILE_PATH_COLUMN,
            crate::ROUTING_KEYWORDS_COLUMN,
            crate::INTENTS_COLUMN,
            crate::CATEGORY_COLUMN,
        ]
    }
}

fn skill_utf8_at(col: SkillDynArrayRef<'_>, idx: usize) -> String {
    col.map(|c| crate::ops::get_utf8_at(c.as_ref(), idx))
        .unwrap_or_default()
}

fn skill_routing_keywords_at(col: SkillDynArrayRef<'_>, idx: usize) -> Vec<String> {
    col.map(|c| crate::ops::get_routing_keywords_at(c.as_ref(), idx))
        .unwrap_or_default()
}

fn skill_intents_at(col: SkillDynArrayRef<'_>, idx: usize) -> Vec<String> {
    col.map(|c| crate::ops::get_intents_at(c.as_ref(), idx))
        .unwrap_or_default()
}

fn routing_keywords_and_intents_json(
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

fn resolve_tool_skill_row_from_columns(
    row_index: usize,
    skill_name: &str,
    category: &str,
    columns: &GetToolsBySkillBatchColumns<'_>,
) -> ResolvedToolSkillRow {
    let tool_name = skill_utf8_at(columns.tool_name, row_index);
    let routing_keywords_raw = skill_routing_keywords_at(columns.routing_keywords, row_index);
    let intents_raw = skill_intents_at(columns.intents, row_index);
    let meta_json = routing_keywords_and_intents_json(&routing_keywords_raw, &intents_raw);

    ResolvedToolSkillRow {
        name: tool_name.clone(),
        skill_name: skill_name.to_string(),
        tool_name,
        file_path: skill_utf8_at(columns.file_path, row_index),
        routing_keywords: skill::resolve_routing_keywords(&meta_json),
        intents: skill::resolve_intents(&meta_json),
        category: category.to_string(),
        input_schema: serde_json::json!({}),
    }
}

fn resolve_tool_skill_row_from_metadata(
    metadata: &serde_json::Value,
    skill_filter: Option<&str>,
) -> Option<ResolvedToolSkillRow> {
    if metadata.get("type").and_then(|value| value.as_str()) != Some("command") {
        return None;
    }
    if let Some(skill) = skill_filter
        && metadata.get("skill_name").and_then(|value| value.as_str()) != Some(skill)
    {
        return None;
    }

    Some(ResolvedToolSkillRow {
        name: metadata
            .get("command")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
        skill_name: metadata
            .get("skill_name")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
        tool_name: metadata
            .get("tool_name")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
        file_path: metadata
            .get("file_path")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
        routing_keywords: skill::resolve_routing_keywords(metadata),
        intents: skill::resolve_intents(metadata),
        category: metadata
            .get("category")
            .and_then(|value| value.as_str())
            .or_else(|| metadata.get("skill_name").and_then(|value| value.as_str()))
            .unwrap_or("")
            .to_string(),
        input_schema: metadata.get("input_schema").map_or_else(
            || serde_json::json!({}),
            skill::normalize_input_schema_value,
        ),
    })
}

fn resolve_tool_skill_row(
    row_index: usize,
    skill_filter: Option<&str>,
    columns: &GetToolsBySkillBatchColumns<'_>,
) -> Option<ResolvedToolSkillRow> {
    let skill_name = skill_utf8_at(columns.skill_name, row_index);
    let category = skill_utf8_at(columns.category, row_index);

    if let Some(metadata_arr) = columns.metadata {
        if metadata_arr.is_null(row_index) {
            return Some(resolve_tool_skill_row_from_columns(
                row_index,
                &skill_name,
                &category,
                columns,
            ));
        }
        let metadata =
            serde_json::from_str::<serde_json::Value>(metadata_arr.value(row_index)).ok()?;
        return resolve_tool_skill_row_from_metadata(&metadata, skill_filter);
    }

    if let Some(skill) = skill_filter
        && skill_name != skill
    {
        return None;
    }
    Some(resolve_tool_skill_row_from_columns(
        row_index,
        &skill_name,
        &category,
        columns,
    ))
}

fn append_get_tools_by_skill_rows_from_batch(
    batch: &lance::deps::arrow_array::RecordBatch,
    skill_filter: Option<&str>,
    tools: &mut Vec<skill::ToolSearchResult>,
) {
    use lance::deps::arrow_array::Array;

    let content = batch.column_by_name(CONTENT_COLUMN).and_then(|column| {
        column
            .as_any()
            .downcast_ref::<lance::deps::arrow_array::StringArray>()
    });
    let Some(content) = content else {
        return;
    };

    let metadata = batch.column_by_name(METADATA_COLUMN).and_then(|column| {
        column
            .as_any()
            .downcast_ref::<lance::deps::arrow_array::StringArray>()
    });
    let columns = GetToolsBySkillBatchColumns {
        content,
        metadata,
        skill_name: batch.column_by_name(crate::SKILL_NAME_COLUMN),
        tool_name: batch.column_by_name(crate::TOOL_NAME_COLUMN),
        file_path: batch.column_by_name(crate::FILE_PATH_COLUMN),
        routing_keywords: batch.column_by_name(crate::ROUTING_KEYWORDS_COLUMN),
        intents: batch.column_by_name(crate::INTENTS_COLUMN),
        category: batch.column_by_name(crate::CATEGORY_COLUMN),
    };

    for row_index in 0..batch.num_rows() {
        let Some(resolved) = resolve_tool_skill_row(row_index, skill_filter, &columns) else {
            continue;
        };
        tools.push(skill::ToolSearchResult {
            name: resolved.name,
            description: columns.content.value(row_index).to_string(),
            input_schema: resolved.input_schema,
            score: 1.0,
            vector_score: None,
            keyword_score: None,
            skill_name: resolved.skill_name,
            tool_name: resolved.tool_name,
            file_path: resolved.file_path,
            routing_keywords: resolved.routing_keywords,
            intents: resolved.intents,
            category: resolved.category,
            parameters: vec![],
        });
    }
}
