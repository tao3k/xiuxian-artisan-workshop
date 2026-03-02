/// Split argument text on commas, but respect triple-quoted strings.
pub(super) fn split_args_respecting_strings(arg_text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut current_start = 0usize;
    let mut in_triple_quote = false;
    let mut triple_quote_char = '\0';
    let mut skip_next_chars = 0usize;

    for (index, character) in arg_text.char_indices() {
        if skip_next_chars > 0 {
            skip_next_chars -= 1;
            continue;
        }

        if in_triple_quote {
            if character == triple_quote_char {
                let remaining = &arg_text[index..];
                if remaining.starts_with("\"\"\"") || remaining.starts_with("'''") {
                    in_triple_quote = false;
                    skip_next_chars = 2;
                }
            }
        } else {
            let remaining = &arg_text[index..];
            if remaining.starts_with("\"\"\"") {
                in_triple_quote = true;
                triple_quote_char = '"';
                skip_next_chars = 2;
            } else if remaining.starts_with("'''") {
                in_triple_quote = true;
                triple_quote_char = '\'';
                skip_next_chars = 2;
            } else if character == ',' {
                result.push(&arg_text[current_start..index]);
                current_start = index + 1;
            }
        }
    }

    if current_start < arg_text.len() {
        result.push(&arg_text[current_start..]);
    }

    result
}

/// Extract value from a string literal (handles triple-quoted strings).
pub(super) fn extract_string_value(value: &str) -> &str {
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
        return &value[1..value.len() - 1];
    }

    value
}
