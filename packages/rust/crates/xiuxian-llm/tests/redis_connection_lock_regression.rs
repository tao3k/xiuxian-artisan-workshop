//! Regression guard for discover cache redis connection lock anti-patterns.

use std::path::PathBuf;

const FORBIDDEN_PATTERNS: &[&str] = &[
    "Arc<Mutex<Option<redis::aio::MultiplexedConnection>>>",
    "Mutex<Option<redis::aio::MultiplexedConnection>>",
];

#[test]
fn discover_cache_backend_does_not_use_mutex_wrapped_multiplexed_connection() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_path = crate_root.join("src/mcp/discover_cache.rs");
    let source = std::fs::read_to_string(&source_path).unwrap_or_else(|error| {
        panic!(
            "failed to read discover cache source file {}: {error}",
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
