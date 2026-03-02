//! Runtime settings loader for omni-agent.
//!
//! Loads and merges:
//! - Embedded system defaults: `omni-agent/resources/config/xiuxian.toml`
//! - Optional system override file: `<PRJ_ROOT>/packages/rust/crates/omni-agent/resources/config/xiuxian.toml`
//! - User overrides: `<PRJ_CONFIG_HOME>/xiuxian-artisan-workshop/xiuxian.toml`
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
    InferenceSettings, McpSettings, MemorySettings, MistralSettings, RuntimeSettings,
    SessionSettings, TelegramAclAllowSettings, TelegramAclControlSettings,
    TelegramAclPrincipalSettings, TelegramAclSettings, TelegramAclSlashSettings,
    TelegramGroupSettings, TelegramSettings, TelegramTopicSettings,
};
