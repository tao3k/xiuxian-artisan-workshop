/// Find all @`skill_command` decorator positions in Python code.
///
/// Uses simple string matching (not regex) to find decorators.
/// Returns `Vec` of (`start_pos`, `end_pos`, `full_decorator_text`).
#[must_use]
pub fn find_skill_command_decorators(content: &str) -> Vec<(usize, usize, String)> {
    let mut decorators = Vec::new();
    let prefix = "@skill_command";

    let mut search_start = 0usize;
    while let Some(start) = content[search_start..].find(prefix) {
        let absolute_start = search_start + start;
        let line_start = content[..absolute_start]
            .rfind('\n')
            .map_or(0, |position| position + 1);
        let after_decorator = &content[absolute_start + prefix.len()..];

        if after_decorator.starts_with('(') {
            if let Some(end_pos) = find_matching_paren(content, absolute_start + prefix.len()) {
                let full_text = &content[line_start..end_pos];
                decorators.push((line_start, end_pos, full_text.to_string()));
                search_start = end_pos;
            } else {
                search_start = absolute_start + prefix.len();
            }
        } else {
            search_start = absolute_start + prefix.len();
        }
    }

    decorators
}

fn find_matching_paren(content: &str, paren_start: usize) -> Option<usize> {
    let search_content = &content[paren_start + 1..];
    let chars: Vec<char> = search_content.chars().collect();

    let mut depth = 1usize;
    let mut in_string = false;
    let mut quote_char = '\0';
    let mut in_triple_quote = false;
    let mut index = 0usize;

    while index < chars.len() {
        let character = chars[index];

        if in_triple_quote {
            if character == quote_char
                && index + 2 < chars.len()
                && chars[index + 1] == quote_char
                && chars[index + 2] == quote_char
            {
                in_triple_quote = false;
                index += 2;
            }
        } else if in_string {
            if character == quote_char && (index == 0 || chars[index - 1] != '\\') {
                in_string = false;
            }
        } else {
            if index + 2 < chars.len()
                && ((chars[index] == '"' && chars[index + 1] == '"' && chars[index + 2] == '"')
                    || (chars[index] == '\''
                        && chars[index + 1] == '\''
                        && chars[index + 2] == '\''))
            {
                in_triple_quote = true;
                quote_char = chars[index];
                index += 3;
                continue;
            }

            match character {
                '"' | '\'' => {
                    in_string = true;
                    quote_char = character;
                }
                '(' => depth += 1,
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(paren_start + 1 + index + 1);
                    }
                }
                _ => {}
            }
        }

        index += 1;
    }

    None
}
