use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::skills::metadata::SnifferRule;

/// TOML structure for rules.toml parsing.
#[derive(Debug, Deserialize)]
struct RulesToml {
    #[serde(default, rename = "match")]
    matches: Vec<RuleMatch>,
}

/// Single match rule in rules.toml.
#[derive(Debug, Deserialize)]
struct RuleMatch {
    #[serde(rename = "type")]
    rule_type: Option<String>,
    pattern: Option<String>,
}

/// Parse extensions/sniffer/rules.toml for sniffer rules.
///
/// Returns a vector of `SnifferRule` extracted from the TOML file.
/// If the file doesn't exist or is invalid, returns an empty vector.
#[inline]
pub(super) fn parse_rules_toml(skill_path: &Path) -> Vec<SnifferRule> {
    let rules_path = skill_path.join("extensions/sniffer/rules.toml");
    if !rules_path.exists() {
        return Vec::new();
    }

    let content = match fs::read_to_string(&rules_path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to read rules.toml: {e}");
            return Vec::new();
        }
    };

    let rules_toml: RulesToml = match toml::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("Failed to parse rules.toml: {e}");
            return Vec::new();
        }
    };

    let mut rules = Vec::new();
    for rule in rules_toml.matches {
        if let (Some(rule_type), Some(pattern)) = (rule.rule_type, rule.pattern) {
            rules.push(SnifferRule::new(rule_type, pattern));
        }
    }

    if log::log_enabled!(log::Level::Debug) {
        log::debug!(
            "Parsed {} sniffer rules from {}",
            rules.len(),
            rules_path.display()
        );
    }

    rules
}
