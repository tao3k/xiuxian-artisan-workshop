use super::*;

#[test]
fn test_complex_dependency_parsing_performance() -> Result<(), Box<dyn std::error::Error>> {
    const PARSE_RUNS: usize = 100;

    // Test the complex format: name = { version = "x.y.z", features = [...] }
    let content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
tokio = { version = "1.49.0", features = ["full", "tracing"] }
serde = { version = "1.0.228", features = ["derive", "rc"] }
serde_json = { version = "1.0.149", features = ["std", "arbitrary_precision"] }
anyhow = { version = "1.0.100", features = ["backtrace"] }
thiserror = { version = "2.0.17", features = ["std"] }
async-trait = { version = "0.1.83", features = ["async-lift"] }
futures = { version = "0.3.31", features = ["async-await", "compat"] }
"#;

    let start = std::time::Instant::now();

    // Parse the same content multiple times
    for _ in 0..PARSE_RUNS {
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;

        let deps = parse_cargo_dependencies(file.path())?;
        assert_eq!(deps.len(), 7);
    }

    let elapsed = start.elapsed();

    // Should complete benchmark corpus within bounded time.
    let max_duration = benchmark_budget(
        std::time::Duration::from_millis(500),
        std::time::Duration::from_millis(1_500),
    );
    assert!(
        elapsed < max_duration,
        "Complex dependency parsing took {:.2}ms for {PARSE_RUNS} iterations, expected < {:.2}ms",
        elapsed.as_secs_f64() * 1000.0,
        max_duration.as_secs_f64() * 1000.0
    );

    println!(
        "Complex dependency parsing: {PARSE_RUNS} iterations = {:.2}ms",
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(())
}
