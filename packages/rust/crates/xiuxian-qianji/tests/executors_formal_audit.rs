//! Integration tests for `xiuxian_qianji::executors::formal_audit`.

use serde_json::json;
use xiuxian_qianji::contracts::{FlowInstruction, QianjiMechanism};
use xiuxian_qianji::executors::formal_audit::FormalAuditMechanism;
use xiuxian_qianji::safety::logic::Invariant;

#[tokio::test]
async fn test_formal_audit_passes() -> Result<(), String> {
    let audit = FormalAuditMechanism {
        invariants: vec![Invariant::MustBeGrounded],
        retry_target_ids: vec!["Analyzer".to_string()],
    };

    let context = json!({
        "analysis_trace": [
            {
                "predicate": "A implies B",
                "has_grounding": true,
                "confidence": 0.95
            }
        ]
    });

    let output = audit.execute(&context).await?;
    assert_eq!(output.data["audit_status"], "passed");
    assert!(matches!(output.instruction, FlowInstruction::Continue));
    Ok(())
}

#[tokio::test]
async fn test_formal_audit_fails() -> Result<(), String> {
    let audit = FormalAuditMechanism {
        invariants: vec![Invariant::MustBeGrounded],
        retry_target_ids: vec!["Analyzer".to_string()],
    };

    let context = json!({
        "analysis_trace": [
            {
                "predicate": "A implies B",
                "has_grounding": false,
                "confidence": 0.95
            }
        ]
    });

    let output = audit.execute(&context).await?;
    assert_eq!(output.data["audit_status"], "failed");
    let FlowInstruction::RetryNodes(nodes) = output.instruction else {
        return Err("expected RetryNodes instruction".to_string());
    };
    assert_eq!(nodes.first().map(String::as_str), Some("Analyzer"));
    Ok(())
}
