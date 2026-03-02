//! Config namespace: agent config and MCP config loading.

mod agent;
mod mcp;
mod settings;
mod xiuxian;

pub use agent::{
    AgentConfig, ContextBudgetStrategy, LITELLM_DEFAULT_URL, McpServerEntry, MemoryConfig,
};
pub use mcp::{McpConfigFile, McpServerEntryFile, load_mcp_config};
pub use settings::{
    DiscordAclAllowSettings, DiscordAclControlSettings, DiscordAclPrincipalSettings,
    DiscordAclSettings, DiscordAclSlashSettings, DiscordSettings, EmbeddingSettings,
    InferenceSettings, McpSettings, MemorySettings, RuntimeSettings, SessionSettings,
    TelegramAclAllowSettings, TelegramAclControlSettings, TelegramAclPrincipalSettings,
    TelegramAclSettings, TelegramAclSlashSettings, TelegramGroupSettings, TelegramSettings,
    TelegramTopicSettings, load_runtime_settings, load_runtime_settings_from_paths,
    runtime_settings_paths, set_config_home_override,
};
pub(crate) use xiuxian::XiuxianConfig;
pub use xiuxian::load_xiuxian_config;

#[cfg(test)]
mod tests_xiuxian;
