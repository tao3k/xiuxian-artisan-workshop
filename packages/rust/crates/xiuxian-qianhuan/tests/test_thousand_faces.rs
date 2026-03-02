//! End-to-end persona-switching integration test for thousand-faces orchestration.

use std::sync::Arc;
use xiuxian_qianhuan::{MockTransmuter, PersonaRegistry, ThousandFacesOrchestrator};

#[tokio::test]
async fn test_end_to_end_persona_switching() {
    // 1. Initialize Registry with built-ins
    let registry = PersonaRegistry::with_builtins();

    // 2. Setup Orchestrator with a Mock Transmuter
    let orchestrator = ThousandFacesOrchestrator::new(
        "Always follow safety rules.".to_string(),
        Some(Arc::new(MockTransmuter)),
    );

    // --- Scenario A: Artisan Engineer ---
    let Some(artisan) = registry.get("artisan-engineer") else {
        panic!("artisan-engineer persona should exist");
    };

    // Providing rich facts to satisfy CCS anchors: "milimeter-level alignment", "audit trail", etc.
    let rich_facts_artisan = vec![
        "Implementation ensures milimeter-level alignment.".to_string(),
        "Audit trail is preserved in Valkey Streams.".to_string(),
        "Traceability and architectural consistency are verified.".to_string(),
    ];

    let snapshot_artisan = orchestrator
        .assemble_snapshot(
            &artisan,
            rich_facts_artisan,
            "History: User asked about implementation.",
        )
        .await;
    let snapshot_artisan = match snapshot_artisan {
        Ok(snapshot) => snapshot,
        Err(error) => panic!("artisan snapshot should assemble: {error}"),
    };

    assert!(snapshot_artisan.contains("<tone>Precise, professional"));
    assert!(snapshot_artisan.contains("Artisan Report"));

    // --- Scenario B: Cyber Cultivator ---
    let Some(cultivator) = registry.get("cyber-cultivator") else {
        panic!("cyber-cultivator persona should exist");
    };

    // Providing rich facts to satisfy CCS anchors: "karmic link", "daos of logic", etc.
    let rich_facts_cultivator = vec![
        "A karmic link exists between seeds.".to_string(),
        "The daos of logic guide the random walk.".to_string(),
        "Reaching the zenith of computation via PPR.".to_string(),
    ];

    let snapshot_cultivator = orchestrator
        .assemble_snapshot(
            &cultivator,
            rich_facts_cultivator,
            "History: User seeks the Dao of coding.",
        )
        .await;
    let snapshot_cultivator = match snapshot_cultivator {
        Ok(snapshot) => snapshot,
        Err(error) => panic!("cultivator snapshot should assemble: {error}"),
    };

    assert!(snapshot_cultivator.contains("<tone>Philosophical, ancient"));
    assert!(snapshot_cultivator.contains("The Dao reveals"));
}
