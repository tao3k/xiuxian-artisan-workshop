//! Agent configuration: inference API, model, API key, MCP server list.

mod agent_defaults;
mod memory_defaults;
mod types;

pub use agent_defaults::LITELLM_DEFAULT_URL;
pub use types::{AgentConfig, ContextBudgetStrategy, McpServerEntry, MemoryConfig};
