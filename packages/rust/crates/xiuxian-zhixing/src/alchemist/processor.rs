use crate::Result;
use crate::interface::{SecureAction, ZhixingLlmInterface};
use std::sync::Arc;

/// The Alchemist (炼金术士)
/// 
/// Responsible for transforming unstructured consciousness-stream (raw text)
/// into structured system actions and knowledge insights.
pub struct Alchemist {
    /// Interface to the LLM (implemented in the agent layer).
    llm: Arc<dyn ZhixingLlmInterface>,
}

impl Alchemist {
    /// Creates a new Alchemist instance.
    #[must_use]
    pub fn new(llm: Arc<dyn ZhixingLlmInterface>) -> Self {
        Self { llm }
    }

    /// Processes raw input text and returns a [`SecureAction`].
    /// 
    /// This is the primary entry point for Discord/Telegram messages.
    /// 
    /// # Errors
    /// Returns an error if the LLM interface fails to select an action.
    pub fn process_input(&self, raw_text: &str) -> Result<SecureAction> {
        log::debug!("Alchemist processing raw input: {raw_text}");
        
        // Use the LLM interface to decide which action to take.
        // The prompt context would ideally include recent task states (omitted here for brevity).
        let action = self.llm.select_action(raw_text)?;
        
        Ok(action)
    }

    /// Extracts insights from a journal entry and formats them for the Knowledge Graph.
    /// 
    /// # Errors
    /// Returns an error if reflection alchemy fails.
    pub fn alchemize_journal(&self, journal_content: &str) -> Result<String> {
        log::info!("Alchemizing journal content for insights.");
        
        let insight = self.llm.alchemize_reflection(journal_content)?;
        
        Ok(insight)
    }
}
