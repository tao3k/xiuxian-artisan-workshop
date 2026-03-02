//! Incremental-first LinkGraph refresh mechanism.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use xiuxian_wendao::LinkGraphIndex;
use xiuxian_wendao::link_graph::LinkGraphRefreshMode;

/// Runtime `Wendao` refresh trigger.
///
/// This mechanism prefers incremental refresh from context-provided
/// changed paths and only falls back to full rebuild when required.
pub struct WendaoRefreshMechanism {
    /// Output context key for refresh telemetry.
    pub output_key: String,
    /// Context key containing changed paths (`string` or `string[]`).
    pub changed_paths_key: String,
    /// Optional context key resolving root directory.
    pub root_dir_key: Option<String>,
    /// Optional static root directory override.
    pub root_dir: Option<String>,
    /// Force full rebuild (ignores incremental preference).
    pub force_full: bool,
    /// Prefer incremental mode even when changed path count crosses threshold.
    pub prefer_incremental: bool,
    /// Allow full fallback when incremental refresh fails.
    pub allow_full_fallback: bool,
    /// Optional explicit threshold when not preferring incremental.
    pub full_rebuild_threshold: Option<usize>,
    /// Optional include directories for `LinkGraph` build.
    pub include_dirs: Vec<String>,
    /// Optional excluded directories for `LinkGraph` build.
    pub excluded_dirs: Vec<String>,
}

#[async_trait]
impl QianjiMechanism for WendaoRefreshMechanism {
    async fn execute(&self, context: &Value) -> Result<QianjiOutput, String> {
        let changed_paths = collect_changed_paths(context, self.changed_paths_key.as_str());
        let root_dir = resolve_root_dir(
            context,
            self.root_dir.as_deref(),
            self.root_dir_key.as_deref(),
        )?;

        if changed_paths.is_empty() && !self.force_full {
            return Ok(QianjiOutput {
                data: json!({
                    self.output_key.clone(): {
                        "mode": "noop",
                        "changed_count": 0,
                        "force_full": false,
                        "fallback": false,
                        "root_dir": root_dir.display().to_string(),
                    }
                }),
                instruction: FlowInstruction::Continue,
            });
        }

        let mut index = build_index(root_dir.as_path(), &self.include_dirs, &self.excluded_dirs)?;

        let mut fallback = false;
        let threshold = if self.prefer_incremental {
            usize::MAX
        } else {
            self.full_rebuild_threshold
                .unwrap_or_else(LinkGraphIndex::incremental_rebuild_threshold)
                .max(1)
        };

        let refresh_mode = if self.force_full {
            fallback = false;
            run_forced_full_refresh(&mut index, &changed_paths)?
        } else {
            match index.refresh_incremental_with_threshold(&changed_paths, threshold) {
                Ok(mode) => mode,
                Err(error) if self.allow_full_fallback => {
                    fallback = true;
                    log::warn!(
                        "qianji wendao_refresh incremental failed, fallback to full rebuild: {error}"
                    );
                    run_forced_full_refresh(&mut index, &changed_paths)?
                }
                Err(error) => {
                    return Err(format!(
                        "wendao_refresh incremental failed without fallback: {error}"
                    ));
                }
            }
        };

        let changed_path_rows = changed_paths
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        Ok(QianjiOutput {
            data: json!({
                self.output_key.clone(): {
                    "mode": refresh_mode_label(refresh_mode),
                    "changed_count": changed_paths.len(),
                    "force_full": self.force_full,
                    "fallback": fallback,
                    "prefer_incremental": self.prefer_incremental,
                    "effective_threshold": threshold,
                    "root_dir": root_dir.display().to_string(),
                    "changed_paths": changed_path_rows,
                }
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

fn build_index(
    root_dir: &Path,
    include_dirs: &[String],
    excluded_dirs: &[String],
) -> Result<LinkGraphIndex, String> {
    match LinkGraphIndex::build_with_cache(root_dir, include_dirs, excluded_dirs) {
        Ok(index) => Ok(index),
        Err(error) if include_dirs.is_empty() && excluded_dirs.is_empty() => {
            log::warn!(
                "qianji wendao_refresh cache bootstrap failed, fallback to build(): {error}"
            );
            LinkGraphIndex::build(root_dir)
        }
        Err(error) => Err(error),
    }
}

fn collect_changed_paths(context: &Value, key: &str) -> Vec<PathBuf> {
    let Some(value) = context
        .get(key)
        .or_else(|| lookup_nested_value(context, key))
    else {
        return Vec::new();
    };
    match value {
        Value::String(single) => {
            let trimmed = single.trim();
            if trimmed.is_empty() {
                Vec::new()
            } else {
                vec![PathBuf::from(trimmed)]
            }
        }
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .collect(),
        _ => Vec::new(),
    }
}

fn lookup_nested_value<'a>(context: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = context;
    for segment in path.split('.') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        current = current.get(segment)?;
    }
    Some(current)
}

fn resolve_root_dir(
    context: &Value,
    explicit: Option<&str>,
    root_dir_key: Option<&str>,
) -> Result<PathBuf, String> {
    if let Some(path) = explicit.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(path));
    }

    if let Some(key) = root_dir_key
        && let Some(path) = context.get(key).and_then(Value::as_str)
        && !path.trim().is_empty()
    {
        return Ok(PathBuf::from(path.trim()));
    }

    for fallback_key in ["project_root", "repo_root", "notebook_root"] {
        if let Some(path) = context.get(fallback_key).and_then(Value::as_str)
            && !path.trim().is_empty()
        {
            return Ok(PathBuf::from(path.trim()));
        }
    }

    if let Ok(path) = std::env::var("PRJ_ROOT")
        && !path.trim().is_empty()
    {
        return Ok(PathBuf::from(path.trim()));
    }

    std::env::current_dir().map_err(|error| format!("failed to resolve current_dir: {error}"))
}

fn run_forced_full_refresh(
    index: &mut LinkGraphIndex,
    changed_paths: &[PathBuf],
) -> Result<LinkGraphRefreshMode, String> {
    let force_paths = if changed_paths.is_empty() {
        vec![PathBuf::from("__qianji_force_full__.md")]
    } else {
        changed_paths.to_vec()
    };
    index.refresh_incremental_with_threshold(&force_paths, 1)
}

fn refresh_mode_label(mode: LinkGraphRefreshMode) -> &'static str {
    match mode {
        LinkGraphRefreshMode::Noop => "noop",
        LinkGraphRefreshMode::Delta => "delta",
        LinkGraphRefreshMode::Full => "full",
    }
}
