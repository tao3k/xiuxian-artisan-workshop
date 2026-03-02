//! Weighted RRF with field boosting (SOTA algorithm).
//! P5: Boost phase runs in parallel (rayon) over name/token/metadata deltas.

use std::collections::HashMap;

use aho_corasick::{AhoCorasick, PatternID};
use lance::deps::arrow_array::StringArray;
use rayon::prelude::*;

use crate::ToolSearchResult;
use crate::keyword::{EXACT_PHRASE_BOOST, NAME_TOKEN_BOOST};

use super::boost::{file_discovery_boost, is_file_discovery_query, metadata_alignment_boost};
use super::match_util::{
    NameMatchResult, build_name_lower_arrow, build_name_token_automaton_with_phrase,
    count_name_token_matches_and_exact,
};
use super::types::HybridSearchResult;

struct EffectiveFusionWeights {
    keyword_sparse: bool,
    vec_weight: f32,
    kw_weight: f32,
}

fn compute_effective_fusion_weights(
    keyword_results_len: usize,
    semantic_weight: f32,
    keyword_weight: f32,
) -> EffectiveFusionWeights {
    let keyword_sparse = keyword_results_len < 2;
    if keyword_sparse {
        EffectiveFusionWeights {
            keyword_sparse,
            vec_weight: 2.0,
            kw_weight: 0.1,
        }
    } else {
        EffectiveFusionWeights {
            keyword_sparse,
            vec_weight: semantic_weight,
            kw_weight: keyword_weight,
        }
    }
}

fn maybe_log_sparse_keyword_fallback(
    weights: &EffectiveFusionWeights,
    keyword_results_len: usize,
    query: &str,
) {
    if log::log_enabled!(log::Level::Debug) && weights.keyword_sparse && keyword_results_len > 0 {
        log::debug!(
            "Smart RRF Fallback: Sparse keyword results ({}) for query '{}', \
             boosting vector weight to {:.1}",
            keyword_results_len,
            query,
            weights.vec_weight
        );
    }
}

fn seed_vector_fusion_map(
    fusion_map: &mut HashMap<String, HybridSearchResult>,
    vector_results: Vec<(String, f32)>,
    k: f32,
    weights: &EffectiveFusionWeights,
) {
    for (rank, (name, score)) in vector_results.into_iter().enumerate() {
        let rrf_score = weights.vec_weight * super::kernels::rrf_term(k, rank);
        let fallback_bonus = if weights.keyword_sparse {
            score * 0.3
        } else {
            0.0
        };
        fusion_map.insert(
            name.clone(),
            HybridSearchResult {
                tool_name: name,
                rrf_score: rrf_score + fallback_bonus,
                vector_score: score,
                keyword_score: 0.0,
            },
        );
    }
}

fn merge_keyword_fusion_scores(
    fusion_map: &mut HashMap<String, HybridSearchResult>,
    keyword_results: Vec<ToolSearchResult>,
    k: f32,
    weights: &EffectiveFusionWeights,
) -> HashMap<String, ToolSearchResult> {
    let mut keyword_context: HashMap<String, ToolSearchResult> = HashMap::new();

    if weights.kw_weight <= 0.05 {
        for result in keyword_results {
            keyword_context.insert(result.tool_name.clone(), result);
        }
        return keyword_context;
    }

    for (rank, result) in keyword_results.into_iter().enumerate() {
        let rrf_score = weights.kw_weight * super::kernels::rrf_term(k, rank);
        let tool_name = result.tool_name.clone();
        if let Some(entry) = fusion_map.get_mut(tool_name.as_str()) {
            entry.rrf_score += rrf_score;
            entry.keyword_score = result.score;
        } else {
            fusion_map.insert(
                tool_name.clone(),
                HybridSearchResult {
                    tool_name: tool_name.clone(),
                    rrf_score,
                    vector_score: 0.0,
                    keyword_score: result.score,
                },
            );
        }
        keyword_context.insert(tool_name, result);
    }

    keyword_context
}

fn compute_name_metadata_boost_deltas(
    keys_ordered: &[String],
    names_lower_array: &StringArray,
    ac_and_exact: Option<&(AhoCorasick, Option<PatternID>)>,
    keyword_context: &HashMap<String, ToolSearchResult>,
    query_parts: &[&str],
    file_discovery_intent: bool,
) -> Vec<f32> {
    (0..keys_ordered.len())
        .into_par_iter()
        .map(|i| {
            let tool_name = &keys_ordered[i];
            let name_lower = names_lower_array.value(i);
            let NameMatchResult {
                token_count: match_count,
                exact_phrase,
            } = ac_and_exact.map_or_else(NameMatchResult::default, |(ac, exact_id)| {
                count_name_token_matches_and_exact(ac, name_lower, *exact_id)
            });

            let mut delta = 0.0;
            if match_count > 0 {
                delta +=
                    f32::from(u16::try_from(match_count).unwrap_or(u16::MAX)) * NAME_TOKEN_BOOST;
            }
            if exact_phrase {
                delta += EXACT_PHRASE_BOOST;
            }
            if let Some(meta) = keyword_context.get(tool_name.as_str()) {
                delta += metadata_alignment_boost(meta, query_parts);
                if file_discovery_intent && file_discovery_boost(meta) {
                    delta += 0.25;
                }
            }
            delta
        })
        .collect()
}

fn apply_boost_deltas(
    fusion_map: &mut HashMap<String, HybridSearchResult>,
    keys_ordered: &[String],
    deltas: Vec<f32>,
) {
    for (i, delta) in deltas.into_iter().enumerate() {
        if let Some(entry) = fusion_map.get_mut(&keys_ordered[i]) {
            entry.rrf_score += delta;
        }
    }
}

fn sorted_fusion_results(
    fusion_map: HashMap<String, HybridSearchResult>,
) -> Vec<HybridSearchResult> {
    let mut results: Vec<_> = fusion_map.into_values().collect();
    results.sort_by(|a, b| {
        b.rrf_score
            .total_cmp(&a.rrf_score)
            .then_with(|| a.tool_name.cmp(&b.tool_name))
    });
    results
}

/// Apply Weighted RRF with Field Boosting.
///
/// Algorithm: weighted vector + keyword streams, smart fallback for sparse keyword results,
/// dynamic field boosting (name token match, exact phrase, metadata alignment).
#[must_use]
pub fn apply_weighted_rrf(
    vector_results: Vec<(String, f32)>,
    keyword_results: Vec<ToolSearchResult>,
    k: f32,
    semantic_weight: f32,
    keyword_weight: f32,
    query: &str,
) -> Vec<HybridSearchResult> {
    let mut fusion_map: HashMap<String, HybridSearchResult> = HashMap::new();
    let query_lower = query.to_lowercase();
    let query_parts: Vec<&str> = query_lower.split_whitespace().collect();
    let file_discovery_intent = is_file_discovery_query(&query_lower, &query_parts);
    let keyword_results_len = keyword_results.len();
    let weights =
        compute_effective_fusion_weights(keyword_results_len, semantic_weight, keyword_weight);
    maybe_log_sparse_keyword_fallback(&weights, keyword_results_len, query);
    seed_vector_fusion_map(&mut fusion_map, vector_results, k, &weights);
    let keyword_context =
        merge_keyword_fusion_scores(&mut fusion_map, keyword_results, k, &weights);

    let (keys_ordered, names_lower_array) = build_name_lower_arrow(fusion_map.keys());
    let ac_and_exact = build_name_token_automaton_with_phrase(&query_parts, &query_lower);
    let deltas = compute_name_metadata_boost_deltas(
        &keys_ordered,
        &names_lower_array,
        ac_and_exact.as_ref(),
        &keyword_context,
        &query_parts,
        file_discovery_intent,
    );
    apply_boost_deltas(&mut fusion_map, &keys_ordered, deltas);
    sorted_fusion_results(fusion_map)
}
