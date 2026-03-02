//! Unit tests for core orchestrator snapshot assembly.

use std::collections::HashMap;
use std::sync::Arc;
use xiuxian_qianhuan::{MockTransmuter, PersonaProfile, ThousandFacesOrchestrator};

#[tokio::test]
async fn test_orchestrator_assembly() {
    let orchestrator = ThousandFacesOrchestrator::new(
        "Genesis Rule 1".to_string(),
        Some(Arc::new(MockTransmuter)),
    );

    let persona = PersonaProfile {
        id: "test".to_string(),
        name: "Cultivator".to_string(),
        voice_tone: "Zen".to_string(),
        style_anchors: vec!["Dao".to_string()],
        cot_template: "Step 1".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
        background: None,
        guidelines: vec![],
    };

    // Enrich fact to satisfy CCS "Dao" anchor
    let snapshot = orchestrator
        .assemble_snapshot(
            &persona,
            vec!["The Dao reveals the truth.".to_string()],
            "Prev history",
        )
        .await;
    let snapshot = match snapshot {
        Ok(snapshot) => snapshot,
        Err(error) => panic!("snapshot assembly should succeed: {error}"),
    };

    assert!(snapshot.contains("<genesis_rules>"));
    assert!(snapshot.contains("Genesis Rule 1"));
    assert!(snapshot.contains("<persona_steering>"));
    assert!(snapshot.contains("<tone>Zen</tone>"));
    assert!(snapshot.contains("<narrative_context>"));

    // Check if simulation worked (MockTransmuter adds persona specific tone)
    assert!(snapshot.contains("The Dao reveals"));
    assert!(snapshot.contains("<working_history>"));
}
