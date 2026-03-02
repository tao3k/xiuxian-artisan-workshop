use super::*;

#[test]
fn test_register_skill_entities_creates_entities_and_relations() {
    let graph = KnowledgeGraph::new();

    let docs = vec![
        SkillDoc {
            id: "git".to_string(),
            doc_type: "skill".to_string(),
            skill_name: "git".to_string(),
            tool_name: String::new(),
            content: "Git version control operations".to_string(),
            routing_keywords: vec![],
        },
        SkillDoc {
            id: "git.smart_commit".to_string(),
            doc_type: "command".to_string(),
            skill_name: "git".to_string(),
            tool_name: "git.smart_commit".to_string(),
            content: "Create a smart commit with AI-generated message".to_string(),
            routing_keywords: vec!["commit".to_string(), "git".to_string()],
        },
        SkillDoc {
            id: "git.status".to_string(),
            doc_type: "command".to_string(),
            skill_name: "git".to_string(),
            tool_name: "git.status".to_string(),
            content: "Show working tree status".to_string(),
            routing_keywords: vec!["status".to_string(), "git".to_string()],
        },
        SkillDoc {
            id: "knowledge".to_string(),
            doc_type: "skill".to_string(),
            skill_name: "knowledge".to_string(),
            tool_name: String::new(),
            content: "Knowledge base operations".to_string(),
            routing_keywords: vec![],
        },
        SkillDoc {
            id: "knowledge.recall".to_string(),
            doc_type: "command".to_string(),
            skill_name: "knowledge".to_string(),
            tool_name: "knowledge.recall".to_string(),
            content: "Recall knowledge from memory".to_string(),
            routing_keywords: vec!["search".to_string(), "recall".to_string()],
        },
    ];

    let result = graph
        .register_skill_entities(&docs)
        .unwrap_or_else(|error| panic!("skill entity registration should succeed: {error}"));

    // 2 skills + 3 tools + 4 unique keywords = 9 entities
    assert!(
        result.entities_added >= 9,
        "Expected >= 9 entities, got {}",
        result.entities_added
    );

    // CONTAINS: git->git.smart_commit, git->git.status, knowledge->knowledge.recall = 3
    // RELATED_TO: git.smart_commit->{commit,git}, git.status->{status,git}, knowledge.recall->{search,recall} = 6
    assert!(
        result.relations_added >= 9,
        "Expected >= 9 relations, got {}",
        result.relations_added
    );

    let stats = graph.get_stats();
    assert_eq!(*stats.entities_by_type.get("SKILL").unwrap_or(&0), 2);
    assert_eq!(*stats.entities_by_type.get("TOOL").unwrap_or(&0), 3);

    let hops = graph.multi_hop_search("git", 2);
    let names: Vec<String> = hops.iter().map(|e| e.name.clone()).collect();
    assert!(
        names.contains(&"git.smart_commit".to_string()),
        "Multi-hop from 'git' should reach 'git.smart_commit', got: {names:?}",
    );
}

#[test]
fn test_register_skill_entities_idempotent() {
    let graph = KnowledgeGraph::new();

    let docs = vec![SkillDoc {
        id: "git".to_string(),
        doc_type: "skill".to_string(),
        skill_name: "git".to_string(),
        tool_name: String::new(),
        content: "Git operations".to_string(),
        routing_keywords: vec![],
    }];

    let r1 = graph
        .register_skill_entities(&docs)
        .unwrap_or_else(|error| panic!("first registration should succeed: {error}"));
    let r2 = graph
        .register_skill_entities(&docs)
        .unwrap_or_else(|error| panic!("second registration should succeed: {error}"));

    assert_eq!(r1.entities_added, 1);
    assert_eq!(r2.entities_added, 0);
    assert_eq!(graph.get_stats().total_entities, 1);
}

#[test]
fn test_register_skill_entities_shared_keyword_creates_graph_connections() {
    let graph = KnowledgeGraph::new();

    let docs = vec![
        SkillDoc {
            id: "knowledge".to_string(),
            doc_type: "skill".to_string(),
            skill_name: "knowledge".to_string(),
            tool_name: String::new(),
            content: "Knowledge skill".to_string(),
            routing_keywords: vec![],
        },
        SkillDoc {
            id: "knowledge.recall".to_string(),
            doc_type: "command".to_string(),
            skill_name: "knowledge".to_string(),
            tool_name: "knowledge.recall".to_string(),
            content: "Recall from knowledge base".to_string(),
            routing_keywords: vec!["search".to_string()],
        },
        SkillDoc {
            id: "researcher".to_string(),
            doc_type: "skill".to_string(),
            skill_name: "researcher".to_string(),
            tool_name: String::new(),
            content: "Research skill".to_string(),
            routing_keywords: vec![],
        },
        SkillDoc {
            id: "researcher.search".to_string(),
            doc_type: "command".to_string(),
            skill_name: "researcher".to_string(),
            tool_name: "researcher.search".to_string(),
            content: "Search the web".to_string(),
            routing_keywords: vec!["search".to_string()],
        },
    ];

    assert!(graph.register_skill_entities(&docs).is_ok());

    let search_rels = graph.get_relations(Some("keyword:search"), None);
    assert!(
        search_rels.len() >= 2,
        "keyword:search should have relations from both tools, got: {}",
        search_rels.len()
    );
}

#[test]
fn test_register_skill_entities_creates_qianji_flow_governs_relation() {
    let graph = KnowledgeGraph::new();

    let docs = vec![SkillDoc {
        id: "agenda".to_string(),
        doc_type: "skill".to_string(),
        skill_name: "agenda".to_string(),
        tool_name: String::new(),
        content: "Flow mapping [[references/agenda_flow.toml#qianji-flow]]".to_string(),
        routing_keywords: vec![],
    }];

    let result = graph
        .register_skill_entities(&docs)
        .unwrap_or_else(|error| panic!("qianji-flow registration should succeed: {error}"));
    assert!(
        result.entities_added >= 2,
        "Expected skill + qianji flow entities, got {}",
        result.entities_added
    );

    let flow = graph.get_entity_by_name("agenda_flow");
    let Some(flow) = flow else {
        panic!("QianjiFlow entity 'agenda_flow' should exist");
    };
    assert_eq!(
        flow.entity_type,
        EntityType::Other("QianjiFlow".to_string())
    );

    let governs = graph.get_relations(Some("agenda"), Some(RelationType::Governs));
    assert!(
        governs
            .iter()
            .any(|relation| relation.source == "agenda" && relation.target == "agenda_flow"),
        "Expected GOVERNS relation agenda -> agenda_flow, got: {governs:?}"
    );
}

#[test]
fn test_register_skill_entities_extracts_qianji_flow_from_command_doc() {
    let graph = KnowledgeGraph::new();

    let docs = vec![SkillDoc {
        id: "agenda.validate".to_string(),
        doc_type: "command".to_string(),
        skill_name: "agenda".to_string(),
        tool_name: "agenda.validate".to_string(),
        content: "Use [[agenda_flow.toml#qianji-flow]] for validation".to_string(),
        routing_keywords: vec!["agenda".to_string()],
    }];

    graph
        .register_skill_entities(&docs)
        .unwrap_or_else(|error| panic!("command qianji-flow registration should succeed: {error}"));

    let governs = graph.get_relations(Some("agenda"), Some(RelationType::Governs));
    assert!(
        governs
            .iter()
            .any(|relation| relation.source == "agenda" && relation.target == "agenda_flow"),
        "Expected command-driven GOVERNS relation agenda -> agenda_flow, got: {governs:?}"
    );
}
