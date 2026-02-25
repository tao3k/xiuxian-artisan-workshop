/// Core processing logic for text alchemy.
pub mod processor;

pub use processor::Alchemist;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::{SecureAction, ZhixingLlmInterface};
    use crate::Result;
    use std::sync::Arc;

    // Mock LLM Implementation for testing
    struct MockLlm;
    impl ZhixingLlmInterface for MockLlm {
        fn select_action(&self, _prompt: &str) -> Result<SecureAction> {
            Ok(SecureAction::UpdateTaskStatus {
                id: uuid::Uuid::new_v4(),
                status: crate::agenda::Status::Done,
            })
        }
        fn alchemize_reflection(&self, _raw_text: &str) -> Result<String> {
            Ok("Insight: Practice makes perfect.".to_string())
        }
    }

    #[tokio::test]
    async fn test_alchemist_processing() {
        let llm = Arc::new(MockLlm);
        let alchemist = Alchemist::new(llm);
        
        let action = alchemist.process_input("I finished the task").unwrap();
        
        match action {
            SecureAction::UpdateTaskStatus { status, .. } => {
                assert_eq!(status, crate::agenda::Status::Done);
            },
            _ => panic!("Expected UpdateTaskStatus action"),
        }
    }
}
