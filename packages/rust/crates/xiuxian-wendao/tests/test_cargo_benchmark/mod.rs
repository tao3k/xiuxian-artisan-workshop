//! Benchmark tests for Cargo.toml parsing performance.
//!
//! These tests measure the performance of parsing Cargo.toml files
//! for dependency extraction.

use std::fmt::Write as FmtWrite;
use std::io::Write as StdWrite;
use tempfile::NamedTempFile;
use xiuxian_wendao::dependency_indexer::parse_cargo_dependencies;

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

fn append_format(content: &mut String, args: std::fmt::Arguments<'_>) {
    if content.write_fmt(args).is_err() {
        unreachable!("formatting into String should not fail");
    }
}

/// Generate a complex Cargo.toml for benchmarking.
fn generate_cargo_toml(dep_count: usize) -> String {
    let mut content = String::from(
        "[package]\nname = \"test-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
    );

    // Add simple dependencies
    for i in 0..dep_count {
        append_format(
            &mut content,
            format_args!("dep{i} = \"{}.{}.{}\"\n", i / 100, (i / 10) % 10, i % 10),
        );
    }

    content.push_str("\n[dev-dependencies]\n");
    for i in 0..(dep_count / 3) {
        append_format(
            &mut content,
            format_args!(
                "dev_dep{i} = \"{}.{}.{}\"\n",
                i / 100,
                (i / 10) % 10,
                i % 10
            ),
        );
    }

    content
}

/// Generate a workspace Cargo.toml for benchmarking.
fn generate_workspace_cargo_toml(member_count: usize, dep_count: usize) -> String {
    let mut content = String::from("[workspace]\nmembers = [");

    // Add workspace members
    for i in 0..member_count {
        let separator = if i + 1 == member_count { "" } else { ", " };
        append_format(&mut content, format_args!("\"crate{i}\"{separator}"));
    }
    content.push_str("]\n\n[workspace.dependencies]\n");

    // Add workspace dependencies with complex format
    for i in 0..dep_count {
        append_format(
            &mut content,
            format_args!(
                "dep{i} = {{ version = \"{}.{}.{}\", features = [\"feature-a\", \"feature-b\"] }}\n",
                i / 100,
                (i / 10) % 10,
                i % 10
            ),
        );
    }

    // Add simple dependencies
    for i in 0..(dep_count / 2) {
        append_format(
            &mut content,
            format_args!(
                "simple_dep{i} = \"{}.{}.{}\"\n",
                i / 100,
                (i / 10) % 10,
                i % 10
            ),
        );
    }

    content
}

/// Benchmark test for parsing Cargo.toml with many dependencies.
mod cargo_toml_parsing_performance;
mod complex_dependency_parsing_performance;
mod parsing_vs_io_overhead;
mod workspace_cargo_toml_parsing_performance;
