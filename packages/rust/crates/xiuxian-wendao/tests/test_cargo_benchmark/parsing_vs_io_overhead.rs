use super::*;

#[test]
fn test_parsing_vs_io_overhead() -> Result<(), Box<dyn std::error::Error>> {
    const DEP_COUNT: usize = 50;
    const PARSE_RUNS: usize = 50;

    // Generate content once
    let content = generate_cargo_toml(DEP_COUNT);

    // Test pure parsing (multiple parses of same content)
    let parse_start = std::time::Instant::now();
    for _ in 0..PARSE_RUNS {
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;
        let _deps = parse_cargo_dependencies(file.path())?;
    }
    let parse_elapsed = parse_start.elapsed();

    // Report the breakdown
    println!(
        "Parsing {DEP_COUNT} deps: {:.2}ms for {PARSE_RUNS} iterations (includes I/O)",
        parse_elapsed.as_secs_f64() * 1000.0
    );

    // Just verify it completes in reasonable time
    let max_duration = benchmark_budget(
        std::time::Duration::from_secs(2),
        std::time::Duration::from_secs(5),
    );
    assert!(
        parse_elapsed < max_duration,
        "Parsing took too long: {:.2}s (budget {:.2}s)",
        parse_elapsed.as_secs_f64(),
        max_duration.as_secs_f64()
    );
    Ok(())
}
