//! Integration tests for `xiuxian_wendao::unified_symbol_py` internals.

use xiuxian_wendao::unified_symbol::UnifiedSymbolIndex;

#[test]
fn test_unified_symbol_creation() {
    let mut index = UnifiedSymbolIndex::new();
    index.add_project_symbol("my_func", "fn", "src/lib.rs:42", "mycrate");
    index.add_external_symbol("spawn", "fn", "task_join_set.rs:1", "tokio");

    let results = index.search_unified("spawn", 10);
    assert_eq!(results.len(), 1);
    assert!(results[0].is_external());
    assert_eq!(results[0].crate_name, "tokio");
}

#[test]
fn test_external_usage() {
    let mut index = UnifiedSymbolIndex::new();
    index.record_external_usage("tokio", "spawn", "src/main.rs:10");
    index.record_external_usage("tokio", "spawn", "src/worker.rs:5");

    let usage = index.find_external_usage("tokio");
    assert_eq!(usage.len(), 2);
}

#[test]
fn test_stats() {
    let mut index = UnifiedSymbolIndex::new();
    index.add_project_symbol("func1", "fn", "src/lib.rs:1", "mycrate");
    index.add_project_symbol("func2", "fn", "src/lib.rs:2", "mycrate");
    index.add_external_symbol("spawn", "fn", "task.rs:1", "tokio");

    let stats = index.stats();
    assert_eq!(stats.total_symbols, 3);
    assert_eq!(stats.project_symbols, 2);
    assert_eq!(stats.external_symbols, 1);
}
