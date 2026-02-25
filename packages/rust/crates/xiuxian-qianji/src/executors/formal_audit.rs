//! Skeptic node: performs formal audit on Analyzer output.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use crate::safety::logic::{Invariant, Proposition};
use async_trait::async_trait;
use serde_json::json;

/// Formally audits LLM traces using LTL-inspired invariants.
pub struct FormalAuditMechanism {
    /// List of invariants to enforce.
    pub invariants: Vec<Invariant>,
    /// Target nodes to trigger if audit fails.
    pub retry_target_ids: Vec<String>,
}

#[async_trait]
impl QianjiMechanism for FormalAuditMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        // 1. Extract Trace (In real system, this parsed from LLM output)
        // Here we simulate trace extraction from context.
        let raw_trace = context.get("analysis_trace").and_then(|v| v.as_array());

        let mut propositions = Vec::new();
        if let Some(arr) = raw_trace {
            for item in arr {
                if let Ok(p) = serde_json::from_value::<Proposition>(item.clone()) {
                    propositions.push(p);
                }
            }
        }

        // 2. Run Audit
        let mut failed = false;
        let mut failure_reasons = Vec::new();

        for inv in &self.invariants {
            if !inv.check(&propositions) {
                failed = true;
                failure_reasons.push("Invariant violation detected during Synapse-Audit.");
            }
        }

        // 3. Decide Flow
        if failed {
            Ok(QianjiOutput {
                data: json!({ "audit_status": "failed", "audit_errors": failure_reasons }),
                instruction: FlowInstruction::RetryNodes(self.retry_target_ids.clone()),
            })
        } else {
            Ok(QianjiOutput {
                data: json!({ "audit_status": "passed" }),
                instruction: FlowInstruction::Continue,
            })
        }
    }

    fn weight(&self) -> f32 {
        2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_formal_audit_passes() {
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

        let output = audit.execute(&context).await.unwrap();
        assert_eq!(output.data["audit_status"], "passed");
        match output.instruction {
            FlowInstruction::Continue => {}
            _ => panic!("Expected Continue instruction"),
        }
    }

    #[tokio::test]
    async fn test_formal_audit_fails() {
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

        let output = audit.execute(&context).await.unwrap();
        assert_eq!(output.data["audit_status"], "failed");
        match output.instruction {
            FlowInstruction::RetryNodes(nodes) => {
                assert_eq!(nodes[0], "Analyzer");
            }
            _ => panic!("Expected RetryNodes instruction"),
        }
    }
}
