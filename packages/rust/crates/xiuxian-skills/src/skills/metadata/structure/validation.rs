use std::path::{Path, PathBuf};

use super::{SkillStructure, StructureItem, StructureItemKind};

const FORBIDDEN_SKILL_MD_LOGIC_TOKENS: &[&str] = &["{{", "{%", "{#"];

/// Validation report for one skill folder.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkillValidationReport {
    /// Whether the inspected skill folder is valid.
    pub valid: bool,
    /// Human-readable issue list describing validation failures.
    pub issues: Vec<String>,
    /// Non-blocking warnings discovered during validation.
    pub warnings: Vec<String>,
}

impl SkillValidationReport {
    #[must_use]
    fn from_findings(issues: Vec<String>, warnings: Vec<String>) -> Self {
        Self {
            valid: issues.is_empty(),
            issues,
            warnings,
        }
    }

    /// Returns `true` when the report contains any issue or warning.
    #[must_use]
    pub fn has_findings(&self) -> bool {
        !self.issues.is_empty() || !self.warnings.is_empty()
    }

    /// Returns number of blocking issues.
    #[must_use]
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    /// Returns number of non-blocking warnings.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Render a human-readable validation report with severity markers.
    #[must_use]
    pub fn render_for_skill(&self, skill_path: &Path) -> Vec<String> {
        if !self.has_findings() {
            return Vec::new();
        }
        let mut lines = Vec::new();
        lines.push(format!(
            "skill validation report for {} (errors: {}, warnings: {})",
            skill_path.display(),
            self.issue_count(),
            self.warning_count()
        ));
        for issue in &self.issues {
            lines.push(format!("[ERROR] {issue}"));
        }
        for warning in &self.warnings {
            lines.push(format!("[WARN] {warning}"));
        }
        lines
    }

    /// Build a machine-readable JSON summary for tooling and CI parsers.
    #[must_use]
    pub fn to_summary_json(&self, skill_path: &Path) -> serde_json::Value {
        let status = if !self.issues.is_empty() {
            "error"
        } else if !self.warnings.is_empty() {
            "warning"
        } else {
            "ok"
        };
        serde_json::json!({
            "skill_path": skill_path.display().to_string(),
            "status": status,
            "valid": self.valid,
            "issue_count": self.issue_count(),
            "warning_count": self.warning_count(),
            "issues": self.issues,
            "warnings": self.warnings,
        })
    }

    /// Build one-line JSON summary suitable for structured log pipelines.
    #[must_use]
    pub fn to_summary_json_line(&self, skill_path: &Path) -> String {
        self.to_summary_json(skill_path).to_string()
    }
}

pub(super) fn validate_skill_path(
    structure: &SkillStructure,
    skill_path: &Path,
) -> SkillValidationReport {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    if !skill_path.exists() {
        issues.push(format!(
            "skill directory does not exist: {}",
            skill_path.display()
        ));
        return SkillValidationReport::from_findings(issues, warnings);
    }
    if !skill_path.is_dir() {
        issues.push(format!(
            "skill path is not a directory: {}",
            skill_path.display()
        ));
        return SkillValidationReport::from_findings(issues, warnings);
    }

    issues.extend(validate_items(skill_path, "required", &structure.required));
    if structure.validation.structure.strict_mode {
        issues.extend(validate_items(
            skill_path,
            "default(strict_mode)",
            &structure.default,
        ));
    }
    if structure.validation.structure.enforce_references_folder {
        let references_path = skill_path.join("references");
        if !(references_path.exists() && references_path.is_dir()) {
            issues.push(format!(
                "missing enforced references directory: {}",
                references_path.display()
            ));
        }
    }
    if structure.validation.frontmatter.prohibit_logic_in_skill_md {
        issues.extend(validate_skill_md_logic_policy(skill_path));
    }
    warnings.extend(validate_out_of_scope_entries(skill_path, structure));

    SkillValidationReport::from_findings(issues, warnings)
}

fn validate_items(skill_path: &Path, scope: &str, items: &[StructureItem]) -> Vec<String> {
    let mut issues = Vec::new();
    for item in items {
        let path = skill_path.join(item.path.as_str());
        if !path.exists() {
            issues.push(format!(
                "missing {} item `{}` at {}",
                scope,
                item.path,
                path.display()
            ));
            continue;
        }
        match item.item_kind() {
            StructureItemKind::File if !path.is_file() => {
                issues.push(format!(
                    "{} item `{}` must be file but found non-file at {}",
                    scope,
                    item.path,
                    path.display()
                ));
            }
            StructureItemKind::Dir if !path.is_dir() => {
                issues.push(format!(
                    "{} item `{}` must be directory but found non-directory at {}",
                    scope,
                    item.path,
                    path.display()
                ));
            }
            _ => {}
        }
    }
    issues
}

fn validate_skill_md_logic_policy(skill_path: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let skill_md_path = resolve_skill_md_path(skill_path);
    let Some(skill_md_path) = skill_md_path else {
        return issues;
    };
    let content = match std::fs::read_to_string(&skill_md_path) {
        Ok(content) => content,
        Err(error) => {
            issues.push(format!(
                "failed to read SKILL.md for logic validation at {}: {}",
                skill_md_path.display(),
                error
            ));
            return issues;
        }
    };
    for token in FORBIDDEN_SKILL_MD_LOGIC_TOKENS {
        if content.contains(token) {
            issues.push(format!(
                "SKILL.md contains forbidden logic token `{}` at {}",
                token,
                skill_md_path.display()
            ));
        }
    }
    issues
}

fn resolve_skill_md_path(skill_path: &Path) -> Option<PathBuf> {
    let uppercase = skill_path.join("SKILL.md");
    if uppercase.is_file() {
        return Some(uppercase);
    }
    let lowercase = skill_path.join("skill.md");
    if lowercase.is_file() {
        return Some(lowercase);
    }
    None
}

fn validate_out_of_scope_entries(skill_path: &Path, structure: &SkillStructure) -> Vec<String> {
    use std::collections::HashSet;

    let mut warnings = Vec::new();
    let mut authorized: HashSet<String> = structure
        .required
        .iter()
        .chain(structure.default.iter())
        .chain(structure.optional.iter())
        .map(|item| normalize_entry_name(item.path.as_str()))
        .collect();
    authorized.insert("SKILL.md".to_string());
    authorized.insert("skill.md".to_string());

    let Ok(entries) = std::fs::read_dir(skill_path) else {
        return warnings;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_ignored_out_of_scope_name(name.as_str()) {
            continue;
        }
        if authorized.contains(name.as_str()) {
            continue;
        }
        warnings.push(format!(
            "out-of-scope entry `{}` detected at {} (not declared in skills.toml)",
            name,
            skill_path.display()
        ));
    }
    warnings
}

fn normalize_entry_name(path: &str) -> String {
    path.trim_end_matches('/').to_string()
}

fn is_ignored_out_of_scope_name(name: &str) -> bool {
    name.starts_with('.') || name == "target"
}
