use super::attachments::attachments_for_parsed_note;
use super::constants::DEFAULT_EXCLUDED_DIR_NAMES;
use super::filters::{merge_excluded_dirs, normalize_include_dir, should_skip_entry};
use super::graphmem::sync_graphmem_state_best_effort;
use crate::link_graph::index::{IndexedSection, LinkGraphIndex, doc_sort_key};
use crate::link_graph::models::{LinkGraphAttachment, LinkGraphDocument};
use crate::link_graph::parser::{ParsedNote, is_supported_note, normalize_alias, parse_note};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use xiuxian_skills::SkillScanner;

struct NormalizedDirectoryFilters {
    include_dirs: Vec<String>,
    excluded_dirs: Vec<String>,
    included: HashSet<String>,
    excluded: HashSet<String>,
}

struct ParsedNoteMaps {
    docs_by_id: HashMap<String, LinkGraphDocument>,
    sections_by_doc: HashMap<String, Vec<IndexedSection>>,
    attachments_by_doc: HashMap<String, Vec<LinkGraphAttachment>>,
    alias_to_doc_id: HashMap<String, String>,
}

struct GraphEdges {
    outgoing: HashMap<String, HashSet<String>>,
    incoming: HashMap<String, HashSet<String>>,
    edge_count: usize,
}

struct CandidateScan {
    candidate_paths: Vec<PathBuf>,
    skill_descriptor_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct SkillPromotionMetadata {
    semantic_name: String,
    routing_keywords: Vec<String>,
    intents: Vec<String>,
}

fn canonicalize_root_dir(root_dir: &Path) -> Result<PathBuf, String> {
    let root = root_dir
        .canonicalize()
        .map_err(|e| format!("invalid notebook root '{}': {e}", root_dir.display()))?;
    if !root.is_dir() {
        return Err(format!(
            "notebook root is not a directory: {}",
            root.display()
        ));
    }
    Ok(root)
}

fn normalize_directory_filters(
    include_dirs: &[String],
    excluded_dirs: &[String],
) -> NormalizedDirectoryFilters {
    let normalized_include_dirs: Vec<String> = include_dirs
        .iter()
        .filter_map(|path| normalize_include_dir(path))
        .collect();
    let normalized_excluded_dirs: Vec<String> =
        merge_excluded_dirs(excluded_dirs, DEFAULT_EXCLUDED_DIR_NAMES);
    let included: HashSet<String> = normalized_include_dirs.iter().cloned().collect();
    let excluded: HashSet<String> = normalized_excluded_dirs.iter().cloned().collect();

    NormalizedDirectoryFilters {
        include_dirs: normalized_include_dirs,
        excluded_dirs: normalized_excluded_dirs,
        included,
        excluded,
    }
}

fn collect_candidate_note_paths(
    root: &Path,
    included: &HashSet<String>,
    excluded: &HashSet<String>,
) -> CandidateScan {
    let mut candidate_paths: Vec<PathBuf> = Vec::new();
    let mut skill_descriptor_paths: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            !should_skip_entry(
                entry.path(),
                entry.file_type().is_dir(),
                root,
                included,
                excluded,
            )
        })
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !entry.file_type().is_file() || !is_supported_note(path) {
            continue;
        }
        if is_skill_descriptor_path(path) {
            skill_descriptor_paths.push(path.to_path_buf());
        }
        candidate_paths.push(path.to_path_buf());
    }
    skill_descriptor_paths.sort();
    skill_descriptor_paths.dedup();
    CandidateScan {
        candidate_paths,
        skill_descriptor_paths,
    }
}

fn parse_candidate_notes(root: &Path, candidate_paths: Vec<PathBuf>) -> Vec<ParsedNote> {
    candidate_paths
        .into_par_iter()
        .filter_map(|path| {
            let content = std::fs::read_to_string(&path).ok()?;
            parse_note(&path, root, &content)
        })
        .collect()
}

fn is_skill_descriptor_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("SKILL.md") || name == "skill.md")
}

fn normalize_skill_tag_token(raw: &str) -> Option<String> {
    let normalized = raw
        .trim()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_ascii_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

fn collect_skill_promotions(
    root: &Path,
    skill_descriptor_paths: &[PathBuf],
) -> HashMap<String, SkillPromotionMetadata> {
    let scanner = SkillScanner::new();
    let mut promotions = HashMap::new();
    for skill_doc in skill_descriptor_paths {
        let Some(skill_root) = skill_doc.parent() else {
            continue;
        };
        let metadata = match scanner.scan_skill(skill_root, None) {
            Ok(Some(metadata)) => metadata,
            Ok(None) => continue,
            Err(error) => {
                log::warn!(
                    "skip skill promotion for {} due to semantic validation error: {}",
                    skill_root.display(),
                    error
                );
                continue;
            }
        };
        let Some(relative) = skill_doc
            .strip_prefix(root)
            .ok()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
        else {
            continue;
        };
        let semantic_name = metadata.skill_name.trim().to_ascii_lowercase();
        if semantic_name.is_empty() {
            continue;
        }
        let routing_keywords = metadata
            .routing_keywords
            .iter()
            .filter_map(|keyword| normalize_skill_tag_token(keyword))
            .collect::<Vec<_>>();
        let intents = metadata
            .intents
            .iter()
            .filter_map(|intent| normalize_skill_tag_token(intent))
            .collect::<Vec<_>>();
        promotions.insert(
            relative,
            SkillPromotionMetadata {
                semantic_name,
                routing_keywords,
                intents,
            },
        );
    }
    promotions
}

fn apply_skill_promotions(
    parsed_notes: &mut [ParsedNote],
    promotions_by_path: &HashMap<String, SkillPromotionMetadata>,
) {
    parsed_notes.par_iter_mut().for_each(|parsed| {
        let Some(metadata) = promotions_by_path.get(parsed.doc.path.as_str()) else {
            return;
        };
        if parsed.doc.doc_type.is_none() {
            parsed.doc.doc_type = Some("skill".to_string());
        }
        let mut tags = parsed.doc.tags.clone();
        tags.push("skill".to_string());
        tags.push(format!("skill:{}", metadata.semantic_name));
        tags.extend(
            metadata
                .routing_keywords
                .iter()
                .map(|keyword| format!("routing:{keyword}")),
        );
        tags.extend(
            metadata
                .intents
                .iter()
                .map(|intent| format!("intent:{intent}")),
        );
        tags.sort();
        tags.dedup();
        parsed.doc.tags = tags;
        parsed.doc.tags_lower = parsed
            .doc
            .tags
            .iter()
            .map(|tag| tag.to_lowercase())
            .collect();
    });
}

fn build_note_maps(parsed_notes: &[ParsedNote]) -> ParsedNoteMaps {
    let mut docs_by_id: HashMap<String, LinkGraphDocument> = HashMap::new();
    let mut sections_by_doc: HashMap<String, Vec<IndexedSection>> = HashMap::new();
    let mut attachments_by_doc: HashMap<String, Vec<LinkGraphAttachment>> = HashMap::new();
    let mut alias_to_doc_id: HashMap<String, String> = HashMap::new();

    for parsed in parsed_notes {
        let doc = &parsed.doc;
        docs_by_id.insert(doc.id.clone(), doc.clone());
        let indexed_sections = parsed
            .sections
            .iter()
            .map(IndexedSection::from_parsed)
            .collect::<Vec<IndexedSection>>();
        sections_by_doc.insert(doc.id.clone(), indexed_sections);
        attachments_by_doc.insert(doc.id.clone(), attachments_for_parsed_note(parsed));

        for alias in [&doc.id, &doc.path, &doc.stem] {
            let key = normalize_alias(alias);
            if key.is_empty() {
                continue;
            }
            alias_to_doc_id.entry(key).or_insert_with(|| doc.id.clone());
        }
        for tag in &doc.tags {
            let Some(skill_alias) = tag.strip_prefix("skill:") else {
                continue;
            };
            let key = normalize_alias(skill_alias);
            if key.is_empty() {
                continue;
            }
            alias_to_doc_id.entry(key).or_insert_with(|| doc.id.clone());
        }
    }

    ParsedNoteMaps {
        docs_by_id,
        sections_by_doc,
        attachments_by_doc,
        alias_to_doc_id,
    }
}

fn build_graph_edges(
    parsed_notes: Vec<ParsedNote>,
    alias_to_doc_id: &HashMap<String, String>,
) -> GraphEdges {
    let mut outgoing: HashMap<String, HashSet<String>> = HashMap::new();
    let mut incoming: HashMap<String, HashSet<String>> = HashMap::new();
    let mut edge_count = 0usize;

    for parsed in parsed_notes {
        let from_id = parsed.doc.id;
        for raw_target in parsed.link_targets {
            let normalized = normalize_alias(&raw_target);
            if normalized.is_empty() {
                continue;
            }
            let Some(to_id) = alias_to_doc_id.get(&normalized).cloned() else {
                continue;
            };
            if to_id == from_id {
                continue;
            }
            let inserted = outgoing
                .entry(from_id.clone())
                .or_default()
                .insert(to_id.clone());
            if inserted {
                incoming.entry(to_id).or_default().insert(from_id.clone());
                edge_count += 1;
            }
        }
    }

    GraphEdges {
        outgoing,
        incoming,
        edge_count,
    }
}

fn build_index_from_parts(
    root: PathBuf,
    filters: NormalizedDirectoryFilters,
    note_maps: ParsedNoteMaps,
    edges: GraphEdges,
) -> LinkGraphIndex {
    let rank_by_id =
        LinkGraphIndex::compute_rank_by_id(&note_maps.docs_by_id, &edges.incoming, &edges.outgoing);
    let mut index = LinkGraphIndex {
        root,
        include_dirs: filters.include_dirs,
        excluded_dirs: filters.excluded_dirs,
        docs_by_id: note_maps.docs_by_id,
        passages_by_id: HashMap::new(),
        sections_by_doc: note_maps.sections_by_doc,
        attachments_by_doc: note_maps.attachments_by_doc,
        alias_to_doc_id: note_maps.alias_to_doc_id,
        outgoing: edges.outgoing,
        incoming: edges.incoming,
        rank_by_id,
        edge_count: edges.edge_count,
    };
    index.rebuild_all_passages();
    index
}

impl LinkGraphIndex {
    /// Build index with excluded directory names (e.g. ".cache", ".git").
    ///
    /// # Errors
    ///
    /// Returns an error when index construction fails.
    pub fn build_with_excluded_dirs(
        root_dir: &Path,
        excluded_dirs: &[String],
    ) -> Result<Self, String> {
        let index = Self::build_with_filters(root_dir, &[], excluded_dirs)?;
        sync_graphmem_state_best_effort(&index);
        Ok(index)
    }

    /// Build index with include/exclude directory filters relative to notebook root.
    ///
    /// # Errors
    ///
    /// Returns an error when root path validation fails.
    pub fn build_with_filters(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
    ) -> Result<Self, String> {
        let root = canonicalize_root_dir(root_dir)?;
        let filters = normalize_directory_filters(include_dirs, excluded_dirs);
        let candidate_scan =
            collect_candidate_note_paths(&root, &filters.included, &filters.excluded);
        let mut parsed_notes = parse_candidate_notes(&root, candidate_scan.candidate_paths);
        let promotions_by_path =
            collect_skill_promotions(&root, &candidate_scan.skill_descriptor_paths);
        apply_skill_promotions(&mut parsed_notes, &promotions_by_path);

        parsed_notes.sort_by(|left, right| doc_sort_key(&left.doc).cmp(&doc_sort_key(&right.doc)));

        let note_maps = build_note_maps(&parsed_notes);
        let graph_edges = build_graph_edges(parsed_notes, &note_maps.alias_to_doc_id);
        Ok(build_index_from_parts(
            root,
            filters,
            note_maps,
            graph_edges,
        ))
    }
}
