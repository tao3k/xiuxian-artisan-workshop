use super::super::{
    LinkGraphDocument, LinkGraphIndex, LinkGraphMatchStrategy, LinkGraphScope,
    LinkGraphSearchOptions, SECTION_AGGREGATION_BETA, SectionCandidate, SectionMatch,
    WEIGHT_FTS_LEXICAL, WEIGHT_FTS_PATH, WEIGHT_FTS_SECTION, WEIGHT_PATH_FUZZY_PATH,
    WEIGHT_PATH_FUZZY_SECTION, score_document, score_document_exact, score_document_regex,
};
use super::context::{SearchExecutionContext, SearchRuntimePolicy};

pub(super) struct DocScoreContext<'a> {
    pub(super) section_candidates: &'a [SectionCandidate],
    pub(super) section_match: Option<&'a SectionMatch>,
    pub(super) section_score: f64,
    pub(super) path_score: f64,
}

impl LinkGraphIndex {
    pub(super) fn score_doc_for_strategy(
        &self,
        doc: &LinkGraphDocument,
        options: &LinkGraphSearchOptions,
        context: &SearchExecutionContext,
        runtime_policy: &SearchRuntimePolicy,
        score_context: &DocScoreContext<'_>,
    ) -> (f64, String) {
        let raw_query = context.raw_query.as_str();
        let (mut doc_score, mut doc_reason) = match options.match_strategy {
            LinkGraphMatchStrategy::Fts if !raw_query.is_empty() => {
                let lexical = score_document(
                    doc,
                    &context.clean_query,
                    &context.query_tokens,
                    options.case_sensitive,
                );
                let blended = (lexical * WEIGHT_FTS_LEXICAL
                    + score_context.section_score * WEIGHT_FTS_SECTION
                    + score_context.path_score * WEIGHT_FTS_PATH)
                    .max(lexical);
                let reason = if let Some(section) = score_context.section_match {
                    format!("fts+{}", section.reason)
                } else {
                    "fts".to_string()
                };
                (blended, reason)
            }
            LinkGraphMatchStrategy::PathFuzzy if !raw_query.is_empty() => {
                let base = score_context.path_score.max(score_context.section_score);
                let blended = if base > 0.0 {
                    (score_context.path_score * WEIGHT_PATH_FUZZY_PATH
                        + score_context.section_score * WEIGHT_PATH_FUZZY_SECTION)
                        .max(base)
                } else {
                    0.0
                };
                let reason = if let Some(section) = score_context.section_match {
                    format!("path_fuzzy+{}", section.reason)
                } else {
                    "path_fuzzy".to_string()
                };
                (blended, reason)
            }
            LinkGraphMatchStrategy::Exact if !raw_query.is_empty() => (
                score_document_exact(doc, &context.clean_query, options.case_sensitive),
                "exact".to_string(),
            ),
            LinkGraphMatchStrategy::Re if !raw_query.is_empty() => (
                context
                    .regex
                    .as_ref()
                    .map_or(0.0, |compiled| score_document_regex(doc, compiled)),
                "regex".to_string(),
            ),
            _ => (1.0, "filtered".to_string()),
        };

        if matches!(runtime_policy.scope, LinkGraphScope::Mixed)
            && runtime_policy.collapse_to_doc
            && !score_context.section_candidates.is_empty()
        {
            let max_section = score_context
                .section_candidates
                .first()
                .map_or(0.0, |row| row.score);
            let section_tail_sum = score_context
                .section_candidates
                .iter()
                .skip(1)
                .map(|row| row.score)
                .sum::<f64>();
            let aggregated =
                (max_section + SECTION_AGGREGATION_BETA * section_tail_sum).clamp(0.0, 1.0);
            if aggregated > doc_score {
                doc_score = aggregated;
                doc_reason.push_str("+section_agg");
            }
        }

        if runtime_policy.semantic_edges_enabled
            && !raw_query.is_empty()
            && matches!(
                options.match_strategy,
                LinkGraphMatchStrategy::Fts | LinkGraphMatchStrategy::PathFuzzy
            )
            && doc_score > 0.0
        {
            let boosted = self.apply_graph_rank_boost(&doc.id, doc_score);
            if boosted > doc_score {
                doc_reason.push_str("+graph_rank");
            }
            doc_score = boosted;
        }

        (doc_score, doc_reason)
    }
}
