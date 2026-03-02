mod prefilter;
mod sections;

use super::super::{
    LinkGraphDocument, LinkGraphIndex, LinkGraphScope, LinkGraphSearchOptions, ScoredSearchRow,
    score_path_fields,
};
use super::context::{SearchExecutionContext, SearchRuntimePolicy};
use super::strategy::DocScoreContext;
use std::collections::HashSet;

impl LinkGraphIndex {
    pub(super) fn evaluate_doc_rows(
        &self,
        doc: &LinkGraphDocument,
        options: &LinkGraphSearchOptions,
        context: &SearchExecutionContext,
        graph_candidates: Option<&HashSet<String>>,
        runtime_policy: &SearchRuntimePolicy,
    ) -> Vec<ScoredSearchRow> {
        let mut out: Vec<ScoredSearchRow> = Vec::new();
        let raw_query = context.raw_query.as_str();

        if self.should_skip_doc_by_filters(doc, options, context, graph_candidates) {
            return out;
        }

        let (section_candidates, section_match, section_score) =
            self.prepare_section_context(doc, context, runtime_policy);

        let path_score = if raw_query.is_empty() {
            0.0
        } else {
            score_path_fields(
                doc,
                &context.clean_query,
                &context.query_tokens,
                context.case_sensitive,
            )
        };

        let score_context = DocScoreContext {
            section_candidates: &section_candidates,
            section_match: section_match.as_ref(),
            section_score,
            path_score,
        };
        let (doc_score, doc_reason) =
            self.score_doc_for_strategy(doc, options, context, runtime_policy, &score_context);

        if !matches!(runtime_policy.scope, LinkGraphScope::SectionOnly) {
            Self::emit_doc_row(&mut out, doc, doc_score, doc_reason, section_match.as_ref());
        }

        let emit_section_rows = runtime_policy.structural_edges_enabled
            && (matches!(runtime_policy.scope, LinkGraphScope::SectionOnly)
                || (matches!(runtime_policy.scope, LinkGraphScope::Mixed)
                    && !runtime_policy.collapse_to_doc));
        if emit_section_rows {
            self.emit_section_rows(
                &mut out,
                doc,
                &section_candidates,
                options,
                raw_query,
                runtime_policy.semantic_edges_enabled,
            );
        }

        out
    }
}
