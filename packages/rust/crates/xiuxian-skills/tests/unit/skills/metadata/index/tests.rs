use super::{DocsAvailable, IndexToolEntry, SkillIndexEntry};

#[test]
fn test_docs_available_default_shape() {
    let docs = DocsAvailable::default();
    assert!(docs.skill_md);
    assert!(!docs.readme);
    assert!(!docs.tests);
}

#[test]
fn test_skill_index_entry_add_tool_and_has_tools() {
    let mut entry = SkillIndexEntry::new(
        "writer".to_string(),
        "Writer skill".to_string(),
        "1.0.0".to_string(),
        "assets/skills/writer".to_string(),
    );

    assert!(!entry.has_tools());
    assert_eq!(entry.authors, vec!["omni-dev-fusion"]);

    entry.add_tool(IndexToolEntry {
        name: "writer.polish_text".to_string(),
        description: "Polish text".to_string(),
        category: "writing".to_string(),
        input_schema: "{\"type\":\"object\"}".to_string(),
        file_hash: "abc123".to_string(),
    });

    assert!(entry.has_tools());
    assert_eq!(entry.tools.len(), 1);
    assert_eq!(entry.tools[0].name, "writer.polish_text");
}
