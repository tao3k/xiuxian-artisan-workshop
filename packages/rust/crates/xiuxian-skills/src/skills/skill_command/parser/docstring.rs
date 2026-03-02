use std::collections::HashMap;

/// Extract docstring from matched function text.
#[must_use]
pub fn extract_docstring_from_text(text: &str) -> String {
    if let Some(start) = text.find("\"\"\"")
        && let Some(end) = text[start + 3..].find("\"\"\"")
    {
        let doc = &text[start + 3..start + 3 + end];
        return doc.trim().to_string();
    }
    if let Some(start) = text.find("'''")
        && let Some(end) = text[start + 3..].find("'''")
    {
        let doc = &text[start + 3..start + 3 + end];
        return doc.trim().to_string();
    }
    String::new()
}

/// Extract parameter descriptions from decorator description text.
///
/// Supports Google-style docstring format:
/// ```python
/// """
/// Tool description.
///
/// Args:
///     - query: str - The search query (required)
///     - limit: int - Maximum number of results
///
/// Returns:
///     List of results
/// """
/// ```
///
/// Returns a `HashMap` mapping parameter name to its description.
#[must_use]
pub fn extract_param_descriptions(description: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    // Find the Args section
    let Some(args_start) = description.find("Args:") else {
        return result;
    };

    let args_section = &description[args_start..];

    // Parse each parameter line: "- param_name: type - description"
    // or "- param_name: description"
    for line in args_section.lines() {
        let trimmed = line.trim();

        // Check if line starts with "- " or "• " (bullet point)
        if !trimmed.starts_with("- ") && !trimmed.starts_with("• ") {
            continue;
        }

        // Remove bullet point prefix ("- " or "• ")
        let content = &trimmed[2..];

        // Find the first colon to separate param name from description
        if let Some(colon_pos) = content.find(':') {
            let param_name = content[..colon_pos].trim();

            // Skip if param_name looks like a type (starts with capital or contains space)
            if param_name.is_empty()
                || param_name.contains(' ')
                || param_name.chars().next().is_some_and(char::is_uppercase)
            {
                continue;
            }

            // Get everything after the colon
            let after_colon = &content[colon_pos + 1..].trim();

            // Extract description after type and separator (- or —)
            let description = if let Some(sep_pos) = after_colon.find(['-', '—', '–']) {
                let desc = &after_colon[sep_pos + 1..].trim();
                // Clean up trailing newlines and whitespace
                desc.trim().to_string()
            } else {
                after_colon.to_string()
            };

            if !description.is_empty() {
                result.insert(param_name.to_string(), description);
            }
        }
    }

    result
}
