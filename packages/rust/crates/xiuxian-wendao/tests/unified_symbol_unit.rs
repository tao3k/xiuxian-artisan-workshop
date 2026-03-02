//! Integration tests for `xiuxian_wendao::unified_symbol`.

use xiuxian_wendao::unified_symbol::{UnifiedSymbol, UnifiedSymbolIndex};

#[test]
fn test_unified_symbol_creation() {
    let proj = UnifiedSymbol::new_project("my_func", "fn", "src/lib.rs:42", "mycrate");
    assert!(proj.is_project());
    assert_eq!(proj.crate_name, "mycrate");

    let ext = UnifiedSymbol::new_external("spawn", "fn", "task_join_set.rs:1", "tokio");
    assert!(ext.is_external());
    assert_eq!(ext.crate_name, "tokio");
}

#[test]
fn test_unified_search() {
    let mut index = UnifiedSymbolIndex::new();

    index.add_project_symbol("my_func", "fn", "src/lib.rs:42", "mycrate");
    index.add_external_symbol("spawn", "fn", "task_join_set.rs:1", "tokio");
    index.add_external_symbol("spawn_local", "mod", "task_join_set.rs:1", "tokio");

    let results = index.search_unified("spawn", 10);
    assert_eq!(results.len(), 2);

    let proj_results = index.search_project("spawn", 10);
    assert_eq!(proj_results.len(), 0);

    let ext_results = index.search_external("spawn", 10);
    assert_eq!(ext_results.len(), 2);
}

#[test]
fn test_external_usage() {
    let mut index = UnifiedSymbolIndex::new();

    index.record_external_usage("tokio", "spawn", "src/main.rs:10");
    index.record_external_usage("tokio", "spawn", "src/worker.rs:5");

    let usage = index.find_external_usage("tokio");
    assert_eq!(usage.len(), 2);
    assert!(usage.contains(&"src/main.rs:10"));
    assert!(usage.contains(&"src/worker.rs:5"));
}

#[test]
fn test_stats() {
    let mut index = UnifiedSymbolIndex::new();

    index.add_project_symbol("func1", "fn", "src/lib.rs:1", "mycrate");
    index.add_project_symbol("func2", "fn", "src/lib.rs:2", "mycrate");
    index.add_external_symbol("spawn", "fn", "task.rs:1", "tokio");
    index.record_external_usage("tokio", "spawn", "src/main.rs:10");

    let stats = index.stats();
    assert_eq!(stats.total_symbols, 3);
    assert_eq!(stats.project_symbols, 2);
    assert_eq!(stats.external_symbols, 1);
    assert_eq!(stats.external_crates, 1);
}
