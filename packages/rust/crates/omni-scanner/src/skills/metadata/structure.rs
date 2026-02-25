use serde::{Deserialize, Serialize};

/// Represents the canonical skill structure as defined in settings.yaml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillStructure {
    /// Required items that must exist in a valid skill.
    pub required: Vec<StructureItem>,
    /// Default items that are created when generating a new skill.
    pub default: Vec<StructureItem>,
    /// Optional items that may be present.
    #[serde(default)]
    pub optional: Vec<StructureItem>,
}

/// A single item in the skill structure definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructureItem {
    /// Path or pattern for the item.
    pub path: String,
    /// Description of what this item represents.
    #[serde(default)]
    pub description: String,
    /// Type of item ("file" or "dir").
    #[serde(default)]
    pub item_type: String,
}

impl Default for SkillStructure {
    fn default() -> Self {
        Self {
            required: vec![StructureItem {
                path: "SKILL.md".to_string(),
                description: "Skill metadata".to_string(),
                item_type: "file".to_string(),
            }],
            default: vec![
                StructureItem {
                    path: "scripts/".to_string(),
                    description: "Standalone executables".to_string(),
                    item_type: "dir".to_string(),
                },
                StructureItem {
                    path: "templates/".to_string(),
                    description: "Jinja2 templates".to_string(),
                    item_type: "dir".to_string(),
                },
                StructureItem {
                    path: "references/".to_string(),
                    description: "Markdown documentation".to_string(),
                    item_type: "dir".to_string(),
                },
            ],
            optional: Vec::new(),
        }
    }
}

impl SkillStructure {
    /// Creates a new `SkillStructure` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the paths of all default directories.
    #[must_use]
    pub fn script_directories(&self) -> Vec<&str> {
        self.default
            .iter()
            .filter(|i| i.item_type == "dir")
            .map(|i| i.path.trim_end_matches('/'))
            .collect()
    }
}
