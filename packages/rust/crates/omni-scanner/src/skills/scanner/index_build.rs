use std::collections::HashMap;
use std::path::Path;

use super::SkillScanner;
use super::references::scan_references;
use super::rules::parse_rules_toml;
use crate::skills::canonical::{CanonicalSkillPayload, CanonicalToolEntry};
use crate::skills::metadata::{
    IndexToolEntry, ReferenceRecord, SkillIndexEntry, SkillMetadata, ToolRecord,
};

impl SkillScanner {
    /// Build a full `SkillIndexEntry` from metadata and tools.
    ///
    /// Combines skill metadata from SKILL.md frontmatter with discovered
    /// tools from the script scanner to create a complete skill index entry.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Skill metadata from SKILL.md
    /// * `tools` - Tools discovered in the skill's scripts directory
    /// * `skill_path` - Path to the skill directory
    #[must_use]
    pub fn build_index_entry(
        &self,
        metadata: SkillMetadata,
        tools: &[ToolRecord],
        skill_path: &Path,
    ) -> SkillIndexEntry {
        let _ = self;
        let path = format!("assets/skills/{}", metadata.skill_name);

        let mut entry = SkillIndexEntry::new(
            metadata.skill_name.clone(),
            metadata.description.clone(),
            metadata.version.clone(),
            path,
        );

        // Add routing keywords
        entry.routing_keywords = metadata.routing_keywords;

        // Add intents
        entry.intents = metadata.intents;

        // Add authors
        entry.authors = metadata.authors;

        // Add require_refs from frontmatter
        entry.require_refs = metadata.require_refs;

        // Add permissions (Zero Trust: empty = no access)
        entry.permissions = metadata.permissions;

        // Add sniffer rules from rules.toml
        entry.sniffing_rules = parse_rules_toml(skill_path);

        // Add tools (tool.tool_name already includes skill_name prefix from tools_scanner)
        let mut seen_names: Vec<String> = Vec::new();
        for tool in tools {
            if !seen_names.contains(&tool.tool_name) {
                seen_names.push(tool.tool_name.clone());
                let tool_entry = IndexToolEntry {
                    name: tool.tool_name.clone(),
                    description: tool.description.clone(),
                    category: tool.category.clone(),
                    input_schema: tool.input_schema.clone(),
                    file_hash: tool.file_hash.clone(),
                };
                entry.add_tool(tool_entry);
            }
        }

        // Scan references/*.md (metadata.for_tools per doc)
        entry.references = scan_references(skill_path, &metadata.skill_name);

        entry
    }

    /// Build the canonical per-skill payload (Rust schema) from metadata, tools, and filesystem.
    ///
    /// Fills `skill_tools` (each tool + `ref_key` → path from references' `for_tools`) and
    /// `references` (`ref_id` → `ReferenceRecord`). Used to wire the agreed schema to the scanner.
    #[must_use]
    pub fn build_canonical_payload(
        &self,
        metadata: SkillMetadata,
        tools: &[ToolRecord],
        skill_path: &Path,
    ) -> CanonicalSkillPayload {
        let _ = self;
        let skill_md_path = skill_path.join("SKILL.md").to_string_lossy().to_string();
        let refs = scan_references(skill_path, &metadata.skill_name);
        let references: HashMap<String, ReferenceRecord> =
            refs.into_iter().map(|r| (r.ref_name.clone(), r)).collect();

        let mut skill_tools: HashMap<String, CanonicalToolEntry> = HashMap::new();
        for tool in tools {
            let mut skill_tool_references = HashMap::new();
            for rec in references.values() {
                if rec
                    .for_tools
                    .as_ref()
                    .is_some_and(|v| v.contains(&tool.tool_name))
                {
                    let ref_key = format!("{}.references.{}", metadata.skill_name, rec.ref_name);
                    skill_tool_references.insert(ref_key, rec.file_path.clone());
                }
            }
            let entry = CanonicalToolEntry {
                tool: tool.clone(),
                skill_tool_references,
            };
            skill_tools.insert(tool.tool_name.clone(), entry);
        }

        CanonicalSkillPayload {
            skill_name: metadata.skill_name.clone(),
            skill_md_path,
            metadata,
            skill_tools,
            references,
        }
    }
}
