//! Integration tests for dynamic system prompt template loading.

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use xiuxian_qianhuan::{PersonaProfile, ThousandFacesOrchestrator};

fn minimal_persona() -> PersonaProfile {
    PersonaProfile {
        id: "dynamic-template-test".to_string(),
        name: "Dynamic Template Tester".to_string(),
        voice_tone: "Precise".to_string(),
        style_anchors: vec![],
        cot_template: "Observe -> compile -> verify".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
        background: None,
        guidelines: vec![],
    }
}

#[tokio::test]
async fn orchestrator_supports_runtime_template_override_directory() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let override_dir = temp.path().join("override_templates");
    fs::create_dir_all(&override_dir)?;
    fs::write(
        override_dir.join("system_prompt_injection.xml.j2"),
        r"<{{ system_prompt_injection_tag }}>
<override_marker>{{ genesis_rules }}|{{ persona_voice_tone }}|{{ history }}</override_marker>
</{{ system_prompt_injection_tag }}>
",
    )?;

    let orchestrator = ThousandFacesOrchestrator::new_with_template_dirs(
        "QH06 Rules".to_string(),
        None,
        &[PathBuf::from(&override_dir)],
    );
    let snapshot = orchestrator
        .assemble_snapshot(
            &minimal_persona(),
            vec!["fact".to_string()],
            "dynamic-history",
        )
        .await?;

    assert!(
        snapshot.contains("<override_marker>QH06 Rules|Precise|dynamic-history</override_marker>")
    );
    assert!(!snapshot.contains("<persona_steering>"));
    Ok(())
}

#[tokio::test]
async fn orchestrator_hot_reloads_template_without_restart() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let override_dir = temp.path().join("hot_reload_templates");
    fs::create_dir_all(&override_dir)?;
    let template_path = override_dir.join("system_prompt_injection.xml.j2");

    fs::write(
        &template_path,
        r"<{{ system_prompt_injection_tag }}>
<marker>v1-{{ history }}</marker>
</{{ system_prompt_injection_tag }}>
",
    )?;

    let orchestrator = ThousandFacesOrchestrator::new_with_template_dirs(
        "QH06 Rules".to_string(),
        None,
        &[PathBuf::from(&override_dir)],
    );

    let first = orchestrator
        .assemble_snapshot(&minimal_persona(), vec!["fact".to_string()], "history-v1")
        .await?;
    assert!(first.contains("<marker>v1-history-v1</marker>"));

    fs::write(
        &template_path,
        r"<{{ system_prompt_injection_tag }}>
<marker>v2-{{ history }}</marker>
</{{ system_prompt_injection_tag }}>
",
    )?;

    let second = orchestrator
        .assemble_snapshot(&minimal_persona(), vec!["fact".to_string()], "history-v2")
        .await?;
    assert!(second.contains("<marker>v2-history-v2</marker>"));
    Ok(())
}

#[tokio::test]
async fn orchestrator_reports_invalid_template_directory() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let invalid_dir = temp.path().join("not_a_dir.txt");
    fs::write(&invalid_dir, "not a directory")?;

    let orchestrator = ThousandFacesOrchestrator::new_with_template_dirs(
        "QH06 Rules".to_string(),
        None,
        &[invalid_dir],
    );
    let Err(error) = orchestrator
        .assemble_snapshot(&minimal_persona(), vec!["fact".to_string()], "history")
        .await
    else {
        return Err(anyhow!("invalid template directory should fail"));
    };
    let error_text = error.to_string();
    assert!(error_text.contains("Template renderer unavailable"));
    assert!(error_text.contains("template path is not a directory"));
    Ok(())
}
