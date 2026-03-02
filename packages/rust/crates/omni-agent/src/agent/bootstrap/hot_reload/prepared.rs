use super::super::service_mount::{ServiceMountCatalog, ServiceMountMeta};
use super::super::zhixing::{ZhixingRuntimeBundle, resolve_project_root};
use super::constants::{
    DEFAULT_WENDAO_INCREMENTAL_EXTENSIONS, DEFAULT_WENDAO_WATCH_PATTERNS, HOT_RELOAD_DOMAIN,
    HOT_RELOAD_TARGET_QIANHUAN_MANIFESTATION, HOT_RELOAD_TARGET_WENDAO_INDEX,
    TARGET_ID_QIANHUAN_MANIFESTATION, TARGET_ID_WENDAO_INDEX,
};
use crate::config::XiuxianConfig;
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use xiuxian_qianhuan::{
    HotReloadInvocation, HotReloadRuntime, HotReloadTarget, resolve_hot_reload_watch_extensions,
    resolve_hot_reload_watch_patterns,
};
use xiuxian_wendao::IncrementalSyncPolicy;

pub(super) fn prepare_qianhuan_manifestation_target(
    runtime_bundle: &ZhixingRuntimeBundle,
) -> PreparedHotReloadTarget {
    let target_id = TARGET_ID_QIANHUAN_MANIFESTATION.to_string();
    let roots = runtime_bundle.manifestation_manager.template_watch_roots();
    if roots.is_empty() {
        return PreparedHotReloadTarget::Skipped {
            mount_id: HOT_RELOAD_TARGET_QIANHUAN_MANIFESTATION,
            detail: format!("id={target_id}, no_watch_roots(external_template_globs_empty)"),
        };
    }
    match Arc::clone(&runtime_bundle.manifestation_manager).hot_reload_target(target_id.as_str()) {
        Ok(target) => {
            let roots = summarize_paths(target.roots());
            let patterns = target.include_globs().join(",");
            PreparedHotReloadTarget::Ready(RegisteredHotReloadTarget::new(
                HOT_RELOAD_TARGET_QIANHUAN_MANIFESTATION,
                target_id.clone(),
                target,
                format!("id={target_id},roots={roots},patterns={patterns}"),
            ))
        }
        Err(error) => PreparedHotReloadTarget::Failed {
            mount_id: HOT_RELOAD_TARGET_QIANHUAN_MANIFESTATION,
            detail: format!("id={target_id}, build failed: {error}"),
        },
    }
}

pub(super) fn prepare_wendao_index_target(
    runtime_bundle: &ZhixingRuntimeBundle,
    xiuxian_cfg: &XiuxianConfig,
) -> PreparedHotReloadTarget {
    let target_id = TARGET_ID_WENDAO_INDEX.to_string();
    let project_root = resolve_project_root();
    let roots = resolve_wendao_watch_roots(
        &project_root,
        &runtime_bundle.heyi.storage.root_dir,
        xiuxian_cfg.wendao.link_graph.watch_dirs.as_ref(),
        xiuxian_cfg.wendao.link_graph.include_dirs.as_ref(),
    );
    if roots.is_empty() {
        return PreparedHotReloadTarget::Skipped {
            mount_id: HOT_RELOAD_TARGET_WENDAO_INDEX,
            detail: format!("id={target_id}, no_roots"),
        };
    }

    let patterns = resolve_wendao_watch_patterns(
        xiuxian_cfg.wendao.link_graph.watch_patterns.as_deref(),
        xiuxian_cfg.wendao.link_graph.watch_extensions.as_deref(),
    );
    let incremental_policy = resolve_wendao_incremental_policy(
        &patterns,
        xiuxian_cfg.wendao.link_graph.watch_extensions.as_deref(),
    );
    let incremental_extensions = incremental_policy.extensions().join("|");
    let heyi = Arc::clone(&runtime_bundle.heyi);

    match HotReloadTarget::new(
        target_id.as_str(),
        roots.clone(),
        patterns.clone(),
        Arc::new(move |invocation| match invocation {
            HotReloadInvocation::LocalPathChange { path } => heyi
                .sync_changed_path_from_disk(path, &incremental_policy)
                .map_err(|error| anyhow!("{error}")),
            HotReloadInvocation::RemoteVersionSync => heyi
                .sync_from_disk()
                .map(|_| true)
                .map_err(|error| anyhow!("{error}")),
        }),
    ) {
        Ok(target) => PreparedHotReloadTarget::Ready(RegisteredHotReloadTarget::new(
            HOT_RELOAD_TARGET_WENDAO_INDEX,
            target_id.clone(),
            target,
            format!(
                "id={target_id},mode=heyi_sync_incremental_or_full,roots={},patterns={},extensions={}",
                summarize_paths(&roots),
                patterns.join(","),
                incremental_extensions
            ),
        )),
        Err(error) => PreparedHotReloadTarget::Failed {
            mount_id: HOT_RELOAD_TARGET_WENDAO_INDEX,
            detail: format!("id={target_id}, build failed: {error}"),
        },
    }
}

pub(super) enum PreparedHotReloadTarget {
    Ready(RegisteredHotReloadTarget),
    Skipped {
        mount_id: &'static str,
        detail: String,
    },
    Failed {
        mount_id: &'static str,
        detail: String,
    },
}

pub(super) struct RegisteredHotReloadTarget {
    mount_id: &'static str,
    target_id: String,
    target: HotReloadTarget,
    detail: String,
}

impl RegisteredHotReloadTarget {
    pub(super) fn new(
        mount_id: &'static str,
        target_id: String,
        target: HotReloadTarget,
        detail: String,
    ) -> Self {
        Self {
            mount_id,
            target_id,
            target,
            detail,
        }
    }
}

pub(super) fn apply_prepared_target(
    runtime: &Arc<HotReloadRuntime>,
    mounts: &mut ServiceMountCatalog,
    target_ids: &mut Vec<String>,
    prepared_target: PreparedHotReloadTarget,
) {
    match prepared_target {
        PreparedHotReloadTarget::Ready(registration) => {
            register_runtime_target(runtime, mounts, target_ids, registration);
        }
        PreparedHotReloadTarget::Skipped { mount_id, detail } => {
            mounts.skipped(
                mount_id,
                HOT_RELOAD_DOMAIN,
                ServiceMountMeta::default().detail(detail),
            );
        }
        PreparedHotReloadTarget::Failed { mount_id, detail } => {
            mounts.failed(
                mount_id,
                HOT_RELOAD_DOMAIN,
                ServiceMountMeta::default().detail(detail),
            );
        }
    }
}

fn register_runtime_target(
    runtime: &Arc<HotReloadRuntime>,
    mounts: &mut ServiceMountCatalog,
    target_ids: &mut Vec<String>,
    registration: RegisteredHotReloadTarget,
) {
    let RegisteredHotReloadTarget {
        mount_id,
        target_id,
        target,
        detail,
    } = registration;
    match runtime.register_target(target) {
        Ok(()) => {
            target_ids.push(target_id);
            mounts.mounted(
                mount_id,
                HOT_RELOAD_DOMAIN,
                ServiceMountMeta::default().detail(detail),
            );
        }
        Err(error) => {
            mounts.failed(
                mount_id,
                HOT_RELOAD_DOMAIN,
                ServiceMountMeta::default()
                    .detail(format!("id={target_id}, register failed: {error}")),
            );
        }
    }
}

pub(super) fn resolve_wendao_incremental_policy(
    patterns: &[String],
    configured_extensions: Option<&[String]>,
) -> IncrementalSyncPolicy {
    let explicit = resolve_hot_reload_watch_extensions(configured_extensions, &[]);
    IncrementalSyncPolicy::from_patterns_and_extensions(
        patterns,
        &explicit,
        DEFAULT_WENDAO_INCREMENTAL_EXTENSIONS,
    )
}

pub(super) fn resolve_wendao_watch_patterns(
    configured_patterns: Option<&[String]>,
    configured_extensions: Option<&[String]>,
) -> Vec<String> {
    resolve_hot_reload_watch_patterns(
        configured_patterns,
        configured_extensions,
        DEFAULT_WENDAO_WATCH_PATTERNS,
    )
}

pub(super) fn resolve_wendao_watch_roots(
    project_root: &Path,
    default_notebook_root: &Path,
    watch_dirs: Option<&Vec<String>>,
    include_dirs: Option<&Vec<String>>,
) -> Vec<PathBuf> {
    let configured = watch_dirs
        .filter(|paths| !paths.is_empty())
        .or(include_dirs.filter(|paths| !paths.is_empty()));
    let mut roots = configured.map_or_else(
        || vec![default_notebook_root.to_path_buf()],
        |paths| {
            paths
                .iter()
                .filter_map(|value| resolve_path(project_root, value))
                .collect::<Vec<_>>()
        },
    );
    roots.sort();
    roots.dedup();
    roots
}

fn resolve_path(project_root: &Path, raw: &str) -> Option<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(stripped) = trimmed.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
        && !home.trim().is_empty()
    {
        return Some(PathBuf::from(home).join(stripped));
    }
    let candidate = PathBuf::from(trimmed);
    if candidate.is_absolute() {
        Some(candidate)
    } else {
        Some(project_root.join(candidate))
    }
}

fn summarize_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(",")
}
