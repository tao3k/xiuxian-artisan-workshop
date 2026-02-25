/// YAML frontmatter structure for knowledge documents.
///
/// ```yaml
/// ---
/// title: "Git Commit Best Practices"
/// description: "Guidelines for writing effective commit messages"
/// category: "pattern"
/// tags: ["git", "commit", "best-practices"]
/// authors: ["developer@example.com"]
/// version: "1.0.0"
/// ---
/// ```
#[derive(Debug, Default, serde::Deserialize)]
pub(super) struct KnowledgeFrontmatter {
    #[serde(default)]
    pub(super) title: Option<String>,
    #[serde(default)]
    pub(super) description: Option<String>,
    #[serde(default)]
    pub(super) category: Option<String>,
    #[serde(default)]
    pub(super) tags: Option<Vec<String>>,
    #[serde(default)]
    pub(super) authors: Option<Vec<String>>,
    #[serde(default)]
    pub(super) source: Option<String>,
    #[serde(default)]
    pub(super) version: Option<String>,
}
