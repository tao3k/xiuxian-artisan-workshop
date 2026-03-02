use super::state::ParsedDirectiveState;
use crate::link_graph::query::helpers::{
    is_boolean_connector_token, paren_balance, parse_directive_key, parse_time_filter,
    split_terms_preserving_quotes,
};

#[path = "scan/directives/mod.rs"]
mod directives;

use directives::apply_directive;

pub(super) fn parse_terms(raw: &str, state: &mut ParsedDirectiveState) -> Vec<String> {
    let terms = split_terms_preserving_quotes(raw);
    let mut residual_terms: Vec<String> = Vec::new();
    let mut index = 0usize;

    while index < terms.len() {
        let term = terms[index].clone();
        let bare = term.trim().to_lowercase();
        if bare == "orphan" {
            state.filters.orphan = true;
            index += 1;
            continue;
        }
        if bare == "tagless" {
            state.filters.tagless = true;
            index += 1;
            continue;
        }
        if bare == "missing_backlink" {
            state.filters.missing_backlink = true;
            index += 1;
            continue;
        }
        if parse_time_filter(
            &term,
            &mut state.created_after,
            &mut state.created_before,
            &mut state.modified_after,
            &mut state.modified_before,
        ) {
            index += 1;
            continue;
        }

        let Some((raw_key, raw_value)) = term.split_once(':') else {
            residual_terms.push(term);
            index += 1;
            continue;
        };

        let (negated_key, key_raw) = parse_directive_key(raw_key);
        let key = key_raw.replace(['-', '.'], "_");
        let mut value = raw_value.trim().to_string();
        if value.is_empty() {
            residual_terms.push(term);
            index += 1;
            continue;
        }

        let mut consumed = index;
        extend_until_balanced(&mut value, &terms, &mut consumed);

        if matches!(key.as_str(), "tag" | "tags") {
            extend_tag_expression(&mut value, &terms, &mut consumed);
        }

        if !apply_directive(&key, &value, negated_key, state, &mut residual_terms) {
            residual_terms.push(format!("{}:{}", raw_key.trim(), value));
        }

        index = consumed + 1;
    }

    residual_terms
}

fn extend_until_balanced(value: &mut String, terms: &[String], consumed: &mut usize) {
    while paren_balance(value) > 0 && *consumed + 1 < terms.len() {
        *consumed += 1;
        append_non_empty_token(value, terms[*consumed].trim());
    }
}

fn extend_tag_expression(value: &mut String, terms: &[String], consumed: &mut usize) {
    while *consumed + 2 < terms.len() && is_boolean_connector_token(&terms[*consumed + 1]) {
        append_non_empty_token(value, terms[*consumed + 1].trim());
        append_non_empty_token(value, terms[*consumed + 2].trim());
        *consumed += 2;
        extend_until_balanced(value, terms, consumed);
    }
}

fn append_non_empty_token(value: &mut String, token: &str) {
    if token.is_empty() {
        return;
    }
    if !value.is_empty() {
        value.push(' ');
    }
    value.push_str(token);
}
