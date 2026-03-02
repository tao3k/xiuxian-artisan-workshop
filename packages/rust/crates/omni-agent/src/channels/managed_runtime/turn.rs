use std::time::Duration;

use crate::agent::Agent;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ForegroundTurnOutcome {
    Succeeded(String),
    Failed {
        reply: String,
        error_chain: String,
        error_kind: &'static str,
    },
    TimedOut {
        reply: String,
    },
    Interrupted {
        reply: String,
    },
}

pub(crate) fn build_session_id(channel: &str, session_key: &str) -> String {
    format!("{channel}:{session_key}")
}

pub(crate) struct ForegroundTurnRequest {
    pub(crate) agent: Arc<Agent>,
    pub(crate) session_id: String,
    pub(crate) content: String,
    pub(crate) timeout_secs: u64,
    pub(crate) timeout_reply: String,
    pub(crate) interrupt_rx: watch::Receiver<u64>,
    pub(crate) interrupt_generation: u64,
    pub(crate) interrupted_reply: String,
}

pub(crate) async fn run_foreground_turn_with_interrupt(
    request: ForegroundTurnRequest,
) -> ForegroundTurnOutcome {
    let ForegroundTurnRequest {
        agent,
        session_id,
        content,
        timeout_secs,
        timeout_reply,
        mut interrupt_rx,
        interrupt_generation,
        interrupted_reply,
    } = request;

    let mut turn_task = tokio::spawn(async move { agent.run_turn(&session_id, &content).await });
    let timeout = tokio::time::sleep(Duration::from_secs(timeout_secs));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            result = &mut turn_task => {
                return match result {
                    Ok(result) => map_turn_execution_result(result),
                    Err(join_error) => ForegroundTurnOutcome::Failed {
                        reply: "Error: foreground worker task failed unexpectedly.".to_string(),
                        error_chain: format!("foreground worker task join error: {join_error}"),
                        error_kind: "runtime_join",
                    },
                };
            }
            () = &mut timeout => {
                abort_turn_task(&mut turn_task).await;
                return ForegroundTurnOutcome::TimedOut { reply: timeout_reply };
            }
            changed = interrupt_rx.changed() => {
                if changed.is_ok() && *interrupt_rx.borrow() != interrupt_generation {
                    abort_turn_task(&mut turn_task).await;
                    return ForegroundTurnOutcome::Interrupted { reply: interrupted_reply };
                }
            }
        }
    }
}

async fn abort_turn_task(turn_task: &mut JoinHandle<Result<String, anyhow::Error>>) {
    turn_task.abort();
    let _ = tokio::time::timeout(Duration::from_millis(100), turn_task).await;
}

fn map_turn_execution_result(result: Result<String, anyhow::Error>) -> ForegroundTurnOutcome {
    match result {
        Ok(output) => ForegroundTurnOutcome::Succeeded(output),
        Err(error) => {
            let error_chain = format!("{error:#}");
            let error_kind = classify_turn_error(&error_chain);
            ForegroundTurnOutcome::Failed {
                reply: format!("Error: {error}"),
                error_chain,
                error_kind,
            }
        }
    }
}

pub(crate) fn classify_turn_error(error: &str) -> &'static str {
    let e = error.to_ascii_lowercase();
    if e.contains("tools/list") {
        "mcp_tools_list"
    } else if e.contains("tools/call") {
        "mcp_tools_call"
    } else if e.contains("transport send error") || e.contains("error sending request") {
        "mcp_transport"
    } else if e.contains("mcp handshake timeout") || e.contains("connect failed") {
        "mcp_connect"
    } else if e.contains("llm") {
        "llm"
    } else {
        "unknown"
    }
}
