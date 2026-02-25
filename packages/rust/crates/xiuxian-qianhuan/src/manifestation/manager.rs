use tera::{Tera, Context};
use serde_json::Value;
use crate::interface::ManifestationInterface;
use anyhow::{Result, anyhow};

/// Manager for the Manifestation (Qianhuan) layer.
/// 
/// Coordinates template rendering and dynamic context injection.
pub struct ManifestationManager {
    /// The template engine (Tera).
    tera: Tera,
}

impl ManifestationManager {
    /// Creates a new `ManifestationManager` with templates loaded from a directory.
    /// 
    /// # Errors
    /// Returns an error if the template glob pattern is invalid or if loading fails.
    pub fn new(templates_glob: &str) -> Result<Self> {
        let tera = Tera::new(templates_glob)
            .map_err(|e| anyhow!("Failed to initialize Tera: {e}"))?;
        Ok(Self { tera })
    }
}

impl ManifestationInterface for ManifestationManager {
    fn render_template(&self, template_name: &str, data: Value) -> Result<String> {
        let context = Context::from_value(data)
            .map_err(|e| anyhow!("Failed to create context: {e}"))?;
            
        self.tera.render(template_name, &context)
            .map_err(|e| anyhow!("Template rendering error: {e}"))
    }

    fn inject_context(&self, state_context: &str) -> String {
        // Implementation of Instance-Adaptive Prompting logic
        match state_context {
            "STALE_TASKS" => {
                "### Steward's Warning: Your Vows are decaying. Focus on completion to avoid mental blockage."
                    .to_string()
            },
            "SUCCESS_STREAK" => {
                "### Steward's Praise: Your path is clear. Knowledge and Action are in harmony."
                    .to_string()
            },
            _ => "### Steward's Presence: Ready to guide your path.".to_string(),
        }
    }
}
