use super::ParsedParameter;
use super::split::split_parameters;

/// Parse parameter names from function signature string.
#[must_use]
pub fn parse_parameters(params_text: &str) -> Vec<String> {
    split_parameters(params_text)
        .iter()
        .filter_map(|segment| {
            let clean = if let Some(colon_pos) = segment.find(':') {
                &segment[..colon_pos]
            } else {
                segment
            };
            let clean = clean.split('=').next().unwrap_or(clean);
            let clean = clean.trim();

            if clean.starts_with('*') && clean != "*" {
                None
            } else if !clean.is_empty() {
                Some(clean.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Parse detailed parameter information from parameter text.
#[must_use]
pub(super) fn parse_detailed_parameters(params_text: &str) -> Vec<ParsedParameter> {
    split_parameters(params_text)
        .iter()
        .map(|segment| segment.trim())
        .filter(|segment| !segment.is_empty())
        .filter_map(|segment| {
            if segment.starts_with('*') && segment != "*" {
                return None;
            }

            let (before_eq, default_str) = if let Some(eq_pos) = segment.find('=') {
                (
                    &segment[..eq_pos],
                    Some(segment[eq_pos + 1..].trim().to_string()),
                )
            } else {
                (segment, None)
            };

            let (name, type_str) = if let Some(colon_pos) = before_eq.find(':') {
                let name_part = before_eq[..colon_pos].trim();
                let type_part = before_eq[colon_pos + 1..].trim().to_string();
                (name_part.to_string(), Some(type_part))
            } else {
                (before_eq.trim().to_string(), None)
            };

            let has_default = default_str.is_some();

            if name.is_empty() {
                None
            } else {
                Some(ParsedParameter {
                    name,
                    type_annotation: type_str,
                    has_default,
                    default_value: default_str,
                })
            }
        })
        .filter(|parameter| !parameter.name.starts_with('*'))
        .collect()
}
