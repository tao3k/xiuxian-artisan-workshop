//! Unit tests for persona registry and profile operations.

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use xiuxian_qianhuan::{MemoryPersonaRecord, PersonaProfile, PersonaProvider, PersonaRegistry};

#[test]
fn test_builtin_loading() {
    let registry = PersonaRegistry::with_builtins();
    let Some(artisan) = registry.get("artisan-engineer") else {
        panic!("Artisan should exist");
    };
    assert_eq!(artisan.name, "Artisan Engineer");
    assert!(artisan.style_anchors.contains(&"audit trail".to_string()));

    let Some(cultivator) = registry.get("cyber-cultivator") else {
        panic!("Cultivator should exist");
    };
    assert_eq!(cultivator.name, "Cyber-Cultivator");

    let Some(agenda_steward) = registry.get("agenda_steward") else {
        panic!("Agenda Steward should exist");
    };
    assert_eq!(agenda_steward.name, "Agenda Steward");

    let Some(strict_teacher) = registry.get("strict_teacher") else {
        panic!("Strict Teacher should exist");
    };
    assert_eq!(strict_teacher.name, "Strict Teacher");
}

#[test]
fn test_custom_registration() {
    let registry = PersonaRegistry::with_builtins();
    let profile = PersonaProfile {
        id: "test".to_string(),
        name: "Test".to_string(),
        voice_tone: "Test".to_string(),
        style_anchors: vec![],
        cot_template: "Test".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
        background: None,
        guidelines: vec![],
    };
    registry.register(profile);
    assert!(registry.get("test").is_some());
}

#[test]
fn test_load_from_dir_toml_only_ignores_yaml() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let persona_dir = temp.path().join("personas");
    fs::create_dir_all(&persona_dir)?;

    fs::write(
        persona_dir.join("strict.toml"),
        r#"
id = "strict-architecture-auditor"
name = "Strict Architecture Auditor"
voice_tone = "Direct and precise."
style_anchors = ["architecture", "invariants"]
cot_template = "1) Analyze constraints -> 2) validate invariants"
forbidden_words = ["maybe"]
"#,
    )?;

    fs::write(
        persona_dir.join("cultivator.yaml"),
        r#"
id: "cyber-cultivator-lite"
name: "Cyber Cultivator Lite"
voice_tone: "Focused and calm."
style_anchors:
  - "dao"
cot_template: "Observe -> reason -> answer"
forbidden_words:
  - "panic"
"#,
    )?;

    let registry = PersonaRegistry::load_from_dir(&persona_dir)?;
    assert!(registry.get("strict-architecture-auditor").is_some());
    assert!(registry.get("cyber-cultivator-lite").is_none());
    Ok(())
}

#[test]
fn test_load_from_dir_allows_extensibility_without_recompile() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let persona_dir = temp.path().join("personas");
    fs::create_dir_all(&persona_dir)?;

    fs::write(
        persona_dir.join("strict_architecture_auditor.toml"),
        r#"
id = "strict-architecture-auditor"
name = "Strict Architecture Auditor"
voice_tone = "No-nonsense, architecture-first."
style_anchors = ["modularization", "contract", "traceability"]
cot_template = "1. Verify architecture contract -> 2. Reject ambiguity -> 3. Propose deterministic fix"
forbidden_words = ["guess", "probably", "quick hack"]
"#,
    )?;

    let registry = PersonaRegistry::load_from_dir(&persona_dir)?;
    let profile = registry
        .get("strict-architecture-auditor")
        .ok_or_else(|| anyhow!("new persona should be discoverable from directory"))?;
    assert_eq!(profile.name, "Strict Architecture Auditor");
    assert!(
        profile
            .style_anchors
            .iter()
            .any(|anchor| anchor == "traceability")
    );
    Ok(())
}

#[test]
fn test_register_from_memory_toml_enforces_id_key() -> Result<()> {
    let registry = PersonaRegistry::new();
    registry.register_from_memory_toml(
        "agenda_steward",
        r#"
name = "Agenda Steward"
voice_tone = "Structured and practical."
style_anchors = ["agenda", "clarity"]
cot_template = "Observe -> draft -> validate"
forbidden_words = ["impossible"]
"#,
    )?;

    let profile = registry
        .get("agenda_steward")
        .ok_or_else(|| anyhow!("memory persona should be loaded by exact id key"))?;
    assert_eq!(profile.id, "agenda_steward");
    assert_eq!(profile.name, "Agenda Steward");
    Ok(())
}

#[test]
fn test_load_from_memory_records_bulk() -> Result<()> {
    let mut registry = PersonaRegistry::new();
    let loaded = registry.load_from_memory_records([
        MemoryPersonaRecord::new(
            "agenda_steward",
            r#"
name = "Agenda Steward"
voice_tone = "Structured."
style_anchors = ["agenda"]
cot_template = "Observe -> draft"
forbidden_words = ["none"]
"#,
        ),
        MemoryPersonaRecord::new(
            "strict_teacher",
            r#"
name = "Strict Teacher"
voice_tone = "Direct."
style_anchors = ["audit"]
cot_template = "Inspect -> critique"
forbidden_words = ["guess"]
"#,
        ),
    ])?;

    assert_eq!(loaded, 2);
    assert!(registry.get("agenda_steward").is_some());
    assert!(registry.get("strict_teacher").is_some());
    Ok(())
}

#[derive(Default)]
struct MockPersonaProvider {
    fetch_calls: AtomicUsize,
}

impl PersonaProvider for MockPersonaProvider {
    fn fetch_persona(&self, id: &str) -> Option<PersonaProfile> {
        self.fetch_calls.fetch_add(1, Ordering::SeqCst);
        if id != "graph_teacher" {
            return None;
        }
        Some(PersonaProfile {
            id: id.to_string(),
            name: "Graph Teacher".to_string(),
            voice_tone: "Strict".to_string(),
            style_anchors: vec!["graph".to_string(), "audit".to_string()],
            cot_template: "Inspect -> critique".to_string(),
            forbidden_words: vec!["guess".to_string()],
            metadata: HashMap::new(),
            background: None,
            guidelines: Vec::new(),
        })
    }
}

#[test]
fn test_provider_fetches_on_cache_miss_and_caches_result() {
    let provider = Arc::new(MockPersonaProvider::default());
    let mut registry = PersonaRegistry::new();
    registry.set_provider(provider.clone());

    let Some(first) = registry.get("graph_teacher") else {
        panic!("provider should return graph_teacher on first miss");
    };
    assert_eq!(first.name, "Graph Teacher");

    let Some(second) = registry.get("graph_teacher") else {
        panic!("persona should be cached after first provider fetch");
    };
    assert_eq!(second.id, "graph_teacher");

    assert_eq!(provider.fetch_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn test_provider_returns_none_for_unknown_persona() {
    let provider = Arc::new(MockPersonaProvider::default());
    let mut registry = PersonaRegistry::new();
    registry.set_provider(provider.clone());

    assert!(registry.get("unknown_persona").is_none());
    assert_eq!(provider.fetch_calls.load(Ordering::SeqCst), 1);
}
