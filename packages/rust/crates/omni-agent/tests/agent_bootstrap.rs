//! Top-level integration harness for `agent::bootstrap` helper contracts.

mod config {
    pub use omni_agent::{MemoryConfig, RuntimeSettings};

    #[derive(Debug, Clone, Default)]
    pub(crate) struct XiuxianConfig {
        _placeholder: bool,
    }
}

mod agent {
    mod bootstrap {
        mod service_mount {
            include!("../src/agent/bootstrap/service_mount.rs");
        }

        mod hot_reload {
            use std::path::{Path, PathBuf};

            use xiuxian_qianhuan::{
                resolve_hot_reload_watch_extensions, resolve_hot_reload_watch_patterns,
            };
            use xiuxian_wendao::IncrementalSyncPolicy;

            const DEFAULT_WENDAO_INCREMENTAL_EXTENSIONS: &[&str] =
                &["md", "markdown", "org", "orgm", "j2", "toml"];
            const DEFAULT_WENDAO_WATCH_PATTERNS: &[&str] = &[
                "**/*.md",
                "**/*.markdown",
                "**/*.org",
                "**/*.orgm",
                "**/*.j2",
                "**/*.toml",
            ];

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
        }

        mod memory {
            use xiuxian_llm::embedding::backend::{
                EmbeddingBackendKind, parse_embedding_backend_kind,
            };
            use xiuxian_macros::env_non_empty;

            use crate::config::RuntimeSettings;

            const DEFAULT_MEMORY_EMBED_BASE_URL: &str = "http://localhost:3002";
            const MISTRAL_SDK_INPROC_LABEL: &str = "inproc://mistral-sdk";

            fn trimmed_non_empty(raw: Option<&str>) -> Option<String> {
                raw.map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToString::to_string)
            }

            pub(super) fn resolve_memory_embed_base_url(
                memory_cfg: &crate::config::MemoryConfig,
                runtime_settings: &RuntimeSettings,
            ) -> String {
                let backend_hint = trimmed_non_empty(memory_cfg.embedding_backend.as_deref())
                    .or_else(|| {
                        trimmed_non_empty(runtime_settings.memory.embedding_backend.as_deref())
                    })
                    .or_else(|| trimmed_non_empty(runtime_settings.embedding.backend.as_deref()));
                if matches!(
                    parse_embedding_backend_kind(backend_hint.as_deref()),
                    Some(EmbeddingBackendKind::MistralSdk)
                ) {
                    return MISTRAL_SDK_INPROC_LABEL.to_string();
                }

                trimmed_non_empty(memory_cfg.embedding_base_url.as_deref())
                    .or_else(|| {
                        trimmed_non_empty(
                            env_non_empty!("OMNI_AGENT_MEMORY_EMBEDDING_BASE_URL").as_deref(),
                        )
                    })
                    .or_else(|| {
                        trimmed_non_empty(env_non_empty!("OMNI_AGENT_EMBED_BASE_URL").as_deref())
                    })
                    .or_else(|| {
                        trimmed_non_empty(runtime_settings.memory.embedding_base_url.as_deref())
                    })
                    .or_else(|| trimmed_non_empty(runtime_settings.embedding.client_url.as_deref()))
                    .or_else(|| {
                        trimmed_non_empty(runtime_settings.embedding.litellm_api_base.as_deref())
                    })
                    .or_else(|| trimmed_non_empty(runtime_settings.mistral.base_url.as_deref()))
                    .unwrap_or_else(|| DEFAULT_MEMORY_EMBED_BASE_URL.to_string())
            }
        }

        mod qianhuan {
            include!("../src/agent/bootstrap/qianhuan.rs");
        }

        mod zhenfa {
            use std::path::PathBuf;
            use std::sync::Arc;

            use xiuxian_wendao::SkillVfsResolver;

            use super::service_mount::{ServiceMountCatalog, ServiceMountMeta};

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
        }

        mod zhixing {
            use std::collections::HashMap;
            use std::path::{Path, PathBuf};

            use xiuxian_qianhuan::{ManifestationManager, MemoryTemplateRecord};
            use xiuxian_wendao::WendaoResourceUri;

            #[derive(Debug, Clone, Default, PartialEq, Eq)]
            pub(super) struct ZhixingSkillTemplateLoadSummary {
                pub(super) linked_ids: usize,
                pub(super) template_records: usize,
                pub(super) loaded_template_names: usize,
            }

            pub(super) fn resolve_project_root() -> PathBuf {
                let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                resolve_project_root_with_env("PRJ_ROOT", &current_dir)
            }

            pub(super) fn resolve_project_root_with_prj_root(
                prj_root: Option<&str>,
                current_dir: &Path,
            ) -> PathBuf {
                resolve_project_root_from_override(prj_root.map(str::to_owned), current_dir)
            }

            fn resolve_project_root_with_env(env_name: &str, current_dir: &Path) -> PathBuf {
                let env_override = std::env::var(env_name).ok();
                resolve_project_root_from_override(env_override, current_dir)
            }

            fn resolve_project_root_from_override(
                env_override: Option<String>,
                current_dir: &Path,
            ) -> PathBuf {
                if let Some(root) = env_override
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .map(PathBuf::from)
                {
                    return root;
                }

                let mut cursor = current_dir.to_path_buf();
                loop {
                    let has_git = cursor.join(".git").exists();
                    let has_system_config = cursor
                        .join("packages")
                        .join("conf")
                        .join("xiuxian.toml")
                        .is_file();
                    if has_git || has_system_config {
                        return cursor;
                    }
                    if !cursor.pop() {
                        return current_dir.to_path_buf();
                    }
                }
            }

            pub(super) fn resolve_prj_data_home(project_root: &Path) -> PathBuf {
                resolve_prj_data_home_from_override(
                    project_root,
                    std::env::var("PRJ_DATA_HOME").ok(),
                )
            }

            pub(super) fn resolve_prj_data_home_with_env(
                project_root: &Path,
                prj_data_home: Option<&str>,
            ) -> PathBuf {
                resolve_prj_data_home_from_override(project_root, prj_data_home.map(str::to_owned))
            }

            fn resolve_prj_data_home_from_override(
                project_root: &Path,
                prj_data_home: Option<String>,
            ) -> PathBuf {
                prj_data_home
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .map_or_else(|| project_root.join(".data"), PathBuf::from)
            }

            pub(super) fn resolve_notebook_root(
                prj_data_home: &Path,
                env_notebook_path: Option<String>,
                config_notebook_path: Option<String>,
            ) -> PathBuf {
                env_notebook_path
                    .map(PathBuf::from)
                    .or_else(|| config_notebook_path.map(PathBuf::from))
                    .unwrap_or_else(|| prj_data_home.join("xiuxian").join("notebook"))
            }

            pub(super) fn resolve_template_globs(
                project_root: &Path,
                config_template_paths: Option<Vec<String>>,
            ) -> Vec<String> {
                let executable_dir = std::env::current_exe()
                    .ok()
                    .and_then(|path| path.parent().map(Path::to_path_buf));
                resolve_template_globs_with_runtime_overrides(
                    project_root,
                    config_template_paths,
                    std::env::var("XIUXIAN_RESOURCE_ROOT").ok(),
                    executable_dir.as_deref(),
                )
            }

            fn dedup_paths_in_order(paths: Vec<PathBuf>) -> Vec<PathBuf> {
                let mut unique = Vec::new();
                for path in paths {
                    if !unique.contains(&path) {
                        unique.push(path);
                    }
                }
                unique
            }

            fn dedup_strings_in_order(values: Vec<String>) -> Vec<String> {
                let mut unique = Vec::new();
                for value in values {
                    if !unique.contains(&value) {
                        unique.push(value);
                    }
                }
                unique
            }

            pub(super) fn resolve_template_globs_with_resource_root(
                project_root: &Path,
                config_template_paths: Option<Vec<String>>,
                resource_root: Option<&str>,
            ) -> Vec<String> {
                resolve_template_globs_with_runtime_overrides(
                    project_root,
                    config_template_paths,
                    resource_root.map(str::to_owned),
                    None,
                )
            }

            fn resolve_template_globs_with_runtime_overrides(
                project_root: &Path,
                config_template_paths: Option<Vec<String>>,
                resource_root_override: Option<String>,
                executable_dir: Option<&Path>,
            ) -> Vec<String> {
                let mut roots = resolve_runtime_template_candidates(
                    project_root,
                    resource_root_override,
                    executable_dir,
                )
                .into_iter()
                .filter(|path| path.is_dir())
                .collect::<Vec<_>>();
                if let Some(custom_paths) = config_template_paths.filter(|paths| !paths.is_empty())
                {
                    roots.extend(
                        custom_paths
                            .into_iter()
                            .filter_map(|value| {
                                let trimmed = value.trim();
                                if trimmed.is_empty() {
                                    return None;
                                }
                                let path = PathBuf::from(trimmed);
                                Some(if path.is_absolute() {
                                    path
                                } else {
                                    project_root.join(path)
                                })
                            })
                            .filter(|path| path.is_dir())
                            .collect::<Vec<_>>(),
                    );
                }
                dedup_paths_in_order(roots)
                    .into_iter()
                    .map(|path| path.join("*.md").to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
            }

            fn resolve_runtime_template_candidates(
                project_root: &Path,
                resource_root_override: Option<String>,
                executable_dir: Option<&Path>,
            ) -> Vec<PathBuf> {
                let mut candidates = Vec::new();

                if let Some(resource_root) = resource_root_override
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .map(PathBuf::from)
                    .map(|path| {
                        if path.is_absolute() {
                            path
                        } else {
                            project_root.join(path)
                        }
                    })
                {
                    candidates.push(
                        resource_root
                            .join("omni-agent")
                            .join("zhixing")
                            .join("templates"),
                    );
                    candidates.push(resource_root.join("zhixing").join("templates"));
                }

                if let Some(executable_dir) = executable_dir {
                    candidates.push(
                        executable_dir
                            .join("..")
                            .join("resources")
                            .join("zhixing")
                            .join("templates"),
                    );
                    candidates.push(
                        executable_dir
                            .join("resources")
                            .join("zhixing")
                            .join("templates"),
                    );
                }

                candidates
            }

            pub(super) fn load_skill_templates_from_embedded_registry(
                manager: &ManifestationManager,
            ) -> Result<ZhixingSkillTemplateLoadSummary, String> {
                const TEMPLATE_CONFIG_TYPE: &str = "template";

                let registry = xiuxian_wendao::build_embedded_wendao_registry().map_err(|error| {
                    format!(
                        "failed to build embedded zhixing wendao registry for skill bridge: {error}"
                    )
                })?;
                let mut links_by_template_id: HashMap<String, Vec<String>> = HashMap::new();
                for file in registry.files() {
                    for (id, links) in file.links_by_id() {
                        let Some(block) = registry.get(id) else {
                            continue;
                        };
                        if !block.config_type.eq_ignore_ascii_case(TEMPLATE_CONFIG_TYPE) {
                            continue;
                        }
                        let entry = links_by_template_id.entry(id.clone()).or_default();
                        for link in links {
                            if !entry.iter().any(|existing| existing == link) {
                                entry.push(link.clone());
                            }
                        }
                    }
                }

                let mut id_links = links_by_template_id.into_iter().collect::<Vec<_>>();
                id_links.sort_by(|(left_id, _), (right_id, _)| left_id.cmp(right_id));

                let mut records = Vec::new();
                let mut linked_ids = 0usize;

                for (id, links) in id_links {
                    let deduped_links = dedup_strings_in_order(links);
                    if deduped_links.is_empty() {
                        continue;
                    }
                    linked_ids += 1;

                    let alias_target = (deduped_links.len() == 1).then(|| id.clone());
                    for link_uri in deduped_links {
                        if WendaoResourceUri::parse(link_uri.as_str()).is_err() {
                            return Err(format!(
                                "template link `{link_uri}` for id `{id}` must use semantic URI `wendao://skills/<name>/references/<entity>`"
                            ));
                        }
                        let Some(content) = xiuxian_wendao::embedded_resource_text_from_wendao_uri(
                            link_uri.as_str(),
                        ) else {
                            return Err(format!(
                                "linked template URI `{link_uri}` for id `{id}` not found in embedded zhixing resources"
                            ));
                        };
                        records.push(MemoryTemplateRecord::new(
                            link_uri,
                            alias_target.clone(),
                            content,
                        ));
                    }
                }

                let template_records = records.len();
                let loaded_template_names =
                    manager
                        .load_templates_from_memory(records)
                        .map_err(|error| {
                            format!("failed to load linked templates into manifestation: {error}")
                        })?;

                Ok(ZhixingSkillTemplateLoadSummary {
                    linked_ids,
                    template_records,
                    loaded_template_names,
                })
            }
        }

        mod tests {
            include!("unit/agent/bootstrap_tests.rs");
        }

        fn lint_symbol_probe() {
            let _ = service_mount::ServiceMountMeta::default().endpoint("probe");
            let _ = service_mount::ServiceMountMeta::default().storage("probe");
            let _ = zhixing::resolve_project_root as fn() -> std::path::PathBuf;
            let _ = zhixing::resolve_prj_data_home as fn(&std::path::Path) -> std::path::PathBuf;
        }

        const _: fn() = lint_symbol_probe;
    }
}
