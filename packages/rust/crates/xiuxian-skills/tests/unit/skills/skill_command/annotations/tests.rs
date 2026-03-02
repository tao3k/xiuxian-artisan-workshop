use super::build_annotations;
use crate::skills::metadata::DecoratorArgs;

#[test]
fn test_read_only_heuristic() {
    let args = DecoratorArgs::default();
    let ann = build_annotations(&args, "get_data", &[]);
    assert!(ann.read_only);
    assert!(ann.is_idempotent());
}

#[test]
fn test_destructive_heuristic() {
    let args = DecoratorArgs::default();
    let ann = build_annotations(&args, "delete_file", &[]);
    assert!(ann.destructive);
    assert!(!ann.is_idempotent());
}

#[test]
fn test_network_heuristic() {
    let args = DecoratorArgs::default();
    let ann = build_annotations(&args, "fetch_url", &[]);
    assert!(ann.is_open_world());
}

#[test]
fn test_explicit_override() {
    let args = DecoratorArgs {
        read_only: Some(false),
        ..DecoratorArgs::default()
    };
    let ann = build_annotations(&args, "get_data", &[]);
    assert!(!ann.read_only);
}
