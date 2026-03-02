use crate::hot_reload::HotReloadTarget;
use crate::interface::ManifestationInterface;
use crate::manifestation::request::{ManifestationRenderRequest, ManifestationTemplateTarget};
use crate::{InjectionWindowConfig, SystemPromptInjectionWindow};
use anyhow::{Context as AnyhowContext, Result, anyhow};
use globset::Glob;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tera::{Context, Tera};
use walkdir::WalkDir;

/// In-memory template payload resolved from external runtime indexes (for example Wendao).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryTemplateRecord {
    /// Exact logical identifier for the template payload.
    pub id: String,
    /// Optional render target alias (for example `daily_agenda.md`).
    pub target: Option<String>,
    /// Raw template source content.
    pub content: String,
}

impl MemoryTemplateRecord {
    /// Creates a memory template record.
    #[must_use]
    pub fn new(id: impl Into<String>, target: Option<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            target,
            content: content.into(),
        }
    }
}

/// Snapshot of one session-level system prompt injection payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSystemPromptInjectionSnapshot {
    /// Unix timestamp when this snapshot was updated (milliseconds).
    pub updated_at_unix_ms: u64,
    /// Number of retained `<qa>` entries after window normalization.
    pub qa_count: usize,
    /// Canonical XML payload.
    pub xml: String,
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

/// Parse and normalize one system prompt injection XML payload.
///
/// # Errors
///
/// Returns an error when XML parsing/normalization fails.
pub fn normalize_session_system_prompt_injection_xml(
    raw_xml: &str,
) -> Result<SessionSystemPromptInjectionSnapshot> {
    let window = SystemPromptInjectionWindow::from_xml(raw_xml, InjectionWindowConfig::default())?;
    Ok(SessionSystemPromptInjectionSnapshot {
        updated_at_unix_ms: now_unix_ms(),
        qa_count: window.len(),
        xml: window.render_xml(),
    })
}

/// Manager for the Manifestation (Qianhuan) layer.
///
/// Coordinates template rendering and dynamic context injection.
pub struct ManifestationManager {
    /// Embedded baseline templates bundled in the binary.
    embedded_templates: Vec<(String, String)>,
    /// Ordered glob patterns used to load templates.
    template_globs: Vec<String>,
    /// Compiled glob matchers and scan roots used for change detection.
    compiled_globs: Vec<CompiledTemplateGlob>,
    /// Session-scoped system prompt injection cache.
    session_prompt_injection: RwLock<HashMap<String, SessionSystemPromptInjectionSnapshot>>,
    /// Hot-reloadable template engine state.
    state: RwLock<ManifestationRuntimeState>,
}

impl ManifestationManager {
    /// Creates an empty `ManifestationManager` without disk or embedded templates.
    ///
    /// This constructor is intended for zero-export runtime loading where
    /// templates are injected from memory records after initialization.
    #[must_use]
    pub fn new_empty() -> Self {
        Self {
            embedded_templates: Vec::new(),
            template_globs: Vec::new(),
            compiled_globs: Vec::new(),
            session_prompt_injection: RwLock::new(HashMap::new()),
            state: RwLock::new(ManifestationRuntimeState {
                tera: Tera::default(),
                snapshot: TemplateSnapshot { files: Vec::new() },
                runtime_templates: BTreeMap::new(),
            }),
        }
    }

    /// Creates a new `ManifestationManager` with templates loaded from multiple glob patterns.
    ///
    /// # Errors
    /// Returns an error if any glob pattern is invalid or if loading fails.
    pub fn new(templates_globs: &[&str]) -> Result<Self> {
        Self::new_with_embedded_templates(templates_globs, &[])
    }

    /// Creates a new `ManifestationManager` with embedded baseline templates
    /// and optional runtime-loaded override globs.
    ///
    /// External templates loaded from `template_globs` override embedded
    /// templates when names collide.
    ///
    /// # Errors
    /// Returns an error if any glob pattern is invalid, if provided templates
    /// are invalid, or if no template can be loaded.
    pub fn new_with_embedded_templates(
        template_globs: &[&str],
        embedded_templates: &[(&str, &str)],
    ) -> Result<Self> {
        let template_patterns = template_globs
            .iter()
            .map(|glob| (*glob).to_string())
            .collect::<Vec<_>>();
        let embedded = embedded_templates
            .iter()
            .map(|(name, source)| ((*name).to_string(), (*source).to_string()))
            .collect::<Vec<_>>();
        let compiled_globs = compile_template_globs(&template_patterns)?;
        let tera = Self::load_tera(&embedded, &compiled_globs, &BTreeMap::new())?;
        let snapshot = capture_snapshot(&compiled_globs)?;

        log::debug!(
            "Registered templates: {:?}",
            tera.get_template_names().collect::<Vec<_>>()
        );

        Ok(Self {
            embedded_templates: embedded,
            template_globs: template_patterns,
            compiled_globs,
            session_prompt_injection: RwLock::new(HashMap::new()),
            state: RwLock::new(ManifestationRuntimeState {
                tera,
                snapshot,
                runtime_templates: BTreeMap::new(),
            }),
        })
    }

    /// Renders one logical target template with runtime-aware context injection.
    ///
    /// # Errors
    ///
    /// Returns an error when template rendering fails.
    pub fn render_request(&self, request: &ManifestationRenderRequest) -> Result<String> {
        let payload = self.build_injected_payload(&request.data, request);
        self.render_template(request.target.template_name(), payload)
    }

    /// Renders a specific logical template target using raw data without runtime enrichment.
    ///
    /// # Errors
    ///
    /// Returns an error when template rendering fails.
    pub fn render_target(
        &self,
        target: &ManifestationTemplateTarget,
        data: Value,
    ) -> Result<String> {
        self.render_template(target.template_name(), data)
    }

    /// Upserts one template payload from runtime memory.
    ///
    /// The record `id` becomes a directly renderable template name.
    /// When `target` is present, the same payload is also registered under
    /// that alias so logical render targets can resolve without file I/O.
    ///
    /// # Errors
    ///
    /// Returns an error when the template payload is invalid or renderer lock
    /// access fails.
    pub fn upsert_template_from_memory(&self, record: MemoryTemplateRecord) -> Result<()> {
        let _ = self.load_templates_from_memory(std::iter::once(record))?;
        Ok(())
    }

    /// Bulk-loads template payloads from runtime memory index records.
    ///
    /// # Errors
    ///
    /// Returns an error when any payload is invalid or renderer lock access
    /// fails.
    pub fn load_templates_from_memory<I>(&self, records: I) -> Result<usize>
    where
        I: IntoIterator<Item = MemoryTemplateRecord>,
    {
        let mut state = self
            .state
            .write()
            .map_err(|_| anyhow!("template renderer lock poisoned"))?;

        let mut loaded_names = 0usize;
        for record in records {
            loaded_names += upsert_runtime_template_record(&mut state.runtime_templates, record);
        }

        let tera = Self::load_tera(
            &self.embedded_templates,
            &self.compiled_globs,
            &state.runtime_templates,
        )?;
        state.tera = tera;

        Ok(loaded_names)
    }

    /// Returns watcher root directories derived from template globs.
    #[must_use]
    pub fn template_watch_roots(&self) -> Vec<PathBuf> {
        let mut roots = self
            .compiled_globs
            .iter()
            .map(|compiled| compiled.root_dir.clone())
            .collect::<Vec<_>>();
        roots.sort();
        roots.dedup();
        roots
    }

    /// Returns the raw template include patterns.
    #[must_use]
    pub fn template_watch_patterns(&self) -> Vec<String> {
        self.template_globs.clone()
    }

    /// Reloads templates when snapshot changes are detected.
    ///
    /// Returns `Ok(true)` when the in-memory renderer was refreshed.
    ///
    /// # Errors
    ///
    /// Returns an error when snapshot capture, template load, or lock access fails.
    pub fn reload_templates_if_changed(&self) -> Result<bool> {
        let current_snapshot = capture_snapshot(&self.compiled_globs)?;

        let runtime_templates = {
            let guard = self
                .state
                .read()
                .map_err(|_| anyhow!("template renderer lock poisoned"))?;
            if guard.snapshot == current_snapshot {
                return Ok(false);
            }
            guard.runtime_templates.clone()
        };

        let tera = Self::load_tera(
            &self.embedded_templates,
            &self.compiled_globs,
            &runtime_templates,
        )?;
        let mut guard = self
            .state
            .write()
            .map_err(|_| anyhow!("template renderer lock poisoned"))?;
        *guard = ManifestationRuntimeState {
            tera,
            snapshot: current_snapshot,
            runtime_templates,
        };
        log::info!("manifestation templates hot-reloaded");
        Ok(true)
    }

    /// Builds a reusable hot-reload target registration for this manager.
    ///
    /// # Errors
    ///
    /// Returns an error when target metadata is invalid.
    pub fn hot_reload_target(
        self: &Arc<Self>,
        target_id: impl Into<String>,
    ) -> Result<HotReloadTarget> {
        let manager = Arc::clone(self);
        HotReloadTarget::new(
            target_id,
            self.template_watch_roots(),
            self.template_watch_patterns(),
            Arc::new(move |_| manager.reload_templates_if_changed()),
        )
    }

    /// Upsert one session-level injection XML payload into in-memory cache.
    ///
    /// # Errors
    ///
    /// Returns an error when payload parsing fails.
    pub fn upsert_session_prompt_injection_xml(
        &self,
        session_id: &str,
        raw_xml: &str,
    ) -> Result<SessionSystemPromptInjectionSnapshot> {
        let snapshot = normalize_session_system_prompt_injection_xml(raw_xml)
            .context("invalid system prompt injection xml payload")?;
        self.upsert_session_prompt_injection_snapshot(session_id, snapshot.clone());
        Ok(snapshot)
    }

    /// Upsert one validated session-level injection snapshot into in-memory cache.
    pub fn upsert_session_prompt_injection_snapshot(
        &self,
        session_id: &str,
        snapshot: SessionSystemPromptInjectionSnapshot,
    ) {
        match self.session_prompt_injection.write() {
            Ok(mut cache) => {
                cache.insert(session_id.to_string(), snapshot);
            }
            Err(_) => {
                log::warn!(
                    "manifestation session prompt injection cache lock poisoned on upsert; drop update"
                );
            }
        }
    }

    /// Inspect one session-level injection snapshot from in-memory cache.
    #[must_use]
    pub fn inspect_session_prompt_injection(
        &self,
        session_id: &str,
    ) -> Option<SessionSystemPromptInjectionSnapshot> {
        let Ok(cache) = self.session_prompt_injection.read() else {
            log::warn!(
                "manifestation session prompt injection cache lock poisoned on inspect; return empty"
            );
            return None;
        };
        cache.get(session_id).cloned()
    }

    /// Clear one session-level injection snapshot from in-memory cache.
    ///
    /// Returns true when a snapshot existed and was removed.
    pub fn clear_session_prompt_injection(&self, session_id: &str) -> bool {
        let Ok(mut cache) = self.session_prompt_injection.write() else {
            log::warn!(
                "manifestation session prompt injection cache lock poisoned on clear; return false"
            );
            return false;
        };
        cache.remove(session_id).is_some()
    }

    fn build_injected_payload(&self, data: &Value, request: &ManifestationRenderRequest) -> Value {
        let mut root = match data.clone() {
            Value::Object(map) => map,
            payload => {
                let mut map = Map::new();
                map.insert("payload".to_string(), payload);
                map
            }
        };

        let mut qianhuan = Map::new();
        if let Some(state_context) = request.runtime.state_context.as_deref() {
            qianhuan.insert("state_context".to_string(), json!(state_context));
            qianhuan.insert(
                "injected_context".to_string(),
                json!(self.inject_context(state_context)),
            );
        }
        if let Some(persona_id) = request.runtime.persona_id.as_deref() {
            qianhuan.insert("persona_id".to_string(), json!(persona_id));
        }
        if let Some(domain) = request.runtime.domain.as_deref() {
            qianhuan.insert("domain".to_string(), json!(domain));
        }
        if !request.runtime.extra.is_empty() {
            qianhuan.insert("extra".to_string(), json!(request.runtime.extra));
        }
        root.insert("qianhuan".to_string(), Value::Object(qianhuan));

        Value::Object(root)
    }

    fn load_tera(
        embedded_templates: &[(String, String)],
        compiled_globs: &[CompiledTemplateGlob],
        runtime_templates: &BTreeMap<String, String>,
    ) -> Result<Tera> {
        let mut tera = Tera::default();

        for (template_name, template_source) in embedded_templates {
            tera.add_raw_template(template_name, template_source)
                .map_err(|error| {
                    anyhow!("failed to add embedded template '{template_name}': {error}")
                })?;
        }

        for (template_name, template_path) in collect_external_templates(compiled_globs)? {
            tera.add_template_file(&template_path, Some(template_name.as_str()))
                .map_err(|error| {
                    anyhow!(
                        "failed to add external template '{}' from {}: {error}",
                        template_name,
                        template_path.display()
                    )
                })?;
        }

        for (template_name, template_source) in runtime_templates {
            tera.add_raw_template(template_name, template_source)
                .map_err(|error| {
                    anyhow!("failed to add memory template '{template_name}' from runtime index: {error}")
                })?;
        }

        if tera.get_template_names().next().is_none() {
            return Err(anyhow!(
                "no manifestation templates available: embedded templates and external globs are both empty"
            ));
        }

        Ok(tera)
    }

    fn refresh_templates_best_effort(&self) {
        if let Err(error) = self.reload_templates_if_changed() {
            log::warn!(
                "manifestation template hot-reload failed; keeping previous template state: {error}"
            );
        }
    }
}

impl ManifestationInterface for ManifestationManager {
    fn render_template(&self, template_name: &str, data: Value) -> Result<String> {
        self.refresh_templates_best_effort();
        let context =
            Context::from_value(data).map_err(|e| anyhow!("Failed to create context: {e}"))?;

        let state = self
            .state
            .read()
            .map_err(|_| anyhow!("template renderer lock poisoned"))?;
        state
            .tera
            .render(template_name, &context)
            .map_err(|e| anyhow!("Template rendering error: {e}"))
    }

    fn inject_context(&self, state_context: &str) -> String {
        match state_context {
            "STALE_TASKS" => {
                "### Cognitive Interface Warning: Your vows are decaying. Focus on completion to avoid mental blockage."
                    .to_string()
            },
            "SUCCESS_STREAK" => {
                "### Cognitive Interface Praise: Your path is clear. Knowledge and Action are in harmony."
                    .to_string()
            },
            _ => "### Cognitive Interface Presence: Ready to guide your path.".to_string(),
        }
    }
}

#[derive(Debug)]
struct ManifestationRuntimeState {
    tera: Tera,
    snapshot: TemplateSnapshot,
    runtime_templates: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TemplateSnapshot {
    files: Vec<TemplateFileStamp>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TemplateFileStamp {
    path: PathBuf,
    modified_unix_millis: u128,
    size_bytes: u64,
}

#[derive(Debug)]
struct CompiledTemplateGlob {
    pattern: String,
    matcher: globset::GlobMatcher,
    root_dir: PathBuf,
}

fn upsert_runtime_template_record(
    runtime_templates: &mut BTreeMap<String, String>,
    record: MemoryTemplateRecord,
) -> usize {
    let MemoryTemplateRecord {
        id,
        target,
        content,
    } = record;

    runtime_templates.insert(id.clone(), content.clone());
    let mut loaded_names = 1usize;

    if let Some(target_name) = target
        && target_name != id
    {
        runtime_templates.insert(target_name, content);
        loaded_names += 1;
    }

    loaded_names
}

fn compile_template_globs(template_globs: &[String]) -> Result<Vec<CompiledTemplateGlob>> {
    template_globs
        .iter()
        .map(|pattern| {
            let glob = Glob::new(pattern)
                .map_err(|error| anyhow!("Invalid template glob '{pattern}': {error}"))?;
            Ok(CompiledTemplateGlob {
                pattern: pattern.clone(),
                matcher: glob.compile_matcher(),
                root_dir: derive_glob_root(pattern),
            })
        })
        .collect()
}

fn derive_glob_root(pattern: &str) -> PathBuf {
    let wildcard_pos = pattern
        .find(|ch| ['*', '?', '[', '{'].contains(&ch))
        .unwrap_or(pattern.len());
    let prefix = &pattern[..wildcard_pos];

    if wildcard_pos == pattern.len() {
        let path = Path::new(prefix);
        return path
            .parent()
            .map(Path::to_path_buf)
            .filter(|value| !value.as_os_str().is_empty())
            .unwrap_or_else(|| PathBuf::from("."));
    }

    let prefix_path = Path::new(prefix);
    if prefix.ends_with('/') || prefix.ends_with('\\') {
        let candidate = PathBuf::from(prefix_path);
        if candidate.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            candidate
        }
    } else {
        prefix_path
            .parent()
            .map(Path::to_path_buf)
            .filter(|value| !value.as_os_str().is_empty())
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

fn capture_snapshot(compiled_globs: &[CompiledTemplateGlob]) -> Result<TemplateSnapshot> {
    let mut files = Vec::new();

    for compiled in compiled_globs {
        if !compiled.root_dir.exists() {
            continue;
        }
        for entry in WalkDir::new(&compiled.root_dir) {
            let entry = entry.map_err(|error| {
                anyhow!(
                    "failed to walk template root {} for pattern {}: {error}",
                    compiled.root_dir.display(),
                    compiled.pattern
                )
            })?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if !compiled.matcher.is_match(path) {
                continue;
            }
            let metadata = std::fs::metadata(path).map_err(|error| {
                anyhow!("failed to stat template file {}: {error}", path.display())
            })?;
            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let modified_unix_millis = modified
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            files.push(TemplateFileStamp {
                path: path.to_path_buf(),
                modified_unix_millis,
                size_bytes: metadata.len(),
            });
        }
    }

    files.sort();
    files.dedup();
    Ok(TemplateSnapshot { files })
}

fn collect_external_templates(
    compiled_globs: &[CompiledTemplateGlob],
) -> Result<Vec<(String, PathBuf)>> {
    let mut discovered: BTreeMap<String, PathBuf> = BTreeMap::new();

    for compiled in compiled_globs {
        if !compiled.root_dir.exists() {
            continue;
        }
        let mut matched_paths = Vec::new();
        for entry in WalkDir::new(&compiled.root_dir) {
            let entry = entry.map_err(|error| {
                anyhow!(
                    "failed to walk template root {} for pattern {}: {error}",
                    compiled.root_dir.display(),
                    compiled.pattern
                )
            })?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if !compiled.matcher.is_match(path) {
                continue;
            }
            matched_paths.push(path.to_path_buf());
        }
        matched_paths.sort();
        for path in matched_paths {
            let template_name = template_name_from_path(&compiled.root_dir, &path);
            discovered.insert(template_name, path);
        }
    }

    Ok(discovered.into_iter().collect())
}

fn template_name_from_path(root_dir: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root_dir).unwrap_or(path);
    let normalized = relative.to_string_lossy().replace('\\', "/");
    if normalized.trim().is_empty() {
        path.file_name().map_or_else(
            || path.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
    } else {
        normalized
    }
}
