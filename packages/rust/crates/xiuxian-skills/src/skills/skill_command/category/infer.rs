use super::rules::infer_category_keyword_match;

/// Infer category from skill name using pattern matching.
///
/// Provides a sensible default category when not explicitly specified
/// in the @`skill_command` decorator.
///
/// # Arguments
///
/// * `skill_name` - Name of the skill (e.g., "git", "filesystem", "writer")
///
/// # Returns
///
/// Inferred category string based on skill name patterns.
#[must_use]
pub fn infer_category_from_skill(skill_name: &str) -> String {
    let name_lower = skill_name.to_lowercase();
    infer_category_keyword_match(name_lower.as_str())
        .unwrap_or(skill_name)
        .to_string()
}
