//! Integration tests for manifestation manager template rendering.

use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;
use xiuxian_qianhuan::interface::ManifestationInterface;
use xiuxian_qianhuan::{
    ManifestationManager, ManifestationRenderRequest, ManifestationRuntimeContext,
    ManifestationTemplateTarget, MemoryTemplateRecord, SessionSystemPromptInjectionSnapshot,
    normalize_session_system_prompt_injection_xml,
};

#[test]
fn manifestation_manager_renders_template() -> Result<()> {
    let dir = tempdir()?;
    let template_path = dir.path().join("test.md.j2");
    fs::write(&template_path, "Hello {{ name }}!")?;

    let glob = format!("{}/*.j2", dir.path().display());
    let manager = ManifestationManager::new(&[glob.as_str()])?;

    let rendered = manager.render_template("test.md.j2", json!({"name": "Daoist"}))?;
    assert_eq!(rendered, "Hello Daoist!");
    Ok(())
}

#[test]
fn manifestation_manager_render_request_injects_runtime_context() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("system_prompt_v2.xml"),
        "<root>{{ qianhuan.persona_id }}|{{ qianhuan.state_context }}|{{ qianhuan.injected_context }}</root>",
    )?;

    let glob = format!("{}/*", dir.path().display());
    let manager = ManifestationManager::new(&[glob.as_str()])?;

    let request = ManifestationRenderRequest {
        target: ManifestationTemplateTarget::SystemPromptV2Xml,
        data: json!({}),
        runtime: ManifestationRuntimeContext {
            state_context: Some("STALE_TASKS".to_string()),
            persona_id: Some("artisan-engineer".to_string()),
            domain: Some("zhixing".to_string()),
            extra: HashMap::default(),
        },
    };

    let rendered = manager.render_request(&request)?;
    assert!(rendered.contains("artisan-engineer"));
    assert!(rendered.contains("STALE_TASKS"));
    assert!(rendered.contains("Cognitive Interface Warning"));
    Ok(())
}

#[test]
fn manifestation_manager_supports_multiple_template_targets() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("daily_agenda.md"), "Agenda: {{ title }}")?;
    fs::write(
        dir.path().join("system_prompt_v2.xml"),
        "<prompt>{{ title }}</prompt>",
    )?;

    let glob = format!("{}/*", dir.path().display());
    let manager = ManifestationManager::new(&[glob.as_str()])?;

    let agenda = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({"title": "Morning Cultivation"}),
    )?;
    assert_eq!(agenda, "Agenda: Morning Cultivation");

    let xml = manager.render_target(
        &ManifestationTemplateTarget::SystemPromptV2Xml,
        json!({"title": "Runtime Persona"}),
    )?;
    assert_eq!(xml, "<prompt>Runtime Persona</prompt>");
    Ok(())
}

#[test]
fn manifestation_manager_hot_reloads_template_without_restart() -> Result<()> {
    let dir = tempdir()?;
    let template_path = dir.path().join("daily_agenda.md");
    fs::write(&template_path, "Agenda v1: {{ title }}")?;

    let glob = format!("{}/*", dir.path().display());
    let manager = ManifestationManager::new(&[glob.as_str()])?;

    let first = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({"title": "Morning"}),
    )?;
    assert_eq!(first, "Agenda v1: Morning");

    fs::write(&template_path, "Agenda v2: {{ title }}")?;

    let second = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({"title": "Morning"}),
    )?;
    assert_eq!(second, "Agenda v2: Morning");
    Ok(())
}

#[test]
fn manifestation_manager_keeps_last_good_template_when_hot_reload_fails() -> Result<()> {
    let dir = tempdir()?;
    let template_path = dir.path().join("daily_agenda.md");
    fs::write(&template_path, "Agenda stable: {{ title }}")?;

    let glob = format!("{}/*", dir.path().display());
    let manager = ManifestationManager::new(&[glob.as_str()])?;

    let baseline = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({"title": "Morning"}),
    )?;
    assert_eq!(baseline, "Agenda stable: Morning");

    fs::write(&template_path, "Agenda broken: {{ title ")?;

    let still_baseline = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({"title": "Morning"}),
    )?;
    assert_eq!(still_baseline, "Agenda stable: Morning");

    fs::write(&template_path, "Agenda recovered: {{ title }}")?;

    let recovered = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({"title": "Morning"}),
    )?;
    assert_eq!(recovered, "Agenda recovered: Morning");
    Ok(())
}

#[test]
fn manifestation_manager_supports_embedded_templates_without_external_globs() -> Result<()> {
    let manager = ManifestationManager::new_with_embedded_templates(
        &[],
        &[("daily_agenda.md", "Embedded Agenda: {{ title }}")],
    )?;

    let rendered = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({"title": "Morning"}),
    )?;
    assert_eq!(rendered, "Embedded Agenda: Morning");
    Ok(())
}

#[test]
fn manifestation_manager_external_templates_override_embedded_templates() -> Result<()> {
    let dir = tempdir()?;
    let template_path = dir.path().join("daily_agenda.md");
    fs::write(&template_path, "External Agenda: {{ title }}")?;

    let glob = format!("{}/*", dir.path().display());
    let manager = ManifestationManager::new_with_embedded_templates(
        &[glob.as_str()],
        &[("daily_agenda.md", "Embedded Agenda: {{ title }}")],
    )?;

    let rendered = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({"title": "Morning"}),
    )?;
    assert_eq!(rendered, "External Agenda: Morning");
    Ok(())
}

#[test]
fn manifestation_manager_loads_templates_from_memory_records() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("daily_agenda.md"),
        "Disk Agenda: {{ title }}",
    )?;

    let glob = format!("{}/*", dir.path().display());
    let manager = ManifestationManager::new(&[glob.as_str()])?;

    manager.load_templates_from_memory([MemoryTemplateRecord::new(
        "draft_agenda.j2",
        Some("daily_agenda.md".to_string()),
        "Memory Agenda: {{ title }}",
    )])?;

    let rendered_target = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({"title": "Morning"}),
    )?;
    assert_eq!(rendered_target, "Memory Agenda: Morning");

    let rendered_id = manager.render_template("draft_agenda.j2", json!({"title": "Morning"}))?;
    assert_eq!(rendered_id, "Memory Agenda: Morning");
    Ok(())
}

#[test]
fn manifestation_manager_tracks_session_prompt_injection_cache() -> Result<()> {
    let manager = ManifestationManager::new_with_embedded_templates(
        &[],
        &[("daily_agenda.md", "Embedded Agenda: {{ title }}")],
    )?;
    let session_id = "telegram:session-cache";

    let xml = r#"
<system_prompt_injection>
  <qa><q>q1</q><a>a1</a></qa>
  <qa><q>q2</q><a>a2</a></qa>
</system_prompt_injection>
"#;
    let snapshot = manager.upsert_session_prompt_injection_xml(session_id, xml)?;
    assert_eq!(snapshot.qa_count, 2);
    assert!(snapshot.xml.contains("<system_prompt_injection>"));

    let loaded = manager
        .inspect_session_prompt_injection(session_id)
        .expect("snapshot should exist in cache");
    assert_eq!(loaded.qa_count, 2);
    assert!(loaded.xml.contains("<q>q1</q>"));

    assert!(manager.clear_session_prompt_injection(session_id));
    assert!(!manager.clear_session_prompt_injection(session_id));
    assert!(
        manager
            .inspect_session_prompt_injection(session_id)
            .is_none()
    );
    Ok(())
}

#[test]
fn manifestation_manager_upserts_prevalidated_session_prompt_injection_snapshot() -> Result<()> {
    let manager = ManifestationManager::new_with_embedded_templates(
        &[],
        &[("daily_agenda.md", "Embedded Agenda: {{ title }}")],
    )?;
    let session_id = "telegram:session-prevalidated";
    let snapshot: SessionSystemPromptInjectionSnapshot =
        normalize_session_system_prompt_injection_xml("<qa><q>Q</q><a>A</a></qa>")?;
    assert_eq!(snapshot.qa_count, 1);

    manager.upsert_session_prompt_injection_snapshot(session_id, snapshot.clone());
    let loaded = manager
        .inspect_session_prompt_injection(session_id)
        .expect("snapshot should be present");
    assert_eq!(loaded.qa_count, snapshot.qa_count);
    assert_eq!(loaded.xml, snapshot.xml);
    Ok(())
}
