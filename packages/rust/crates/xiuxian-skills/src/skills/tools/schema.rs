use crate::skills::skill_command::parser::{ParsedParameter, extract_param_descriptions};

pub(super) fn generate_input_schema(parameters: &[ParsedParameter], description: &str) -> String {
    let param_descriptions = extract_param_descriptions(description);

    if log::log_enabled!(log::Level::Debug) {
        log::debug!(
            "generate_input_schema: params={:?}, desc_len={}, param_descs={:?}",
            parameters.iter().map(|p| &p.name).collect::<Vec<_>>(),
            description.len(),
            param_descriptions
        );
    }

    let mut props = serde_json::Map::new();
    let mut required_params: Vec<String> = Vec::new();

    let param_debug_info: Vec<_> = if log::log_enabled!(log::Level::Debug) {
        parameters
            .iter()
            .map(|p| {
                if let Some(d) = param_descriptions.get(&p.name) {
                    format!("Found: {} = {}", p.name, d)
                } else {
                    format!("Fallback: {}", p.name)
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    for param in parameters {
        let desc = if let Some(d) = param_descriptions.get(&param.name) {
            d.clone()
        } else {
            format!("Parameter: {}", param.name)
        };

        let mut schema = param.to_json_schema_property();
        if let serde_json::Value::Object(ref mut schema_obj) = schema {
            schema_obj.insert("description".to_string(), serde_json::json!(desc));
        }

        props.insert(param.name.clone(), schema);

        if !param.has_default {
            required_params.push(param.name.clone());
        }
    }

    if !param_debug_info.is_empty() {
        log::debug!("Param processing: {param_debug_info:?}");
    }

    serde_json::json!({
        "type": "object",
        "properties": props,
        "required": required_params
    })
    .to_string()
}
