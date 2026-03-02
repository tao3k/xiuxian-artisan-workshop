//! Rust agent: one-turn loop with LLM + MCP tools; HTTP gateway.
//!
//! - **B.1**: Session store (in-memory or omni-window), LLM client (OpenAI-compatible chat API).
//! - **B.2**: One turn: user message → prompt + tools/list → LLM → `tool_calls` → MCP tools/call → repeat until done.

/// Compile-time embedded resource tree rooted at `omni-agent/resources`.
pub static RESOURCES: ::include_dir::Dir<'_> =
    ::include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources");

mod agent;
mod channels;
mod config;
mod contracts;
mod embedding;
mod env_parse;
mod gateway;
mod jobs;
mod llm;
mod mcp;
mod observability;
mod session;
mod shortcuts;
#[doc(hidden)]
pub mod test_support;
mod tools;

pub use agent::{
    Agent, MemoryRecallLatencyBucketsSnapshot, MemoryRecallMetricsSnapshot, NativeToolRegistry,
    SessionContextBudgetClassSnapshot, SessionContextBudgetSnapshot, SessionContextMode,
    SessionContextSnapshotInfo, SessionContextStats, SessionContextWindowInfo,
    SessionMemoryRecallDecision, SessionMemoryRecallSnapshot,
    native_tools::registry::{NativeTool, NativeToolCallContext},
    native_tools::zhixing::{AgendaViewTool, JournalRecordTool, TaskAddTool},
    notification::{NotificationDispatcher, NotificationProvider},
    prune_messages_for_token_budget, summarise_drained_turns,
};
pub use channels::{
    Channel, ChannelMessage, DEFAULT_REDIS_KEY_PREFIX, DISCORD_MAX_MESSAGE_LENGTH,
    DiscordAclOverrides, DiscordChannel, DiscordCommandAdminRule, DiscordControlCommandPolicy,
    DiscordIngressApp, DiscordIngressBuildRequest, DiscordIngressRunRequest, DiscordRuntimeConfig,
    DiscordSessionPartition, DiscordSlashCommandPolicy, ForegroundQueueMode,
    RecipientCommandAdminUsersMutation, SessionGate, TELEGRAM_MAX_MESSAGE_LENGTH,
    TelegramAclOverrides, TelegramChannel, TelegramCommandAdminRule, TelegramControlCommandPolicy,
    TelegramRuntimeConfig, TelegramSessionPartition, TelegramSlashCommandPolicy,
    TelegramWebhookApp, TelegramWebhookControlPolicyBuildRequest,
    TelegramWebhookPartitionBuildRequest, TelegramWebhookPolicyRunRequest,
    TelegramWebhookRunRequest, WebhookDedupBackend, WebhookDedupConfig,
    build_discord_acl_overrides, build_discord_command_admin_rule, build_discord_ingress_app,
    build_discord_ingress_app_with_control_command_policy,
    build_discord_ingress_app_with_partition_and_control_command_policy,
    build_telegram_acl_overrides, build_telegram_acl_overrides_from_settings,
    build_telegram_command_admin_rule, build_telegram_webhook_app,
    build_telegram_webhook_app_with_control_command_policy,
    build_telegram_webhook_app_with_partition, chunk_marker_reserve_chars,
    decorate_chunk_for_telegram, markdown_to_telegram_html, markdown_to_telegram_markdown_v2,
    run_discord_gateway, run_discord_ingress, run_telegram, run_telegram_webhook,
    run_telegram_webhook_with_control_command_policy, run_telegram_with_control_command_policy,
    split_message_for_discord, split_message_for_telegram,
};
pub use config::{
    AgentConfig, ContextBudgetStrategy, DiscordSettings, EmbeddingSettings, InferenceSettings,
    LITELLM_DEFAULT_URL, McpConfigFile, McpServerEntry, McpServerEntryFile, McpSettings,
    MemoryConfig, MemorySettings, RuntimeSettings, SessionSettings, TelegramAclAllowSettings,
    TelegramAclControlSettings, TelegramAclPrincipalSettings, TelegramAclSettings,
    TelegramAclSlashSettings, TelegramSettings, load_mcp_config, load_runtime_settings,
    load_runtime_settings_from_paths, set_config_home_override,
};
pub use contracts::{
    DiscoverConfidence, DiscoverMatch, GraphExecutionPlan, GraphPlanStep, GraphPlanStepKind,
    GraphWorkflowMode, MemoryGateDecision, MemoryGateVerdict, OmegaDecision, OmegaFallbackPolicy,
    OmegaRiskLevel, OmegaRoute, OmegaToolTrustClass, RouteTrace, RouteTraceGraphStep,
    RouteTraceInjection, WorkflowBridgeMode,
};
pub use embedding::EmbeddingClient;
pub use gateway::{
    DEFAULT_STDIO_SESSION_ID, GatewayHealthResponse, GatewayMcpHealthResponse, GatewayState,
    MessageRequest, MessageResponse, router, run_http, run_stdio, validate_message_request,
};
pub use jobs::{
    HeartbeatProbeState, JobCompletion, JobCompletionKind, JobHealthState, JobManager,
    JobManagerConfig, JobMetricsSnapshot, JobState, JobStatusSnapshot, RecurringScheduleConfig,
    RecurringScheduleOutcome, TurnRunner, classify_heartbeat_probe_result, classify_job_health,
    run_recurring_schedule,
};
pub use mcp::{
    McpClientPool, McpDiscoverCacheStatsSnapshot, McpPoolConnectConfig,
    McpToolsListCacheStatsSnapshot, connect_pool,
};
pub use session::{
    BoundedSessionStore, ChatMessage, FunctionCall, SessionStore, SessionSummarySegment,
    ToolCallOut,
};
pub use shortcuts::parse_react_shortcut;
pub use tools::{parse_qualified_tool_name, qualify_tool_name};
