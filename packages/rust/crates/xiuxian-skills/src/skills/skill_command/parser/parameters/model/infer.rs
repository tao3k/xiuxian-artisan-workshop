pub(super) fn infer_json_type(type_annotation: Option<&str>) -> serde_json::Value {
    let Some(type_str) = type_annotation.map(str::trim) else {
        return serde_json::json!("string");
    };

    if type_str.contains("Optional") || type_str.contains("| None") {
        return infer_base_json_type(type_str);
    }

    if type_str.starts_with("Literal")
        && let Some(values) = parse_literal_values(type_str)
    {
        return serde_json::json!({
            "type": "string",
            "enum": values,
        });
    }

    infer_base_json_type(type_str)
}

fn infer_base_json_type(type_str: &str) -> serde_json::Value {
    if type_str.starts_with("list[") || type_str.starts_with("List[") {
        let inner_type = infer_list_inner_type(type_str);
        return serde_json::json!({
            "type": "array",
            "items": { "type": inner_type },
        });
    }

    if type_str.starts_with("dict[") || type_str.starts_with("Dict[") {
        return serde_json::json!({
            "type": "object",
            "additionalProperties": true,
        });
    }

    match type_str.to_lowercase().as_str() {
        "int" | "integer" => serde_json::json!("integer"),
        "float" | "number" => serde_json::json!("number"),
        "bool" | "boolean" => serde_json::json!("boolean"),
        _ => serde_json::json!("string"),
    }
}

fn infer_list_inner_type(type_str: &str) -> &'static str {
    let Some(start) = type_str.find('[') else {
        return "string";
    };
    let Some(end) = type_str.rfind(']') else {
        return "string";
    };

    match type_str[start + 1..end].trim() {
        "int" | "integer" => "integer",
        "float" | "number" => "number",
        "bool" | "boolean" => "boolean",
        _ => "string",
    }
}

fn parse_literal_values(type_str: &str) -> Option<Vec<serde_json::Value>> {
    let start = type_str.find('[')?;
    let end = type_str.rfind(']')?;
    let values_str = &type_str[start + 1..end];

    let values = values_str
        .split(',')
        .map(|value| {
            let value = value.trim().trim_matches(|c| c == '"' || c == '\'');
            serde_json::json!(value)
        })
        .collect();
    Some(values)
}
