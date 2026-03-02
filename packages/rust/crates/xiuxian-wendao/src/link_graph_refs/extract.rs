use super::model::LinkGraphEntityRef;
use super::regex::{WIKILINK_REGEX, WIKILINK_REGEX_EXACT};
use std::collections::HashSet;

/// Extract all entity references from note content.
///
/// Supports:
/// - `[[EntityName]]` - reference by name
/// - `[[EntityName#type]]` - reference with type hint (`rust`, `py`, `pattern`, etc.)
/// - `[[EntityName|alias]]` - reference with alias (alias is ignored)
///
/// # Arguments
///
/// * `content` - The note body content to search
///
/// # Returns
///
/// Vector of extracted entity references (deduplicated)
pub fn extract_entity_refs(content: &str) -> Vec<LinkGraphEntityRef> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut refs: Vec<LinkGraphEntityRef> = Vec::new();

    for caps in WIKILINK_REGEX.captures_iter(content) {
        let Some(name_match) = caps.get(1) else {
            continue;
        };
        let name = name_match.as_str().trim().to_string();
        let entity_type = caps.get(2).map(|m| m.as_str().trim().to_string());
        let Some(original_match) = caps.get(0) else {
            continue;
        };
        let original = original_match.as_str().to_string();

        // Deduplicate by name
        if !seen.contains(&name) {
            seen.insert(name.clone());
            refs.push(LinkGraphEntityRef::new(name, entity_type, original));
        }
    }

    refs
}

/// Extract entity references from multiple notes (batch processing).
///
/// More efficient than calling `extract_entity_refs` individually
/// when processing many notes.
///
/// # Arguments
///
/// * `notes` - Vector of (`note_id`, content) tuples
///
/// # Returns
///
/// Vector of (`note_id`, `entity_references`) tuples
#[must_use]
pub fn extract_entity_refs_batch<'a>(
    notes: &[(&'a str, &'a str)],
) -> Vec<(&'a str, Vec<LinkGraphEntityRef>)> {
    notes
        .iter()
        .map(|(note_id, content)| (*note_id, extract_entity_refs(content)))
        .collect()
}

/// Find notes that reference a given entity name.
///
/// # Arguments
///
/// * `entity_name` - The entity name to search for
/// * `contents` - Vector of (`note_id`, content) tuples to search
///
/// # Returns
///
/// Vector of note IDs that reference the entity
#[must_use]
pub fn find_notes_referencing_entity<'a>(
    entity_name: &str,
    contents: &[(&'a str, &'a str)],
) -> Vec<&'a str> {
    let lower_name = entity_name.to_lowercase();
    let wikilink_pattern = format!("[[{entity_name}]]");
    let wikilink_pattern_typed = format!("[[{entity_name}#");

    contents
        .iter()
        .filter(|(_, content)| {
            let lower = content.to_lowercase();
            lower.contains(&lower_name)
                || lower.contains(&wikilink_pattern.to_lowercase())
                || lower.contains(&wikilink_pattern_typed.to_lowercase())
        })
        .map(|(note_id, _)| *note_id)
        .collect()
}

/// Count entity references in content.
#[must_use]
pub fn count_entity_refs(content: &str) -> usize {
    WIKILINK_REGEX.captures_iter(content).count()
}

/// Validate entity reference format.
#[must_use]
pub fn is_valid_entity_ref(text: &str) -> bool {
    WIKILINK_REGEX_EXACT.is_match(text)
}

/// Parse a single entity reference string.
#[must_use]
pub fn parse_entity_ref(text: &str) -> Option<LinkGraphEntityRef> {
    let caps = WIKILINK_REGEX_EXACT.captures(text)?;
    let name = caps.get(1)?.as_str().trim().to_string();
    Some(LinkGraphEntityRef::new(
        name,
        caps.get(2).map(|m| m.as_str().trim().to_string()),
        text.to_string(),
    ))
}
