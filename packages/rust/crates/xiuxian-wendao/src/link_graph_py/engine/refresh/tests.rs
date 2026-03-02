use super::plan_apply::{select_refresh_strategy, strategy_label_and_reason};

#[test]
fn link_graph_refresh_strategy_is_noop_when_no_changes() {
    let strategy = select_refresh_strategy(false, 0, 256);
    assert_eq!(strategy_label_and_reason(&strategy), ("noop", "noop"));
}

#[test]
fn link_graph_refresh_strategy_respects_force_full() {
    let strategy = select_refresh_strategy(true, 1, 256);
    assert_eq!(strategy_label_and_reason(&strategy), ("full", "force_full"));
}

#[test]
fn link_graph_refresh_strategy_prefers_incremental_when_threshold_exceeded() {
    let strategy = select_refresh_strategy(false, 300, 256);
    assert_eq!(
        strategy_label_and_reason(&strategy),
        ("delta", "threshold_exceeded_incremental")
    );
}
