//! Integration tests for `omni-executor` query builder.

use omni_executor::{QueryAction, QueryBuilder};

#[test]
fn test_basic_query() {
    let query = QueryBuilder::new("ls")
        .source("packages/python/core/**/*.py")
        .where_clause("size > 2kb")
        .select(&["name", "size"])
        .build();

    assert!(query.contains("ls packages/python/core/**/*.py"));
    assert!(query.contains("where size > 2kb"));
    assert!(query.contains("select name size"));
    assert!(query.contains("to json --raw"));
}

#[test]
fn test_sort_and_limit() {
    let query = QueryBuilder::new("ls")
        .source(".")
        .sort_by_desc("size")
        .take(5)
        .build();

    assert!(query.contains("sort-by size --reverse"));
    assert!(query.contains("first 5"));
}

#[test]
fn test_unsafe_predicate_rejected() {
    let query = QueryBuilder::new("ls")
        .where_clause("size > 1kb; rm -rf /")
        .build();

    assert!(!query.contains("where size > 1kb; rm -rf /"));
    assert!(query.contains("to json --raw"));
}

#[test]
fn test_closure_query() {
    let query = QueryBuilder::new("ls")
        .where_closure("$row.size > 1kb")
        .build();

    assert!(query.contains("where { |row| $row.size > 1kb }"));
}

#[test]
fn test_mutation_mode_no_json() {
    let query = QueryBuilder::new("save")
        .source("content.txt")
        .with_action_type(QueryAction::Mutate)
        .build();

    assert!(!query.contains("to json"));
}

#[test]
fn test_build_raw() {
    let query = QueryBuilder::new("ls")
        .source(".")
        .where_clause("size > 1kb")
        .build_raw();

    assert!(query.contains("ls ."));
    assert!(query.contains("where size > 1kb"));
    assert!(!query.contains("to json"));
}
