//! Skeptic node: performs formal audit on Analyzer output.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
#[cfg(feature = "llm")]
use crate::executors::annotation::ContextAnnotator;
use crate::safety::logic::{Invariant, Proposition};
use async_trait::async_trait;
#[cfg(feature = "llm")]
use serde_json::Value;
use serde_json::json;
#[cfg(feature = "llm")]
use std::sync::Arc;
#[cfg(feature = "llm")]
use xiuxian_llm::llm::{ChatMessage, ChatRequest, LlmClient};
#[cfg(feature = "llm")]
use xiuxian_zhenfa::ZhenfaTransmuter;

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

#[cfg(feature = "llm")]
fn context_non_empty_string(context: &Value, key: &str) -> Option<String> {
    context
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

#[cfg(feature = "llm")]
fn resolve_model_for_request(context: &Value, default_model: &str) -> String {
    if let Some(explicit_override) = context_non_empty_string(context, "llm_model") {
        return explicit_override;
    }
    let default_trimmed = default_model.trim();
    if !default_trimmed.is_empty() {
        return default_trimmed.to_string();
    }
    if let Some(fallback) = context_non_empty_string(context, "llm_model_fallback") {
        return fallback;
    }
    default_trimmed.to_string()
}

#[cfg(feature = "llm")]
fn extract_xml_score(text: &str) -> Option<f32> {
    ZhenfaTransmuter::get_tag_f32(text, "score")
}

#[cfg(feature = "llm")]
fn score_to_memrl_reward(score: f32) -> f32 {
    score.clamp(0.0, 1.0)
}

/// LLM-driven formal audit controller (Synaptic Flow V2).
#[cfg(feature = "llm")]
pub struct LlmAugmentedAuditMechanism {
    /// Node-local context annotator used to generate critique prompts.
    pub annotator: ContextAnnotator,
    /// LLM client used for critique generation.
    pub client: Arc<dyn LlmClient>,
    /// Default model name used unless context override is present.
    pub model: String,
    /// Score threshold below which retry is required.
    pub threshold_score: f32,
    /// Maximum allowed retries before hard stop to prevent runaway loops.
    pub max_retries: u32,
    /// Target nodes to trigger if audit score is below threshold.
    pub retry_target_ids: Vec<String>,
    /// Context key used to persist retry counter across loop iterations.
    pub retry_counter_key: String,
    /// Output key used for raw critique text.
    pub output_key: String,
    /// Output key used for numeric score extraction.
    pub score_key: String,
}

#[cfg(feature = "llm")]
#[async_trait]
impl QianjiMechanism for LlmAugmentedAuditMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let annotation_output = self.annotator.execute(context).await?;
        let Value::Object(mut data) = annotation_output.data else {
            return Err("LlmAugmentedAuditMechanism expected annotation output object".to_string());
        };

        let prompt = data
            .get(&self.annotator.output_key)
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "LlmAugmentedAuditMechanism missing annotated prompt at key `{}`",
                    self.annotator.output_key
                )
            })?;

        let user_query = context
            .get("request")
            .or_else(|| context.get("query"))
            .or_else(|| context.get("raw_facts"))
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("Critique the agenda and emit an XML <score> tag.");

        let request = ChatRequest {
            model: resolve_model_for_request(context, &self.model),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_query.to_string(),
                },
            ],
            temperature: 0.1,
        };

        let critique = self
            .client
            .chat(request)
            .await
            .map_err(|error| format!("LLM formal audit execution failed: {error}"))?;
        let parsed_score = extract_xml_score(&critique);
        let score = parsed_score.unwrap_or(0.0);
        let failed = score < self.threshold_score;
        let retry_count = context
            .get(&self.retry_counter_key)
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0);
        let mut audit_errors: Vec<String> = Vec::new();
        if parsed_score.is_none() {
            audit_errors.push("LLM audit score missing or invalid; defaulted to 0.0.".to_string());
        }

        data.insert(self.output_key.clone(), Value::String(critique));
        data.insert(self.score_key.clone(), json!(score));
        data.insert(
            "memrl_reward".to_string(),
            json!(score_to_memrl_reward(score)),
        );
        data.insert("memrl_signal_source".to_string(), json!("formal_audit.llm"));
        if let Some(memrl_episode_id) = context_non_empty_string(context, "memrl_episode_id")
            .or_else(|| context_non_empty_string(context, "episode_id"))
        {
            data.insert("memrl_episode_id".to_string(), json!(memrl_episode_id));
        }
        data.insert("audit_threshold".to_string(), json!(self.threshold_score));
        data.insert(self.retry_counter_key.clone(), json!(retry_count));
        if failed {
            let next_retry_count = retry_count.saturating_add(1);
            data.insert(self.retry_counter_key.clone(), json!(next_retry_count));
            audit_errors.push("LLM audit score below threshold.".to_string());
            if next_retry_count > self.max_retries {
                audit_errors.push(format!(
                    "LLM audit retry budget exceeded (max_retries={}).",
                    self.max_retries
                ));
                data.insert("audit_retry_exhausted".to_string(), json!(true));
                data.insert("audit_status".to_string(), json!("failed"));
                data.insert("audit_errors".to_string(), json!(audit_errors));
                return Ok(QianjiOutput {
                    data: Value::Object(data),
                    instruction: FlowInstruction::Abort(
                        "formal_audit.max_retries_exceeded".to_string(),
                    ),
                });
            }

            data.insert("audit_status".to_string(), json!("failed"));
            data.insert("audit_errors".to_string(), json!(audit_errors));
            return Ok(QianjiOutput {
                data: Value::Object(data),
                instruction: FlowInstruction::RetryNodes(self.retry_target_ids.clone()),
            });
        }

        data.insert("audit_status".to_string(), json!("passed"));
        Ok(QianjiOutput {
            data: Value::Object(data),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        2.0
    }
}
