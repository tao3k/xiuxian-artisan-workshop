//! Regression guard for redis connection lock anti-patterns.

use std::path::PathBuf;

const FORBIDDEN_PATTERNS: &[&str] = &[
    "Arc<Mutex<Option<redis::aio::MultiplexedConnection>>>",
    "Mutex<Option<redis::aio::MultiplexedConnection>>",
];

const GUARDED_FILES: &[&str] = &[
    "src/session/redis_backend/backend.rs",
    "src/channels/telegram/channel/send_gate.rs",
    "src/channels/telegram/session_gate/valkey/mod.rs",
    "src/channels/telegram/idempotency.rs",
    "src/agent/zhenfa/valkey_hooks.rs",
];

#[test]
fn redis_backends_do_not_use_mutex_wrapped_multiplexed_connection() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for relative in GUARDED_FILES {
        let source_path = crate_root.join(relative);
        let source = std::fs::read_to_string(&source_path).unwrap_or_else(|error| {
            panic!(
                "failed to read guarded source file {}: {error}",
                source_path.display()
            )
        });
        for pattern in FORBIDDEN_PATTERNS {
            assert!(
                !source.contains(pattern),
                "forbidden redis connection lock pattern `{pattern}` found in {}",
                source_path.display()
            );
        }
    }
}
