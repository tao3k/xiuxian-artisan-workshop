//! CLI argument model for the `xiuxian-tui` binary.

/// Omni TUI - Headless-compatible renderer for Python Agent events.
#[derive(clap::Parser, Debug)]
#[command(name = "xiuxian-tui")]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Unix socket path for IPC.
    #[arg(short, long)]
    pub(crate) socket: String,

    /// Connection role: "server" (binds socket) or "client" (connects to Python).
    #[arg(long, default_value = "client")]
    pub(crate) role: String,

    /// Parent process PID (for cleanup on parent death).
    #[arg(short, long)]
    pub(crate) pid: Option<i32>,

    /// Run in headless mode (no TUI rendering, just process events).
    #[arg(long, default_value = "false")]
    pub(crate) headless: bool,
}
