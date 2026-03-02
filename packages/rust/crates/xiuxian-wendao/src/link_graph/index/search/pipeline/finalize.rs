use super::super::super::{
    LinkGraphHit, LinkGraphIndex, LinkGraphSearchOptions, ScoredSearchRow,
    deterministic_random_key, sort_hits,
};
use std::collections::{HashMap, HashSet};

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search) fn finalize_search_rows(
        &self,
        mut rows: Vec<ScoredSearchRow>,
        options: &LinkGraphSearchOptions,
        bounded: usize,
        doc_boosts: Option<&HashMap<String, f64>>,
    ) -> Vec<LinkGraphHit> {
        let mut boosted_doc_ids: HashSet<String> = HashSet::new();
        if let Some(boosts) = doc_boosts
            && !boosts.is_empty()
        {
            for row in &mut rows {
                let doc_id = self
                    .resolve_doc_id(&row.hit.path)
                    .or_else(|| self.resolve_doc_id(&row.hit.stem));
                let Some(doc_id) = doc_id else {
                    continue;
                };
                let Some(boost) = boosts.get(doc_id) else {
                    continue;
                };
                if *boost <= 0.0 {
                    continue;
                }
                let bounded_boost = boost.clamp(0.0, 1.0);
                let bounded_score = row.hit.score.clamp(0.0, 1.0);
                row.hit.score =
                    (bounded_score + (1.0 - bounded_score) * bounded_boost).clamp(0.0, 1.0);
                let reason = row.hit.match_reason.get_or_insert_with(String::new);
                if !reason.contains("agentic_provisional") {
                    if !reason.is_empty() {
                        reason.push('+');
                    }
                    reason.push_str("agentic_provisional");
                }
                boosted_doc_ids.insert(doc_id.to_string());
            }

            for (doc_id, boost) in boosts {
                if *boost <= 0.0 || boosted_doc_ids.contains(doc_id) {
                    continue;
                }
                let Some(doc) = self.docs_by_id.get(doc_id) else {
                    continue;
                };
                let bounded_boost = boost.clamp(0.0, 1.0);
                let injected_score = (0.25 + bounded_boost * 0.5).clamp(0.0, 1.0);
                rows.push(ScoredSearchRow {
                    hit: LinkGraphHit {
                        stem: doc.stem.clone(),
                        title: doc.title.clone(),
                        path: doc.path.clone(),
                        doc_type: doc.doc_type.clone(),
                        tags: doc.tags.clone(),
                        score: injected_score,
                        best_section: None,
                        match_reason: Some("agentic_provisional_injection".to_string()),
                    },
                    created_ts: doc.created_ts,
                    modified_ts: doc.modified_ts,
                    word_count: doc.word_count,
                    random_key: deterministic_random_key(&doc.stem, &doc.path),
                });
            }
        }

        sort_hits(&mut rows, &options.sort_terms);
        rows.truncate(bounded);
        rows.into_iter().map(|row| row.hit).collect()
    }
}
