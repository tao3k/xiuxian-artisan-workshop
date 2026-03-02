//! Integration tests for `xiuxian_wendao::types`.

use xiuxian_wendao::{KnowledgeCategory, KnowledgeEntry, KnowledgeSearchQuery};

#[test]
fn test_knowledge_entry_creation() {
    let entry = KnowledgeEntry::new(
        "test-001".to_string(),
        "Test Entry".to_string(),
        "Test content".to_string(),
        KnowledgeCategory::Note,
    );

    assert_eq!(entry.id, "test-001");
    assert_eq!(entry.title, "Test Entry");
    assert_eq!(entry.category, KnowledgeCategory::Note);
    assert_eq!(entry.version, 1);
}

#[test]
fn test_knowledge_entry_with_tags() {
    let entry = KnowledgeEntry::new(
        "test-002".to_string(),
        "Tagged Entry".to_string(),
        "Content".to_string(),
        KnowledgeCategory::Pattern,
    )
    .with_tags(vec!["rust".to_string(), "patterns".to_string()])
    .with_source(Some("docs/test.md".to_string()));

    assert_eq!(entry.tags.len(), 2);
    assert_eq!(entry.source, Some("docs/test.md".to_string()));
}

#[test]
fn test_search_query() {
    let query = KnowledgeSearchQuery::new("error handling".to_string())
        .with_category(KnowledgeCategory::Error)
        .with_tags(vec!["exception".to_string()])
        .with_limit(10);

    assert_eq!(query.query, "error handling");
    assert_eq!(query.limit, 10);
}
