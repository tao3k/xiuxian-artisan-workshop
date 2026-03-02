use super::state::{ParsedDirectiveState, has_related_ppr_options};
use crate::link_graph::models::{
    LinkGraphMatchStrategy, LinkGraphSearchOptions, LinkGraphSortTerm, LinkGraphTagFilter,
};
use crate::link_graph::query::helpers::{infer_strategy_from_residual, is_default_sort_terms};

fn merge_match_strategy(
    base: &mut LinkGraphSearchOptions,
    residual_terms: &[String],
    state: &ParsedDirectiveState,
) {
    if base.match_strategy != LinkGraphMatchStrategy::Fts {
        return;
    }
    if let Some(strategy) = state.match_strategy {
        base.match_strategy = strategy;
        return;
    }
    let residual = residual_terms.join(" ");
    if let Some(inferred) = infer_strategy_from_residual(&residual) {
        base.match_strategy = inferred;
    }
}

fn merge_case_and_sort(base: &mut LinkGraphSearchOptions, state: &ParsedDirectiveState) {
    if !base.case_sensitive
        && let Some(case_sensitive) = state.case_sensitive
    {
        base.case_sensitive = case_sensitive;
    }
    if is_default_sort_terms(&base.sort_terms) && !state.sort_terms.is_empty() {
        base.sort_terms.clone_from(&state.sort_terms);
    }
    if base.sort_terms.is_empty() {
        base.sort_terms = vec![LinkGraphSortTerm::default()];
    }
}

fn merge_tag_and_link_filters(base: &mut LinkGraphSearchOptions, state: &ParsedDirectiveState) {
    if !state.tags_all.is_empty() || !state.tags_any.is_empty() || !state.tags_not.is_empty() {
        let parsed_tag_filter = LinkGraphTagFilter {
            all: state.tags_all.clone(),
            any: state.tags_any.clone(),
            not_tags: state.tags_not.clone(),
        };
        if base.filters.tags.is_none() {
            base.filters.tags = Some(parsed_tag_filter);
        }
    }
    if !state.link_to.seeds.is_empty() && base.filters.link_to.is_none() {
        base.filters.link_to = Some(state.link_to.clone());
    }
    if !state.linked_by.seeds.is_empty() && base.filters.linked_by.is_none() {
        base.filters.linked_by = Some(state.linked_by.clone());
    }
}

fn merge_related_filters(base: &mut LinkGraphSearchOptions, state: &ParsedDirectiveState) {
    let parsed_related_has_ppr = has_related_ppr_options(&state.related_ppr);
    if base.filters.related.is_none() {
        if !state.related.seeds.is_empty() {
            let mut related = state.related.clone();
            if parsed_related_has_ppr {
                related.ppr = Some(state.related_ppr.clone());
            }
            base.filters.related = Some(related);
        }
    } else if let Some(base_related) = base.filters.related.as_mut() {
        if base_related.max_distance.is_none() && state.related.max_distance.is_some() {
            base_related.max_distance = state.related.max_distance;
        }
        if base_related.ppr.is_none() && parsed_related_has_ppr {
            base_related.ppr = Some(state.related_ppr.clone());
        }
    }
}

fn merge_search_filters(base: &mut LinkGraphSearchOptions, state: &ParsedDirectiveState) {
    if base.filters.include_paths.is_empty() && !state.filters.include_paths.is_empty() {
        base.filters
            .include_paths
            .clone_from(&state.filters.include_paths);
    }
    if base.filters.exclude_paths.is_empty() && !state.filters.exclude_paths.is_empty() {
        base.filters
            .exclude_paths
            .clone_from(&state.filters.exclude_paths);
    }
    if base.filters.mentions_of.is_empty() && !state.filters.mentions_of.is_empty() {
        base.filters
            .mentions_of
            .clone_from(&state.filters.mentions_of);
    }
    if base.filters.mentioned_by_notes.is_empty() && !state.filters.mentioned_by_notes.is_empty() {
        base.filters
            .mentioned_by_notes
            .clone_from(&state.filters.mentioned_by_notes);
    }
    if !base.filters.orphan && state.filters.orphan {
        base.filters.orphan = true;
    }
    if !base.filters.tagless && state.filters.tagless {
        base.filters.tagless = true;
    }
    if !base.filters.missing_backlink && state.filters.missing_backlink {
        base.filters.missing_backlink = true;
    }
    if base.filters.scope.is_none() {
        base.filters.scope = state.scope;
    }
    if base.filters.max_heading_level.is_none() {
        base.filters.max_heading_level = state.max_heading_level;
    }
    if base.filters.max_tree_hops.is_none() {
        base.filters.max_tree_hops = state.max_tree_hops;
    }
    if base.filters.collapse_to_doc.is_none() {
        base.filters.collapse_to_doc = state.collapse_to_doc;
    }
    if base.filters.edge_types.is_empty() && !state.edge_types.is_empty() {
        base.filters.edge_types.clone_from(&state.edge_types);
    }
    if base.filters.per_doc_section_cap.is_none() {
        base.filters.per_doc_section_cap = state.per_doc_section_cap;
    }
    if base.filters.min_section_words.is_none() {
        base.filters.min_section_words = state.min_section_words;
    }
}

fn merge_time_filters(base: &mut LinkGraphSearchOptions, state: &ParsedDirectiveState) {
    if base.created_after.is_none() {
        base.created_after = state.created_after;
    }
    if base.created_before.is_none() {
        base.created_before = state.created_before;
    }
    if base.modified_after.is_none() {
        base.modified_after = state.modified_after;
    }
    if base.modified_before.is_none() {
        base.modified_before = state.modified_before;
    }
}

pub(super) fn merge_into_base(
    base: &mut LinkGraphSearchOptions,
    residual_terms: &[String],
    state: &ParsedDirectiveState,
) {
    merge_match_strategy(base, residual_terms, state);
    merge_case_and_sort(base, state);
    merge_tag_and_link_filters(base, state);
    merge_related_filters(base, state);
    merge_search_filters(base, state);
    merge_time_filters(base, state);
}
