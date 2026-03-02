use super::super::super::{
    LinkGraphDocument, LinkGraphIndex, LinkGraphScope, SectionCandidate, SectionMatch,
};
use super::super::context::{SearchExecutionContext, SearchRuntimePolicy};

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search) fn prepare_section_context(
        &self,
        doc: &LinkGraphDocument,
        context: &SearchExecutionContext,
        runtime_policy: &SearchRuntimePolicy,
    ) -> (Vec<SectionCandidate>, Option<SectionMatch>, f64) {
        let mut section_candidates = if runtime_policy.structural_edges_enabled {
            self.section_candidates(&doc.id, context, runtime_policy)
        } else {
            Vec::new()
        };
        if matches!(
            runtime_policy.scope,
            LinkGraphScope::SectionOnly | LinkGraphScope::Mixed
        ) {
            section_candidates.retain(|row| !row.heading_path.trim().is_empty());
        }
        if section_candidates.len() > runtime_policy.per_doc_section_cap {
            section_candidates.truncate(runtime_policy.per_doc_section_cap);
        }
        let section_match = Self::best_section_match(&section_candidates);
        let section_score = section_match.as_ref().map_or(0.0, |row| row.score);
        (section_candidates, section_match, section_score)
    }
}
