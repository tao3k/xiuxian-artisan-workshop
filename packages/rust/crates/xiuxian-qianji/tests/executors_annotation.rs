//! Integration tests for `xiuxian_qianji::executors::annotation`.

use std::sync::Arc;

use serde_json::json;
use xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator;
use xiuxian_qianhuan::persona::PersonaRegistry;
use xiuxian_qianji::contracts::{NodeQianhuanExecutionMode, QianjiMechanism};
use xiuxian_qianji::executors::annotation::ContextAnnotator;

#[tokio::test]
async fn context_annotator_can_load_persona_via_wendao_uri() {
    let annotator = ContextAnnotator {
        orchestrator: Arc::new(ThousandFacesOrchestrator::new(
            "keep plans executable".to_string(),
            None,
        )),
        registry: Arc::new(PersonaRegistry::new()),
        persona_id: "$wendao://skills/agenda-management/references/steward.md".to_string(),
        template_target: Some(
            "$wendao://skills/agenda-management/references/draft_agenda.j2".to_string(),
        ),
        execution_mode: NodeQianhuanExecutionMode::Isolated,
        input_keys: vec!["raw_facts".to_string()],
        history_key: "qianhuan_history".to_string(),
        output_key: "annotated_prompt".to_string(),
    };

    let output = annotator
        .execute(&json!({
            "raw_facts": "agenda planning execution Draft a realistic schedule Translate user intent into tasks Audit agenda quality"
        }))
        .await
        .unwrap_or_else(|error| panic!("annotation execution should succeed: {error}"));
    let Some(persona_id) = output
        .data
        .get("annotated_persona_id")
        .and_then(serde_json::Value::as_str)
    else {
        panic!("expected annotated_persona_id in annotation output");
    };
    assert!(
        persona_id == "pragmatic_agenda_steward"
            || persona_id == "professional_identity_the_clockwork_guardian",
        "unexpected persona id: {persona_id}"
    );
    let Some(template_target) = output
        .data
        .get("annotated_template_target")
        .and_then(serde_json::Value::as_str)
    else {
        panic!("expected annotated_template_target in annotation output");
    };
    assert_eq!(
        template_target,
        "wendao://skills/agenda-management/references/draft_agenda.j2"
    );
}
