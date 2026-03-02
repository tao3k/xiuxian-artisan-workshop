use super::snapshot_path;

#[test]
fn org_snapshot_directory_is_reserved_for_future_parser() {
    let dir = snapshot_path("parser/org");
    assert!(
        dir.exists(),
        "expected parser/org snapshot directory to exist for upcoming org parser coverage"
    );
}

#[test]
#[ignore = "org parser snapshots will be enabled when org parser lands"]
fn org_parser_snapshot_contract_placeholder() {
    // Intentionally empty: this test reserves the contract slot for future org parser snapshots.
}
