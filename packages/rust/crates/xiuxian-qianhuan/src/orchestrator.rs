//! Multi-layer orchestrator for xiuxian-qianhuan prompt assembly.

use std::path::PathBuf;
use std::sync::Arc;

use crate::calibration::AdversarialOrchestrator;
use crate::error::InjectionError;
use crate::manifestation::templates::{
    SystemPromptTemplateRenderer, resolve_system_prompt_template_dirs,
};
use crate::persona::PersonaProfile;
use crate::transmuter::ToneTransmuter;
use tera::Context;

/// Logical layers used to compose an injection snapshot.
pub enum InjectionLayer {
    /// L0: immutable safety and governance rules.
    Genesis,
    /// L1: persona tone and reasoning style steering.
    Persona,
    /// L2: transformed narrative/knowledge blocks.
    Narrative,
    /// L3: recency/working-memory context.
    Working,
    /// 2026 Extension: Calibration feedback from Skeptic.
    Calibration,
}

/// Assembles layered prompt snapshots with optional narrative transmutation.
pub struct ThousandFacesOrchestrator {
    genesis_rules: String,
    transmuter: Option<Arc<dyn ToneTransmuter>>,
    template_renderer: std::result::Result<SystemPromptTemplateRenderer, String>,
    /// Optional adversarial calibrator for post-assembly alignment loops.
    pub calibrator: Option<Arc<AdversarialOrchestrator>>,
}

impl ThousandFacesOrchestrator {
    /// Creates a new orchestrator with fixed genesis rules and optional transmuter.
    #[must_use]
    pub fn new(genesis_rules: String, transmuter: Option<Arc<dyn ToneTransmuter>>) -> Self {
        Self::new_with_template_dirs(genesis_rules, transmuter, &[])
    }

    /// Creates a new orchestrator with optional additional template directories.
    ///
    /// Template resolution always includes internal system resources and default
    /// user override path, and appends any caller-provided directories.
    #[must_use]
    pub fn new_with_template_dirs(
        genesis_rules: String,
        transmuter: Option<Arc<dyn ToneTransmuter>>,
        template_dirs: &[PathBuf],
    ) -> Self {
        let resolved_template_dirs = resolve_system_prompt_template_dirs(template_dirs);
        let template_renderer = SystemPromptTemplateRenderer::from_dirs(&resolved_template_dirs)
            .map_err(|error| {
                format!(
                    "failed to initialize system prompt template renderer from [{}]: {error}",
                    resolved_template_dirs
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            });
        if let Err(error) = template_renderer.as_ref() {
            log::warn!("qianhuan template renderer unavailable: {error}");
        }

        Self {
            genesis_rules,
            transmuter,
            template_renderer,
            calibrator: None,
        }
    }

    /// Assembles the final XML system prompt snapshot asynchronously.
    ///
    /// Narrative blocks are passed through the configured transmuter when present.
    ///
    /// # Errors
    ///
    /// Returns [`InjectionError`] when context completeness is below threshold,
    /// tone transmutation fails, or generated XML is invalid.
    pub async fn assemble_snapshot(
        &self,
        persona: &PersonaProfile,
        narrative_blocks: Vec<String>,
        history: &str,
    ) -> Result<String, InjectionError> {
        // 2026 CCS Gating: Evaluate if facts support the persona (Ref: Agent-G)
        let (ccs, missing_anchors) = Self::calculate_ccs_with_missing(persona, &narrative_blocks);
        if ccs < 0.65 {
            return Err(InjectionError::ContextInsufficient {
                ccs,
                missing_info: missing_anchors.join(", "),
            });
        }

        let narrative_entries = if let Some(ref transmuter) = self.transmuter {
            let mut shifted = Vec::with_capacity(narrative_blocks.len());
            for block in narrative_blocks {
                shifted.push(transmuter.transmute(&block, persona).await?);
            }
            shifted
        } else {
            narrative_blocks
        };

        let mut context = Context::new();
        context.insert("genesis_rules", &self.genesis_rules);
        context.insert("persona_name", &persona.name);
        context.insert("persona_voice_tone", &persona.voice_tone);
        if let Some(bg) = &persona.background {
            context.insert("persona_background", bg);
        }
        context.insert("persona_guidelines", &persona.guidelines);
        context.insert("persona_cot_template", &persona.cot_template);
        context.insert("persona_style_anchors", &persona.style_anchors);
        context.insert("persona_forbidden_words", &persona.forbidden_words);
        context.insert("persona_metadata", &persona.metadata);
        context.insert("narrative_entries", &narrative_entries);
        context.insert("history", history);

        let template_renderer = self.template_renderer.as_ref().map_err(|error| {
            InjectionError::XmlValidationError(format!("Template renderer unavailable: {error}"))
        })?;
        let final_xml = template_renderer.render(context).map_err(|error| {
            InjectionError::XmlValidationError(format!("Template rendering failed: {error}"))
        })?;

        // 2026 Integrity Check: Validate XML balance before returning
        Self::validate_xml(&final_xml)?;

        Ok(final_xml)
    }

    fn validate_xml(xml: &str) -> Result<(), InjectionError> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(xml);
        let mut stack = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    stack.push(name);
                }
                Ok(Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if let Some(open_tag) = stack.pop() {
                        if open_tag != name {
                            return Err(InjectionError::XmlValidationError(format!(
                                "Mismatched tag: expected </{open_tag}>, found </{name}>"
                            )));
                        }
                    } else {
                        return Err(InjectionError::XmlValidationError(format!(
                            "Unexpected closing tag: </{name}>"
                        )));
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(InjectionError::XmlValidationError(format!(
                        "Malformed XML structure: {e}"
                    )));
                }
                _ => {}
            }
        }

        if let Some(open_tag) = stack.pop() {
            return Err(InjectionError::XmlValidationError(format!(
                "Unclosed tag: <{open_tag}>"
            )));
        }

        Ok(())
    }

    /// Calculates Context Completeness Score (CCS) and identifies missing anchors.
    fn calculate_ccs_with_missing(
        persona: &PersonaProfile,
        narrative: &[String],
    ) -> (f64, Vec<String>) {
        if persona.style_anchors.is_empty() {
            return (1.0, Vec::new());
        }
        if narrative.is_empty() {
            return (0.0, persona.style_anchors.clone());
        }

        let mut missing = Vec::new();
        let mut matches: usize = 0;
        for anchor in &persona.style_anchors {
            let anchor_lower = anchor.to_lowercase();
            let mut found = false;
            for block in narrative {
                if block.to_lowercase().contains(&anchor_lower) {
                    found = true;
                    break;
                }
            }
            if found {
                matches += 1;
            } else {
                missing.push(anchor.clone());
            }
        }

        let total = u32::try_from(persona.style_anchors.len()).unwrap_or(u32::MAX);
        let matched_count = u32::try_from(matches).unwrap_or(total);
        let score = if total == 0 {
            1.0
        } else {
            (f64::from(matched_count) / f64::from(total)).clamp(0.0, 1.0)
        };
        (score, missing)
    }
}
