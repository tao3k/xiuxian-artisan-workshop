//! MCP pool retry and fallback behavior tests.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Result, anyhow};
use rmcp::model::{CallToolResult, ListToolsResult};
use xiuxian_llm::mcp::{run_tool_call_with_retry, run_tools_list_with_fallback};

#[tokio::test]
async fn run_tools_list_with_fallback_uses_next_client_on_non_retryable_error() -> Result<()> {
    let attempts = Arc::new(Mutex::new(Vec::<usize>::new()));
    let reconnect_count = Arc::new(AtomicUsize::new(0));

    let result = run_tools_list_with_fallback(
        2,
        0,
        {
            let attempts = Arc::clone(&attempts);
            move |client_index| {
                let attempts = Arc::clone(&attempts);
                async move {
                    attempts
                        .lock()
                        .map_err(|_| anyhow!("attempt log lock poisoned"))?
                        .push(client_index);
                    if client_index == 0 {
                        Err(anyhow!("boom"))
                    } else {
                        Ok(ListToolsResult::with_all_items(vec![]))
                    }
                }
            }
        },
        {
            let reconnect_count = Arc::clone(&reconnect_count);
            move |_| {
                let reconnect_count = Arc::clone(&reconnect_count);
                async move {
                    reconnect_count.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            }
        },
    )
    .await?;

    let attempts = attempts
        .lock()
        .map_err(|_| anyhow!("attempt log lock poisoned"))?;
    assert_eq!(*attempts, vec![0, 1]);
    assert_eq!(reconnect_count.load(Ordering::Relaxed), 0);
    assert_eq!(result.tools.len(), 0);
    Ok(())
}

#[tokio::test]
async fn run_tools_list_with_fallback_retries_same_client_for_retryable_error() -> Result<()> {
    let call_count = Arc::new(AtomicUsize::new(0));
    let reconnect_count = Arc::new(AtomicUsize::new(0));

    let result = run_tools_list_with_fallback(
        1,
        0,
        {
            let call_count = Arc::clone(&call_count);
            move |_| {
                let call_count = Arc::clone(&call_count);
                async move {
                    let attempt = call_count.fetch_add(1, Ordering::Relaxed);
                    if attempt == 0 {
                        Err(anyhow!("connection refused"))
                    } else {
                        Ok(ListToolsResult::with_all_items(vec![]))
                    }
                }
            }
        },
        {
            let reconnect_count = Arc::clone(&reconnect_count);
            move |_| {
                let reconnect_count = Arc::clone(&reconnect_count);
                async move {
                    reconnect_count.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            }
        },
    )
    .await?;

    assert_eq!(call_count.load(Ordering::Relaxed), 2);
    assert_eq!(reconnect_count.load(Ordering::Relaxed), 1);
    assert_eq!(result.tools.len(), 0);
    Ok(())
}

#[tokio::test]
async fn run_tool_call_with_retry_reconnects_on_retryable_error() -> Result<()> {
    let call_count = Arc::new(AtomicUsize::new(0));
    let reconnect_count = Arc::new(AtomicUsize::new(0));

    let result = run_tool_call_with_retry(
        "tools/call:test",
        0,
        30,
        {
            let call_count = Arc::clone(&call_count);
            move || {
                let call_count = Arc::clone(&call_count);
                async move {
                    let attempt = call_count.fetch_add(1, Ordering::Relaxed);
                    if attempt == 0 {
                        Err(anyhow!("connection refused"))
                    } else {
                        Ok(CallToolResult::success(vec![]))
                    }
                }
            }
        },
        {
            let reconnect_count = Arc::clone(&reconnect_count);
            move || {
                let reconnect_count = Arc::clone(&reconnect_count);
                async move {
                    reconnect_count.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            }
        },
    )
    .await?;

    assert_eq!(call_count.load(Ordering::Relaxed), 2);
    assert_eq!(reconnect_count.load(Ordering::Relaxed), 1);
    assert_eq!(result.is_error, Some(false));
    Ok(())
}

#[tokio::test]
async fn run_tool_call_with_retry_skips_reconnect_for_non_retryable_error() {
    let reconnect_count = Arc::new(AtomicUsize::new(0));

    let result = run_tool_call_with_retry(
        "tools/call:test",
        0,
        30,
        || async { Err(anyhow!("boom")) },
        {
            let reconnect_count = Arc::clone(&reconnect_count);
            move || {
                let reconnect_count = Arc::clone(&reconnect_count);
                async move {
                    reconnect_count.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            }
        },
    )
    .await;

    assert!(result.is_err());
    assert_eq!(reconnect_count.load(Ordering::Relaxed), 0);
}
