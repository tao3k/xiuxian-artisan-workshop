use super::valkey_hooks::{build_zhenfa_orchestrator_hooks, resolve_zhenfa_valkey_hook_config};
use crate::config::XiuxianConfig;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, ZhenfaOrchestrator, ZhenfaRegistry, ZhenfaTool};

#[test]
fn resolve_zhenfa_valkey_hook_config_returns_none_without_url() {
    let config = XiuxianConfig::default();
    assert!(resolve_zhenfa_valkey_hook_config(&config).is_none());
}

#[test]
fn resolve_zhenfa_valkey_hook_config_applies_defaults() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.valkey.url = Some("redis://127.0.0.1:6379/0".to_string());
    let resolved = resolve_zhenfa_valkey_hook_config(&config)
        .unwrap_or_else(|| panic!("valkey config should resolve"));
    assert_eq!(resolved.url, "redis://127.0.0.1:6379/0");
    assert_eq!(resolved.key_prefix, "omni:zhenfa");
    assert_eq!(resolved.cache_ttl_seconds, 120);
    assert_eq!(resolved.lock_ttl_seconds, 15);
    assert_eq!(resolved.audit_stream, "dispatch.audit");
}

#[test]
fn build_zhenfa_orchestrator_hooks_returns_hooks_when_url_is_configured() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.valkey.url = Some("redis://127.0.0.1:6379/0".to_string());
    let hooks = build_zhenfa_orchestrator_hooks(&config)
        .unwrap_or_else(|| panic!("zhenfa valkey hooks should be enabled"));
    assert!(hooks.cache.is_some());
    assert!(hooks.mutation_lock.is_some());
    assert!(hooks.audit_sink.is_some());
}

struct LiveCachedTool {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl ZhenfaTool for LiveCachedTool {
    fn id(&self) -> &'static str {
        "live.cache.echo"
    }

    fn definition(&self) -> Value {
        json!({
            "name": self.id(),
            "description": "Live cached echo tool for valkey hook e2e verification",
            "parameters": {
                "type": "object",
                "properties": { "value": { "type": "string" } },
                "required": ["value"]
            }
        })
    }

    async fn call_native(&self, _ctx: &ZhenfaContext, args: Value) -> Result<String, ZhenfaError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let value = args
            .get("value")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        Ok(format!("<cached>{value}</cached>"))
    }

    fn cache_key(&self, _ctx: &ZhenfaContext, args: &Value) -> Option<String> {
        let value = args.get("value")?.as_str().unwrap_or_default();
        Some(format!("live.cache.echo::{value}"))
    }
}

struct LiveMutationTool {
    calls: Arc<AtomicUsize>,
    hold_ms: u64,
}

#[async_trait]
impl ZhenfaTool for LiveMutationTool {
    fn id(&self) -> &'static str {
        "live.mutation.write"
    }

    fn definition(&self) -> Value {
        json!({
            "name": self.id(),
            "description": "Live mutation tool for lock verification",
            "parameters": {
                "type": "object",
                "properties": { "value": { "type": "string" } },
                "required": ["value"]
            }
        })
    }

    async fn call_native(&self, _ctx: &ZhenfaContext, args: Value) -> Result<String, ZhenfaError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(self.hold_ms)).await;
        let value = args
            .get("value")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        Ok(format!("<mutation>{value}</mutation>"))
    }

    fn mutation_scope(&self, _ctx: &ZhenfaContext, _args: &Value) -> Option<String> {
        Some("agenda.write.live".to_string())
    }
}

fn resolve_live_valkey_url() -> Option<String> {
    for key in ["VALKEY_URL", "XIUXIAN_WENDAO_VALKEY_URL"] {
        let value = std::env::var(key).ok().unwrap_or_default();
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn purge_valkey_prefix(url: &str, key_prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open(url)?;
    let mut connection = client.get_connection()?;
    let pattern = format!("{key_prefix}:*");
    let mut cursor = 0_u64;
    loop {
        let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(&pattern)
            .arg("COUNT")
            .arg(200)
            .query(&mut connection)?;
        if !keys.is_empty() {
            let _: i64 = redis::cmd("DEL").arg(keys).query(&mut connection)?;
        }
        if next_cursor == 0 {
            break;
        }
        cursor = next_cursor;
    }
    Ok(())
}

fn stream_len(url: &str, key: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let client = redis::Client::open(url)?;
    let mut connection = client.get_connection()?;
    let len: u64 = redis::cmd("XLEN")
        .arg(key)
        .query(&mut connection)
        .unwrap_or(0);
    Ok(len)
}

async fn assert_cached_dispatch_uses_valkey_cache(
    orchestrator: &ZhenfaOrchestrator,
    ctx: &ZhenfaContext,
    cached_calls: &Arc<AtomicUsize>,
) {
    let first_cached = orchestrator
        .dispatch("live.cache.echo", ctx, json!({ "value": "alpha" }))
        .await
        .unwrap_or_else(|error| panic!("first cached dispatch should succeed: {error}"));
    let second_cached = orchestrator
        .dispatch("live.cache.echo", ctx, json!({ "value": "alpha" }))
        .await
        .unwrap_or_else(|error| {
            panic!("second cached dispatch should succeed from cache: {error}")
        });
    assert_eq!(first_cached, "<cached>alpha</cached>");
    assert_eq!(second_cached, "<cached>alpha</cached>");
    assert_eq!(
        cached_calls.load(Ordering::SeqCst),
        1,
        "second deterministic dispatch should be served from valkey cache"
    );
}

async fn assert_mutation_lock_contention(
    orchestrator: &ZhenfaOrchestrator,
    ctx: &ZhenfaContext,
    mutation_calls: &Arc<AtomicUsize>,
) {
    let orchestrator_clone = orchestrator.clone();
    let ctx_clone = ctx.clone();
    let first_mutation = tokio::spawn(async move {
        orchestrator_clone
            .dispatch(
                "live.mutation.write",
                &ctx_clone,
                json!({ "value": "first" }),
            )
            .await
    });
    tokio::time::sleep(Duration::from_millis(40)).await;
    let lock_error = match orchestrator
        .dispatch("live.mutation.write", ctx, json!({ "value": "second" }))
        .await
    {
        Ok(value) => {
            panic!("second mutation dispatch should fail while lock is held, got output: {value}")
        }
        Err(error) => error,
    };
    match lock_error {
        ZhenfaError::Execution {
            code: Some(code),
            message,
        } => {
            assert_eq!(code, -32011);
            assert!(
                message.contains("mutation lock busy"),
                "expected lock busy message, got: {message}"
            );
        }
        other => panic!("expected lock busy execution error, got: {other}"),
    }
    let first_mutation_output = first_mutation
        .await
        .unwrap_or_else(|error| panic!("join first mutation task: {error}"))
        .unwrap_or_else(|error| panic!("first mutation dispatch should succeed: {error}"));
    assert_eq!(first_mutation_output, "<mutation>first</mutation>");
    assert_eq!(
        mutation_calls.load(Ordering::SeqCst),
        1,
        "lock contention path should prevent second mutation execution"
    );
}

#[tokio::test]
#[ignore = "requires live valkey server and socket access"]
async fn live_valkey_hooks_cover_cache_lock_and_audit_stream() {
    let Some(valkey_url) = resolve_live_valkey_url() else {
        eprintln!("skip: set VALKEY_URL or XIUXIAN_WENDAO_VALKEY_URL for live zhenfa hook test");
        return;
    };

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|error| panic!("system clock before unix epoch: {error}"))
        .as_nanos();
    let key_prefix = format!("omni:zhenfa:test:live:{}:{nanos}", std::process::id());
    let mut config = XiuxianConfig::default();
    config.zhenfa.valkey.url = Some(valkey_url.clone());
    config.zhenfa.valkey.key_prefix = Some(key_prefix.clone());
    config.zhenfa.valkey.cache_ttl_seconds = Some(60);
    config.zhenfa.valkey.lock_ttl_seconds = Some(5);
    config.zhenfa.valkey.audit_stream = Some("dispatch.audit".to_string());

    purge_valkey_prefix(&valkey_url, &key_prefix)
        .unwrap_or_else(|error| panic!("purge live valkey prefix before test: {error}"));

    let hooks = build_zhenfa_orchestrator_hooks(&config)
        .unwrap_or_else(|| panic!("zhenfa valkey hooks should be enabled for live test"));
    let mut registry = ZhenfaRegistry::new();
    let cached_calls = Arc::new(AtomicUsize::new(0));
    let mutation_calls = Arc::new(AtomicUsize::new(0));
    registry.register(Arc::new(LiveCachedTool {
        calls: Arc::clone(&cached_calls),
    }));
    registry.register(Arc::new(LiveMutationTool {
        calls: Arc::clone(&mutation_calls),
        hold_ms: 220,
    }));
    let orchestrator = ZhenfaOrchestrator::with_hooks(registry, hooks);
    let mut ctx = ZhenfaContext::default();
    ctx.session_id = Some("live:valkey:hooks".to_string());

    assert_cached_dispatch_uses_valkey_cache(&orchestrator, &ctx, &cached_calls).await;
    assert_mutation_lock_contention(&orchestrator, &ctx, &mutation_calls).await;

    let audit_stream = format!("{key_prefix}:stream:dispatch.audit");
    let audit_count = stream_len(&valkey_url, &audit_stream)
        .unwrap_or_else(|error| panic!("read audit stream len: {error}"));
    assert!(
        audit_count >= 4,
        "expected >=4 audit events (success + cached + lock fail + success), got {audit_count}"
    );

    purge_valkey_prefix(&valkey_url, &key_prefix)
        .unwrap_or_else(|error| panic!("purge live valkey prefix after test: {error}"));
}
