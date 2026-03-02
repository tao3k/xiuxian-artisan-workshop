use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use omni_agent::DEFAULT_STDIO_SESSION_ID;

#[derive(Parser)]
#[command(name = "omni-agent")]
#[command(about = "Rust agent: LLM + MCP tools. Gateway, stdio, or repl (interactive / one-shot).")]
pub(crate) struct Cli {
    /// Override config directory (same semantics as Python `--conf`).
    #[arg(long, global = true)]
    pub(crate) conf: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum TelegramChannelMode {
    Polling,
    Webhook,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum ChannelProvider {
    Telegram,
    Discord,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum DiscordRuntimeMode {
    Gateway,
    Ingress,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum WebhookDedupBackendMode {
    Memory,
    Valkey,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Run HTTP server (POST /message). Default bind: 0.0.0.0:8080
    Gateway {
        /// Listen address (e.g. 0.0.0.0:8080)
        #[arg(long, default_value = "0.0.0.0:8080")]
        bind: String,

        /// Per-turn timeout in seconds (default: 300)
        #[arg(long)]
        turn_timeout: Option<u64>,

        /// Max concurrent agent turns (default: 4; omit for no limit)
        #[arg(long)]
        max_concurrent: Option<usize>,

        /// Path to mcp.json (default: .mcp.json)
        #[arg(long, default_value = ".mcp.json")]
        mcp_config: PathBuf,
    },
    /// Read lines from stdin, run turn, print output. Exit on EOF or Ctrl+C.
    Stdio {
        /// Session ID for conversation (default: default)
        #[arg(long, default_value = DEFAULT_STDIO_SESSION_ID)]
        session_id: String,

        /// Path to mcp.json (default: .mcp.json)
        #[arg(long, default_value = ".mcp.json")]
        mcp_config: PathBuf,
    },
    /// REPL: interact with the model (complex intents, tool use). One-shot with --query, or interactive loop.
    Repl {
        /// Run one turn with this intent and exit (no interactive loop).
        #[arg(long)]
        query: Option<String>,

        /// Session ID for conversation (default: default)
        #[arg(long, default_value = DEFAULT_STDIO_SESSION_ID)]
        session_id: String,

        /// Path to mcp.json (default: .mcp.json)
        #[arg(long, default_value = ".mcp.json")]
        mcp_config: PathBuf,
    },
    /// Run recurring scheduled jobs via `JobManager`.
    Schedule {
        /// Prompt executed on every schedule tick.
        #[arg(long)]
        prompt: String,

        /// Tick interval in seconds.
        #[arg(long, default_value_t = 300)]
        interval_secs: u64,

        /// Optional number of submissions before exit.
        #[arg(long)]
        max_runs: Option<u64>,

        /// Logical schedule id for session namespacing.
        #[arg(long, default_value = "default")]
        schedule_id: String,

        /// Session prefix for generated schedule job sessions.
        #[arg(long, default_value = "scheduler")]
        session_prefix: String,

        /// Recipient identifier attached to job records/completions.
        #[arg(long, default_value = "scheduler")]
        recipient: String,

        /// Grace period (seconds) to drain in-flight completions before exit.
        #[arg(long, default_value_t = 30)]
        wait_for_completion_secs: u64,

        /// Path to mcp.json (default: .mcp.json)
        #[arg(long, default_value = ".mcp.json")]
        mcp_config: PathBuf,
    },
    /// Run messaging channel runtime (`telegram` or `discord`).
    Channel {
        /// Channel provider.
        #[arg(long, value_enum, default_value_t = ChannelProvider::Telegram)]
        provider: ChannelProvider,

        /// Bot token (`TELEGRAM_BOT_TOKEN` for Telegram, `DISCORD_BOT_TOKEN` for Discord).
        #[arg(long)]
        bot_token: Option<String>,

        /// Path to mcp.json (default: .mcp.json)
        #[arg(long, default_value = ".mcp.json")]
        mcp_config: PathBuf,

        /// Telegram transport mode (`polling` for single instance, `webhook` for multi-instance).
        #[arg(long, value_enum)]
        mode: Option<TelegramChannelMode>,

        /// Webhook listen address (used only when `--mode webhook`).
        #[arg(long)]
        webhook_bind: Option<String>,

        /// Webhook path (used only when `--mode webhook`).
        #[arg(long)]
        webhook_path: Option<String>,

        /// Telegram webhook secret token (required when `--mode webhook`; or `TELEGRAM_WEBHOOK_SECRET` env).
        #[arg(long)]
        webhook_secret_token: Option<String>,

        /// Discord session partition (`guild_channel_user`, `channel`, `user`, `guild_user`).
        #[arg(long)]
        session_partition: Option<String>,

        /// Discord inbound queue capacity.
        #[arg(long)]
        inbound_queue_capacity: Option<usize>,

        /// Discord foreground turn timeout in seconds.
        #[arg(long)]
        turn_timeout_secs: Option<u64>,

        /// Discord runtime transport (`gateway` for production, `ingress` for synthetic ingress probes).
        #[arg(long, value_enum)]
        discord_runtime_mode: Option<DiscordRuntimeMode>,

        /// Webhook dedup backend (`valkey` recommended for multi-node, `memory` for single node).
        #[arg(long, value_enum)]
        webhook_dedup_backend: Option<WebhookDedupBackendMode>,

        /// Valkey URL for webhook dedup (or `XIUXIAN_WENDAO_VALKEY_URL` env).
        #[arg(long)]
        valkey_url: Option<String>,

        /// TTL (seconds) for webhook dedup keys.
        #[arg(long)]
        webhook_dedup_ttl_secs: Option<u64>,

        /// Key prefix for webhook dedup keys in Valkey/Redis.
        #[arg(long)]
        webhook_dedup_key_prefix: Option<String>,

        /// Verbose logs: show user messages and bot replies (also enables debug-level tracing).
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Run one embedding request to warm up backend/model before channel startup.
    EmbeddingWarmup {
        /// Warmup input text.
        #[arg(long, default_value = "embedding warmup")]
        text: String,

        /// Optional explicit embedding model override.
        #[arg(long)]
        model: Option<String>,

        /// Only execute warmup when effective embedding backend is `mistral_sdk`.
        #[arg(long, default_value_t = false)]
        mistral_sdk_only: bool,
    },
}
