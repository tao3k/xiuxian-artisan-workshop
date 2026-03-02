use super::service_mount::{ServiceMountCatalog, ServiceMountMeta};
use super::zhixing::ZhixingRuntimeBundle;
use crate::config::XiuxianConfig;
use crate::env_parse::{parse_bool_from_env, parse_positive_u64_from_env};
#[cfg(test)]
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use xiuxian_qianhuan::{HotReloadDriver, HotReloadRuntime};
#[cfg(test)]
use xiuxian_wendao::IncrementalSyncPolicy;

mod backend;
mod constants;
mod prepared;

use backend::resolve_version_backend;
use constants::{
    DEFAULT_HOT_RELOAD_DEBOUNCE_MS, DEFAULT_HOT_RELOAD_SYNC_INTERVAL_MS, HOT_RELOAD_DOMAIN,
    XIUXIAN_HOT_RELOAD_DEBOUNCE_MS_ENV, XIUXIAN_HOT_RELOAD_ENABLED_ENV,
    XIUXIAN_HOT_RELOAD_SYNC_INTERVAL_MS_ENV,
};
use prepared::{
    apply_prepared_target, prepare_qianhuan_manifestation_target, prepare_wendao_index_target,
};

pub(super) async fn start_hot_reload_driver(
    zhixing_runtime: Option<&ZhixingRuntimeBundle>,
    xiuxian_cfg: &XiuxianConfig,
    mounts: &mut ServiceMountCatalog,
) -> Option<HotReloadDriver> {
    if !is_hot_reload_enabled() {
        mounts.skipped(
            "hot_reload.driver",
            HOT_RELOAD_DOMAIN,
            ServiceMountMeta::default().detail("disabled(by_env)"),
        );
        return None;
    }

    let backend = resolve_version_backend(xiuxian_cfg, mounts);
    let backend_name = if backend.is_some() { "valkey" } else { "local" };
    let runtime = Arc::new(HotReloadRuntime::new(backend));
    let mut target_ids = Vec::new();

    if let Some(bundle) = zhixing_runtime {
        let prepared_targets = vec![
            prepare_qianhuan_manifestation_target(bundle),
            prepare_wendao_index_target(bundle, xiuxian_cfg),
        ];
        for prepared_target in prepared_targets {
            apply_prepared_target(&runtime, mounts, &mut target_ids, prepared_target);
        }
    }

    if target_ids.is_empty() {
        mounts.skipped(
            "hot_reload.driver",
            HOT_RELOAD_DOMAIN,
            ServiceMountMeta::default().detail("disabled(no_targets_registered)"),
        );
        return None;
    }

    let debounce_ms = resolve_hot_reload_debounce_ms();
    let sync_interval = Duration::from_millis(resolve_hot_reload_sync_interval_ms());
    match HotReloadDriver::start(Arc::clone(&runtime), debounce_ms, sync_interval).await {
        Ok(Some(driver)) => {
            mounts.mounted(
                "hot_reload.driver",
                HOT_RELOAD_DOMAIN,
                ServiceMountMeta::default().detail(format!(
                    "backend={backend_name}, debounce_ms={debounce_ms}, sync_interval_ms={}, targets={}",
                    sync_interval.as_millis(),
                    target_ids.join(",")
                )),
            );
            Some(driver)
        }
        Ok(None) => {
            mounts.skipped(
                "hot_reload.driver",
                HOT_RELOAD_DOMAIN,
                ServiceMountMeta::default().detail("disabled(no_watch_roots)"),
            );
            None
        }
        Err(error) => {
            mounts.failed(
                "hot_reload.driver",
                HOT_RELOAD_DOMAIN,
                ServiceMountMeta::default().detail(format!("start failed: {error}")),
            );
            None
        }
    }
}

fn is_hot_reload_enabled() -> bool {
    parse_bool_from_env(XIUXIAN_HOT_RELOAD_ENABLED_ENV).unwrap_or(true)
}

fn resolve_hot_reload_debounce_ms() -> u64 {
    parse_positive_u64_from_env(XIUXIAN_HOT_RELOAD_DEBOUNCE_MS_ENV)
        .unwrap_or(DEFAULT_HOT_RELOAD_DEBOUNCE_MS)
        .max(1)
}

fn resolve_hot_reload_sync_interval_ms() -> u64 {
    parse_positive_u64_from_env(XIUXIAN_HOT_RELOAD_SYNC_INTERVAL_MS_ENV)
        .unwrap_or(DEFAULT_HOT_RELOAD_SYNC_INTERVAL_MS)
        .max(100)
}

#[cfg(test)]
pub(super) fn resolve_wendao_incremental_policy(
    patterns: &[String],
    configured_extensions: Option<&[String]>,
) -> IncrementalSyncPolicy {
    prepared::resolve_wendao_incremental_policy(patterns, configured_extensions)
}

#[cfg(test)]
pub(super) fn resolve_wendao_watch_patterns(
    configured_patterns: Option<&[String]>,
    configured_extensions: Option<&[String]>,
) -> Vec<String> {
    prepared::resolve_wendao_watch_patterns(configured_patterns, configured_extensions)
}

#[cfg(test)]
pub(super) fn resolve_wendao_watch_roots(
    project_root: &Path,
    default_notebook_root: &Path,
    watch_dirs: Option<&Vec<String>>,
    include_dirs: Option<&Vec<String>>,
) -> Vec<PathBuf> {
    prepared::resolve_wendao_watch_roots(
        project_root,
        default_notebook_root,
        watch_dirs,
        include_dirs,
    )
}
