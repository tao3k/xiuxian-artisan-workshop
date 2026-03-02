mod infer;

/// Represents a parsed parameter with its type annotation and default value info.
#[derive(Debug, Clone)]
pub struct ParsedParameter {
    /// Parameter name.
    pub name: String,
    /// Python type annotation (e.g., "str", "int", "list[str]").
    pub type_annotation: Option<String>,
    /// Whether the parameter has a default value.
    pub has_default: bool,
    /// Default value as a string (e.g., "10", "'default'", "None").
    pub default_value: Option<String>,
}

impl ParsedParameter {
    /// Check if this parameter is optional (has a default value).
    #[must_use]
    pub fn is_optional(&self) -> bool {
        self.has_default
    }

    /// Infer JSON Schema type from Python type annotation.
    #[must_use]
    pub fn infer_json_type(&self) -> serde_json::Value {
        infer::infer_json_type(self.type_annotation.as_deref())
    }

    /// Generate JSON Schema property for this parameter.
    #[must_use]
    pub fn to_json_schema_property(&self) -> serde_json::Value {
        let mut schema = serde_json::Map::new();
        let json_type = self.infer_json_type();
        schema.insert("type".to_string(), json_type);

        if let Some(ref default) = self.default_value
            && default != "None"
        {
            schema.insert("default".to_string(), serde_json::json!(default));
        }

        serde_json::Value::Object(schema)
    }
}
