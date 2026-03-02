use std::collections::HashMap;
use std::path::Path;

use super::super::SkillScanner;
use super::super::references::scan_references;
use crate::skills::canonical::{CanonicalSkillPayload, CanonicalToolEntry};
use crate::skills::metadata::{ReferenceRecord, SkillMetadata, ToolRecord};

impl SkillScanner {
    /// Build the canonical per-skill payload (Rust schema) from metadata, tools, and filesystem.
    ///
    /// Fills `skill_tools` (each tool + `ref_key` -> path from references' `for_tools`) and
    /// `references` (`ref_id` -> `ReferenceRecord`). Used to wire the agreed schema to the scanner.
    #[must_use]
    pub fn build_canonical_payload(
        &self,
        metadata: SkillMetadata,
        tools: &[ToolRecord],
        skill_path: &Path,
    ) -> CanonicalSkillPayload {
        let _ = self;
        let skill_md_path = skill_path.join("SKILL.md").to_string_lossy().to_string();

        let references: HashMap<String, ReferenceRecord> =
            scan_references(skill_path, &metadata.skill_name)
                .into_iter()
                .map(|record| (record.ref_name.clone(), record))
                .collect();

        let skill_tools: HashMap<String, CanonicalToolEntry> = tools
            .iter()
            .map(|tool| {
                let skill_tool_references = references_for_tool(
                    references.values(),
                    tool.tool_name.as_str(),
                    metadata.skill_name.as_str(),
                );
                let entry = CanonicalToolEntry {
                    tool: tool.clone(),
                    skill_tool_references,
                };
                (tool.tool_name.clone(), entry)
            })
            .collect();

        CanonicalSkillPayload {
            skill_name: metadata.skill_name.clone(),
            skill_md_path,
            metadata,
            skill_tools,
            references,
        }
    }
}

fn references_for_tool<'a>(
    references: impl Iterator<Item = &'a ReferenceRecord>,
    tool_name: &str,
    skill_name: &str,
) -> HashMap<String, String> {
    let mut skill_tool_references = HashMap::new();
    for reference in references {
        if !applies_to_tool(reference, tool_name) {
            continue;
        }
        let ref_key = format!("{skill_name}.references.{}", reference.ref_name);
        skill_tool_references.insert(ref_key, reference.file_path.clone());
    }
    skill_tool_references
}

fn applies_to_tool(reference: &ReferenceRecord, tool_name: &str) -> bool {
    reference
        .for_tools
        .as_ref()
        .is_some_and(|tools| tools.iter().any(|candidate| candidate == tool_name))
}
