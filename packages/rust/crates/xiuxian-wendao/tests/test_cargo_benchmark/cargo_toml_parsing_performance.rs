use super::*;

#[test]
fn test_cargo_toml_parsing_performance() -> Result<(), Box<dyn std::error::Error>> {
    const DEP_COUNT: usize = 100;
    const PARSE_RUNS: usize = 20;

    let start = std::time::Instant::now();

    // Parse multiple Cargo.toml files
    for _ in 0..PARSE_RUNS {
        let content = generate_cargo_toml(DEP_COUNT);
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;

        let deps = parse_cargo_dependencies(file.path())?;
        assert!(!deps.is_empty());
    }

    let elapsed = start.elapsed();

    // Should parse benchmark corpus within bounded time.
    let max_duration = benchmark_budget(
        std::time::Duration::from_secs(1),
        std::time::Duration::from_secs(3),
    );
    assert!(
        elapsed < max_duration,
        "Cargo.toml parsing took {:.2}s for {PARSE_RUNS} files x {DEP_COUNT} deps, expected < {:.2}s",
        elapsed.as_secs_f64(),
        max_duration.as_secs_f64()
    );

    println!(
        "Cargo.toml parsing: {PARSE_RUNS} files x {DEP_COUNT} deps = {:.2}ms",
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(())
}
