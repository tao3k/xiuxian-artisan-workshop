//! LLM analysis mechanism for high-precision reasoning.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use std::sync::Arc;
use xiuxian_llm::llm::{ChatMessage, ChatRequest, LlmClient};

/// Mechanism responsible for performing LLM inference based on annotated context.
pub struct LlmAnalyzer {
    /// Thread-safe client for LLM communication.
    pub client: Arc<dyn LlmClient>,
    /// Target model name.
    pub model: String,
    /// Context keys to extract and format into the prompt.
    pub context_keys: Vec<String>,
    /// The template/base prompt for the system.
    pub prompt_template: String,
    /// The output key to store the result.
    pub output_key: String,
}

#[async_trait]
impl QianjiMechanism for LlmAnalyzer {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let mut final_prompt = self.prompt_template.clone();
        
        // Very basic interpolation from context keys or fallback to appending
        for key in &self.context_keys {
            if let Some(val) = context.get(key) {
                let val_str = if let Some(s) = val.as_str() {
                    s.to_string()
                } else {
                    val.to_string()
                };
                
                let placeholder = format!("{{{{{}}}}}", key);
                if final_prompt.contains(&placeholder) {
                    final_prompt = final_prompt.replace(&placeholder, &val_str);
                } else {
                    final_prompt.push_str(&format!("\n\n[{key}]:\n{val_str}"));
                }
            }
        }
        
        let user_query = context
            .get("request")
            .or_else(|| context.get("query"))
            .and_then(|v| v.as_str())
            .unwrap_or("Proceed.");

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: final_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_query.to_string(),
                },
            ],
            temperature: 0.1,
        };

        let conclusion = self
            .client
            .chat(request)
            .await
            .map_err(|e| format!("LLM execution failed: {}", e))?;

        let mut data = serde_json::Map::new();
        data.insert(self.output_key.clone(), serde_json::Value::String(conclusion));

        Ok(QianjiOutput {
            data: serde_json::Value::Object(data),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    struct MockLlmClient;

    #[async_trait]
    impl LlmClient for MockLlmClient {
        async fn chat(&self, request: ChatRequest) -> Result<String> {
            if request.messages[0].content == "You are an Artisan." {
                Ok("I am ready.".to_string())
            } else {
                anyhow::bail!("Invalid prompt");
            }
        }
    }

    #[tokio::test]
    async fn test_llm_analyzer_success() {
        let client = Arc::new(MockLlmClient);
        let analyzer = LlmAnalyzer {
            client,
            model: "test-model".to_string(),
        };

        let context = json!({
            "annotated_prompt": "You are an Artisan.",
            "query": "Who are you?"
        });

        let output = analyzer.execute(&context).await.unwrap();
        assert_eq!(output.data["analysis_conclusion"], "I am ready.");
        match output.instruction {
            FlowInstruction::Continue => {}
            _ => panic!("Expected Continue"),
        }
    }

    #[tokio::test]
    async fn test_llm_analyzer_missing_prompt() {
        let client = Arc::new(MockLlmClient);
        let analyzer = LlmAnalyzer {
            client,
            model: "test-model".to_string(),
        };

        let context = json!({
            "query": "Who are you?"
        });

        let result = analyzer.execute(&context).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing 'annotated_prompt'");
    }
}
