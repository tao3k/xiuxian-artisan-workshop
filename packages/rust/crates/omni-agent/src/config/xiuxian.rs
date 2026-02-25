//! Configuration loader for `xiuxian.toml`.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Root configuration of xiuxian.toml
#[derive(Debug, Clone, Deserialize, Default)]
pub struct XiuxianConfig {
    #[serde(default)]
    pub llm: LlmConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct LlmConfig {
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
    #[serde(default)]
    pub providers: HashMap<String, LlmProviderConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LlmProviderConfig {
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub model_aliases: HashMap<String, String>,
}

pub fn get_xiuxian_config_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push("packages");
    path.push("conf");
    path.push("xiuxian.toml");

    // Check if the user has overridden it in XDG_CONFIG_HOME
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        let user_path = PathBuf::from(config_home)
            .join("omni-dev-fusion")
            .join("xiuxian.toml");
        if user_path.exists() {
            return user_path;
        }
    }

    // Check ~/.config/omni-dev-fusion/xiuxian.toml
    if let Some(home) = dirs::home_dir() {
        let user_path = home
            .join(".config")
            .join("omni-dev-fusion")
            .join("xiuxian.toml");
        if user_path.exists() {
            return user_path;
        }
    }

    path
}

pub fn load_xiuxian_config() -> XiuxianConfig {
    let path = get_xiuxian_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("Failed to parse {}: {}", path.display(), e);
                XiuxianConfig::default()
            }
        }
    } else {
        tracing::warn!(
            "xiuxian.toml not found at {}. Using defaults.",
            path.display()
        );
        XiuxianConfig::default()
    }
}
