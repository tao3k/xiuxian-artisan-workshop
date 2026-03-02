use super::hot_reload::{
    resolve_wendao_incremental_policy, resolve_wendao_watch_patterns, resolve_wendao_watch_roots,
};
use super::memory::resolve_memory_embed_base_url;
use super::qianhuan::init_persona_registries;
use super::service_mount::{ServiceMountCatalog, ServiceMountStatus};
use super::zhenfa::build_skill_vfs_resolver_from_roots;
use super::zhixing::{
    load_skill_templates_from_embedded_registry, resolve_notebook_root,
    resolve_prj_data_home_with_env, resolve_project_root_with_prj_root, resolve_template_globs,
    resolve_template_globs_with_resource_root,
};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use xiuxian_qianhuan::{ManifestationInterface, ManifestationManager};

#[test]
fn resolve_project_root_prefers_prj_root_env() {
    let resolved =
        resolve_project_root_with_prj_root(Some("/tmp/xiuxian-root"), Path::new("/tmp/project"));
    assert_eq!(resolved, PathBuf::from("/tmp/xiuxian-root"));
}

#[test]
fn resolve_prj_data_home_prefers_env_then_defaults() {
    let project_root = Path::new("/tmp/project");
    assert_eq!(
        resolve_prj_data_home_with_env(project_root, Some("/tmp/custom-data")),
        PathBuf::from("/tmp/custom-data")
    );
    assert_eq!(
        resolve_prj_data_home_with_env(project_root, None),
        PathBuf::from("/tmp/project/.data")
    );
}

#[test]
fn resolve_notebook_root_precedence() {
    let data_home = Path::new("/tmp/project/.data");

    let from_env = resolve_notebook_root(
        data_home,
        Some("/tmp/notebook-env".to_string()),
        Some("/tmp/notebook-config".to_string()),
    );
    assert_eq!(from_env, PathBuf::from("/tmp/notebook-env"));

    let from_config = resolve_notebook_root(data_home, None, Some("/tmp/notebook-config".into()));
    assert_eq!(from_config, PathBuf::from("/tmp/notebook-config"));

    let fallback = resolve_notebook_root(data_home, None, None);
    assert_eq!(
        fallback,
        PathBuf::from("/tmp/project/.data/xiuxian/notebook")
    );
}

#[test]
fn resolve_memory_embed_base_url_uses_inproc_label_for_mistral_sdk_backend() {
    let memory_cfg = crate::config::MemoryConfig {
        embedding_backend: Some("mistral_sdk".to_string()),
        embedding_base_url: Some("http://127.0.0.1:3002".to_string()),
        ..crate::config::MemoryConfig::default()
    };

    let mut runtime_settings = crate::config::RuntimeSettings::default();
    runtime_settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());
    runtime_settings.mistral.base_url = Some("http://127.0.0.1:11500".to_string());

    let resolved = resolve_memory_embed_base_url(&memory_cfg, &runtime_settings);
    assert_eq!(resolved, "inproc://mistral-sdk");
}

#[test]
fn resolve_template_globs_prefers_configured_existing_paths() {
    let project_root = std::env::temp_dir().join(format!(
        "xiuxian-template-globs-project-{}",
        std::process::id()
    ));
    let relative_templates = project_root.join("custom/templates");
    let absolute_templates = std::env::temp_dir().join(format!(
        "xiuxian-template-globs-absolute-{}",
        std::process::id()
    ));
    if let Err(error) = fs::create_dir_all(&relative_templates) {
        panic!("create relative templates dir: {error}");
    }
    if let Err(error) = fs::create_dir_all(&absolute_templates) {
        panic!("create absolute templates dir: {error}");
    }

    let globs = resolve_template_globs(
        &project_root,
        Some(vec![
            "custom/templates".to_string(),
            absolute_templates.display().to_string(),
            "   ".to_string(),
        ]),
    );
    assert_eq!(
        globs,
        vec![
            relative_templates.join("*.md").display().to_string(),
            absolute_templates.join("*.md").display().to_string()
        ]
    );

    let _ = fs::remove_dir_all(&project_root);
    let _ = fs::remove_dir_all(&absolute_templates);
}

#[test]
fn resolve_template_globs_returns_empty_when_no_external_paths_exist() {
    let project_root = Path::new("/tmp/project");
    let globs = resolve_template_globs(project_root, None);
    assert!(globs.is_empty());
}

#[test]
fn resolve_template_globs_prefers_xiuxian_resource_root_when_present() {
    let temp_root = std::env::temp_dir().join(format!(
        "xiuxian-resource-root-{}-{}",
        std::process::id(),
        "bootstrap-tests"
    ));
    let template_root = temp_root
        .join("omni-agent")
        .join("zhixing")
        .join("templates");
    if let Err(error) = fs::create_dir_all(&template_root) {
        panic!("create temp template root: {error}");
    }

    let globs = resolve_template_globs_with_resource_root(
        Path::new("/tmp/project"),
        None,
        Some(temp_root.to_string_lossy().as_ref()),
    );
    assert_eq!(
        globs[0],
        template_root.join("*.md").to_string_lossy().into_owned()
    );

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn load_skill_templates_from_embedded_registry_uses_semantic_wendao_uri_links() {
    let manager = ManifestationManager::new_with_embedded_templates(
        &[],
        &[("probe.md", "Skill bridge probe: {{ marker }}")],
    )
    .unwrap_or_else(|error| panic!("create manifestation manager probe: {error}"));
    let summary = load_skill_templates_from_embedded_registry(&manager)
        .unwrap_or_else(|error| panic!("load skill templates from embedded registry: {error}"));
    if summary.linked_ids == 0 {
        assert_eq!(summary.template_records, 0);
        assert_eq!(summary.loaded_template_names, 0);
    } else {
        assert!(summary.template_records >= summary.linked_ids);
        assert!(summary.loaded_template_names >= 1);
    }

    let rendered = manager
        .render_template(
            "probe.md",
            json!({
                "marker": "ok"
            }),
        )
        .unwrap_or_else(|error| panic!("render probe template after bridge load: {error}"));
    assert!(rendered.contains("Skill bridge probe: ok"));
}

#[test]
fn init_persona_registries_uses_provider_backed_empty_registry() {
    let project_root = Path::new("/tmp/project");
    let xiuxian_cfg = crate::config::XiuxianConfig::default();
    let mut mounts = ServiceMountCatalog::new();

    let registries = init_persona_registries(project_root, &xiuxian_cfg, &mut mounts);
    let mount_records = mounts.finish();

    assert_eq!(registries.internal.len(), 0);
    assert!(
        mount_records.iter().any(|record| {
            record.service == "qianhuan.persona_registry.graph_provider"
                && record.status == ServiceMountStatus::Mounted
        }),
        "graph provider mount record should be present"
    );
    assert!(
        mount_records.iter().any(|record| {
            record.service == "qianhuan.persona_registry.user"
                && record.status == ServiceMountStatus::Skipped
        }),
        "user registry deprecation mount record should be present"
    );
}

#[test]
fn build_skill_vfs_resolver_from_empty_roots_mounts_embedded_resources() {
    let mut mounts = ServiceMountCatalog::new();
    let resolver = build_skill_vfs_resolver_from_roots(&[], &mut mounts)
        .unwrap_or_else(|| panic!("skill vfs resolver should be available with embedded mount"));
    let content = resolver
        .read_semantic("wendao://skills/agenda-management/references/steward.md")
        .unwrap_or_else(|error| panic!("embedded semantic resource should resolve: {error}"));
    assert!(
        content.contains("Agenda Steward Persona") || content.contains("Clockwork Guardian"),
        "unexpected steward persona content"
    );

    let records = mounts.finish();
    let mount_record = records
        .iter()
        .find(|record| record.service == "zhenfa.skill_vfs")
        .unwrap_or_else(|| panic!("expected zhenfa.skill_vfs mount record"));
    assert_eq!(mount_record.status, ServiceMountStatus::Mounted);
    let detail = mount_record.detail.as_deref().unwrap_or_default();
    assert!(
        detail.contains("roots=none") && detail.contains("embedded=true"),
        "unexpected mount detail: {detail}"
    );
}

#[test]
fn resolve_wendao_watch_roots_prefers_configured_watch_dirs() {
    let project_root = Path::new("/tmp/project");
    let roots = resolve_wendao_watch_roots(
        project_root,
        Path::new("/tmp/project/.data/xiuxian/notebook"),
        Some(&vec![
            "docs".to_string(),
            "/opt/shared-notes".to_string(),
            " ".to_string(),
        ]),
        None,
    );
    assert_eq!(
        roots,
        vec![
            PathBuf::from("/opt/shared-notes"),
            PathBuf::from("/tmp/project/docs")
        ]
    );
}

#[test]
fn resolve_wendao_watch_roots_falls_back_to_default_notebook_root() {
    let project_root = Path::new("/tmp/project");
    let roots = resolve_wendao_watch_roots(
        project_root,
        Path::new("/tmp/project/.data/xiuxian/notebook"),
        None,
        None,
    );
    assert_eq!(
        roots,
        vec![PathBuf::from("/tmp/project/.data/xiuxian/notebook")]
    );
}

#[test]
fn resolve_wendao_incremental_policy_prefers_explicit_extensions() {
    let patterns = vec!["**/*.md".to_string(), "**/*.markdown".to_string()];
    let configured = vec!["org".to_string(), "j2".to_string(), "toml".to_string()];
    let policy = resolve_wendao_incremental_policy(&patterns, Some(&configured));
    assert_eq!(
        policy.extensions(),
        &["j2".to_string(), "org".to_string(), "toml".to_string()]
    );
}

#[test]
fn resolve_wendao_incremental_policy_extracts_from_patterns_when_no_override() {
    let patterns = vec!["**/*.{md,org,template.md.j2}".to_string()];
    let policy = resolve_wendao_incremental_policy(&patterns, None);
    assert_eq!(
        policy.extensions(),
        &["j2".to_string(), "md".to_string(), "org".to_string()]
    );
}

#[test]
fn resolve_wendao_watch_patterns_prefers_configured_patterns() {
    let patterns = vec!["**/SKILL.md".to_string(), "docs/**/*.md".to_string()];
    let resolved = resolve_wendao_watch_patterns(Some(&patterns), None);
    assert_eq!(resolved, patterns);
}

#[test]
fn resolve_wendao_watch_patterns_derives_from_extensions_when_patterns_absent() {
    let extensions = vec!["org".to_string(), "j2".to_string(), "toml".to_string()];
    let resolved = resolve_wendao_watch_patterns(None, Some(&extensions));
    assert_eq!(
        resolved,
        vec![
            "**/*.org".to_string(),
            "**/*.j2".to_string(),
            "**/*.toml".to_string()
        ]
    );
}

#[test]
fn resolve_wendao_watch_patterns_falls_back_to_default_set() {
    let resolved = resolve_wendao_watch_patterns(None, None);
    assert_eq!(
        resolved,
        vec![
            "**/*.md".to_string(),
            "**/*.markdown".to_string(),
            "**/*.org".to_string(),
            "**/*.orgm".to_string(),
            "**/*.j2".to_string(),
            "**/*.toml".to_string()
        ]
    );
}

#[test]
fn resolve_wendao_watch_patterns_normalizes_extensions_and_drops_invalid_tokens() {
    let extensions = vec![
        " .ORG ".to_string(),
        "J2".to_string(),
        "bad^token".to_string(),
        String::new(),
    ];
    let resolved = resolve_wendao_watch_patterns(None, Some(&extensions));
    assert_eq!(
        resolved,
        vec!["**/*.org".to_string(), "**/*.j2".to_string()]
    );
}
