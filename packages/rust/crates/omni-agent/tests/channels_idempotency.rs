#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use omni_agent::{DEFAULT_REDIS_KEY_PREFIX, WebhookDedupBackend, WebhookDedupConfig};

#[tokio::test]
async fn memory_store_marks_duplicate_ids() -> anyhow::Result<()> {
    let store = WebhookDedupConfig::default().build_store()?;
    assert!(!store.is_duplicate(42).await?);
    assert!(store.is_duplicate(42).await?);
    Ok(())
}

#[tokio::test]
async fn memory_store_expires_ids_after_ttl() -> anyhow::Result<()> {
    const TTL_SECS: u64 = 1;
    const MAX_WAIT_SECS: u64 = 2;
    const POLL_INTERVAL_MS: u64 = 50;

    let store = WebhookDedupConfig {
        backend: WebhookDedupBackend::Memory,
        ttl_secs: TTL_SECS,
    }
    .build_store()?;
    assert!(!store.is_duplicate(7).await?);

    let wait_started = tokio::time::Instant::now();
    loop {
        if !store.is_duplicate(7).await? {
            break;
        }

        if wait_started.elapsed() >= Duration::from_secs(MAX_WAIT_SECS) {
            panic!("memory dedup entry did not expire within {MAX_WAIT_SECS}s");
        }

        tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    Ok(())
}

#[test]
fn redis_config_normalizes_empty_prefix() {
    let config = WebhookDedupConfig {
        backend: WebhookDedupBackend::Redis {
            url: "redis://valkey.local:6379/0".to_string(),
            key_prefix: "".to_string(),
        },
        ttl_secs: 0,
    }
    .normalized();
    assert_eq!(config.ttl_secs, 1);
    match config.backend {
        WebhookDedupBackend::Redis { key_prefix, .. } => {
            assert_eq!(key_prefix, DEFAULT_REDIS_KEY_PREFIX);
        }
        WebhookDedupBackend::Memory => panic!("unexpected memory backend"),
    }
}

#[tokio::test]
#[ignore = "requires live valkey server and network access"]
async fn redis_store_marks_duplicate_ids() -> anyhow::Result<()> {
    let url = std::env::var("VALKEY_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("skip: set VALKEY_URL for live dedup test"))?;
    let run_id = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros();
    let key_prefix = format!("omni-agent:test:dedup:{run_id}");

    let store = WebhookDedupConfig {
        backend: WebhookDedupBackend::Redis { url, key_prefix },
        ttl_secs: 600,
    }
    .build_store()?;
    assert!(!store.is_duplicate(42).await?);
    assert!(store.is_duplicate(42).await?);
    Ok(())
}
