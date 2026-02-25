use std::fs;
use std::path::{Path, PathBuf};

use super::SkillScanner;
use super::frontmatter::SkillFrontmatter;
use crate::frontmatter::extract_frontmatter;
use crate::skills::metadata::{ReferencePath, SkillMetadata, SkillStructure};

impl SkillScanner {
    /// Create a new skill scanner with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Get the default skill structure (from settings.yaml).
    #[must_use]
    pub fn default_structure() -> SkillStructure {
        SkillStructure::default()
    }

    /// Validate a skill directory against the canonical structure.
    ///
    /// Returns `true` if the skill has all required files.
    #[must_use]
    pub fn validate_structure(skill_path: &Path, structure: &SkillStructure) -> bool {
        if !skill_path.exists() {
            return false;
        }

        // Check that all required files exist
        for item in &structure.required {
            if item.item_type == "file" {
                let required_path = skill_path.join(&item.path);
                if !required_path.exists() {
                    log::debug!(
                        "Missing required file: {:?} for skill: {}",
                        item.path,
                        skill_path.display()
                    );
                    return false;
                }
            }
        }

        true
    }

    /// Scan a single skill directory and extract its metadata.
    ///
    /// Returns `Ok(Some(metadata))` if SKILL.md is found and valid.
    /// Returns `Ok(None)` if SKILL.md is missing.
    /// Returns `Err(...)` if SKILL.md exists but cannot be parsed.
    ///
    /// # Arguments
    ///
    /// * `skill_path` - Path to the skill directory (e.g., "assets/skills/writer")
    /// * `structure` - Optional skill structure for validation (uses default if None)
    ///
    /// # Errors
    ///
    /// Returns an error if `SKILL.md` cannot be read or parsed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let scanner = SkillScanner::new();
    /// let metadata = scanner.scan_skill(PathBuf::from("assets/skills/writer"), None).unwrap();
    ///
    /// match metadata {
    ///     Some(m) => println!("Found skill: {}", m.skill_name),
    ///     None => println!("No SKILL.md found"),
    /// }
    /// ```
    pub fn scan_skill(
        &self,
        skill_path: &Path,
        structure: Option<&SkillStructure>,
    ) -> Result<Option<SkillMetadata>, Box<dyn std::error::Error>> {
        let skill_md_path = skill_path.join("SKILL.md");

        if !skill_md_path.exists() {
            log::debug!("SKILL.md not found for skill: {}", skill_path.display());
            return Ok(None);
        }

        // Validate structure if provided
        if let Some(structure) = structure
            && !Self::validate_structure(skill_path, structure)
        {
            log::warn!(
                "Skill at {} does not match required structure",
                skill_path.display()
            );
        }

        let content = fs::read_to_string(&skill_md_path)?;
        let metadata = self.parse_skill_md(&content, skill_path)?;

        log::info!(
            "Scanned skill metadata: {} (v{}) - {} keywords",
            metadata.skill_name,
            metadata.version,
            metadata.routing_keywords.len()
        );

        Ok(Some(metadata))
    }

    /// Scan all skills in a base directory with parallel processing.
    ///
    /// Returns a vector of skill metadata for all skills with valid SKILL.md.
    /// Skills without SKILL.md are silently skipped.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Path to the skills directory (e.g., "assets/skills")
    /// * `structure` - Optional skill structure for validation (uses default if None)
    ///
    /// # Errors
    ///
    /// Returns an error if the skills directory cannot be read.
    pub fn scan_all(
        &self,
        base_path: &Path,
        structure: Option<&SkillStructure>,
    ) -> Result<Vec<SkillMetadata>, Box<dyn std::error::Error>> {
        use rayon::prelude::*;
        use std::sync::Arc;

        if !base_path.exists() {
            log::warn!("Skills base directory not found: {}", base_path.display());
            return Ok(Vec::new());
        }

        // Collect all skill directories first
        let skill_dirs: Vec<PathBuf> = fs::read_dir(base_path)?
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect();

        // Arc wrap structure for thread-safe sharing
        let validate_struct = structure.map(|s| Arc::new(s.clone()));

        // Process in parallel using rayon
        let metadatas: Vec<SkillMetadata> = skill_dirs
            .par_iter()
            .filter_map(|skill_path| self.scan_skill_inner(skill_path, validate_struct.as_deref()))
            .collect();

        log::info!(
            "Scanned {} skills from {}",
            metadatas.len(),
            base_path.display()
        );
        Ok(metadatas)
    }

    /// Internal helper for parallel skill scanning.
    #[inline]
    fn scan_skill_inner(
        &self,
        skill_path: &Path,
        structure: Option<&SkillStructure>,
    ) -> Option<SkillMetadata> {
        let skill_md_path = skill_path.join("SKILL.md");
        if !skill_md_path.exists() {
            return None;
        }

        // Validate structure if provided
        if let Some(structure) = structure
            && !Self::validate_structure(skill_path, structure)
        {
            log::warn!(
                "Skill at {} does not match required structure",
                skill_path.display()
            );
        }

        // Read and parse the file
        let content = fs::read_to_string(&skill_md_path).ok()?;
        let metadata = self.parse_skill_md(&content, skill_path).ok()?;

        log::info!(
            "Scanned skill metadata: {} (v{}) - {} keywords",
            metadata.skill_name,
            metadata.version,
            metadata.routing_keywords.len()
        );

        Some(metadata)
    }

    /// Parse YAML frontmatter from SKILL.md content.
    ///
    /// This is a public method to allow external parsing if needed.
    ///
    /// # Arguments
    ///
    /// * `content` - Raw content of the SKILL.md file
    /// * `skill_path` - Path to the skill directory (for extracting skill name)
    ///
    /// # Errors
    ///
    /// Returns an error if YAML frontmatter cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let content = std::fs::read_to_string("assets/skills/writer/SKILL.md").unwrap();
    /// let metadata = scanner.parse_skill_md(&content, PathBuf::from("writer")).unwrap();
    /// ```
    pub fn parse_skill_md(
        &self,
        content: &str,
        skill_path: &Path,
    ) -> Result<SkillMetadata, Box<dyn std::error::Error>> {
        let _ = self;
        // Extract skill name from path if not already set
        let skill_name = skill_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Find YAML frontmatter (between first and second ---)
        let Some(frontmatter) = extract_frontmatter(content) else {
            log::warn!("No YAML frontmatter found in SKILL.md for: {skill_name}");
            return Ok(SkillMetadata {
                skill_name,
                ..SkillMetadata::default()
            });
        };

        // Parse YAML frontmatter
        let frontmatter_data: SkillFrontmatter = serde_yaml::from_str(&frontmatter)
            .map_err(|e| anyhow::anyhow!("Failed to parse SKILL.md frontmatter: {e}"))?;

        // Extract from metadata block (new format)
        let (version, routing_keywords, authors, intents, require_refs, repository, permissions) =
            if let Some(meta) = &frontmatter_data.metadata {
                (
                    meta.version.clone().unwrap_or_default(),
                    meta.routing_keywords.clone().unwrap_or_default(),
                    // Support both "author" (single) and "authors" (multiple)
                    if let Some(authors_vec) = &meta.authors {
                        authors_vec.clone()
                    } else if let Some(a) = &meta.author {
                        vec![a.clone()]
                    } else {
                        Vec::new()
                    },
                    meta.intents.clone().unwrap_or_default(),
                    meta.require_refs.clone().unwrap_or_default(),
                    meta.source.clone().unwrap_or_default(),
                    meta.permissions.clone().unwrap_or_default(),
                )
            } else {
                log::warn!("No metadata block found in SKILL.md for: {skill_name}");
                (
                    String::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    String::new(),
                    Vec::new(),
                )
            };

        Ok(SkillMetadata {
            skill_name,
            version,
            description: frontmatter_data.description.unwrap_or_default(),
            routing_keywords,
            authors,
            intents,
            require_refs: require_refs
                .into_iter()
                .filter_map(|r| ReferencePath::new(r).ok())
                .collect(),
            repository,
            permissions,
        })
    }
}

impl Default for SkillScanner {
    fn default() -> Self {
        Self::new()
    }
}
