use super::*;

#[test]
fn test_entity_from_dict() {
    let data = serde_json::json!({
        "name": "Claude Code",
        "entity_type": "TOOL",
        "description": "AI coding assistant",
        "source": "docs/tools.md",
        "aliases": ["claude", "claude-dev"],
        "confidence": 0.95
    });

    let Some(entity) = entity_from_dict(&data) else {
        panic!("entity_from_dict should return an entity");
    };
    assert_eq!(entity.name, "Claude Code");
    assert!(matches!(entity.entity_type, EntityType::Tool));
    assert_eq!(entity.aliases.len(), 2);
}

#[test]
fn test_save_and_load_graph() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let graph_path = temp_dir.path().join("test_graph.json");
    let graph_path_str = graph_path.to_string_lossy().into_owned();

    {
        let graph = KnowledgeGraph::new();

        let entity1 = Entity::new(
            "tool:python".to_string(),
            "Python".to_string(),
            EntityType::Skill,
            "Programming language".to_string(),
        );
        let entity2 = Entity::new(
            "tool:claude-code".to_string(),
            "Claude Code".to_string(),
            EntityType::Tool,
            "AI coding assistant".to_string(),
        );

        assert!(graph.add_entity(entity1).is_ok());
        assert!(graph.add_entity(entity2).is_ok());

        let relation = Relation::new(
            "Claude Code".to_string(),
            "Python".to_string(),
            RelationType::Uses,
            "Claude Code uses Python".to_string(),
        );
        assert!(graph.add_relation(&relation).is_ok());
        graph.save_to_file(&graph_path_str)?;
    }

    {
        let mut graph = KnowledgeGraph::new();
        graph.load_from_file(&graph_path_str)?;

        let stats = graph.get_stats();
        assert_eq!(stats.total_entities, 2);
        assert_eq!(stats.total_relations, 1);

        let python = graph.get_entity_by_name("Python");
        let Some(python) = python else {
            panic!("Python entity should exist after load");
        };
        assert_eq!(python.entity_type, EntityType::Skill);

        let relations = graph.get_relations(None, None);
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].source, "Claude Code");
    }
    Ok(())
}

#[test]
fn test_export_as_json() -> Result<(), Box<dyn std::error::Error>> {
    let graph = KnowledgeGraph::new();

    let entity = Entity::new(
        "project:omni".to_string(),
        "Omni Dev Fusion".to_string(),
        EntityType::Project,
        "Development environment".to_string(),
    );

    assert!(graph.add_entity(entity).is_ok());

    let json = graph.export_as_json()?;
    assert!(json.contains("Omni Dev Fusion"));
    assert!(json.contains("entities"));
    assert!(json.contains("relations"));
    Ok(())
}

#[test]
fn test_export_import_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let graph_path = temp_dir.path().join("roundtrip.json");
    let graph_path_str = graph_path.to_string_lossy().into_owned();

    let graph1 = KnowledgeGraph::new();

    let entities = vec![
        ("Python", EntityType::Skill),
        ("Rust", EntityType::Skill),
        ("Claude Code", EntityType::Tool),
        ("Omni Dev Fusion", EntityType::Project),
    ];

    for (name, etype) in &entities {
        let entity = Entity::new(
            format!(
                "{}:{}",
                etype.to_string().to_lowercase(),
                name.to_lowercase().replace(' ', "_")
            ),
            name.to_string(),
            etype.clone(),
            format!("Description of {name}"),
        );
        assert!(graph1.add_entity(entity).is_ok());
    }

    let relations = vec![
        ("Claude Code", "Python", RelationType::Uses),
        ("Claude Code", "Rust", RelationType::Uses),
        ("Omni Dev Fusion", "Claude Code", RelationType::CreatedBy),
    ];

    for (source, target, rtype) in &relations {
        let relation = Relation::new(
            source.to_string(),
            target.to_string(),
            rtype.clone(),
            format!("{source} -> {target}"),
        );
        assert!(graph1.add_relation(&relation).is_ok());
    }

    graph1.save_to_file(&graph_path_str)?;

    let mut graph2 = KnowledgeGraph::new();
    graph2.load_from_file(&graph_path_str)?;

    let stats1 = graph1.get_stats();
    let stats2 = graph2.get_stats();
    assert_eq!(stats1.total_entities, stats2.total_entities);
    assert_eq!(stats1.total_relations, stats2.total_relations);
    Ok(())
}
