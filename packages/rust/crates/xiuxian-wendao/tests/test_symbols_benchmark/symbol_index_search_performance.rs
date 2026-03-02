use super::*;

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
fn test_symbol_index_search_performance() {
    const SYMBOL_COUNT: usize = 5000;

    let mut index = SymbolIndex::new();

    // Add many symbols to the index
    for i in 0..SYMBOL_COUNT {
        index.add_symbols(
            &format!("crate_{}", i % 10),
            &[ExternalSymbol {
                name: format!("SymbolName{i}"),
                kind: if i % 5 == 0 {
                    SymbolKind::Struct
                } else if i % 5 == 1 {
                    SymbolKind::Function
                } else if i % 5 == 2 {
                    SymbolKind::Enum
                } else {
                    SymbolKind::Trait
                },
                file: PathBuf::from(format!("file_{}.rs", i % 100)),
                line: i,
                crate_name: format!("crate_{}", i % 10),
            }],
        );
    }

    // Benchmark search
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let results = index.search("SymbolName", 50);
        assert!(!results.is_empty());
    }
    let elapsed = start.elapsed();

    // Should complete 100 searches quickly
    let max_duration = benchmark_budget(
        std::time::Duration::from_millis(500),
        std::time::Duration::from_millis(750),
    );
    let max_duration_ms = max_duration.as_secs_f64() * 1000.0;
    assert!(
        elapsed < max_duration,
        "Symbol search took {:.2}ms for {} symbols, expected < {:.2}ms (set {} >= 1.0 to tune)",
        elapsed.as_secs_f64() * 1000.0,
        SYMBOL_COUNT,
        max_duration_ms,
        BENCH_SLACK_ENV
    );

    println!(
        "Symbol index search: {} symbols, 100 searches = {:.2}ms",
        SYMBOL_COUNT,
        elapsed.as_secs_f64() * 1000.0
    );
}
