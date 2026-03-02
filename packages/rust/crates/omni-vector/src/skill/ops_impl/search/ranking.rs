/// Parse `skill_name = 'value'` from a `where_filter` string for Rust-side filtering
/// (Lance filter on dictionary columns can return no rows).
fn parse_skill_name_from_where_filter(where_filter: &str) -> Option<String> {
    let prefix = "skill_name = '";
    let f = where_filter.trim();
    if !f.starts_with(prefix) {
        return None;
    }
    let rest = f.get(prefix.len()..)?;
    let mut end = 0usize;
    let mut it = rest.char_indices();
    while let Some((i, c)) = it.next() {
        if c == '\'' {
            if rest.get(i + 1..)?.starts_with('\'') {
                it.next();
                end = i + 2;
                continue;
            }
            end = i;
            break;
        }
        end = i + c.len_utf8();
    }
    Some(rest[..end].replace("''", "'"))
}

fn normalize_query_terms(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '*' || c == '.' || c == '_'))
        .filter(|t| !t.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn query_has_file_discovery_intent(query_parts: &[String]) -> bool {
    query_parts.iter().any(|part| {
        matches!(
            part.as_str(),
            "find" | "list" | "file" | "files" | "directory" | "folder" | "path" | "glob"
        ) || part.starts_with("*.")
    })
}

fn canonical_tool_name_from_result_meta(meta: &serde_json::Value, row_id: &str) -> Option<String> {
    let skill_name = meta
        .get("skill_name")
        .and_then(|s| s.as_str())
        .map_or("", str::trim);
    let tool_name = meta
        .get("tool_name")
        .and_then(|s| s.as_str())
        .map_or("", str::trim);
    if skill::is_routable_tool_name(tool_name) && tool_name.contains('.') {
        return Some(tool_name.to_string());
    }
    if !skill_name.is_empty() && skill::is_routable_tool_name(tool_name) {
        let candidate = format!("{skill_name}.{tool_name}");
        if skill::is_routable_tool_name(&candidate) {
            return Some(candidate);
        }
    }

    let command = meta
        .get("command")
        .and_then(|s| s.as_str())
        .map_or("", str::trim);
    if !skill_name.is_empty() && !command.is_empty() {
        let candidate = format!("{skill_name}.{command}");
        if skill::is_routable_tool_name(&candidate) {
            return Some(candidate);
        }
    }

    if skill::is_routable_tool_name(command) {
        return Some(command.to_string());
    }
    if skill::is_routable_tool_name(row_id) {
        return Some(row_id.to_string());
    }
    None
}

fn tool_metadata_alignment_boost(tool: &skill::ToolSearchResult, query_parts: &[String]) -> f32 {
    if query_parts.is_empty() {
        return 0.0;
    }

    let mut boost = 0.0f32;
    let category = tool.category.to_lowercase();
    let description = tool.description.to_lowercase();

    for term in query_parts {
        if term.len() <= 2 {
            continue;
        }
        if !category.is_empty() && category.contains(term) {
            boost += 0.05;
        }
        if description.contains(term) {
            boost += 0.03;
        }
        if tool
            .routing_keywords
            .iter()
            .any(|k| k.to_lowercase().contains(term))
        {
            boost += 0.07;
        }
        if tool.intents.iter().any(|i| i.to_lowercase().contains(term)) {
            boost += 0.08;
        }
    }

    boost.min(0.50)
}

fn tool_file_discovery_match(tool: &skill::ToolSearchResult) -> bool {
    let tool_name = tool.tool_name.to_lowercase();
    if tool_name == "advanced_tools.smart_find" {
        return true;
    }

    let category = tool.category.to_lowercase();
    let description = tool.description.to_lowercase();
    let terms = [
        "find",
        "file",
        "files",
        "directory",
        "folder",
        "path",
        "glob",
    ];
    terms.iter().any(|t| {
        category.contains(t)
            || description.contains(t)
            || tool
                .routing_keywords
                .iter()
                .any(|k| k.to_lowercase().contains(t))
            || tool.intents.iter().any(|i| i.to_lowercase().contains(t))
    })
}
