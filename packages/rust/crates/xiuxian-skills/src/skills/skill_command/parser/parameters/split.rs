/// Split parameters respecting nested brackets and `|` unions in type annotations.
///
/// This handles cases like `dict[str, Any] | None` by not splitting on commas inside
/// type annotations with brackets.
pub(super) fn split_parameters(params_text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;

    for character in params_text.chars() {
        match character {
            '(' | '[' => {
                depth += 1;
                current.push(character);
            }
            ')' | ']' => {
                depth = depth.saturating_sub(1);
                current.push(character);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    result.push(trimmed);
                }
                current.clear();
            }
            _ => current.push(character),
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        result.push(trimmed);
    }

    result
}
