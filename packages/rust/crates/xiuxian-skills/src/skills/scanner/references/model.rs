use serde::Deserialize;

/// Frontmatter for reference docs under `references/*.md`.
/// Contract:
/// - `type` is mandatory for every reference document.
/// - all semantic fields must be nested under `metadata`.
#[derive(Debug, Deserialize)]
pub(super) struct ReferenceFrontmatter {
    #[serde(rename = "type")]
    pub(super) metadata_type: UnifiedMetadataType,
    pub(super) metadata: ReferenceMetadataBlock,
}

/// Contents of the `metadata` block in reference front matter.
#[derive(Debug, Deserialize)]
pub(super) struct ReferenceMetadataBlock {
    /// Tool(s) this reference is for, full name e.g. `git.smart_commit`
    /// (string or array).
    #[serde(default, rename = "for_tools")]
    pub(super) for_tools: Option<serde_yaml::Value>,
    #[serde(default)]
    pub(super) title: Option<String>,
    /// Persona role class required when `type = "persona"`.
    #[serde(default)]
    pub(super) role_class: Option<String>,
    /// Optional description (reserved for future use on `ReferenceRecord`).
    #[serde(default, rename = "description")]
    pub(super) _description: Option<String>,
    #[serde(default)]
    pub(super) routing_keywords: Option<Vec<String>>,
    #[serde(default)]
    pub(super) intents: Option<Vec<String>>,
}

/// Mandatory `type` discriminator for markdown frontmatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum UnifiedMetadataType {
    Skill,
    Persona,
    Knowledge,
    Template,
    Workflow,
    Prompt,
}
