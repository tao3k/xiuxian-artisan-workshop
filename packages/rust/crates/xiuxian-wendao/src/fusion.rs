//! Fusion Recall Boost — high-performance Rust implementation.
//!
//! Pure computation: apply `LinkGraph` link/tag proximity boost to recall results.
//! `Python` provides a thin wrapper (`LinkGraph` data fetch); all score computation runs here.

use std::collections::{HashMap, HashSet};

/// Apply `LinkGraph` link and tag proximity boost to recall results.
///
/// For each pair of results (`i`, `j`) where stems share a `LinkGraph` link or tag:
/// - Add `link_boost` to both scores when stems are bidirectionally linked
/// - Add `tag_boost` to both scores when stems share tags
///
/// Results are re-sorted by score (descending) in place.
pub fn apply_link_graph_proximity_boost(
    results: &mut [RecallResult],
    stem_links: &HashMap<String, HashSet<String>>,
    stem_tags: &HashMap<String, HashSet<String>>,
    link_boost: f64,
    tag_boost: f64,
) {
    if results.len() < 2 {
        return;
    }

    for i in 0..results.len() {
        let stem1 = stem_from_source(&results[i].source);
        let Some(links1) = stem_links.get(&stem1) else {
            continue;
        };

        for j in (i + 1)..results.len() {
            let stem2 = stem_from_source(&results[j].source);
            let Some(links2) = stem_links.get(&stem2) else {
                continue;
            };

            let mut add_link = false;
            if links1.contains(&stem2) || links2.contains(&stem1) {
                add_link = true;
            }

            let mut add_tag = false;
            if let (Some(tags1), Some(tags2)) = (stem_tags.get(&stem1), stem_tags.get(&stem2))
                && !tags1.is_disjoint(tags2)
            {
                add_tag = true;
            }

            if add_link {
                results[i].score += link_boost;
                results[j].score += link_boost;
            }
            if add_tag {
                results[i].score += tag_boost;
                results[j].score += tag_boost;
            }
        }
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Extract stem from source path (filename without extension).
fn stem_from_source(source: &str) -> String {
    source
        .rsplit('/')
        .next()
        .unwrap_or(source)
        .rsplit('.')
        .nth(1)
        .map_or_else(|| source.to_string(), std::string::ToString::to_string)
}

/// Recall result for boost computation.
#[derive(Debug, Clone)]
pub struct RecallResult {
    pub source: String,
    pub score: f64,
    pub content: String,
    pub title: String,
}

impl RecallResult {
    pub fn new(source: String, score: f64, content: String, title: String) -> Self {
        Self {
            source,
            score,
            content,
            title,
        }
    }
}
