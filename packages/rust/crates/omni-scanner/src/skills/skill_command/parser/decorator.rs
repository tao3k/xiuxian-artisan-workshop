use crate::skills::metadata::DecoratorArgs;

/// Find all @`skill_command` decorator positions in Python code.
///
/// Uses simple string matching (not regex) to find decorators.
/// Returns `Vec` of (`start_pos`, `end_pos`, `full_decorator_text`).
#[must_use]
pub fn find_skill_command_decorators(content: &str) -> Vec<(usize, usize, String)> {
    let mut decorators = Vec::new();
    let prefix = "@skill_command";

    // Find all occurrences of @skill_command
    let mut search_start = 0usize;
    while let Some(start) = content[search_start..].find(prefix) {
        let absolute_start = search_start + start;
        let line_start = content[..absolute_start].rfind('\n').map_or(0, |p| p + 1);

        // Find the end of the decorator (matching closing paren or end of line)
        let after_decorator = &content[absolute_start + prefix.len()..];

        // Check if this is a valid decorator call with parentheses
        if after_decorator.starts_with('(') {
            let mut depth = 1;
            let mut end_offset = None;

            let mut in_string = false;
            let mut quote_char = '\0';
            let mut in_triple_quote = false;

            let paren_start = absolute_start + prefix.len();
            let search_content = &content[paren_start + 1..];

            let mut i = 0;
            let chars: Vec<char> = search_content.chars().collect();

            while i < chars.len() {
                let c = chars[i];

                if in_triple_quote {
                    if c == quote_char
                        && i + 2 < chars.len()
                        && chars[i + 1] == quote_char
                        && chars[i + 2] == quote_char
                    {
                        in_triple_quote = false;
                        i += 2;
                    }
                } else if in_string {
                    if c == quote_char && (i == 0 || chars[i - 1] != '\\') {
                        in_string = false;
                    }
                } else {
                    // Check for triple quotes
                    if i + 2 < chars.len()
                        && ((chars[i] == '"' && chars[i + 1] == '"' && chars[i + 2] == '"')
                            || (chars[i] == '\'' && chars[i + 1] == '\'' && chars[i + 2] == '\''))
                    {
                        in_triple_quote = true;
                        quote_char = chars[i];
                        i += 2;
                        i += 1;
                        continue;
                    }

                    if c == '"' || c == '\'' {
                        in_string = true;
                        quote_char = c;
                    } else if c == '(' {
                        depth += 1;
                    } else if c == ')' {
                        depth -= 1;
                        if depth == 0 {
                            end_offset = Some(paren_start + 1 + i + 1);
                            break;
                        }
                    }
                }
                i += 1;
            }

            if let Some(end_pos) = end_offset {
                // Extract the full decorator text including @ sign
                let full_text = &content[line_start..end_pos];
                decorators.push((line_start, end_pos, full_text.to_string()));
                search_start = end_pos;
            } else {
                search_start = absolute_start + prefix.len();
            }
        } else {
            // Invalid format, skip
            search_start = absolute_start + prefix.len();
        }
    }

    decorators
}

/// Parse decorator arguments from decorator text handling triple-quoted strings.
#[must_use]
pub fn parse_decorator_args(decorator_text: &str) -> DecoratorArgs {
    let mut args = DecoratorArgs::default();

    // Extract the arguments from within the parentheses
    // Need to handle triple-quoted strings that may contain )
    if let Some(open_paren) = decorator_text.find('(') {
        let arg_text = extract_decorator_arg_text(decorator_text, open_paren + 1);

        // Parse key=value pairs with proper triple-quoted string handling
        let parts = split_args_respecting_strings(&arg_text);

        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            // Parse key=value
            if let Some(eq_pos) = part.find('=') {
                let key = &part[..eq_pos].trim();
                let value = &part[eq_pos + 1..].trim();

                match *key {
                    "name" => {
                        let cleaned = extract_string_value(value);
                        args.name = Some(cleaned.to_string());
                    }
                    "description" => {
                        let cleaned = extract_string_value(value);
                        args.description = Some(cleaned.to_string());
                    }
                    "category" => {
                        let cleaned = extract_string_value(value);
                        args.category = Some(cleaned.to_string());
                    }
                    "destructive" => {
                        args.destructive = Some(value.trim().eq_ignore_ascii_case("True"));
                    }
                    "read_only" => {
                        args.read_only = Some(value.trim().eq_ignore_ascii_case("True"));
                    }
                    "resource_uri" => {
                        let cleaned = extract_string_value(value);
                        args.resource_uri = Some(cleaned.to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    args
}

/// Extract text between parentheses, respecting triple-quoted strings.
fn extract_decorator_arg_text(text: &str, start: usize) -> String {
    let mut depth = 1;
    let mut in_triple_quote = false;
    let mut triple_quote_char = '\0';
    let mut result = String::new();

    for (i, c) in text[start..].char_indices() {
        let absolute_pos = start + i;

        if in_triple_quote {
            if c == triple_quote_char {
                // Check for closing triple quote (3 consecutive quotes)
                let remaining = &text[absolute_pos..];
                if remaining.starts_with("\"\"\"") || remaining.starts_with("'''") {
                    // Include the closing triple quote and exit triple quote mode
                    result.push_str(&text[start + i..start + i + 3]);
                    in_triple_quote = false;
                    // Don't return yet! Continue to find the closing paren
                    continue;
                }
            }
            result.push(c);
        } else {
            // Check for start of triple quote
            let remaining = &text[absolute_pos..];
            if remaining.starts_with("\"\"\"") {
                in_triple_quote = true;
                triple_quote_char = '"';
                result.push_str("\"\"\"");
            } else if c == '(' {
                depth += 1;
                result.push(c);
            } else if c == ')' {
                depth -= 1;
                if depth == 0 {
                    return result;
                }
                result.push(c);
            } else {
                result.push(c);
            }
        }
    }

    result
}

/// Split argument text on commas, but respect triple-quoted strings.
fn split_args_respecting_strings(arg_text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut current_start = 0usize;
    let mut in_triple_quote = false;
    let mut triple_quote_char = '\0';
    let mut skip_next_chars = 0usize;

    for (i, c) in arg_text.char_indices() {
        // Skip characters if needed (after detecting triple quote end)
        if skip_next_chars > 0 {
            skip_next_chars -= 1;
            continue;
        }

        if in_triple_quote {
            // Check if we're at the end of triple quote (3 consecutive quotes)
            if c == triple_quote_char {
                let remaining = &arg_text[i..];
                if remaining.starts_with("\"\"\"") || remaining.starts_with("'''") {
                    in_triple_quote = false;
                    skip_next_chars = 2; // Skip the next 2 quote characters
                }
            }
        } else {
            // Check for start of triple quote
            let remaining = &arg_text[i..];
            if remaining.starts_with("\"\"\"") {
                in_triple_quote = true;
                triple_quote_char = '"';
                skip_next_chars = 2; // Skip the next 2 quote characters
            } else if remaining.starts_with("'''") {
                in_triple_quote = true;
                triple_quote_char = '\'';
                skip_next_chars = 2; // Skip the next 2 quote characters
            } else if c == ',' {
                // Split at comma
                result.push(&arg_text[current_start..i]);
                current_start = i + 1;
            }
        }
    }

    // Add the last part
    if current_start < arg_text.len() {
        result.push(&arg_text[current_start..]);
    }

    result
}

/// Extract value from a string literal (handles triple-quoted strings).
fn extract_string_value(value: &str) -> &str {
    // Check for triple-quoted strings
    if let Some(stripped) = value.strip_prefix("\"\"\"") {
        if let Some(end) = stripped.find("\"\"\"") {
            return &stripped[..end];
        }
    } else if let Some(stripped) = value.strip_prefix("'''") {
        if let Some(end) = stripped.find("'''") {
            return &stripped[..end];
        }
    } else if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        // Single-quoted string
        return &value[1..value.len() - 1];
    }
    // Not a string or unclosed string, return as-is
    value
}
