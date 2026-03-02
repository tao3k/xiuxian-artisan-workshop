use crate::skills::metadata::{ReferencePath, SkillMetadata};

use super::super::super::frontmatter::{SkillFrontmatter, SkillMetadataBlock};

#[derive(Default)]
struct ParsedMetadataFields {
    version: String,
    routing_keywords: Vec<String>,
    authors: Vec<String>,
    intents: Vec<String>,
    require_refs: Vec<String>,
    repository: String,
    permissions: Vec<String>,
}

pub(super) fn build_skill_metadata(
    skill_name: String,
    frontmatter_data: Option<SkillFrontmatter>,
) -> SkillMetadata {
    let Some(frontmatter_data) = frontmatter_data else {
        return SkillMetadata {
            skill_name,
            ..SkillMetadata::default()
        };
    };

    let semantic_name = frontmatter_data
        .name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(|| skill_name.clone(), ToString::to_string);

    let fields = parse_metadata_fields(frontmatter_data.metadata.as_ref(), &skill_name);

    SkillMetadata {
        skill_name: semantic_name,
        version: fields.version,
        description: frontmatter_data.description.unwrap_or_default(),
        routing_keywords: fields.routing_keywords,
        authors: fields.authors,
        intents: fields.intents,
        require_refs: fields
            .require_refs
            .into_iter()
            .filter_map(|reference| ReferencePath::new(reference).ok())
            .collect(),
        repository: fields.repository,
        permissions: fields.permissions,
    }
}

fn parse_metadata_fields(
    metadata: Option<&SkillMetadataBlock>,
    skill_name: &str,
) -> ParsedMetadataFields {
    let Some(metadata) = metadata else {
        log::warn!("No metadata block found in SKILL.md for: {skill_name}");
        return ParsedMetadataFields::default();
    };

    ParsedMetadataFields {
        version: metadata.version.clone().unwrap_or_default(),
        routing_keywords: metadata.routing_keywords.clone().unwrap_or_default(),
        authors: parse_authors(metadata),
        intents: metadata.intents.clone().unwrap_or_default(),
        require_refs: metadata.require_refs.clone().unwrap_or_default(),
        repository: metadata.source.clone().unwrap_or_default(),
        permissions: metadata.permissions.clone().unwrap_or_default(),
    }
}

fn parse_authors(metadata: &SkillMetadataBlock) -> Vec<String> {
    if let Some(authors) = &metadata.authors {
        return authors.clone();
    }

    if let Some(author) = &metadata.author {
        return vec![author.clone()];
    }

    Vec::new()
}
