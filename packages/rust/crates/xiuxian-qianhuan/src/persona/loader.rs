use super::profile::PersonaProfile;
use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(super) fn collect_persona_files(path: &Path) -> Result<Vec<PathBuf>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    if !path.is_dir() {
        return Err(anyhow!(
            "persona path is not a directory: {}",
            path.display()
        ));
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry
            .with_context(|| format!("failed to walk persona directory {}", path.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let file_path = entry.into_path();
        let Some(ext) = file_path
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
        else {
            continue;
        };
        if ext == "toml" {
            files.push(file_path);
        }
    }
    files.sort();
    Ok(files)
}

pub(super) fn parse_profile_from_file(path: &Path) -> Result<PersonaProfile> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read persona file {}", path.display()))?;
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("toml") => toml::from_str::<PersonaProfile>(&content)
            .with_context(|| format!("failed to parse TOML persona file {}", path.display())),
        Some(other) => Err(anyhow!(
            "unsupported persona file extension '{other}' for {}",
            path.display()
        )),
        None => Err(anyhow!("persona file has no extension: {}", path.display())),
    }
}
