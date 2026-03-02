use super::{ToolEnrichment, ToolRecord};
use crate::skills::metadata::ToolAnnotations;

#[test]
fn test_new_initializes_optional_fields_with_defaults() {
    let record = ToolRecord::new(
        "writer.polish_text".to_string(),
        "Polish text".to_string(),
        "writer".to_string(),
        "/tmp/writer/scripts/main.py".to_string(),
        "polish_text".to_string(),
    );

    assert_eq!(record.tool_name, "writer.polish_text");
    assert_eq!(record.description, "Polish text");
    assert_eq!(record.skill_name, "writer");
    assert_eq!(record.file_path, "/tmp/writer/scripts/main.py");
    assert_eq!(record.function_name, "polish_text");
    assert!(record.execution_mode.is_empty());
    assert!(record.keywords.is_empty());
    assert!(record.intents.is_empty());
    assert!(record.file_hash.is_empty());
    assert!(record.input_schema.is_empty());
    assert!(record.docstring.is_empty());
    assert!(record.category.is_empty());
    assert_eq!(record.annotations, ToolAnnotations::default());
    assert!(record.parameters.is_empty());
    assert!(record.skill_tools_refers.is_empty());
    assert!(record.resource_uri.is_empty());
}

#[test]
fn test_with_enrichment_applies_all_enriched_fields() {
    let mut annotations = ToolAnnotations::open_world();
    annotations.set_idempotent(true);

    let enrichment = ToolEnrichment {
        execution_mode: "script".to_string(),
        keywords: vec!["writer".to_string(), "polish".to_string()],
        intents: vec!["edit_text".to_string()],
        file_hash: "hash123".to_string(),
        docstring: "Polish text body".to_string(),
        category: "writing".to_string(),
        annotations: annotations.clone(),
        parameters: vec!["text".to_string()],
        input_schema: "{\"type\":\"object\"}".to_string(),
        skill_tools_refers: vec!["writer.normalize".to_string()],
        resource_uri: "omni://skill/writer/polish_text".to_string(),
    };

    let record = ToolRecord::with_enrichment(
        "writer.polish_text".to_string(),
        "Polish text".to_string(),
        "writer".to_string(),
        "/tmp/writer/scripts/main.py".to_string(),
        "polish_text".to_string(),
        enrichment,
    );

    assert_eq!(record.execution_mode, "script");
    assert_eq!(record.keywords, vec!["writer", "polish"]);
    assert_eq!(record.intents, vec!["edit_text"]);
    assert_eq!(record.file_hash, "hash123");
    assert_eq!(record.docstring, "Polish text body");
    assert_eq!(record.category, "writing");
    assert_eq!(record.annotations, annotations);
    assert_eq!(record.parameters, vec!["text"]);
    assert_eq!(record.input_schema, "{\"type\":\"object\"}");
    assert_eq!(record.skill_tools_refers, vec!["writer.normalize"]);
    assert_eq!(record.resource_uri, "omni://skill/writer/polish_text");
}
