use super::ReferenceRecord;

#[test]
fn test_deserialize_for_skills_and_for_tools_from_string() -> Result<(), Box<dyn std::error::Error>>
{
    let record: ReferenceRecord = serde_yaml::from_str(
        r#"
ref_name: "run_research_graph"
title: "Run Research Graph"
skill_name: "researcher"
file_path: "assets/skills/researcher/references/run_research_graph.md"
for_skills: "researcher"
for_tools: "researcher.run_research_graph"
"#,
    )?;

    assert_eq!(record.for_skills, vec!["researcher".to_string()]);
    assert_eq!(
        record.for_tools,
        Some(vec!["researcher.run_research_graph".to_string()])
    );

    Ok(())
}

#[test]
fn test_deserialize_for_skills_and_for_tools_from_sequence()
-> Result<(), Box<dyn std::error::Error>> {
    let record: ReferenceRecord = serde_yaml::from_str(
        r#"
ref_name: "graph_doc"
title: "Graph Doc"
skill_name: "researcher"
file_path: "assets/skills/researcher/references/graph_doc.md"
for_skills:
  - "researcher"
  - "writer"
for_tools:
  - "researcher.run_research_graph"
  - "writer.polish_text"
"#,
    )?;

    assert_eq!(
        record.for_skills,
        vec!["researcher".to_string(), "writer".to_string()]
    );
    assert_eq!(
        record.for_tools,
        Some(vec![
            "researcher.run_research_graph".to_string(),
            "writer.polish_text".to_string()
        ])
    );

    Ok(())
}

#[test]
fn test_applies_to_tool_matches_only_configured_tools() {
    let record = ReferenceRecord::new(
        "run_research_graph".to_string(),
        "Run Research Graph".to_string(),
        "researcher".to_string(),
        "assets/skills/researcher/references/run_research_graph.md".to_string(),
    )
    .with_for_tools(Some(vec![
        "researcher.run_research_graph".to_string(),
        "writer.polish_text".to_string(),
    ]));

    assert!(record.applies_to_tool("researcher.run_research_graph"));
    assert!(record.applies_to_tool("writer.polish_text"));
    assert!(!record.applies_to_tool("researcher.search_docs"));
}
