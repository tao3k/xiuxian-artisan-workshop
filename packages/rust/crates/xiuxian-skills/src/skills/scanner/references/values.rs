/// Derive unique skill names from full tool names.
/// Example: `"researcher.run_research_graph"` -> `"researcher"`.
pub(super) fn skills_from_tool_list(tools: &[String]) -> Vec<String> {
    let mut skills: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.split('.').next().map(String::from))
        .filter(|skill| !skill.is_empty())
        .collect();
    skills.sort();
    skills.dedup();
    skills
}

fn yaml_value_to_string_vec(value: &serde_yaml::Value) -> Vec<String> {
    match value {
        serde_yaml::Value::String(text) => {
            if text.is_empty() {
                Vec::new()
            } else {
                vec![text.clone()]
            }
        }
        serde_yaml::Value::Sequence(sequence) => sequence
            .iter()
            .filter_map(|item| item.as_str().map(String::from))
            .filter(|item| !item.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

pub(super) fn yaml_value_to_opt_string_vec(value: &serde_yaml::Value) -> Option<Vec<String>> {
    let entries = yaml_value_to_string_vec(value);
    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}
