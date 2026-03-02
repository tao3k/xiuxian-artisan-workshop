use std::path::Path;

use super::super::SkillScanner;
use super::super::references::scan_references;
use super::super::rules::parse_rules_toml;
use crate::skills::metadata::{IndexToolEntry, SkillIndexEntry, SkillMetadata, ToolRecord};

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

        entry.routing_keywords = metadata.routing_keywords;
        entry.intents = metadata.intents;
        entry.authors = metadata.authors;
        entry.require_refs = metadata.require_refs;
        entry.permissions = metadata.permissions;
        entry.sniffing_rules = parse_rules_toml(skill_path);

        add_unique_tools(&mut entry, tools);
        entry.references = scan_references(skill_path, &metadata.skill_name);
        entry
    }
}

fn add_unique_tools(entry: &mut SkillIndexEntry, tools: &[ToolRecord]) {
    let mut seen_names: Vec<String> = Vec::new();
    for tool in tools {
        if seen_names.contains(&tool.tool_name) {
            continue;
        }
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
