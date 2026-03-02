use super::super::super::{LinkGraphIndex, LinkGraphSearchOptions, ScoredSearchRow};
use super::super::context::{SearchExecutionContext, SearchRuntimePolicy};
use std::collections::HashSet;

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search) fn collect_search_rows(
        &self,
        options: &LinkGraphSearchOptions,
        context: &SearchExecutionContext,
        graph_candidates: Option<&HashSet<String>>,
        runtime_policy: &SearchRuntimePolicy,
    ) -> Vec<ScoredSearchRow> {
        self.docs_by_id
            .values()
            .flat_map(|doc| {
                self.evaluate_doc_rows(doc, options, context, graph_candidates, runtime_policy)
            })
            .collect()
    }
}
