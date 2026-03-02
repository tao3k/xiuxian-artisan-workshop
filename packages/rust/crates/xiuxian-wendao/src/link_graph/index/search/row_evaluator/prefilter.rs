use super::super::super::{LinkGraphDocument, LinkGraphIndex, LinkGraphSearchOptions};
use super::super::context::SearchExecutionContext;
use std::collections::HashSet;

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search) fn should_skip_doc_by_filters(
        &self,
        doc: &LinkGraphDocument,
        options: &LinkGraphSearchOptions,
        context: &SearchExecutionContext,
        graph_candidates: Option<&HashSet<String>>,
    ) -> bool {
        if let Some(allowed_ids) = graph_candidates
            && !allowed_ids.contains(&doc.id)
        {
            return true;
        }
        if !Self::matches_temporal_filters(doc, options) {
            return true;
        }
        !self.matches_structured_filters(doc, options, context)
    }
}
