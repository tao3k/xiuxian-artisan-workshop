use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::frontmatter::extract_frontmatter;
use crate::skills::metadata::ReferenceRecord;

/// Frontmatter for reference docs under `references/*.md` (see skill-data-hierarchy-and-references.md).
/// All reference fields live under a single `metadata` block (same pattern as SKILL.md).
#[derive(Debug, Deserialize)]
struct ReferenceFrontmatter {
    #[serde(default)]
    metadata: Option<ReferenceMetadataBlock>,
}

/// Contents of the `metadata` block in reference front matter.
#[derive(Debug, Deserialize)]
struct ReferenceMetadataBlock {
    /// Tool(s) this reference is for, full name e.g. `git.smart_commit` (string or array).
    #[serde(default, rename = "for_tools")]
    for_tools: Option<serde_yaml::Value>,
    #[serde(default)]
    title: Option<String>,
    /// Optional description (reserved for future use on `ReferenceRecord`).
    #[serde(default)]
    #[allow(dead_code)]
    description: Option<String>,
    #[serde(default)]
    routing_keywords: Option<Vec<String>>,
    #[serde(default)]
    intents: Option<Vec<String>>,
}

/// Derive unique skill names from a list of full tool names
/// (e.g. `"researcher.run_research_graph"` -> `"researcher"`).
fn skills_from_tool_list(tools: &[String]) -> Vec<String> {
    let mut skills: Vec<String> = tools
        .iter()
        .filter_map(|t| t.split('.').next().map(String::from))
        .filter(|s| !s.is_empty())
        .collect();
    skills.sort();
    skills.dedup();
    skills
}

fn yaml_value_to_string_vec(v: &serde_yaml::Value) -> Vec<String> {
    match v {
        serde_yaml::Value::String(s) => {
            if s.is_empty() {
                vec![]
            } else {
                vec![s.clone()]
            }
        }
        serde_yaml::Value::Sequence(seq) => seq
            .iter()
            .filter_map(|x| x.as_str().map(String::from))
            .filter(|s| !s.is_empty())
            .collect(),
        _ => vec![],
    }
}

fn yaml_value_to_opt_string_vec(v: &serde_yaml::Value) -> Option<Vec<String>> {
    let vec = yaml_value_to_string_vec(v);
    if vec.is_empty() { None } else { Some(vec) }
}

/// Scan `references/` under a skill directory and return reference records.
/// Parses YAML frontmatter for `for_tools` only; `for_skills` and `skill_name` are derived from tool full names.
pub(super) fn scan_references(skill_path: &Path, skill_name: &str) -> Vec<ReferenceRecord> {
    let refs_dir = skill_path.join("references");
    if !refs_dir.is_dir() {
        return Vec::new();
    }

    let mut records = Vec::new();
    let read_dir = match fs::read_dir(&refs_dir) {
        Ok(d) => d,
        Err(e) => {
            log::debug!("Could not read references dir {}: {e}", refs_dir.display());
            return records;
        }
    };

    for entry in read_dir.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let ref_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let file_path = path.to_string_lossy().to_string();

        let fm = extract_frontmatter(&content)
            .and_then(|s| serde_yaml::from_str::<ReferenceFrontmatter>(&s).ok());
        let meta = fm.as_ref().and_then(|f| f.metadata.as_ref());
        let for_tools = meta
            .and_then(|m| m.for_tools.as_ref())
            .and_then(yaml_value_to_opt_string_vec);
        let for_skills: Vec<String> = for_tools
            .as_ref()
            .map(|tools| skills_from_tool_list(tools))
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| vec![skill_name.to_string()]);
        let skill_name_primary = for_skills
            .first()
            .cloned()
            .unwrap_or_else(|| skill_name.to_string());
        let title = meta
            .and_then(|m| m.title.clone())
            .unwrap_or_else(|| ref_name.clone());

        let mut rec = ReferenceRecord::new(ref_name, title, skill_name_primary.clone(), file_path);
        rec.for_skills = for_skills;
        rec.for_tools = for_tools;
        if let Some(m) = meta {
            let mut kw = Vec::new();
            if let Some(ref r) = m.routing_keywords {
                kw.extend(r.clone());
            }
            if let Some(ref i) = m.intents {
                kw.extend(i.clone());
            }
            if !kw.is_empty() {
                rec.keywords = kw;
            }
        }
        records.push(rec);
    }

    if log::log_enabled!(log::Level::Debug) && !records.is_empty() {
        log::debug!(
            "Scanned {} reference(s) for skill {}",
            records.len(),
            skill_name
        );
    }
    records
}
