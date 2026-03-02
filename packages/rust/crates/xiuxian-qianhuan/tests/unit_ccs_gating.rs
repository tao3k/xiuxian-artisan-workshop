//! Unit tests for Context Confidence Score gating behavior.

use std::collections::HashMap;
use xiuxian_qianhuan::{InjectionError, PersonaProfile, ThousandFacesOrchestrator};

#[tokio::test]
async fn test_ccs_gating_success() {
    let orchestrator = ThousandFacesOrchestrator::new("Rules".to_string(), None);
    let persona = PersonaProfile {
        id: "engineer".to_string(),
        name: "Engineer".to_string(),
        voice_tone: "Precise".to_string(),
        style_anchors: vec!["latency".to_string(), "memory".to_string()],
        cot_template: "T".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
        background: None,
        guidelines: vec![],
    };

    // Scenario: Fact contains all persona anchors
    let narrative = vec!["System latency is low and memory usage is optimized.".to_string()];
    let result = orchestrator
        .assemble_snapshot(&persona, narrative, "History")
        .await;

    // CCS should be 1.0 > 0.65
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ccs_gating_failure() {
    let orchestrator = ThousandFacesOrchestrator::new("Rules".to_string(), None);
    let persona = PersonaProfile {
        id: "engineer".to_string(),
        name: "Engineer".to_string(),
        voice_tone: "Precise".to_string(),
        style_anchors: vec![
            "latency".to_string(),
            "memory".to_string(),
            "throughput".to_string(),
        ],
        cot_template: "T".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
        background: None,
        guidelines: vec![],
    };

    // Scenario: Fact only mentions 1 out of 3 anchors
    let narrative = vec!["Memory is full.".to_string()];
    let result = orchestrator
        .assemble_snapshot(&persona, narrative, "History")
        .await;

    // CCS = 1/3 = 0.33 < 0.65 -> Should fail
    assert!(result.is_err());
    match result {
        Err(InjectionError::ContextInsufficient { ccs, missing_info }) => {
            assert!(ccs < 0.65);
            assert!(missing_info.contains("latency") || missing_info.contains("throughput"));
        }
        Err(other) => panic!("expected ContextInsufficient error, got {other}"),
        Ok(_) => panic!("expected CCS gating failure"),
    }
}
