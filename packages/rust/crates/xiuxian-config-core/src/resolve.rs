use crate::cache::{build_file_stamps, cache_key, store_cached_merged, try_get_cached_merged};
use crate::{ArrayMergeStrategy, ConfigCascadeSpec, ConfigCoreError};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

/// Resolve layered files and return merged TOML value.
///
/// Merge order:
/// 1. Embedded defaults (`spec.embedded_toml`) as base.
/// 2. If any `xiuxian.toml` exists in `PRJ_CONFIG_HOME`, merge `[spec.namespace]` from each candidate.
/// 3. If no `xiuxian.toml` exists, merge standalone orphan file(s) as fallback.
///
/// This resolver uses an internal read-through cache keyed by namespace/spec/path
/// and invalidated by file metadata stamps.
///
/// # Errors
///
/// Returns [`ConfigCoreError`] on parse/read failure or `SSoT` conflict.
pub fn resolve_and_merge_toml(spec: ConfigCascadeSpec<'_>) -> Result<toml::Value, ConfigCoreError> {
    let project_root = resolve_project_root();
    let config_home = resolve_config_home(project_root.as_deref());
    resolve_and_merge_toml_with_paths(spec, project_root.as_deref(), config_home.as_deref())
}

/// Resolve layered files and return merged TOML value with explicit paths.
///
/// This is intended for deterministic testing and runtime call sites that already
/// resolved `project_root` and `config_home`.
///
/// # Errors
///
/// Returns [`ConfigCoreError`] on parse/read failure or `SSoT` conflict.
pub fn resolve_and_merge_toml_with_paths(
    spec: ConfigCascadeSpec<'_>,
    project_root: Option<&Path>,
    config_home: Option<&Path>,
) -> Result<toml::Value, ConfigCoreError> {
    let resolved_config_home = normalize_config_home(project_root, config_home);
    let mut global_paths =
        existing_config_files(global_candidates(resolved_config_home.as_deref()));
    let mut orphan_paths = existing_config_files(orphan_candidates(
        resolved_config_home.as_deref(),
        spec.orphan_file,
    ));
    global_paths.sort();
    orphan_paths.sort();
    global_paths.dedup();
    orphan_paths.dedup();

    if !global_paths.is_empty() && !orphan_paths.is_empty() {
        let orphans = orphan_paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ConfigCoreError::RedundantOrphan {
            namespace: spec.namespace.to_string(),
            orphans,
        });
    }

    let tracked_files = tracked_files(&global_paths, &orphan_paths);
    let file_stamps = build_file_stamps(tracked_files.as_slice());
    let key = cache_key(spec, resolved_config_home.as_deref());
    if let Some(cached) = try_get_cached_merged(&key, file_stamps.as_slice()) {
        return Ok(cached);
    }

    let mut merged: toml::Value =
        toml::from_str(spec.embedded_toml).map_err(|source| ConfigCoreError::ParseEmbedded {
            namespace: spec.namespace.to_string(),
            source,
        })?;

    if global_paths.is_empty() {
        for orphan_path in orphan_paths {
            let orphan_value = read_toml(orphan_path.as_path())?;
            merge_values(&mut merged, orphan_value, spec.array_merge_strategy);
        }
    } else {
        for path in global_paths {
            let global_root = read_toml(path.as_path())?;
            if let Some(namespace_value) = extract_namespace_value(&global_root, spec.namespace) {
                merge_values(&mut merged, namespace_value, spec.array_merge_strategy);
            }
        }
    }

    store_cached_merged(key, file_stamps, &merged);
    Ok(merged)
}

/// Resolve layered files and deserialize merged config into target type.
///
/// # Errors
///
/// Returns [`ConfigCoreError`] on resolve/merge failure or deserialize failure.
pub fn resolve_and_load<T>(spec: ConfigCascadeSpec<'_>) -> Result<T, ConfigCoreError>
where
    T: DeserializeOwned,
{
    let merged = resolve_and_merge_toml(spec)?;
    merged
        .try_into()
        .map_err(|source| ConfigCoreError::DeserializeMerged {
            namespace: spec.namespace.to_string(),
            source,
        })
}

/// Resolve layered files and deserialize merged config using explicit paths.
///
/// # Errors
///
/// Returns [`ConfigCoreError`] on resolve/merge failure or deserialize failure.
pub fn resolve_and_load_with_paths<T>(
    spec: ConfigCascadeSpec<'_>,
    project_root: Option<&Path>,
    config_home: Option<&Path>,
) -> Result<T, ConfigCoreError>
where
    T: DeserializeOwned,
{
    let merged = resolve_and_merge_toml_with_paths(spec, project_root, config_home)?;
    merged
        .try_into()
        .map_err(|source| ConfigCoreError::DeserializeMerged {
            namespace: spec.namespace.to_string(),
            source,
        })
}

fn read_toml(path: &Path) -> Result<toml::Value, ConfigCoreError> {
    let content = std::fs::read_to_string(path).map_err(|source| ConfigCoreError::ReadFile {
        path: path.display().to_string(),
        source,
    })?;
    toml::from_str::<toml::Value>(&content).map_err(|source| ConfigCoreError::ParseFile {
        path: path.display().to_string(),
        source,
    })
}

fn merge_values(dst: &mut toml::Value, src: toml::Value, array_strategy: ArrayMergeStrategy) {
    match (dst, src) {
        (toml::Value::Table(dst_table), toml::Value::Table(src_table)) => {
            for (key, src_value) in src_table {
                if let Some(dst_value) = dst_table.get_mut(&key) {
                    merge_values(dst_value, src_value, array_strategy);
                } else {
                    dst_table.insert(key, src_value);
                }
            }
        }
        (toml::Value::Array(dst_array), toml::Value::Array(src_array))
            if matches!(array_strategy, ArrayMergeStrategy::Append) =>
        {
            dst_array.extend(src_array);
        }
        (dst_value, src_value) => {
            *dst_value = src_value;
        }
    }
}

fn resolve_project_root() -> Option<PathBuf> {
    if let Some(path) = std::env::var("PRJ_ROOT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        let candidate = PathBuf::from(path);
        if candidate.is_absolute() {
            return Some(candidate);
        }
        if let Ok(current_dir) = std::env::current_dir() {
            return Some(current_dir.join(candidate));
        }
        return None;
    }

    let mut cursor = std::env::current_dir().ok()?;
    loop {
        if cursor.join(".git").exists() {
            return Some(cursor);
        }
        if !cursor.pop() {
            break;
        }
    }
    None
}

fn resolve_config_home(project_root: Option<&Path>) -> Option<PathBuf> {
    std::env::var("PRJ_CONFIG_HOME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else if let Some(root) = project_root {
                root.join(path)
            } else {
                path
            }
        })
        .or_else(|| project_root.map(|root| root.join(".config")))
}

fn normalize_config_home(
    project_root: Option<&Path>,
    config_home: Option<&Path>,
) -> Option<PathBuf> {
    match config_home {
        Some(path) if path.is_absolute() => Some(path.to_path_buf()),
        Some(path) => project_root.map(|root| root.join(path)),
        None => project_root.map(|root| root.join(".config")),
    }
}

fn global_candidates(config_home: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(config_home) = config_home {
        candidates.push(
            config_home
                .join("xiuxian-artisan-workshop")
                .join("xiuxian.toml"),
        );
    }
    candidates
}

fn orphan_candidates(config_home: Option<&Path>, orphan_file: &str) -> Vec<PathBuf> {
    if orphan_file.trim().is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    if let Some(config_home) = config_home {
        candidates.push(
            config_home
                .join("xiuxian-artisan-workshop")
                .join(orphan_file),
        );
    }
    candidates
}

fn existing_config_files(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.into_iter().filter(|path| path.is_file()).collect()
}

fn tracked_files(global_paths: &[PathBuf], orphan_paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::with_capacity(global_paths.len() + orphan_paths.len());
    files.extend(global_paths.iter().cloned());
    files.extend(orphan_paths.iter().cloned());
    files
}

fn extract_namespace_value(root: &toml::Value, namespace: &str) -> Option<toml::Value> {
    let mut cursor = root;
    for segment in namespace.split('.') {
        let key = segment.trim();
        if key.is_empty() {
            return None;
        }
        let table = cursor.as_table()?;
        cursor = table.get(key)?;
    }
    Some(cursor.clone())
}
