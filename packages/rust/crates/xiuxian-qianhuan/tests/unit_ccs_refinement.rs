//! Unit tests for detailed CCS deficiency reporting.

use std::collections::HashMap;
use xiuxian_qianhuan::{InjectionError, PersonaProfile, ThousandFacesOrchestrator};

#[tokio::test]
async fn test_ccs_detailed_missing_info_identification() {
    let orchestrator = ThousandFacesOrchestrator::new("Rules".to_string(), None);

    // Artisan Persona requires 3 specific anchors
    let persona = PersonaProfile {
        id: "artisan".to_string(),
        name: "Artisan".to_string(),
        voice_tone: "Precise".to_string(),
        style_anchors: vec![
            "milimeter-level alignment".to_string(),
            "audit trail".to_string(),
            "traceability".to_string(),
        ],
        cot_template: "T".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
        background: None,
        guidelines: vec![],
    };

    // Scenario: Fact only supports "traceability"
    let narrative = vec!["The code has full traceability records.".to_string()];

    let result = orchestrator
        .assemble_snapshot(&persona, narrative, "History")
        .await;

    // Must fail with a specific error carrying the MISSING anchors
    match result {
        Err(InjectionError::ContextInsufficient { ccs, missing_info }) => {
            assert!(ccs < 0.5);
            // "traceability" was found, so it should NOT be in missing_info
            assert!(!missing_info.contains("traceability"));
            // These two were NOT found, must be reported
            assert!(missing_info.contains("milimeter-level alignment"));
            assert!(missing_info.contains("audit trail"));
        }
        _ => panic!("Expected ContextInsufficient error with missing info detail"),
    }
}
