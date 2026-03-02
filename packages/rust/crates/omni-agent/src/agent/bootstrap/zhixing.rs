use super::service_mount::{ServiceMountCatalog, ServiceMountMeta};
use crate::config::load_xiuxian_config;
use crate::env_parse::{
    parse_positive_u64_from_env, parse_positive_usize_from_env, resolve_valkey_url_env,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use xiuxian_qianhuan::{
    ManifestationInterface, ManifestationManager, MemoryTemplateRecord, PersonaProfile,
};
use xiuxian_wendao::WendaoResourceUri;

const XIUXIAN_WENDAO_NOTEBOOK_PATH_ENV: &str = "XIUXIAN_WENDAO_NOTEBOOK_PATH";
const XIUXIAN_RESOURCE_ROOT_ENV: &str = "XIUXIAN_RESOURCE_ROOT";
const XIUXIAN_ZHIXING_PERSONA_ID_ENV: &str = "XIUXIAN_ZHIXING_PERSONA_ID";
const XIUXIAN_ZHIXING_REMINDER_KEY_PREFIX_ENV: &str = "XIUXIAN_ZHIXING_REMINDER_KEY_PREFIX";
const XIUXIAN_ZHIXING_REMINDER_POLL_INTERVAL_SECONDS_ENV: &str =
    "XIUXIAN_ZHIXING_REMINDER_POLL_INTERVAL_SECONDS";
const XIUXIAN_ZHIXING_REMINDER_POLL_BATCH_SIZE_ENV: &str =
    "XIUXIAN_ZHIXING_REMINDER_POLL_BATCH_SIZE";
const OMNI_AGENT_NOTIFICATION_RECIPIENT_ENV: &str = "OMNI_AGENT_NOTIFICATION_RECIPIENT";
const DEFAULT_ZHIXING_PERSONA_ID: &str = "agenda_steward";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ZhixingSkillTemplateLoadSummary {
    pub(super) linked_ids: usize,
    pub(super) template_records: usize,
    pub(super) loaded_template_names: usize,
}

pub(super) struct ZhixingRuntimeBundle {
    pub(super) heyi: Arc<super::super::ZhixingHeyi>,
    pub(super) manifestation_manager: Arc<ManifestationManager>,
}

pub(super) fn resolve_project_root() -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    resolve_project_root_with_env("PRJ_ROOT", &current_dir)
}

#[cfg(test)]
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

fn resolve_project_root_from_override(env_override: Option<String>, current_dir: &Path) -> PathBuf {
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
    resolve_prj_data_home_from_override(project_root, std::env::var("PRJ_DATA_HOME").ok())
}

#[cfg(test)]
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
        std::env::var(XIUXIAN_RESOURCE_ROOT_ENV).ok(),
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

fn collect_embedded_utf8_files_under(root: &include_dir::Dir<'_>) -> Vec<(String, String)> {
    let root_prefix = root.path().to_string_lossy().replace('\\', "/");
    let mut files = Vec::new();
    collect_embedded_utf8_files(root, root_prefix.as_str(), &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn collect_embedded_utf8_files(
    dir: &include_dir::Dir<'_>,
    root_prefix: &str,
    out: &mut Vec<(String, String)>,
) {
    for file in dir.files() {
        let Some(content) = file.contents_utf8() else {
            continue;
        };
        let path = file.path().to_string_lossy().replace('\\', "/");
        let relative_path = path
            .strip_prefix(root_prefix)
            .map_or(path.as_str(), |value| value.trim_start_matches('/'))
            .to_string();
        out.push((relative_path, content.to_string()));
    }
    for child in dir.dirs() {
        collect_embedded_utf8_files(child, root_prefix, out);
    }
}

#[cfg(test)]
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
    let mut roots =
        resolve_runtime_template_candidates(project_root, resource_root_override, executable_dir)
            .into_iter()
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
    if let Some(custom_paths) = config_template_paths.filter(|paths| !paths.is_empty()) {
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
        format!("failed to build embedded zhixing wendao registry for skill bridge: {error}")
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
            let Some(content) =
                xiuxian_wendao::embedded_resource_text_from_wendao_uri(link_uri.as_str())
            else {
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
    let loaded_template_names = manager
        .load_templates_from_memory(records)
        .map_err(|error| format!("failed to load linked templates into manifestation: {error}"))?;

    Ok(ZhixingSkillTemplateLoadSummary {
        linked_ids,
        template_records,
        loaded_template_names,
    })
}

fn resolve_active_persona_id(xiuxian_toml_cfg: &crate::config::XiuxianConfig) -> String {
    std::env::var(XIUXIAN_ZHIXING_PERSONA_ID_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or(xiuxian_toml_cfg.wendao.zhixing.persona_id.clone())
        .unwrap_or_else(|| DEFAULT_ZHIXING_PERSONA_ID.to_string())
}

fn resolve_active_persona(
    persona_registries: &super::qianhuan::LoadedPersonaRegistries,
    persona_id: &str,
    mounts: &mut ServiceMountCatalog,
) -> Option<PersonaProfile> {
    if let Some(persona) = persona_registries.internal.get(persona_id) {
        mounts.mounted(
            "zhixing.persona",
            "orchestration",
            ServiceMountMeta::default().detail(format!(
                "id={}, source=internal, name={}",
                persona.id, persona.name
            )),
        );
        return Some(persona);
    }

    if persona_id != "cyber-cultivator"
        && let Some(fallback) = persona_registries.internal.get("cyber-cultivator")
    {
        mounts.mounted(
            "zhixing.persona",
            "orchestration",
            ServiceMountMeta::default().detail(format!(
                "id={}, source=internal_fallback(cyber-cultivator), name={}",
                fallback.id, fallback.name
            )),
        );
        return Some(fallback);
    }

    mounts.skipped(
        "zhixing.persona",
        "orchestration",
        ServiceMountMeta::default().detail(format!("id={persona_id}, not_found")),
    );
    None
}

fn init_manifestation_manager(
    project_root: &Path,
    template_paths: Option<Vec<String>>,
    notebook_root: &Path,
    mounts: &mut ServiceMountCatalog,
) -> Option<(
    Arc<ManifestationManager>,
    Vec<String>,
    ZhixingSkillTemplateLoadSummary,
)> {
    let template_globs = resolve_template_globs(project_root, template_paths);
    let template_glob_refs = template_globs
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let builtin_templates = xiuxian_zhixing::RESOURCES
        .get_dir("zhixing/templates")
        .map(collect_embedded_utf8_files_under)
        .unwrap_or_default();
    let builtin_template_refs = builtin_templates
        .iter()
        .map(|(name, source)| (name.as_str(), source.as_str()))
        .collect::<Vec<_>>();

    match xiuxian_qianhuan::ManifestationManager::new_with_embedded_templates(
        &template_glob_refs,
        &builtin_template_refs,
    ) {
        Ok(manager) => {
            let manager = Arc::new(manager);
            match load_skill_templates_from_embedded_registry(manager.as_ref()) {
                Ok(summary) => Some((manager, template_globs, summary)),
                Err(error) => {
                    mounts.failed(
                        "zhixing.skill_template_bridge",
                        "workflow",
                        ServiceMountMeta::default()
                            .storage(notebook_root.display().to_string())
                            .detail(format!("embedded skill bridge failed: {error}")),
                    );
                    None
                }
            }
        }
        Err(error) => {
            mounts.failed(
                "zhixing.heyi",
                "workflow",
                ServiceMountMeta::default()
                    .storage(notebook_root.display().to_string())
                    .detail(format!("manifestation init failed: {error}")),
            );
            None
        }
    }
}

pub(super) fn init_zhixing_runtime(
    persona_registries: &super::qianhuan::LoadedPersonaRegistries,
    mounts: &mut ServiceMountCatalog,
) -> Option<ZhixingRuntimeBundle> {
    let xiuxian_toml_cfg = load_xiuxian_config();
    let project_root = resolve_project_root();
    let prj_data_home = resolve_prj_data_home(&project_root);
    let scope_key = project_root.to_string_lossy().into_owned();

    let notebook_root = resolve_notebook_root(
        &prj_data_home,
        std::env::var(XIUXIAN_WENDAO_NOTEBOOK_PATH_ENV).ok(),
        xiuxian_toml_cfg.wendao.zhixing.notebook_path.clone(),
    );

    let time_zone = std::env::var("TZ")
        .ok()
        .or(xiuxian_toml_cfg.wendao.zhixing.time_zone.clone())
        .unwrap_or_else(|| "UTC".to_string());
    let active_persona = resolve_active_persona(
        persona_registries,
        &resolve_active_persona_id(&xiuxian_toml_cfg),
        mounts,
    );

    let storage = Arc::new(xiuxian_zhixing::storage::MarkdownStorage::new(
        notebook_root.clone(),
    ));
    let graph = if let Ok(Some(cached_graph)) =
        xiuxian_wendao::kg_cache::load_from_valkey_cached(&scope_key)
    {
        Arc::new(cached_graph)
    } else {
        Arc::new(xiuxian_wendao::graph::KnowledgeGraph::new())
    };

    let (manifestation_manager, template_globs, skill_template_summary) =
        init_manifestation_manager(
            &project_root,
            xiuxian_toml_cfg.wendao.zhixing.template_paths.clone(),
            &notebook_root,
            mounts,
        )?;
    let manifestation: Arc<dyn ManifestationInterface> = manifestation_manager.clone();
    let reminder_queue = resolve_reminder_queue_store(&xiuxian_toml_cfg, &scope_key, mounts);
    match super::super::ZhixingHeyi::new(
        graph,
        Arc::clone(&manifestation),
        storage,
        scope_key,
        &time_zone,
    )
    .map(|heyi| {
        heyi.with_reminder_queue(reminder_queue)
            .with_active_persona(active_persona)
    }) {
        Ok(heyi) => {
            heyi.backfill_reminder_queue();
            mounts.mounted(
                "zhixing.heyi",
                "workflow",
                ServiceMountMeta::default()
                    .storage(notebook_root.display().to_string())
                    .detail(format!(
                        "time_zone={time_zone}, embedded_baseline=true, skill_registry=embedded, skill_linked_ids={}, skill_template_records={}, skill_template_names={}, template_globs={}",
                        skill_template_summary.linked_ids,
                        skill_template_summary.template_records,
                        skill_template_summary.loaded_template_names,
                        if template_globs.is_empty() {
                            "<none>".to_string()
                        } else {
                            template_globs.join(",")
                        }
                    )),
            );
            let heyi = Arc::new(heyi);
            Some(ZhixingRuntimeBundle {
                heyi,
                manifestation_manager,
            })
        }
        Err(error) => {
            mounts.failed(
                "zhixing.heyi",
                "workflow",
                ServiceMountMeta::default()
                    .storage(notebook_root.display().to_string())
                    .detail(format!("heyi init failed: {error}")),
            );
            None
        }
    }
}

pub(super) fn mount_zhixing_services(
    heyi: &Arc<super::super::ZhixingHeyi>,
    native_tools: &mut super::super::NativeToolRegistry,
    mounts: &mut ServiceMountCatalog,
) {
    native_tools.register(Arc::new(
        super::super::native_tools::zhixing::JournalRecordTool {
            heyi: Arc::clone(heyi),
        },
    ));
    native_tools.register(Arc::new(super::super::native_tools::zhixing::TaskAddTool {
        heyi: Arc::clone(heyi),
    }));
    native_tools.register(Arc::new(
        super::super::native_tools::zhixing::AgendaViewTool {
            heyi: Arc::clone(heyi),
        },
    ));
    mounts.mounted(
        "zhixing.native_tools",
        "tooling",
        ServiceMountMeta::default().detail("tools=journal.record,task.add,agenda.view"),
    );

    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let _watcher_handle = Arc::clone(heyi).start_timer_watcher(tx);
    let poll_interval_seconds = heyi.reminder_queue.as_ref().map_or(
        60,
        xiuxian_zhixing::ReminderQueueStore::poll_interval_seconds,
    );
    let backend = if heyi.reminder_queue.is_some() {
        "valkey_due_queue"
    } else {
        "graph_scan"
    };
    mounts.mounted(
        "zhixing.timer_watcher",
        "scheduler",
        ServiceMountMeta::default().detail(format!(
            "interval={poll_interval_seconds}s, queue=100, backend={backend}"
        )),
    );

    let default_notification_recipient = resolve_default_notification_recipient();
    let dispatcher = Arc::new(super::super::notification::NotificationDispatcher::new());
    mounts.mounted(
        "notification.dispatcher",
        "notification",
        ServiceMountMeta::default().detail(format!(
            "providers=telegram,discord,linux,llm,default_recipient={}",
            default_notification_recipient
                .as_deref()
                .unwrap_or("<none>")
        )),
    );
    spawn_reminder_notification_worker(
        rx,
        dispatcher,
        Arc::clone(heyi),
        default_notification_recipient,
    );
}

fn spawn_reminder_notification_worker(
    mut rx: tokio::sync::mpsc::Receiver<xiuxian_zhixing::ReminderSignal>,
    dispatcher: Arc<super::super::notification::NotificationDispatcher>,
    heyi: Arc<super::super::ZhixingHeyi>,
    fallback_recipient: Option<String>,
) {
    tokio::spawn(async move {
        dispatcher
            .register(Arc::new(
                super::super::notification::telegram::TelegramProvider::new(),
            ))
            .await;
        dispatcher
            .register(Arc::new(
                super::super::notification::discord::DiscordProvider::new(),
            ))
            .await;
        dispatcher
            .register(Arc::new(
                super::super::notification::linux::LinuxProvider::new(),
            ))
            .await;
        dispatcher
            .register(Arc::new(super::super::notification::llm::LlmProvider::new()))
            .await;

        while let Some(signal) = rx.recv().await {
            let Some(recipient) = signal
                .recipient
                .as_deref()
                .or(fallback_recipient.as_deref())
            else {
                tracing::warn!(
                    task_title = %signal.title,
                    "reminder recipient missing in task metadata and no default recipient configured; skipping reminder"
                );
                continue;
            };

            let content = match heyi.render_reminder_notice_markdown_v2(&signal) {
                Ok(payload) => payload,
                Err(error) => {
                    tracing::warn!(
                        task_title = %signal.title,
                        error = %error,
                        "failed to render reminder_notice.md; skipping reminder dispatch"
                    );
                    continue;
                }
            };
            if let Err(error) = dispatcher.dispatch(recipient, &content).await {
                tracing::warn!(
                    task_title = %signal.title,
                    recipient = %recipient,
                    error = %error,
                    "failed to dispatch reminder notification"
                );
            }
        }
    });
}

fn resolve_default_notification_recipient() -> Option<String> {
    let config_value = load_xiuxian_config()
        .wendao
        .zhixing
        .notification_recipient
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let env_value = std::env::var(OMNI_AGENT_NOTIFICATION_RECIPIENT_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    config_value.or(env_value)
}

fn resolve_reminder_queue_store(
    xiuxian_toml_cfg: &crate::config::XiuxianConfig,
    scope_key: &str,
    mounts: &mut ServiceMountCatalog,
) -> Option<xiuxian_zhixing::ReminderQueueStore> {
    let valkey_url = resolve_valkey_url_env()
        .or(xiuxian_toml_cfg
            .wendao
            .zhixing
            .reminder_queue
            .valkey_url
            .clone())
        .or(xiuxian_toml_cfg.wendao.link_graph.cache.valkey_url.clone());
    let Some(valkey_url) = valkey_url else {
        mounts.mounted(
            "zhixing.reminder_queue",
            "scheduler",
            ServiceMountMeta::default().detail("disabled(no_valkey_url)"),
        );
        return None;
    };

    let key_prefix = std::env::var(XIUXIAN_ZHIXING_REMINDER_KEY_PREFIX_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or(xiuxian_toml_cfg
            .wendao
            .zhixing
            .reminder_queue
            .key_prefix
            .clone());
    let poll_interval_seconds = parse_positive_u64_from_env(
        XIUXIAN_ZHIXING_REMINDER_POLL_INTERVAL_SECONDS_ENV,
    )
    .or(xiuxian_toml_cfg
        .wendao
        .zhixing
        .reminder_queue
        .poll_interval_seconds);
    let poll_batch_size = parse_positive_usize_from_env(
        XIUXIAN_ZHIXING_REMINDER_POLL_BATCH_SIZE_ENV,
    )
    .or(xiuxian_toml_cfg
        .wendao
        .zhixing
        .reminder_queue
        .poll_batch_size);

    let settings = xiuxian_zhixing::ReminderQueueSettings::with_defaults(
        valkey_url,
        key_prefix,
        poll_interval_seconds,
        poll_batch_size,
    );
    match xiuxian_zhixing::ReminderQueueStore::new(settings.clone(), scope_key.to_string()) {
        Ok(store) => {
            mounts.mounted(
                "zhixing.reminder_queue",
                "scheduler",
                ServiceMountMeta::default().detail(format!(
                    "enabled(prefix={}, interval={}s, batch={})",
                    settings.key_prefix, settings.poll_interval_seconds, settings.poll_batch_size
                )),
            );
            Some(store)
        }
        Err(error) => {
            mounts.failed(
                "zhixing.reminder_queue",
                "scheduler",
                ServiceMountMeta::default().detail(format!("init failed: {error}")),
            );
            None
        }
    }
}
