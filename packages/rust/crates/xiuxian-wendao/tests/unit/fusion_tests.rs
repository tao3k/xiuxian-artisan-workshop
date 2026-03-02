use super::*;

#[test]
fn test_apply_link_graph_proximity_boost_bidirectional_link() {
    let mut results = vec![
        RecallResult::new("note-a.md".into(), 0.8, String::new(), String::new()),
        RecallResult::new("note-b.md".into(), 0.7, String::new(), String::new()),
    ];
    let mut stem_links = HashMap::new();
    stem_links.insert("note-a".into(), HashSet::from(["note-b".into()]));
    stem_links.insert("note-b".into(), HashSet::from(["note-a".into()]));
    let stem_tags: HashMap<String, HashSet<String>> = HashMap::new();

    apply_link_graph_proximity_boost(&mut results, &stem_links, &stem_tags, 0.12, 0.08);

    assert!((results[0].score - 0.92).abs() < 1e-6);
    assert!((results[1].score - 0.82).abs() < 1e-6);
}

#[test]
fn test_stem_from_source() {
    assert_eq!(stem_from_source("docs/note-a.md"), "note-a");
    assert_eq!(stem_from_source("note-b.md"), "note-b");
}
