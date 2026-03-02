//! Integration tests for CCS gating across orchestrator and personas.

use std::sync::Arc;
use xiuxian_qianhuan::{
    InjectionError, MockTransmuter, PersonaRegistry, ThousandFacesOrchestrator,
};

#[tokio::test]
async fn test_full_chain_ccs_enforcement() {
    // 1. Load the real builtin Artisan Persona
    let registry = PersonaRegistry::with_builtins();
    let Some(artisan) = registry.get("artisan-engineer") else {
        panic!("artisan-engineer persona should exist");
    };

    // 2. Setup Orchestrator
    let orchestrator = ThousandFacesOrchestrator::new(
        "Standard Core Rules".to_string(),
        Some(Arc::new(MockTransmuter)),
    );

    // 3. Scenario: Background is too thin for the Artisan's high standards
    // Artisan style anchors include: "milimeter-level alignment", "audit trail"
    let thin_narrative = vec!["The system is fast.".to_string()];

    let result = orchestrator
        .assemble_snapshot(&artisan, thin_narrative, "User asked for audit.")
        .await;

    // 4. Verification: Should fail because "audit trail" is not grounded in the facts
    assert!(result.is_err());
    match result {
        Err(InjectionError::ContextInsufficient { ccs, missing_info }) => {
            assert!(ccs < 0.65);
            assert!(!missing_info.trim().is_empty());
        }
        Err(other) => panic!("expected ContextInsufficient error, got {other}"),
        Ok(_) => panic!("expected CCS gating failure"),
    }
}
