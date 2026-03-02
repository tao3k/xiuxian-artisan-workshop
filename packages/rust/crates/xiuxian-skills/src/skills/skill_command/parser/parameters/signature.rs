use super::ParsedParameter;
use super::parse::{parse_detailed_parameters, parse_parameters};

/// Extract parameter names from full function signature text.
#[must_use]
pub fn extract_parameters_from_text(func_text: &str) -> Vec<String> {
    extract_parameter_text(func_text).map_or_else(Vec::new, parse_parameters)
}

/// Extract detailed parameter information from function signature text.
///
/// Returns a vector of `ParsedParameter` with name, type, default info.
#[must_use]
pub fn extract_parsed_parameters(func_text: &str) -> Vec<ParsedParameter> {
    extract_parameter_text(func_text).map_or_else(Vec::new, parse_detailed_parameters)
}

fn extract_parameter_text(func_text: &str) -> Option<&str> {
    let open_paren = func_text.find('(')?;
    let mut depth = 1usize;
    let mut close_paren = None;
    let search_content = &func_text[open_paren + 1..];

    for (index, character) in search_content.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close_paren = Some(open_paren + 1 + index);
                    break;
                }
            }
            _ => {}
        }
    }

    close_paren.map(|end_pos| &func_text[open_paren + 1..end_pos])
}
