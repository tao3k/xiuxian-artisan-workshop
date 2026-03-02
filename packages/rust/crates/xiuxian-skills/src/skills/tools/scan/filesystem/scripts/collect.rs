use std::path::Path;

use crate::skills::metadata::ToolRecord;

use super::super::super::super::ToolsScanner;
use super::entries::script_paths_in_directory;

pub(super) fn collect_tools_from_directory(
    scanner: &ToolsScanner,
    scripts_dir: &Path,
    skill_name: &str,
    skill_keywords: &[String],
    skill_intents: &[String],
) -> Result<Vec<ToolRecord>, Box<dyn std::error::Error>> {
    let mut tools = Vec::new();
    let script_paths = script_paths_in_directory(scripts_dir);

    for script_path in script_paths {
        let parsed_tools = parse_script_file(
            scanner,
            &script_path,
            skill_name,
            skill_keywords,
            skill_intents,
        )?;
        tools.extend(parsed_tools);
    }

    Ok(tools)
}

fn parse_script_file(
    scanner: &ToolsScanner,
    script_path: &Path,
    skill_name: &str,
    skill_keywords: &[String],
    skill_intents: &[String],
) -> Result<Vec<ToolRecord>, Box<dyn std::error::Error>> {
    let parsed_tools =
        scanner.parse_script(script_path, skill_name, skill_keywords, skill_intents)?;

    if !parsed_tools.is_empty() {
        log::debug!(
            "ToolsScanner: Found {} tools in {}",
            parsed_tools.len(),
            script_path.display()
        );
    }

    Ok(parsed_tools)
}
