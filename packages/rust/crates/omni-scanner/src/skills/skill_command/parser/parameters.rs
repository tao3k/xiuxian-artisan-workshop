/// Parse parameter names from function signature string.
#[must_use]
pub fn parse_parameters(params_text: &str) -> Vec<String> {
    split_parameters(params_text)
        .iter()
        .filter_map(|s| {
            // Handle "param: Type" -> "param"
            let clean = if let Some(colon_pos) = s.find(':') {
                &s[..colon_pos]
            } else {
                s
            };
            // Handle "param=default" -> "param"
            let clean = clean.split('=').next().unwrap_or(clean);
            let clean = clean.trim();

            // Skip *args and **kwargs
            if clean.starts_with('*') && clean != "*" {
                None
            } else if !clean.is_empty() {
                Some(clean.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Split parameters respecting nested brackets and `|` unions in type annotations.
///
/// This handles cases like `dict[str, Any] | None` by not splitting on commas inside
/// type annotations with brackets.
fn split_parameters(params_text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;

    for c in params_text.chars() {
        if c == '(' {
            depth += 1;
            current.push(c);
        } else if c == ')' {
            depth = depth.saturating_sub(1);
            current.push(c);
        } else if c == '[' {
            depth += 1;
            current.push(c);
        } else if c == ']' {
            depth = depth.saturating_sub(1);
            current.push(c);
        } else if c == ',' && depth == 0 {
            // Only split on comma if not inside brackets
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                result.push(trimmed);
            }
            current.clear();
        } else {
            current.push(c);
        }
    }

    // Don't forget the last parameter
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        result.push(trimmed);
    }

    result
}

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
        let Some(ref type_str) = self.type_annotation else {
            return serde_json::json!("string");
        };

        let type_str = type_str.trim();

        // Handle Optional/Union with None
        if type_str.contains("Optional") || type_str.contains("| None") {
            // Return minimal schema for optional types
            return self.infer_base_json_type();
        }

        // Handle Literal types for enums
        if type_str.starts_with("Literal")
            && let Some(start) = type_str.find('[')
            && let Some(end) = type_str.rfind(']')
        {
            let values_str = &type_str[start + 1..end];
            let values: Vec<serde_json::Value> = values_str
                .split(',')
                .map(|v| {
                    let v = v.trim().trim_matches(|c| c == '"' || c == '\'');
                    serde_json::json!(v)
                })
                .collect();
            return serde_json::json!({
                "type": "string",
                "enum": values
            });
        }

        self.infer_base_json_type()
    }

    /// Infer JSON Schema type for base types (without Optional/Union wrapper).
    fn infer_base_json_type(&self) -> serde_json::Value {
        let type_str = self.type_annotation.as_deref().unwrap_or("");

        // Handle generic types like list[str], dict[str, int], etc.
        if type_str.starts_with("list[") || type_str.starts_with("List[") {
            let inner = if let Some(start) = type_str.find('[') {
                if let Some(end) = type_str.rfind(']') {
                    &type_str[start + 1..end]
                } else {
                    "string"
                }
            } else {
                "string"
            };

            let inner_type = match inner.trim() {
                "int" | "integer" => "integer",
                "float" | "number" => "number",
                "bool" | "boolean" => "boolean",
                _ => "string",
            };

            return serde_json::json!({
                "type": "array",
                "items": { "type": inner_type }
            });
        }

        // Handle dict types like dict[str, str], Dict[str, int], etc.
        if type_str.starts_with("dict[") || type_str.starts_with("Dict[") {
            return serde_json::json!({
                "type": "object",
                "additionalProperties": true
            });
        }

        // Handle basic types
        let normalized = type_str.to_lowercase();
        match normalized.as_str() {
            "int" | "integer" => serde_json::json!("integer"),
            "float" | "number" => serde_json::json!("number"),
            "bool" | "boolean" => serde_json::json!("boolean"),
            _ => serde_json::json!("string"),
        }
    }

    /// Generate JSON Schema property for this parameter.
    #[must_use]
    pub fn to_json_schema_property(&self) -> serde_json::Value {
        let mut schema = serde_json::Map::new();

        // Add type
        let json_type = self.infer_json_type();
        schema.insert("type".to_string(), json_type);

        // Add description from docstring if available
        // (Description is added separately in generate_input_schema)

        // Add default value if present
        if let Some(ref default) = self.default_value {
            // Only add default if it's not None
            if default != "None" {
                schema.insert("default".to_string(), serde_json::json!(default));
            }
        }

        serde_json::Value::Object(schema)
    }
}

/// Extract parameter names from full function signature text.
#[must_use]
pub fn extract_parameters_from_text(func_text: &str) -> Vec<String> {
    // Find the parameter list between parentheses, handling nested parentheses
    if let Some(open_paren) = func_text.find('(') {
        let mut depth = 1;
        let mut close_paren = None;
        let search_content = &func_text[open_paren + 1..];

        for (i, c) in search_content.char_indices() {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth == 0 {
                    close_paren = Some(open_paren + 1 + i);
                    break;
                }
            }
        }

        if let Some(end_pos) = close_paren {
            let params_text = &func_text[open_paren + 1..end_pos];
            return parse_parameters(params_text);
        }
    }
    Vec::new()
}

/// Extract detailed parameter information from function signature text.
///
/// Returns a vector of `ParsedParameter` with name, type, default info.
#[must_use]
pub fn extract_parsed_parameters(func_text: &str) -> Vec<ParsedParameter> {
    // Find the parameter list between parentheses, handling nested parentheses
    if let Some(open_paren) = func_text.find('(') {
        let mut depth = 1;
        let mut close_paren = None;
        let search_content = &func_text[open_paren + 1..];

        for (i, c) in search_content.char_indices() {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth == 0 {
                    close_paren = Some(open_paren + 1 + i);
                    break;
                }
            }
        }

        if let Some(end_pos) = close_paren {
            let params_text = &func_text[open_paren + 1..end_pos];
            return parse_detailed_parameters(params_text);
        }
    }
    Vec::new()
}

/// Parse detailed parameter information from parameter text.
fn parse_detailed_parameters(params_text: &str) -> Vec<ParsedParameter> {
    split_parameters(params_text)
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(|s| {
            // Parse "param: Type = default" format

            // Check for *args and **kwargs
            if s.starts_with('*') && s != "*" {
                return None;
            }

            // Split on default value "="
            let (before_eq, default_str) = if let Some(eq_pos) = s.find('=') {
                (&s[..eq_pos], Some(s[eq_pos + 1..].trim().to_string()))
            } else {
                (s, None)
            };

            // Split on type annotation ":"
            let (name, type_str) = if let Some(colon_pos) = before_eq.find(':') {
                let name_part = before_eq[..colon_pos].trim();
                let type_part = before_eq[colon_pos + 1..].trim().to_string();
                (name_part.to_string(), Some(type_part))
            } else {
                (before_eq.trim().to_string(), None)
            };

            let has_default = default_str.is_some();

            if name.is_empty() {
                None
            } else {
                Some(ParsedParameter {
                    name,
                    type_annotation: type_str,
                    has_default,
                    default_value: default_str,
                })
            }
        })
        .filter(|p| !p.name.starts_with('*'))
        .collect()
}
