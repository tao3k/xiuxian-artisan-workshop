//! Integration tests for `omni-io` watcher configuration.

#![cfg(feature = "notify")]

use omni_io::WatcherConfig;

#[tokio::test]
async fn test_watcher_config() {
    let config = WatcherConfig::default();
    assert!(config.patterns.contains(&"**/*".to_string()));
    assert!(config.exclude.iter().any(|e| e.contains("*.pyc")));
}
