use std::fs;
use std::path::{Path, PathBuf};

mod markdown;
mod org;

fn snapshot_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(relative)
}

fn read_snapshot(relative: &str) -> String {
    let path = snapshot_path(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read snapshot {}: {error}", path.display());
    })
}

fn assert_snapshot_eq(relative: &str, actual: &str) {
    let expected = read_snapshot(relative);
    assert!(
        expected == actual,
        "snapshot mismatch: {relative}\n--- expected ---\n{expected}\n--- actual ---\n{actual}"
    );
}
