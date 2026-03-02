use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::agent::memory_state::MemoryStateBackend;
use crate::agent::zhenfa::{ZhenfaRuntimeDeps, ZhenfaToolBridge};
use crate::config::XiuxianConfig;
use omni_memory::EpisodeStore;
use xiuxian_wendao::{LinkGraphIndex, SkillVfsResolver};

use super::service_mount::{ServiceMountCatalog, ServiceMountMeta};
use super::zhixing::ZhixingRuntimeBundle;

pub(super) fn build_global_link_graph_index(
    xiuxian_cfg: &XiuxianConfig,
    zhixing_runtime: Option<&ZhixingRuntimeBundle>,
    service_mounts: &mut ServiceMountCatalog,
) -> Option<Arc<LinkGraphIndex>> {
    let root_dir = resolve_link_graph_root_dir(xiuxian_cfg, zhixing_runtime);
    let Some(root_dir) = root_dir else {
        service_mounts.skipped(
            "wendao.link_graph_index",
            "storage",
            ServiceMountMeta::default().detail("no_notebook_root_configured"),
        );
        return None;
    };

    let endpoint = root_dir.to_string_lossy().to_string();
    match LinkGraphIndex::build(&root_dir) {
        Ok(index) => {
            service_mounts.mounted(
                "wendao.link_graph_index",
                "storage",
                ServiceMountMeta::default()
                    .endpoint(endpoint)
                    .detail("mode=startup_singleton"),
            );
            Some(Arc::new(index))
        }
        Err(error) => {
            tracing::warn!(
                event = "agent.bootstrap.wendao.index_failed",
                root_dir = %root_dir.display(),
                error = %error,
                "failed to build startup wendao link graph index"
            );
            service_mounts.skipped(
                "wendao.link_graph_index",
                "storage",
                ServiceMountMeta::default()
                    .endpoint(endpoint)
                    .detail(format!("index_build_failed: {error}")),
            );
            None
        }
    }
}

/// Initialize optional zhenfa tool bridge from merged `xiuxian.toml`.
pub(super) fn init_zhenfa_tool_bridge(
    xiuxian_cfg: &XiuxianConfig,
    zhixing_runtime: Option<&ZhixingRuntimeBundle>,
    global_link_graph_index: Option<Arc<LinkGraphIndex>>,
    memory_store: Option<Arc<EpisodeStore>>,
    memory_state_backend: Option<Arc<MemoryStateBackend>>,
    service_mounts: &mut ServiceMountCatalog,
) -> Option<Arc<ZhenfaToolBridge>> {
    let skill_vfs_resolver = build_skill_vfs_resolver(service_mounts);
    let deps = ZhenfaRuntimeDeps {
        manifestation_manager: zhixing_runtime
            .map(|runtime| Arc::clone(&runtime.manifestation_manager)),
        link_graph_index: global_link_graph_index,
        skill_vfs_resolver,
        memory_store,
        memory_state_backend,
    };

    if let Some(bridge) = ZhenfaToolBridge::from_xiuxian_config(xiuxian_cfg, &deps) {
        let endpoint = bridge.base_url().map(str::to_string);
        let detail = format!(
            "mode=native;tools={};valkey_hooks={}",
            bridge.tool_count(),
            if bridge.valkey_hooks_enabled() {
                "enabled"
            } else {
                "disabled"
            }
        );
        service_mounts.mounted(
            "zhenfa.tool_bridge",
            "tooling",
            ServiceMountMeta::default()
                .endpoint(endpoint.unwrap_or_else(|| "in_process".to_string()))
                .detail(detail),
        );
        return Some(Arc::new(bridge));
    }

    service_mounts.skipped(
        "zhenfa.tool_bridge",
        "tooling",
        ServiceMountMeta::default().detail("disabled(no_native_tools_enabled)"),
    );
    None
}

fn resolve_link_graph_root_dir(
    xiuxian_cfg: &XiuxianConfig,
    zhixing_runtime: Option<&ZhixingRuntimeBundle>,
) -> Option<PathBuf> {
    if let Some(runtime) = zhixing_runtime {
        return Some(runtime.heyi.storage.root_dir.clone());
    }

    xiuxian_cfg
        .wendao
        .zhixing
        .notebook_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn build_skill_vfs_resolver(
    service_mounts: &mut ServiceMountCatalog,
) -> Option<Arc<SkillVfsResolver>> {
    let mut roots = resolve_skill_vfs_roots();
    roots.retain(|path| path.exists() && path.is_dir());
    dedup_paths(&mut roots);
    build_skill_vfs_resolver_from_roots(roots.as_slice(), service_mounts)
}

pub(super) fn build_skill_vfs_resolver_from_roots(
    roots: &[PathBuf],
    service_mounts: &mut ServiceMountCatalog,
) -> Option<Arc<SkillVfsResolver>> {
    match SkillVfsResolver::from_roots_with_embedded(roots) {
        Ok(resolver) => {
            let namespaces = resolver.index().namespace_count();
            let roots_detail = if roots.is_empty() {
                "none".to_string()
            } else {
                roots
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            };
            service_mounts.mounted(
                "zhenfa.skill_vfs",
                "storage",
                ServiceMountMeta::default().detail(format!(
                    "namespaces={namespaces},roots={roots_detail},embedded=true"
                )),
            );
            Some(Arc::new(resolver))
        }
        Err(error) => {
            let roots_detail = roots
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(",");
            service_mounts.skipped(
                "zhenfa.skill_vfs",
                "storage",
                ServiceMountMeta::default().detail(format!(
                    "skill_vfs_build_failed: {error};roots={roots_detail}"
                )),
            );
            None
        }
    }
}

fn resolve_skill_vfs_roots() -> Vec<PathBuf> {
    let project_root = super::zhixing::resolve_project_root();
    let crates_root = project_root.join("packages").join("rust").join("crates");
    let mut roots = discover_crate_skill_roots(crates_root.as_path());
    roots.push(project_root.join("assets").join("skills"));

    if let Some(config_home) = std::env::var("PRJ_CONFIG_HOME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        roots.push(config_home.join("xiuxian-artisan-workshop").join("skills"));
    } else {
        roots.push(
            project_root
                .join(".config")
                .join("xiuxian-artisan-workshop")
                .join("skills"),
        );
    }

    if let Some(resource_root) = std::env::var("XIUXIAN_RESOURCE_ROOT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        let normalized = if resource_root.is_absolute() {
            resource_root
        } else {
            project_root.join(resource_root)
        };
        roots.push(normalized.join("skills"));
    }

    if let Ok(executable_path) = std::env::current_exe()
        && let Some(executable_dir) = executable_path.parent()
    {
        roots.push(executable_dir.join("resources").join("skills"));
        roots.push(executable_dir.join("..").join("resources").join("skills"));
    }

    roots
}

fn dedup_paths(paths: &mut Vec<PathBuf>) {
    let mut unique = Vec::new();
    for path in std::mem::take(paths) {
        if !unique.contains(&path) {
            unique.push(path);
        }
    }
    *paths = unique;
}

fn discover_crate_skill_roots(crates_root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(crates_root) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(|crate_dir| crate_dir.join("resources").join("skills"))
        .collect()
}
