//! End-to-end markdown bridge validation for zero-export runtime loading.

use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use xiuxian_qianhuan::{
    ManifestationManager, ManifestationRenderRequest, ManifestationRuntimeContext,
    ManifestationTemplateTarget, MemoryPersonaRecord, MemoryTemplateRecord, PersonaRegistry,
};
use xiuxian_wendao::{
    MarkdownConfigBlock, MarkdownConfigMemoryIndex, extract_markdown_config_blocks,
};

#[test]
fn markdown_bridge_loads_persona_and_template_from_ast_memory() -> Result<()> {
    let markdown = r#"
## Persona: Agenda Steward
<!-- id: "agenda_steward", type: "persona" -->

```toml
name = "Agenda Steward"
voice_tone = "Structured and practical."
style_anchors = ["agenda", "clarity"]
cot_template = "Observe -> draft -> validate"
forbidden_words = ["impossible"]
```

## Template: Daily Agenda
<!-- id: "draft_agenda.j2", type: "template", target: "daily_agenda.md" -->

```jinja2
Agenda owner: {{ user }}
Persona: {{ qianhuan.persona_id }}
Task: {{ task }}
```
"#;

    let blocks = extract_markdown_config_blocks(markdown);
    let index = MarkdownConfigMemoryIndex::from_blocks(blocks.clone());
    assert!(index.get("agenda_steward").is_some());
    assert!(index.get("draft_agenda.j2").is_some());

    let mut registry = PersonaRegistry::new();
    let loaded_personas = registry.load_from_memory_records(persona_records(&blocks))?;
    assert_eq!(loaded_personas, 1);
    assert!(registry.get("agenda_steward").is_some());

    let manager = ManifestationManager::new_empty();
    let loaded_templates = manager.load_templates_from_memory(template_records(&blocks))?;
    assert_eq!(loaded_templates, 2);

    let request = ManifestationRenderRequest {
        target: ManifestationTemplateTarget::DailyAgenda,
        data: json!({
            "user": "Taogege",
            "task": "Validate markdown-config bridge",
        }),
        runtime: ManifestationRuntimeContext {
            state_context: None,
            persona_id: Some("agenda_steward".to_string()),
            domain: Some("zhixing".to_string()),
            extra: HashMap::new(),
        },
    };
    let rendered = manager.render_request(&request)?;

    assert!(rendered.contains("Agenda owner: Taogege"));
    assert!(rendered.contains("Persona: agenda_steward"));
    assert!(rendered.contains("Task: Validate markdown-config bridge"));
    Ok(())
}

fn template_records(blocks: &[MarkdownConfigBlock]) -> Vec<MemoryTemplateRecord> {
    blocks
        .iter()
        .filter(|block| block.config_type.eq_ignore_ascii_case("template"))
        .map(|block| {
            MemoryTemplateRecord::new(
                block.id.clone(),
                block.target.clone(),
                block.content.clone(),
            )
        })
        .collect()
}

fn persona_records(blocks: &[MarkdownConfigBlock]) -> Vec<MemoryPersonaRecord> {
    blocks
        .iter()
        .filter(|block| block.config_type.eq_ignore_ascii_case("persona"))
        .map(|block| MemoryPersonaRecord::new(block.id.clone(), block.content.clone()))
        .collect()
}
