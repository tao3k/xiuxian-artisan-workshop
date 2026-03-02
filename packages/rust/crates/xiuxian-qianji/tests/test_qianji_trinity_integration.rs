//! Trinity integration tests for Qianji annotation and calibration flow.

use serde_json::json;
use std::sync::Arc;
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_qianji::executors::annotation::ContextAnnotator;
use xiuxian_qianji::executors::calibration::SynapseCalibrator;
use xiuxian_qianji::{NodeQianhuanExecutionMode, QianjiEngine, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

#[tokio::test]
async fn test_qianji_trinity_integration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let _index = Arc::new(LinkGraphIndex::build(temp.path())?);
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety rules.".to_string(),
        None,
    ));
    let registry = Arc::new(PersonaRegistry::with_builtins());

    let mut engine = QianjiEngine::new();
    let annotator = Arc::new(ContextAnnotator {
        orchestrator: orchestrator.clone(),
        registry: registry.clone(),
        persona_id: "artisan-engineer".to_string(),
        template_target: None,
        execution_mode: NodeQianhuanExecutionMode::Isolated,
        input_keys: vec!["raw_facts".to_string()],
        history_key: "qianhuan_history".to_string(),
        output_key: "annotated_prompt".to_string(),
    });
    let calibrator = Arc::new(SynapseCalibrator {
        target_node_id: "Annotator".to_string(),
        drift_threshold: 0.5,
    });

    let a = engine.add_mechanism("Annotator", annotator);
    let c = engine.add_mechanism("Calibrator", calibrator);
    engine.add_link(a, c, None, 1.0);

    let scheduler = QianjiScheduler::new(engine);
    let result = scheduler
        .run(json!({
            "raw_facts": "Implementation ensures milimeter-level alignment and audit trail traceability.",
            "drift_score": 0.1
        }))
        .await?;

    let annotated_prompt = result["annotated_prompt"]
        .as_str()
        .ok_or_else(|| std::io::Error::other("annotated_prompt should be a string"))?;
    assert!(
        annotated_prompt.contains("<system_prompt_injection>"),
        "annotated prompt should contain system prompt injection marker"
    );
    Ok(())
}
