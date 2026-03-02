//! Integration tests for the `LinkGraph` enhancer pipeline and config parsing.

use xiuxian_wendao::enhancer::infer_relations;
use xiuxian_wendao::{
    LinkGraphEntityRef, MarkdownConfigMemoryIndex, NoteFrontmatter, NoteInput, enhance_note,
    enhance_notes_batch, extract_markdown_config_blocks, extract_markdown_config_links_by_id,
    parse_frontmatter,
};

#[test]
fn parse_frontmatter_basic() {
    let content =
        "---\ntitle: My Note\ndescription: A test\ntags:\n  - python\n  - rust\n---\n# Content";
    let fm = parse_frontmatter(content);
    assert_eq!(fm.title.as_deref(), Some("My Note"));
    assert_eq!(fm.description.as_deref(), Some("A test"));
    assert_eq!(fm.tags, vec!["python", "rust"]);
}

#[test]
fn parse_frontmatter_skill_metadata() {
    let content = "---\nname: git\ndescription: Git ops\nmetadata:\n  routing_keywords:\n    - commit\n    - branch\n  intents:\n    - version_control\n---\n# SKILL";
    let fm = parse_frontmatter(content);
    assert_eq!(fm.name.as_deref(), Some("git"));
    assert_eq!(fm.routing_keywords, vec!["commit", "branch"]);
    assert_eq!(fm.intents, vec!["version_control"]);
}

#[test]
fn parse_frontmatter_empty_document() {
    let fm = parse_frontmatter("# No frontmatter");
    assert!(fm.title.is_none());
    assert!(fm.tags.is_empty());
}

#[test]
fn parse_frontmatter_malformed_payload() {
    let fm = parse_frontmatter("---\n: bad [[\n---\n");
    assert!(fm.title.is_none());
}

#[test]
fn infer_relations_documented_in() {
    let refs = vec![LinkGraphEntityRef::new(
        "Python".to_string(),
        None,
        "[[Python]]".to_string(),
    )];
    let fm = NoteFrontmatter::default();
    let relations = infer_relations("docs/test.md", "Test Doc", &fm, &refs);

    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].source, "Python");
    assert_eq!(relations[0].relation_type, "DOCUMENTED_IN");
}

#[test]
fn infer_relations_skill_contains() {
    let fm = NoteFrontmatter {
        name: Some("git".to_string()),
        ..Default::default()
    };
    let relations = infer_relations("assets/skills/git/SKILL.md", "Git Skill", &fm, &[]);

    let contains: Vec<_> = relations
        .iter()
        .filter(|relation| relation.relation_type == "CONTAINS")
        .collect();
    assert_eq!(contains.len(), 1);
    assert_eq!(contains[0].source, "git");
}

#[test]
fn infer_relations_tags() {
    let fm = NoteFrontmatter {
        tags: vec!["search".to_string(), "vector".to_string()],
        ..Default::default()
    };
    let relations = infer_relations("docs/test.md", "Test", &fm, &[]);

    let tag_relations: Vec<_> = relations
        .iter()
        .filter(|relation| relation.relation_type == "RELATED_TO")
        .collect();
    assert_eq!(tag_relations.len(), 2);
}

#[test]
fn enhance_note_full() {
    let input = NoteInput {
        path: "docs/test.md".to_string(),
        title: "Test Doc".to_string(),
        content: "---\ntitle: Test\ntags:\n  - demo\n---\nContent with [[Python#lang]] ref"
            .to_string(),
    };

    let result = enhance_note(&input);
    assert_eq!(result.frontmatter.title.as_deref(), Some("Test"));
    assert_eq!(result.entity_refs.len(), 1);
    assert_eq!(result.entity_refs[0].name, "Python");
    assert_eq!(result.entity_refs[0].entity_type.as_deref(), Some("lang"));
    assert!(result.ref_stats.total_refs >= 1);
    assert!(result.inferred_relations.len() >= 2);
}

#[test]
fn enhance_notes_batch_parallel() {
    let inputs = vec![
        NoteInput {
            path: "a.md".to_string(),
            title: "A".to_string(),
            content: "About [[X]]".to_string(),
        },
        NoteInput {
            path: "b.md".to_string(),
            title: "B".to_string(),
            content: "About [[Y]] and [[Z]]".to_string(),
        },
    ];

    let results = enhance_notes_batch(&inputs);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].entity_refs.len(), 1);
    assert_eq!(results[1].entity_refs.len(), 2);
}

#[test]
fn extract_markdown_config_blocks_jinja2_under_tagged_heading() {
    let markdown = r#"
## Template: Draft Agenda
<!-- id: "draft_agenda.j2", type: "template", target: "daily_agenda.md" -->

```jinja2
Agenda: {{ title }}
```

```python
print("ignore")
```
"#;

    let blocks = extract_markdown_config_blocks(markdown);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].id, "draft_agenda.j2");
    assert_eq!(blocks[0].config_type, "template");
    assert_eq!(blocks[0].target.as_deref(), Some("daily_agenda.md"));
    assert_eq!(blocks[0].language, "jinja2");
    assert!(blocks[0].content.contains("Agenda: {{ title }}"));
}

#[test]
fn extract_markdown_config_blocks_keeps_scope_for_deeper_headings() {
    let markdown = r#"
## Template: Parent
<!-- id: "parent_template.j2", type: "template", target: "daily_agenda.md" -->

### Context
Details.

```jinja2
Parent template payload
```
"#;

    let blocks = extract_markdown_config_blocks(markdown);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].id, "parent_template.j2");
    assert_eq!(blocks[0].heading, "Template: Parent");
    assert!(blocks[0].content.contains("Parent template payload"));
}

#[test]
fn markdown_config_memory_index_exact_lookup_by_id() {
    let markdown = r#"
## Template: Draft Agenda
<!-- id: "draft_agenda.j2", type: "template", target: "daily_agenda.md" -->

```jinja2
Agenda: {{ title }}
```
"#;

    let index = MarkdownConfigMemoryIndex::from_markdown(markdown);
    assert_eq!(index.len(), 1);
    let Some(block) = index.get("draft_agenda.j2") else {
        panic!("expected exact id to resolve in O(1) memory index");
    };
    assert_eq!(block.target.as_deref(), Some("daily_agenda.md"));
}

#[test]
fn extract_markdown_config_blocks_persona_toml_under_tagged_heading() {
    let markdown = r#"
## Persona: Agenda Steward
<!-- id: "agenda_steward", type: "persona" -->

```toml
name = "Agenda Steward"
voice_tone = "Structured and practical."
style_anchors = ["agenda"]
cot_template = "Observe -> draft -> validate"
forbidden_words = ["impossible"]
```
"#;

    let blocks = extract_markdown_config_blocks(markdown);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].id, "agenda_steward");
    assert_eq!(blocks[0].config_type, "persona");
    assert!(blocks[0].target.is_none());
    assert_eq!(blocks[0].language, "toml");
    assert!(blocks[0].content.contains("name = \"Agenda Steward\""));
}

#[test]
fn extract_markdown_config_links_by_id_tracks_tagged_heading_scope() {
    let markdown = r#"
## Persona: Agenda Steward
<!-- id: "agenda_steward", type: "persona" -->
Use [persona config](./personas/agenda_steward.toml).
Use [[templates/draft_agenda.j2]].
Ignore [external](https://example.com/docs).

### Child Notes
Use [daily](./templates/daily_agenda.md#today).

## Persona: Strict Teacher
<!-- id: "strict_teacher", type: "persona" -->
Use [teacher](../shared/strict_teacher.toml).
"#;

    let links = extract_markdown_config_links_by_id(markdown, "zhixing/skill.md");

    assert_eq!(
        links.get("agenda_steward"),
        Some(&vec![
            "zhixing/personas/agenda_steward.toml".to_string(),
            "zhixing/templates/draft_agenda.j2".to_string(),
            "zhixing/templates/daily_agenda.md".to_string()
        ])
    );
    assert_eq!(
        links.get("strict_teacher"),
        Some(&vec!["shared/strict_teacher.toml".to_string()])
    );
}

#[test]
fn extract_markdown_config_links_by_id_keeps_wendao_resource_uri_targets() {
    let markdown = r#"
## Template: Draft Agenda
<!-- id: "draft_agenda.j2", type: "template", target: "draft_agenda.j2" -->
Use semantic address:
[template](wendao://skills/agenda-management/references/draft_agenda.j2#v1).
"#;

    let links =
        extract_markdown_config_links_by_id(markdown, "zhixing/skills/agenda-management/SKILL.md");
    assert_eq!(
        links.get("draft_agenda.j2"),
        Some(&vec![
            "wendao://skills/agenda-management/references/draft_agenda.j2".to_string()
        ])
    );
}
