use crate::skills::metadata::DecoratorArgs;

use super::strings::{extract_string_value, split_args_respecting_strings};

/// Parse decorator arguments from decorator text handling triple-quoted strings.
#[must_use]
pub fn parse_decorator_args(decorator_text: &str) -> DecoratorArgs {
    let mut args = DecoratorArgs::default();

    if let Some(open_paren) = decorator_text.find('(') {
        let arg_text = extract_decorator_arg_text(decorator_text, open_paren + 1);
        let parts = split_args_respecting_strings(&arg_text);

        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

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
    let mut depth = 1usize;
    let mut in_triple_quote = false;
    let mut triple_quote_char = '\0';
    let mut result = String::new();

    for (offset, character) in text[start..].char_indices() {
        let absolute_pos = start + offset;

        if in_triple_quote {
            if character == triple_quote_char {
                let remaining = &text[absolute_pos..];
                if remaining.starts_with("\"\"\"") || remaining.starts_with("'''") {
                    result.push_str(&text[start + offset..start + offset + 3]);
                    in_triple_quote = false;
                    continue;
                }
            }
            result.push(character);
        } else {
            let remaining = &text[absolute_pos..];
            if remaining.starts_with("\"\"\"") {
                in_triple_quote = true;
                triple_quote_char = '"';
                result.push_str("\"\"\"");
            } else if remaining.starts_with("'''") {
                in_triple_quote = true;
                triple_quote_char = '\'';
                result.push_str("'''");
            } else if character == '(' {
                depth += 1;
                result.push(character);
            } else if character == ')' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return result;
                }
                result.push(character);
            } else {
                result.push(character);
            }
        }
    }

    result
}
