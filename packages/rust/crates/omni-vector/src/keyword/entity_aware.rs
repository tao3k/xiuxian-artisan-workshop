//! Entity-Aware Search Enhancement
//!
//! Integrates knowledge graph entities with vector/keyword search for improved recall.
//! When entities are provided, they boost results that contain or are related to those entities.

use crate::HybridSearchResult;
use crate::skill::ToolSearchResult;
use aho_corasick::AhoCorasick;
use std::collections::HashSet;

/// Result type for entity-aware search
#[derive(Debug, Clone)]
pub struct EntityAwareSearchResult {
    /// Full result with base scores
    pub base: HybridSearchResult,
    /// Entity matches that contributed to boosting
    pub entity_matches: Vec<EntityMatch>,
    /// Final boosted score
    pub boosted_score: f32,
}

/// An entity that matched in the search
#[derive(Debug, Clone)]
pub struct EntityMatch {
    /// Entity name
    pub entity_name: String,
    /// Entity type (PERSON, TOOL, CONCEPT, etc.)
    pub entity_type: String,
    /// Confidence score of the match
    pub confidence: f32,
    /// How the entity was matched (`name_match`, `metadata_match`, etc.)
    pub match_type: EntityMatchType,
}

/// How an entity was matched in the search
#[derive(Debug, Clone)]
pub enum EntityMatchType {
    /// Entity name exactly matched content
    NameMatch,
    /// Entity aliases matched content
    AliasMatch,
    /// Entity was mentioned in metadata
    MetadataMatch,
    /// Entity is related to a matched result
    RelatedEntity,
}

/// Cached entity data for efficient matching
struct CachedEntity {
    original: EntityMatch,
    name_lower: String,
}

/// Apply entity boosting to hybrid search results
///
/// # Arguments
///
/// * `results` - Base hybrid search results
/// * `entities` - Entities from knowledge graph to boost with
/// * `entity_weight` - Weight for entity contribution (default 0.3)
/// * `metadata` - Document metadata to check for entity mentions
///
/// # Returns
///
/// Entity-aware results with boosted scores
#[must_use]
pub fn apply_entity_boost(
    results: Vec<HybridSearchResult>,
    entities: Vec<EntityMatch>,
    entity_weight: f32,
    metadata: Option<&[serde_json::Value]>,
) -> Vec<EntityAwareSearchResult> {
    // Pre-compute lowercase entity names for efficient matching
    let cached_entities: Vec<CachedEntity> = entities
        .into_iter()
        .map(|entity| CachedEntity {
            name_lower: entity.entity_name.to_lowercase(),
            original: entity,
        })
        .collect();

    // Aho-Corasick over entity names: one automaton, O(n+m) per haystack instead of O(entities * contains)
    let (entity_ac, pattern_to_cached_idx) = build_entity_name_automaton(&cached_entities);

    let mut aware_results: Vec<EntityAwareSearchResult> = Vec::new();

    for result in results {
        let tool_name_lower = result.tool_name.to_lowercase();
        let mut matched_entities = Vec::new();
        let mut matched_names = HashSet::new();

        collect_entity_matches_in_text(
            &tool_name_lower,
            &cached_entities,
            entity_ac.as_ref(),
            &pattern_to_cached_idx,
            None,
            &mut matched_names,
            &mut matched_entities,
        );
        if let Some(meta_list) = metadata {
            collect_metadata_entity_matches(
                meta_list,
                &cached_entities,
                entity_ac.as_ref(),
                &pattern_to_cached_idx,
                &mut matched_names,
                &mut matched_entities,
            );
        }

        let entity_boost = calculate_entity_boost(&matched_entities, entity_weight);

        // Apply boost to RRF score
        let boosted_score = result.rrf_score * (1.0 + entity_boost);

        aware_results.push(EntityAwareSearchResult {
            base: result,
            entity_matches: matched_entities,
            boosted_score,
        });
    }

    // Sort by boosted score
    aware_results.sort_by(|a, b| {
        b.boosted_score
            .partial_cmp(&a.boosted_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    aware_results
}

fn build_entity_name_automaton(
    cached_entities: &[CachedEntity],
) -> (Option<AhoCorasick>, Vec<usize>) {
    let mut patterns: Vec<&str> = Vec::new();
    let mut pattern_to_cached_idx: Vec<usize> = Vec::new();
    for (index, cached) in cached_entities.iter().enumerate() {
        if !cached.name_lower.is_empty() {
            patterns.push(cached.name_lower.as_str());
            pattern_to_cached_idx.push(index);
        }
    }
    if patterns.is_empty() {
        return (None, Vec::new());
    }
    match AhoCorasick::new(patterns) {
        Ok(ac) => (Some(ac), pattern_to_cached_idx),
        Err(_) => (None, Vec::new()),
    }
}

fn collect_metadata_entity_matches(
    metadata: &[serde_json::Value],
    cached_entities: &[CachedEntity],
    entity_ac: Option<&AhoCorasick>,
    pattern_to_cached_idx: &[usize],
    matched_names: &mut HashSet<String>,
    matched_entities: &mut Vec<EntityMatch>,
) {
    let metadata_match_type = EntityMatchType::MetadataMatch;
    for meta in metadata {
        let Some(content) = meta.get("content").and_then(|value| value.as_str()) else {
            continue;
        };
        let content_lower = content.to_lowercase();
        collect_entity_matches_in_text(
            &content_lower,
            cached_entities,
            entity_ac,
            pattern_to_cached_idx,
            Some(&metadata_match_type),
            matched_names,
            matched_entities,
        );
    }
}

fn collect_entity_matches_in_text(
    text_lower: &str,
    cached_entities: &[CachedEntity],
    entity_ac: Option<&AhoCorasick>,
    pattern_to_cached_idx: &[usize],
    match_type_override: Option<&EntityMatchType>,
    matched_names: &mut HashSet<String>,
    matched_entities: &mut Vec<EntityMatch>,
) {
    if let Some(ac) = entity_ac {
        for mat in ac.find_iter(text_lower) {
            let Some(&cached_index) = pattern_to_cached_idx.get(mat.pattern().as_usize()) else {
                continue;
            };
            push_unique_entity_match(
                &cached_entities[cached_index],
                match_type_override,
                matched_names,
                matched_entities,
            );
        }
        return;
    }
    for cached in cached_entities {
        if text_lower.contains(&cached.name_lower) {
            push_unique_entity_match(cached, match_type_override, matched_names, matched_entities);
        }
    }
}

fn push_unique_entity_match(
    cached: &CachedEntity,
    match_type_override: Option<&EntityMatchType>,
    matched_names: &mut HashSet<String>,
    matched_entities: &mut Vec<EntityMatch>,
) {
    if !matched_names.insert(cached.name_lower.clone()) {
        return;
    }
    let mut matched_entity = cached.original.clone();
    if let Some(match_type) = match_type_override {
        matched_entity.match_type = match_type.clone();
    }
    matched_entities.push(matched_entity);
}

fn calculate_entity_boost(matched_entities: &[EntityMatch], entity_weight: f32) -> f32 {
    if matched_entities.is_empty() {
        return 0.0;
    }
    let match_count_f32 =
        u16::try_from(matched_entities.len()).map_or(f32::from(u16::MAX), f32::from);
    let avg_confidence = matched_entities
        .iter()
        .map(|entity| entity.confidence)
        .sum::<f32>()
        / match_count_f32;
    let match_bonus = match_count_f32 * entity_weight * 0.5;
    avg_confidence * entity_weight + match_bonus
}

/// Apply triple RRF fusion with entity awareness
///
/// Combines semantic, keyword, and entity signals using RRF fusion
#[must_use]
pub fn apply_triple_rrf(
    semantic_results: Vec<(String, f32)>,
    keyword_results: Vec<ToolSearchResult>,
    entity_results: Vec<EntityAwareSearchResult>,
    k: f32,
) -> Vec<EntityAwareSearchResult> {
    use std::collections::HashMap;

    let mut fusion_map: HashMap<String, EntityAwareSearchResult> = HashMap::new();

    // Process semantic results
    for (rank, (name, score)) in semantic_results.into_iter().enumerate() {
        let rrf = crate::rrf_term(k, rank);
        fusion_map.insert(
            name.clone(),
            EntityAwareSearchResult {
                base: HybridSearchResult {
                    tool_name: name.clone(),
                    rrf_score: rrf,
                    vector_score: score,
                    keyword_score: 0.0,
                },
                entity_matches: Vec::new(),
                boosted_score: rrf,
            },
        );
    }

    // Process keyword results
    for (rank, result) in keyword_results.into_iter().enumerate() {
        let rrf = crate::rrf_term(k, rank);
        let name = result.tool_name.clone();

        if let Some(existing) = fusion_map.get_mut(&name) {
            existing.base.rrf_score += rrf;
            existing.base.keyword_score = result.score;
            existing.boosted_score = existing.base.rrf_score;
        } else {
            fusion_map.insert(
                name.clone(),
                EntityAwareSearchResult {
                    base: HybridSearchResult {
                        tool_name: name,
                        rrf_score: rrf,
                        vector_score: 0.0,
                        keyword_score: result.score,
                    },
                    entity_matches: Vec::new(),
                    boosted_score: rrf,
                },
            );
        }
    }

    // Process entity-aware results (already boosted)
    for result in entity_results {
        let name = result.base.tool_name.clone();
        if let Some(existing) = fusion_map.get_mut(&name) {
            // Blend with existing
            existing.base.rrf_score = existing.base.rrf_score.midpoint(result.boosted_score);
            existing.entity_matches.extend(result.entity_matches);
        } else {
            fusion_map.insert(name, result);
        }
    }

    // Sort and return
    let mut results: Vec<_> = fusion_map.into_values().collect();
    results.sort_by(|a, b| {
        b.boosted_score
            .partial_cmp(&a.boosted_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

/// Constants for entity boosting
pub const ENTITY_WEIGHT: f32 = 0.3;
/// Minimum confidence score for an entity match to be considered
pub const ENTITY_CONFIDENCE_THRESHOLD: f32 = 0.7;
/// Maximum number of entity matches per result
pub const MAX_ENTITY_MATCHES: usize = 10;
