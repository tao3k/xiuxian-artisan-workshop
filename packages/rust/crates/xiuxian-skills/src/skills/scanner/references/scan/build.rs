use crate::frontmatter::strict_parse;
use crate::skills::metadata::ReferenceRecord;

use super::super::model::{ReferenceFrontmatter, ReferenceMetadataBlock, UnifiedMetadataType};
use super::super::values::{skills_from_tool_list, yaml_value_to_opt_string_vec};

pub(super) fn parse_reference_metadata_strict(
    content: &str,
    file_path: &std::path::Path,
) -> Result<ReferenceMetadataBlock, String> {
    let parsed: ReferenceFrontmatter = strict_parse(content).map_err(|error| {
        format!(
            "invalid YAML frontmatter in reference markdown {}: {}",
            file_path.display(),
            error
        )
    })?;
    validate_unified_metadata_contract(parsed.metadata_type, &parsed.metadata, file_path)?;
    Ok(parsed.metadata)
}

fn validate_unified_metadata_contract(
    metadata_type: UnifiedMetadataType,
    metadata: &ReferenceMetadataBlock,
    file_path: &std::path::Path,
) -> Result<(), String> {
    if matches!(metadata_type, UnifiedMetadataType::Persona)
        && metadata
            .role_class
            .as_deref()
            .map(str::trim)
            .is_none_or(str::is_empty)
    {
        return Err(format!(
            "invalid persona metadata in {}: `metadata.role_class` is required when type=persona",
            file_path.display()
        ));
    }

    Ok(())
}

pub(super) fn build_reference_record(
    ref_name: String,
    file_path: String,
    fallback_skill_name: &str,
    metadata: Option<&ReferenceMetadataBlock>,
) -> ReferenceRecord {
    let for_tools = metadata
        .and_then(|meta| meta.for_tools.as_ref())
        .and_then(yaml_value_to_opt_string_vec);

    let for_skills = derive_for_skills(for_tools.as_ref(), fallback_skill_name);
    let primary_skill_name = for_skills
        .first()
        .cloned()
        .unwrap_or_else(|| fallback_skill_name.to_string());
    let title = metadata
        .and_then(|meta| meta.title.clone())
        .unwrap_or_else(|| ref_name.clone());

    let mut record = ReferenceRecord::new(ref_name, title, primary_skill_name, file_path);
    record.for_skills = for_skills;
    record.for_tools = for_tools;

    if let Some(meta) = metadata {
        let keywords = merged_keywords(meta);
        if !keywords.is_empty() {
            record.keywords = keywords;
        }
    }

    record
}

fn derive_for_skills(for_tools: Option<&Vec<String>>, fallback_skill_name: &str) -> Vec<String> {
    for_tools
        .map(|tools| skills_from_tool_list(tools))
        .filter(|skills| !skills.is_empty())
        .unwrap_or_else(|| vec![fallback_skill_name.to_string()])
}

fn merged_keywords(metadata: &ReferenceMetadataBlock) -> Vec<String> {
    let mut keywords = Vec::new();
    if let Some(routing_keywords) = metadata.routing_keywords.as_ref() {
        keywords.extend(routing_keywords.clone());
    }
    if let Some(intents) = metadata.intents.as_ref() {
        keywords.extend(intents.clone());
    }
    keywords
}
