use super::*;

fn hit(path: &str, title: &str, doc_type: Option<&str>, tags: Vec<&str>) -> LinkGraphDisplayHit {
    LinkGraphDisplayHit {
        stem: "entry".to_string(),
        title: title.to_string(),
        path: path.to_string(),
        doc_type: doc_type.map(str::to_string),
        tags: tags.into_iter().map(str::to_string).collect(),
        score: 0.9,
        best_section: String::new(),
        match_reason: String::new(),
    }
}

#[test]
fn infer_hit_type_prefers_doc_type_mapping_over_tags_and_path() {
    let display_hit = hit("agenda/today.md", "Today", Some("journal"), vec!["agenda"]);
    assert_eq!(infer_hit_type(&display_hit), "journal");
}

#[test]
fn infer_hit_type_accepts_namespaced_doc_type_values() {
    let display_hit = hit("notes/entry.md", "Entry", Some("zhixing/agenda"), vec![]);
    assert_eq!(infer_hit_type(&display_hit), "agenda");
}

#[test]
fn infer_hit_type_prefers_tag_mapping_over_path() {
    let display_hit = hit("notes/entry.md", "Entry", None, vec!["journal"]);
    assert_eq!(infer_hit_type(&display_hit), "journal");
}

#[test]
fn infer_hit_type_uses_path_when_metadata_is_missing() {
    let display_hit = hit("agenda/today.md", "Today", None, vec![]);
    assert_eq!(infer_hit_type(&display_hit), "agenda");
}

#[test]
fn infer_hit_type_marks_non_markdown_as_attachment() {
    let display_hit = hit("assets/diagram.png", "Diagram", None, vec![]);
    assert_eq!(infer_hit_type(&display_hit), "attachment");
}
