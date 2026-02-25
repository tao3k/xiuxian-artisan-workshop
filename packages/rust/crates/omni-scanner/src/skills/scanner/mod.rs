//! Skill Scanner - Parses SKILL.md files for metadata and routing keywords.
//!
//! This module provides the `SkillScanner` struct which scans skill directories
//! and extracts metadata from SKILL.md YAML frontmatter.
//!
//! # Architecture
//!
//! Follows the skill structure defined in `settings.yaml` under `skills.architecture`:
//! - Required: `SKILL.md` - Skill metadata (YAML frontmatter) and system prompts
//! - Default: `scripts/` - Standalone executables (tools)
//!
//! # Example
//!
//! ```ignore
//! use skills_scanner::SkillScanner;
//!
//! let scanner = SkillScanner::new();
//! let metadatas = scanner.scan_all(PathBuf::from("assets/skills")).unwrap();
//!
//! for metadata in metadatas {
//!     println!("Skill: {} - {} keywords", metadata.skill_name, metadata.routing_keywords.len());
//! }
//! ```

mod frontmatter;
mod index_build;
mod references;
mod rules;
mod scan;

/// Skill Scanner - Extracts metadata from SKILL.md files.
///
/// Scans skill directories to extract:
/// - Skill name, version, description
/// - `routing_keywords` for hybrid search
/// - Authors and intents
///
/// # Usage
///
/// ```ignore
/// use skills_scanner::SkillScanner;
///
/// let scanner = SkillScanner::new();
///
/// // Scan single skill
/// let metadata = scanner.scan_skill(PathBuf::from("assets/skills/writer")).unwrap();
///
/// // Scan all skills
/// let all_metadatas = scanner.scan_all(PathBuf::from("assets/skills")).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct SkillScanner;

// Note: Comprehensive tests are in tests/test_skill_scanner.rs
