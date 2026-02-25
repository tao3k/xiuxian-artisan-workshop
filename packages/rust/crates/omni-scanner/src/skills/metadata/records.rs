use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

/// Represents a discovered template file within a skill.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TemplateRecord {
    /// Name of the template.
    pub template_name: String,
    /// Description of the template's purpose.
    pub description: String,
    /// Skill this template belongs to.
    pub skill_name: String,
    /// Path to the template file.
    pub file_path: String,
    /// Variable names used in the template.
    pub variables: Vec<String>,
    /// Preview of the template content.
    #[serde(default)]
    pub content_preview: String,
    /// Keywords for template discovery.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Hash of the template file.
    #[serde(default)]
    pub file_hash: String,
}

impl TemplateRecord {
    /// Creates a new `TemplateRecord` with required fields.
    #[must_use]
    pub fn new(
        template_name: String,
        description: String,
        skill_name: String,
        file_path: String,
        variables: Vec<String>,
    ) -> Self {
        Self {
            template_name,
            description,
            skill_name,
            file_path,
            variables,
            content_preview: String::new(),
            keywords: Vec::new(),
            file_hash: String::new(),
        }
    }
}

// =============================================================================
// Reference Record
// =============================================================================

/// Deserialize a single string or array of strings into `Vec<String>`.
fn de_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;
    struct StringOrVec;
    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string or array of strings")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Vec<String>, E> {
            Ok(if v.is_empty() {
                vec![]
            } else {
                vec![v.to_string()]
            })
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Vec<String>, A::Error> {
            let mut out = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                if !s.is_empty() {
                    out.push(s);
                }
            }
            Ok(out)
        }
    }
    deserializer.deserialize_any(StringOrVec)
}

/// Deserialize `Option` of string or array of strings into `Option<Vec<String>>`.
fn de_opt_string_or_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;
    struct OptStringOrVec;
    impl<'de> Visitor<'de> for OptStringOrVec {
        type Value = Option<Vec<String>>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("optional string or array of strings")
        }
        fn visit_none<E: de::Error>(self) -> Result<Option<Vec<String>>, E> {
            Ok(None)
        }
        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            d: D2,
        ) -> Result<Option<Vec<String>>, D2::Error> {
            let v = de_string_or_vec(d)?;
            Ok(if v.is_empty() { None } else { Some(v) })
        }
    }
    deserializer.deserialize_option(OptStringOrVec)
}

/// Represents a reference document discovered in a skill's `references/` directory.
///
/// See `docs/reference/skill-data-hierarchy-and-references.md`: references are
/// subordinate to the skill; each may be tied to one or more skills/tools (e.g. graph docs).
/// `for_skills` and `for_tools` are lists so one reference can apply to multiple skills or tools.
#[derive(Debug, Clone, Deserialize, Serialize, SchemarsJsonSchema, PartialEq, Eq)]
pub struct ReferenceRecord {
    /// Name of the reference (e.g. filename stem).
    pub ref_name: String,
    /// Title of the reference document (from frontmatter or first heading).
    pub title: String,
    /// Primary skill (first of `for_skills` or parent path); kept for backward compatibility.
    pub skill_name: String,
    /// Path to the reference file (relative to repo or absolute).
    pub file_path: String,
    /// List of skills this reference applies to; derived from `for_tools` (skill part of each `skill.tool`) or from path when `for_tools` is absent.
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub for_skills: Vec<String>,
    /// If set, list of full tool names this reference is for (e.g. `["git.smart_commit", "researcher.run_research_graph"]`).
    #[serde(default, deserialize_with = "de_opt_string_or_vec", alias = "for_tool")]
    pub for_tools: Option<Vec<String>>,
    /// Document type: `"reference"` for references/*.md; `"comprehensive"` reserved for SKILL.md.
    #[serde(default = "default_ref_doc_type")]
    pub doc_type: String,
    /// Preview of the content.
    #[serde(default)]
    pub content_preview: String,
    /// Keywords for reference discovery.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Section headings in the document.
    #[serde(default)]
    pub sections: Vec<String>,
    /// Hash of the reference file.
    #[serde(default)]
    pub file_hash: String,
}

fn default_ref_doc_type() -> String {
    "reference".to_string()
}

impl ReferenceRecord {
    /// Creates a new `ReferenceRecord` with required fields.
    #[must_use]
    pub fn new(ref_name: String, title: String, skill_name: String, file_path: String) -> Self {
        let for_skills = if skill_name.is_empty() {
            vec![]
        } else {
            vec![skill_name.clone()]
        };
        Self {
            ref_name,
            title,
            skill_name,
            file_path,
            for_skills,
            for_tools: None,
            doc_type: "reference".to_string(),
            content_preview: String::new(),
            keywords: Vec::new(),
            sections: Vec::new(),
            file_hash: String::new(),
        }
    }

    /// Builder: set optional list of tools this reference is for.
    #[must_use]
    pub fn with_for_tools(mut self, for_tools: Option<Vec<String>>) -> Self {
        self.for_tools = for_tools;
        self
    }

    /// Returns true if this reference applies to the given full tool name.
    #[must_use]
    pub fn applies_to_tool(&self, full_tool_name: &str) -> bool {
        self.for_tools
            .as_ref()
            .is_some_and(|v| v.iter().any(|t| t.as_str() == full_tool_name))
    }
}

// =============================================================================
// Asset Record
// =============================================================================

/// Represents an asset file discovered in a skill.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AssetRecord {
    /// Name of the asset.
    pub asset_name: String,
    /// Title of the asset.
    pub title: String,
    /// Skill this asset belongs to.
    pub skill_name: String,
    /// Path to the asset file.
    pub file_path: String,
    /// Preview of the asset content.
    #[serde(default)]
    pub content_preview: String,
    /// Keywords for asset discovery.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Hash of the asset file.
    #[serde(default)]
    pub file_hash: String,
}

impl AssetRecord {
    /// Creates a new `AssetRecord` with required fields.
    #[must_use]
    pub fn new(asset_name: String, title: String, skill_name: String, file_path: String) -> Self {
        Self {
            asset_name,
            title,
            skill_name,
            file_path,
            content_preview: String::new(),
            keywords: Vec::new(),
            file_hash: String::new(),
        }
    }
}

// =============================================================================
// Data Record
// =============================================================================

/// Represents a data file discovered in a skill.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DataRecord {
    /// Name of the data.
    pub data_name: String,
    /// Format of the data (e.g., "json", "csv").
    pub format: String,
    /// Skill this data belongs to.
    pub skill_name: String,
    /// Path to the data file.
    pub file_path: String,
    /// Field names in the data.
    pub fields: Vec<String>,
    /// Preview of the data content.
    #[serde(default)]
    pub content_preview: String,
    /// Keywords for data discovery.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Hash of the data file.
    #[serde(default)]
    pub file_hash: String,
}

impl DataRecord {
    /// Creates a new `DataRecord` with required fields.
    #[must_use]
    pub fn new(
        data_name: String,
        format: String,
        skill_name: String,
        file_path: String,
        fields: Vec<String>,
    ) -> Self {
        Self {
            data_name,
            format,
            skill_name,
            file_path,
            fields,
            content_preview: String::new(),
            keywords: Vec::new(),
            file_hash: String::new(),
        }
    }
}

// =============================================================================
// Test Record
// =============================================================================

/// Represents a test file discovered in a skill.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TestRecord {
    /// Name of the test.
    pub test_name: String,
    /// Skill this test belongs to.
    pub skill_name: String,
    /// Path to the test file.
    pub file_path: String,
    /// Names of test functions.
    pub test_functions: Vec<String>,
    /// Names of test classes.
    pub test_classes: Vec<String>,
    /// Docstring of the test module.
    #[serde(default)]
    pub docstring: String,
    /// Keywords for test discovery.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Hash of the test file.
    #[serde(default)]
    pub file_hash: String,
}

impl TestRecord {
    /// Creates a new `TestRecord` with required fields.
    #[must_use]
    pub fn new(
        test_name: String,
        skill_name: String,
        file_path: String,
        test_functions: Vec<String>,
        test_classes: Vec<String>,
    ) -> Self {
        Self {
            test_name,
            skill_name,
            file_path,
            test_functions,
            test_classes,
            docstring: String::new(),
            keywords: Vec::new(),
            file_hash: String::new(),
        }
    }
}

/// Represents a discovered MCP Resource from @`skill_resource` decorated functions.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResourceRecord {
    /// Resource name (from decorator or function name).
    pub name: String,
    /// Human-readable description of the resource.
    pub description: String,
    /// Full resource URI (e.g., `<omni://skill/knowledge/graph_stats>`).
    pub resource_uri: String,
    /// MIME type of the resource content.
    pub mime_type: String,
    /// Name of the skill this resource belongs to.
    pub skill_name: String,
    /// File path where the resource provider is defined.
    pub file_path: String,
    /// Name of the function implementing this resource.
    pub function_name: String,
    /// Hash of the source file for change detection.
    pub file_hash: String,
}

impl ResourceRecord {
    /// Create a new `ResourceRecord`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        description: String,
        resource_uri: String,
        mime_type: String,
        skill_name: String,
        file_path: String,
        function_name: String,
        file_hash: String,
    ) -> Self {
        Self {
            name,
            description,
            resource_uri,
            mime_type,
            skill_name,
            file_path,
            function_name,
            file_hash,
        }
    }
}

// =============================================================================
// Prompt Record - MCP Prompt metadata
// =============================================================================

/// Represents a discovered MCP Prompt from @prompt decorated functions.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PromptRecord {
    /// Prompt name (from decorator or function name).
    pub name: String,
    /// Human-readable description of the prompt.
    pub description: String,
    /// Name of the skill this prompt belongs to.
    pub skill_name: String,
    /// File path where the prompt is defined.
    pub file_path: String,
    /// Name of the function implementing this prompt.
    pub function_name: String,
    /// Hash of the source file for change detection.
    pub file_hash: String,
    /// Parameter names for the prompt template.
    #[serde(default)]
    pub parameters: Vec<String>,
}

impl PromptRecord {
    /// Create a new `PromptRecord`.
    #[must_use]
    pub fn new(
        name: String,
        description: String,
        skill_name: String,
        file_path: String,
        function_name: String,
        file_hash: String,
        parameters: Vec<String>,
    ) -> Self {
        Self {
            name,
            description,
            skill_name,
            file_path,
            function_name,
            file_hash,
            parameters,
        }
    }
}
