use pyo3::prelude::*;

use crate::link_graph_refs::{
    count_entity_refs, extract_entity_refs, get_ref_stats, is_valid_entity_ref, parse_entity_ref,
};

use super::py_types::{PyLinkGraphEntityRef, PyLinkGraphRefStats};

/// Extract entity references from note content (Rust-accelerated).
#[pyfunction]
#[pyo3(signature = (content))]
#[must_use]
pub fn link_graph_extract_entity_refs(content: &str) -> Vec<PyLinkGraphEntityRef> {
    extract_entity_refs(content)
        .into_iter()
        .map(|item| PyLinkGraphEntityRef { inner: item })
        .collect()
}

/// Get entity reference statistics for content.
#[pyfunction]
#[pyo3(signature = (content))]
#[must_use]
pub fn link_graph_get_ref_stats(content: &str) -> PyLinkGraphRefStats {
    let stats = get_ref_stats(content);
    PyLinkGraphRefStats { inner: stats }
}

/// Parse a single entity reference string.
#[pyfunction]
#[pyo3(signature = (text))]
#[must_use]
pub fn link_graph_parse_entity_ref(text: &str) -> Option<PyLinkGraphEntityRef> {
    parse_entity_ref(text).map(|item| PyLinkGraphEntityRef { inner: item })
}

/// Check if text is a valid entity reference.
#[pyfunction]
#[pyo3(signature = (text))]
#[must_use]
pub fn link_graph_is_valid_ref(text: &str) -> bool {
    is_valid_entity_ref(text)
}

/// Count entity references in content.
#[pyfunction]
#[pyo3(signature = (content))]
#[must_use]
pub fn link_graph_count_refs(content: &str) -> usize {
    count_entity_refs(content)
}

/// Find notes referencing an entity (Python-friendly API).
#[pyfunction]
#[pyo3(signature = (entity_name, contents))]
#[must_use]
pub fn link_graph_find_referencing_notes(entity_name: &str, contents: Vec<String>) -> Vec<usize> {
    let lower_name = entity_name.to_lowercase();
    let wikilink_pattern = format!("[[{entity_name}]]");
    let wikilink_pattern_typed = format!("[[{entity_name}#");
    let wikilink_pattern_lower = wikilink_pattern.to_lowercase();
    let wikilink_pattern_typed_lower = wikilink_pattern_typed.to_lowercase();

    contents
        .into_iter()
        .enumerate()
        .filter_map(|(idx, content)| {
            let lower = content.to_lowercase();
            if lower.contains(&lower_name)
                || lower.contains(&wikilink_pattern_lower)
                || lower.contains(&wikilink_pattern_typed_lower)
            {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}
