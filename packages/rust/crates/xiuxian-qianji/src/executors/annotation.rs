//! Context annotation mechanism using Qianhuan.

use crate::contracts::{FlowInstruction, NodeQianhuanExecutionMode, QianjiMechanism, QianjiOutput};
use crate::scheduler::preflight::{
    context_value_to_text, lookup_context_path, resolve_semantic_content,
    resolve_semantic_reference, resolve_wendao_uri_with_zhenfa,
};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator;
use xiuxian_qianhuan::persona::{PersonaProfile, PersonaRegistry};
use xiuxian_wendao::{WendaoResourceUri, parse_frontmatter};

/// Mechanism responsible for transmuting raw facts into persona-aligned context snapshots.
pub struct ContextAnnotator {
    /// Reference to the `ThousandFaces` orchestrator.
    pub orchestrator: Arc<ThousandFacesOrchestrator>,
    /// Reference to the Persona Registry.
    pub registry: Arc<PersonaRegistry>,
    /// Target persona ID defined in the registry.
    pub persona_id: String,
    /// Optional logical template target associated with this node.
    pub template_target: Option<String>,
    /// Context window behavior for this annotation node.
    pub execution_mode: NodeQianhuanExecutionMode,
    /// Whitelisted context keys that can be marshaled into narrative blocks.
    pub input_keys: Vec<String>,
    /// History key used when execution mode is `appended`.
    pub history_key: String,
    /// Context key where the rendered snapshot is stored.
    pub output_key: String,
}

impl ContextAnnotator {
    fn collect_narrative_blocks(&self, context: &Value) -> Result<Vec<String>, String> {
        let mut blocks = Vec::new();
        for key in &self.input_keys {
            if key.trim_start().starts_with('$') {
                let text = resolve_semantic_content(key, context)?;
                if !text.trim().is_empty() {
                    blocks.push(text);
                }
                continue;
            }
            if let Some(value) = lookup_context_path(context, key)
                && let Some(text) = context_value_to_text(value)
            {
                blocks.push(text);
            }
        }

        if blocks.is_empty() {
            match context.get("raw_facts") {
                Some(value) => {
                    if let Some(text) = context_value_to_text(value) {
                        blocks.push(text);
                    }
                }
                None => blocks.push(String::new()),
            }
        }

        Ok(blocks)
    }

    fn resolve_history_seed(&self, context: &Value) -> String {
        match self.execution_mode {
            NodeQianhuanExecutionMode::Isolated => String::new(),
            NodeQianhuanExecutionMode::Appended => context
                .get(&self.history_key)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        }
    }

    fn metadata_key(&self, suffix: &str) -> String {
        if self.output_key == "annotated_prompt" {
            format!("annotated_{suffix}")
        } else {
            format!("{}_{}", self.output_key, suffix)
        }
    }

    fn merge_history_for_appended_mode(
        &self,
        current_history: &str,
        snapshot: &str,
    ) -> Option<String> {
        if self.execution_mode != NodeQianhuanExecutionMode::Appended {
            return None;
        }
        if current_history.is_empty() {
            return Some(snapshot.to_string());
        }
        Some(format!("{current_history}\n{snapshot}"))
    }

    fn resolve_persona_profile(&self, persona_reference: &str) -> Result<PersonaProfile, String> {
        if persona_reference.trim_start().starts_with("wendao://") {
            return self.resolve_wendao_persona_profile(persona_reference);
        }
        self.registry
            .get(persona_reference)
            .ok_or_else(|| format!("Persona '{persona_reference}' not found"))
    }

    fn resolve_wendao_persona_profile(&self, uri: &str) -> Result<PersonaProfile, String> {
        let parsed_uri = WendaoResourceUri::parse(uri)
            .map_err(|error| format!("invalid persona semantic URI '{uri}': {error}"))?;
        let canonical_uri = parsed_uri.canonical_uri();
        let markdown = resolve_wendao_uri_with_zhenfa(canonical_uri.as_str())?;
        let parsed_profile =
            persona_profile_from_markdown(canonical_uri.as_str(), markdown.as_str());
        if let Some(existing) = self.registry.get(parsed_profile.id.as_str()) {
            return Ok(existing);
        }
        Ok(parsed_profile)
    }
}

#[async_trait]
impl QianjiMechanism for ContextAnnotator {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let narrative_blocks = self.collect_narrative_blocks(context)?;
        let history_seed = self.resolve_history_seed(context);
        let persona_reference = resolve_semantic_reference(&self.persona_id, context)?;

        let persona = self.resolve_persona_profile(persona_reference.as_str())?;
        let persona_id = persona.id.clone();

        // --- REAL-TIME BATTLE REPORTING ---
        println!(
            "\n\033[1;34m[Node: {}]\033[0m Activating Avatar: \033[1;33m{}\033[0m",
            self.output_key, persona_id
        );
        if self.execution_mode == NodeQianhuanExecutionMode::Appended {
            println!("  > Mode: Appended (Preserving Session Context)");
        }
        // ----------------------------------

        let snapshot = self
            .orchestrator
            .assemble_snapshot(&persona, narrative_blocks, &history_seed)
            .await
            .map_err(|e| format!("Qianhuan annotation failed: {e}"))?;

        let mut data = serde_json::Map::new();
        data.insert(self.output_key.clone(), json!(snapshot));
        data.insert(self.metadata_key("persona_id"), json!(persona_id));
        data.insert(
            self.metadata_key("execution_mode"),
            json!(self.execution_mode.as_str()),
        );
        data.insert(
            self.metadata_key("input_keys"),
            json!(self.input_keys.clone()),
        );
        if self.execution_mode == NodeQianhuanExecutionMode::Appended {
            data.insert(
                self.metadata_key("history_key"),
                json!(self.history_key.clone()),
            );
        }
        if let Some(template_target) = self.template_target.as_deref() {
            data.insert(
                self.metadata_key("template_target"),
                json!(resolve_semantic_reference(template_target, context)?),
            );
        }
        if let Some(updated_history) =
            self.merge_history_for_appended_mode(&history_seed, &snapshot)
        {
            data.insert(self.history_key.clone(), json!(updated_history));
        }

        Ok(QianjiOutput {
            data: Value::Object(data),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        8.0
    }
}

fn persona_profile_from_markdown(uri: &str, markdown: &str) -> PersonaProfile {
    let frontmatter = parse_frontmatter(markdown);
    let background = strip_markdown_frontmatter(markdown);
    let heading_title = extract_markdown_h1(background.as_str());
    let parsed_uri = WendaoResourceUri::parse(uri).ok();
    let fallback_name = parsed_uri
        .as_ref()
        .and_then(|value| {
            Path::new(value.entity_name())
                .file_stem()
                .and_then(|stem| stem.to_str())
        })
        .map_or_else(|| "Persona".to_string(), humanize_identifier);
    let persona_name = frontmatter
        .title
        .as_deref()
        .or(heading_title.as_deref())
        .map(strip_persona_suffix)
        .filter(|value| !value.trim().is_empty())
        .map_or(fallback_name, ToString::to_string);
    let persona_id = persona_id_from_name(persona_name.as_str());
    let operating_profile = extract_section_bullets(background.as_str(), "Operating profile");
    let behavior_contract = extract_section_bullets(background.as_str(), "Behavior contract");
    let mut style_anchors = frontmatter.routing_keywords;
    style_anchors.extend(frontmatter.intents);
    style_anchors = dedup_non_empty(style_anchors);

    let mut metadata = HashMap::new();
    metadata.insert("source_uri".to_string(), uri.to_string());

    PersonaProfile {
        id: persona_id,
        name: persona_name,
        voice_tone: if operating_profile.is_empty() {
            "Calm, practical, and context-grounded.".to_string()
        } else {
            operating_profile.join(" ")
        },
        background: Some(background),
        guidelines: if behavior_contract.is_empty() {
            vec!["Respond with concise and actionable guidance.".to_string()]
        } else {
            behavior_contract
        },
        style_anchors,
        cot_template:
            "Extract constraints, reason about feasibility, then produce one executable output."
                .to_string(),
        forbidden_words: Vec::new(),
        metadata,
    }
}

fn strip_persona_suffix(raw: &str) -> &str {
    let trimmed = raw.trim();
    trimmed
        .strip_suffix(" Persona")
        .or_else(|| trimmed.strip_suffix(" persona"))
        .unwrap_or(trimmed)
        .trim()
}

fn strip_markdown_frontmatter(markdown: &str) -> String {
    let normalized = markdown.replace("\r\n", "\n");
    if let Some(rest) = normalized.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        return rest[end + "\n---\n".len()..].trim().to_string();
    }
    normalized.trim().to_string()
}

fn extract_section_bullets(content: &str, heading: &str) -> Vec<String> {
    let heading_token = format!("{}:", heading.trim().to_ascii_lowercase());
    let mut in_section = false;
    let mut bullets = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !in_section {
            if trimmed.to_ascii_lowercase() == heading_token {
                in_section = true;
            }
            continue;
        }
        if trimmed.starts_with('#') {
            break;
        }
        if trimmed.ends_with(':') && !trimmed.starts_with("- ") && !trimmed.starts_with("* ") {
            break;
        }
        if let Some(value) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            let value = value.trim();
            if !value.is_empty() {
                bullets.push(value.to_string());
            }
            continue;
        }
        if trimmed.is_empty() {
            if !bullets.is_empty() {
                break;
            }
            continue;
        }
        if bullets.is_empty() {
            bullets.push(trimmed.to_string());
        } else {
            break;
        }
    }
    bullets
}

fn extract_markdown_h1(content: &str) -> Option<String> {
    content
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn humanize_identifier(identifier: &str) -> String {
    let mut out = String::new();
    for (index, part) in identifier
        .split(['-', '_'])
        .filter(|value| !value.trim().is_empty())
        .enumerate()
    {
        if index > 0 {
            out.push(' ');
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.push_str(chars.as_str());
        }
    }
    if out.is_empty() {
        "Persona".to_string()
    } else {
        out
    }
}

fn persona_id_from_name(name: &str) -> String {
    let mut id = String::new();
    let mut previous_was_separator = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            id.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !id.is_empty() {
            id.push('_');
            previous_was_separator = true;
        }
    }
    id.trim_matches('_').to_string()
}

fn dedup_non_empty(values: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !out.iter().any(|existing: &String| existing == trimmed) {
            out.push(trimmed.to_string());
        }
    }
    out
}
