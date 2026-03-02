//! Integration tests for hot-reload version backend adapters.

use anyhow::Result;
use xiuxian_qianhuan::ValkeyHotReloadVersionBackend;

#[test]
fn valkey_hot_reload_backend_rejects_invalid_url() {
    let result = ValkeyHotReloadVersionBackend::new("://invalid", None);
    assert!(result.is_err());
}

#[test]
fn valkey_hot_reload_backend_accepts_valid_url_shape() -> Result<()> {
    let backend = ValkeyHotReloadVersionBackend::new("redis://127.0.0.1:6379/0", Some("test"))?;
    let _ = backend;
    Ok(())
}
