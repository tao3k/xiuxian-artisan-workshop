use super::super::service_mount::{ServiceMountCatalog, ServiceMountMeta};
use super::constants::{
    HOT_RELOAD_DOMAIN, XIUXIAN_HOT_RELOAD_VALKEY_KEY_PREFIX_ENV, XIUXIAN_HOT_RELOAD_VALKEY_URL_ENV,
};
use crate::config::XiuxianConfig;
use crate::env_parse::resolve_valkey_url_env;
use std::sync::Arc;
use xiuxian_qianhuan::{HotReloadVersionBackend, ValkeyHotReloadVersionBackend};

pub(super) fn resolve_version_backend(
    xiuxian_cfg: &XiuxianConfig,
    mounts: &mut ServiceMountCatalog,
) -> Option<Arc<dyn HotReloadVersionBackend>> {
    let valkey_url = std::env::var(XIUXIAN_HOT_RELOAD_VALKEY_URL_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or(xiuxian_cfg.wendao.link_graph.cache.valkey_url.clone())
        .or_else(resolve_valkey_url_env);

    let Some(valkey_url) = valkey_url else {
        mounts.skipped(
            "hot_reload.version_backend",
            HOT_RELOAD_DOMAIN,
            ServiceMountMeta::default().detail("backend=local(no_valkey_url)"),
        );
        return None;
    };

    let key_prefix = std::env::var(XIUXIAN_HOT_RELOAD_VALKEY_KEY_PREFIX_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or(xiuxian_cfg.wendao.link_graph.cache.key_prefix.clone());

    match ValkeyHotReloadVersionBackend::new(&valkey_url, key_prefix.as_deref()) {
        Ok(backend) => {
            mounts.mounted(
                "hot_reload.version_backend",
                HOT_RELOAD_DOMAIN,
                ServiceMountMeta::default()
                    .endpoint(valkey_url)
                    .detail(format!(
                        "backend=valkey,key_prefix={}",
                        key_prefix.unwrap_or_default()
                    )),
            );
            Some(Arc::new(backend))
        }
        Err(error) => {
            mounts.failed(
                "hot_reload.version_backend",
                HOT_RELOAD_DOMAIN,
                ServiceMountMeta::default().detail(format!("backend=local(init_failed:{error})")),
            );
            None
        }
    }
}
