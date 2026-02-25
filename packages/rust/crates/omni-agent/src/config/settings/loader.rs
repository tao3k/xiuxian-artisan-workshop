use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::RuntimeSettings;

const DEFAULT_SYSTEM_SETTINGS_RELATIVE_PATH: &str = "packages/conf/settings.yaml";
const DEFAULT_USER_SETTINGS_RELATIVE_PATH: &str = "omni-dev-fusion/settings.yaml";
const DEFAULT_CONFIG_HOME_RELATIVE_PATH: &str = ".config";
static CONFIG_HOME_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

#[must_use]
pub fn load_runtime_settings() -> RuntimeSettings {
    let (system_path, user_path) = runtime_settings_paths();
    load_runtime_settings_from_paths(&system_path, &user_path)
}

#[doc(hidden)]
pub fn runtime_settings_paths() -> (PathBuf, PathBuf) {
    let root = project_root();
    let system_path = root.join(DEFAULT_SYSTEM_SETTINGS_RELATIVE_PATH);
    let user_path = resolve_config_home(&root).join(DEFAULT_USER_SETTINGS_RELATIVE_PATH);
    (system_path, user_path)
}

#[doc(hidden)]
#[must_use]
pub fn load_runtime_settings_from_paths(system: &Path, user: &Path) -> RuntimeSettings {
    load_one(system).merge(load_one(user))
}

fn load_one(path: &Path) -> RuntimeSettings {
    if !path.exists() {
        return RuntimeSettings::default();
    }
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) => {
            tracing::warn!(
                path = %path.display(),
                error = %error,
                "failed to read settings file; ignoring"
            );
            return RuntimeSettings::default();
        }
    };
    match serde_yaml::from_str::<RuntimeSettings>(&raw) {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(
                path = %path.display(),
                error = %error,
                "failed to parse settings yaml; ignoring file"
            );
            RuntimeSettings::default()
        }
    }
}

fn project_root() -> PathBuf {
    std::env::var("PRJ_ROOT").ok().map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        |value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            } else {
                PathBuf::from(trimmed)
            }
        },
    )
}

/// Set config-home override (used by CLI `--conf`).
///
/// The path can be absolute, or relative to `PRJ_ROOT`/cwd.
pub fn set_config_home_override(path: impl Into<PathBuf>) {
    let path = path.into();
    if path.as_os_str().is_empty() {
        return;
    }
    if CONFIG_HOME_OVERRIDE.set(path.clone()).is_err()
        && let Some(current) = CONFIG_HOME_OVERRIDE.get()
        && current != &path
    {
        tracing::warn!(
            current = %current.display(),
            ignored = %path.display(),
            "config home override already set; ignoring subsequent value"
        );
    }
}

fn resolve_config_home(project_root: &Path) -> PathBuf {
    if let Some(path) = CONFIG_HOME_OVERRIDE.get() {
        return absolutize(project_root, path.clone());
    }

    let configured = std::env::var("PRJ_CONFIG_HOME")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_CONFIG_HOME_RELATIVE_PATH.to_string());
    absolutize(project_root, PathBuf::from(configured))
}

fn absolutize(project_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    }
}
