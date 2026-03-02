//! Integration tests for dependency indexer core APIs.

use std::path::PathBuf;
use std::time::Instant;

use xiuxian_wendao::dependency_indexer::{DependencyConfig, DependencyIndexer};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
const BENCH_SLACK_ENV: &str = "OMNI_WENDAO_BENCH_SLACK_FACTOR";
const DEFAULT_BENCH_SLACK_FACTOR: f64 = 2.0;

fn benchmark_slack_factor() -> f64 {
    std::env::var(BENCH_SLACK_ENV)
        .ok()
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|factor| factor.is_finite() && *factor >= 1.0)
        .unwrap_or(DEFAULT_BENCH_SLACK_FACTOR)
}

fn benchmark_runtime_multiplier() -> f64 {
    if std::env::var_os("NEXTEST_RUN_ID").is_some() {
        6.0
    } else {
        1.0
    }
}

fn benchmark_budget(local: std::time::Duration, ci: std::time::Duration) -> std::time::Duration {
    let baseline = if std::env::var_os("CI").is_some() {
        ci
    } else {
        local
    };
    baseline.mul_f64(benchmark_slack_factor() * benchmark_runtime_multiplier())
}

#[test]
fn test_indexer_creation() {
    let indexer = DependencyIndexer::new(".", None);
    assert_eq!(indexer.stats().total_crates, 0);
}

#[test]
fn test_config_default() {
    let config = DependencyConfig::default();
    assert_eq!(config.project_root, ".");
}

#[test]
fn test_build_performance() -> TestResult {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .ok_or_else(|| {
            std::io::Error::other("failed to resolve workspace root from CARGO_MANIFEST_DIR")
        })?
        .to_path_buf();
    let project_root_str = project_root.to_string_lossy().to_string();
    let mut config_file = tempfile::NamedTempFile::new()?;
    std::io::Write::write_all(
        &mut config_file,
        br#"[[ast_symbols_external]]
type = "rust"
registry = "cargo"
manifests = ["**/Cargo.toml"]"#,
    )?;

    let mut indexer = DependencyIndexer::new(
        &project_root_str,
        Some(config_file.path().to_string_lossy().as_ref()),
    );

    let start = Instant::now();
    let result = indexer.build(false);
    let elapsed = start.elapsed();

    let max_duration = benchmark_budget(
        std::time::Duration::from_secs(2),
        std::time::Duration::from_secs(4),
    );
    let max_secs = max_duration.as_secs_f64();
    let crates_indexed = result.crates_indexed;
    let total_symbols = result.total_symbols;
    assert!(
        elapsed < max_duration,
        "Build should complete in under {max_secs:.2}s (set {BENCH_SLACK_ENV} >= 1.0 to tune), took: {elapsed:?}",
    );
    assert!(
        crates_indexed >= 10,
        "Should index at least 10 crates, got: {crates_indexed}"
    );
    assert!(
        total_symbols >= 100,
        "Should extract at least 100 symbols, got: {total_symbols}"
    );
    Ok(())
}
