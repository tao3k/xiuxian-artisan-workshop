use super::super::super::state::ParsedDirectiveState;
use crate::link_graph::models::LinkGraphMatchStrategy;
use crate::link_graph::query::helpers::{parse_bool, parse_list_values, parse_sort_term};

pub(super) fn apply(
    key: &str,
    value: &str,
    state: &mut ParsedDirectiveState,
    residual_terms: &mut Vec<String>,
) -> bool {
    match key {
        "match" | "strategy" | "match_strategy" => {
            state.match_strategy = Some(LinkGraphMatchStrategy::from_alias(value));
            true
        }
        "query" | "q" => {
            let query_parts = parse_list_values(value);
            if query_parts.is_empty() {
                let trimmed = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim()
                    .to_string();
                if !trimmed.is_empty() {
                    residual_terms.push(trimmed);
                }
            } else {
                residual_terms.push(query_parts.join(" "));
            }
            true
        }
        "id" => {
            let parsed = parse_list_values(value).into_iter().next().or_else(|| {
                let trimmed = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim()
                    .to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            });
            if let Some(direct_id) = parsed {
                state.direct_id = Some(direct_id);
            }
            true
        }
        "sort" => {
            let mut parsed_any = false;
            for item in parse_list_values(value) {
                state.sort_terms.push(parse_sort_term(&item));
                parsed_any = true;
            }
            if !parsed_any {
                state.sort_terms.push(parse_sort_term(value));
            }
            true
        }
        "case" | "case_sensitive" => {
            state.case_sensitive = parse_bool(value);
            true
        }
        "limit" | "top" | "n" | "k" => {
            if let Ok(limit) = value.parse::<usize>()
                && limit > 0
            {
                state.limit_override = Some(limit);
            }
            true
        }
        _ => false,
    }
}
