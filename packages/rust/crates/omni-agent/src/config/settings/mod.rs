//! Runtime settings loader for omni-agent.
//!
//! Loads and merges:
//! - System defaults: `<PRJ_ROOT>/packages/conf/settings.yaml`
//! - User overrides:  `<PRJ_CONFIG_HOME>/omni-dev-fusion/settings.yaml`
//!
//! Merge precedence is user over system.

mod loader;
mod merge;
mod types;

pub use loader::{
    load_runtime_settings, load_runtime_settings_from_paths, runtime_settings_paths,
    set_config_home_override,
};
pub use types::{
    DiscordAclAllowSettings, DiscordAclControlSettings, DiscordAclPrincipalSettings,
    DiscordAclSettings, DiscordAclSlashSettings, DiscordSettings, EmbeddingSettings,
    InferenceSettings, McpSettings, MemorySettings, RuntimeSettings, SessionSettings,
    TelegramAclAllowSettings, TelegramAclControlSettings, TelegramAclPrincipalSettings,
    TelegramAclSettings, TelegramAclSlashSettings, TelegramGroupSettings, TelegramSettings,
    TelegramTopicSettings,
};
