use super::super::super::state::{ParsedDirectiveState, parse_ppr_subgraph_mode};
use crate::link_graph::query::helpers::{parse_bool, parse_list_values, push_unique_many};

fn parse_positive_usize(value: &str) -> Option<usize> {
    value.parse::<usize>().ok().filter(|distance| *distance > 0)
}

fn parse_alpha(value: &str) -> Option<f64> {
    value
        .parse::<f64>()
        .ok()
        .filter(|alpha| (0.0..=1.0).contains(alpha))
}

fn parse_positive_f64(value: &str) -> Option<f64> {
    value.parse::<f64>().ok().filter(|tol| *tol > 0.0)
}

fn apply_link_to_directive(
    key: &str,
    value: &str,
    negated_key: bool,
    state: &mut ParsedDirectiveState,
) -> bool {
    match key {
        "to" | "link_to" => {
            if negated_key {
                state.link_to.negate = true;
            }
            push_unique_many(&mut state.link_to.seeds, parse_list_values(value));
            true
        }
        "to_not" | "no_link_to" | "link_to_not" => {
            state.link_to.negate = true;
            push_unique_many(&mut state.link_to.seeds, parse_list_values(value));
            true
        }
        "link_to_negate" => {
            state.link_to.negate = parse_bool(value).unwrap_or(state.link_to.negate);
            true
        }
        "link_to_recursive" => {
            state.link_to.recursive = parse_bool(value).unwrap_or(state.link_to.recursive);
            true
        }
        "link_to_max_distance" => {
            if let Some(distance) = parse_positive_usize(value) {
                state.link_to.max_distance = Some(distance);
            }
            true
        }
        _ => false,
    }
}

fn apply_linked_by_directive(
    key: &str,
    value: &str,
    negated_key: bool,
    state: &mut ParsedDirectiveState,
) -> bool {
    match key {
        "from" | "linked_by" => {
            if negated_key {
                state.linked_by.negate = true;
            }
            push_unique_many(&mut state.linked_by.seeds, parse_list_values(value));
            true
        }
        "from_not" | "no_linked_by" | "linked_by_not" => {
            state.linked_by.negate = true;
            push_unique_many(&mut state.linked_by.seeds, parse_list_values(value));
            true
        }
        "linked_by_negate" => {
            state.linked_by.negate = parse_bool(value).unwrap_or(state.linked_by.negate);
            true
        }
        "linked_by_recursive" => {
            state.linked_by.recursive = parse_bool(value).unwrap_or(state.linked_by.recursive);
            true
        }
        "linked_by_max_distance" => {
            if let Some(distance) = parse_positive_usize(value) {
                state.linked_by.max_distance = Some(distance);
            }
            true
        }
        _ => false,
    }
}

fn apply_related_seed_values(value: &str, state: &mut ParsedDirectiveState) {
    for item in parse_list_values(value) {
        if let Some((seed, distance_raw)) = item.rsplit_once('~') {
            let cleaned_seed = seed.trim();
            if !cleaned_seed.is_empty() {
                push_unique_many(&mut state.related.seeds, vec![cleaned_seed.to_string()]);
            }
            if let Some(distance) = parse_positive_usize(distance_raw.trim()) {
                state.related.max_distance = Some(distance);
            }
        } else {
            push_unique_many(&mut state.related.seeds, vec![item]);
        }
    }
}

fn apply_related_directive(key: &str, value: &str, state: &mut ParsedDirectiveState) -> bool {
    match key {
        "related" => {
            apply_related_seed_values(value, state);
            true
        }
        "max_distance" | "distance" | "hops" => {
            if let Some(distance) = parse_positive_usize(value) {
                state.related.max_distance = Some(distance);
            }
            true
        }
        "related_ppr_alpha" | "ppr_alpha" => {
            if let Some(alpha) = parse_alpha(value) {
                state.related_ppr.alpha = Some(alpha);
            }
            true
        }
        "related_ppr_max_iter" | "ppr_max_iter" => {
            if let Some(max_iter) = parse_positive_usize(value) {
                state.related_ppr.max_iter = Some(max_iter);
            }
            true
        }
        "related_ppr_tol" | "ppr_tol" => {
            if let Some(tol) = parse_positive_f64(value) {
                state.related_ppr.tol = Some(tol);
            }
            true
        }
        "related_ppr_subgraph_mode" | "ppr_subgraph_mode" => {
            state.related_ppr.subgraph_mode = parse_ppr_subgraph_mode(value);
            true
        }
        _ => false,
    }
}

pub(super) fn apply(
    key: &str,
    value: &str,
    negated_key: bool,
    state: &mut ParsedDirectiveState,
) -> bool {
    apply_link_to_directive(key, value, negated_key, state)
        || apply_linked_by_directive(key, value, negated_key, state)
        || apply_related_directive(key, value, state)
}
