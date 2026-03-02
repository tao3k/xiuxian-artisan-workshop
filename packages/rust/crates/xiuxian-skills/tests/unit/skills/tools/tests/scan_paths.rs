use crate::skills::tools::ToolsScanner;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_scan_paths_multiple_files() -> TestResult {
    let scanner = ToolsScanner::new();
    let files = vec![
        (
            "/virtual/skill/scripts/tool_a.py".to_string(),
            r#"
@skill_command(name="tool_a")
def tool_a(param: str) -> str:
    '''Tool A implementation.'''
    return param
"#
            .to_string(),
        ),
        (
            "/virtual/skill/scripts/tool_b.py".to_string(),
            r#"
@skill_command(name="tool_b")
def tool_b(value: int) -> int:
    '''Tool B implementation.'''
    return value * 2
"#
            .to_string(),
        ),
    ];

    let tools = scanner.scan_paths(&files, "test_skill", &[], &[])?;

    assert_eq!(tools.len(), 2);
    assert!(tools.iter().any(|t| t.tool_name == "test_skill.tool_a"));
    assert!(tools.iter().any(|t| t.tool_name == "test_skill.tool_b"));

    Ok(())
}

#[test]
fn test_scan_paths_skips_init() -> TestResult {
    let scanner = ToolsScanner::new();
    let files = vec![
        (
            "/virtual/skill/scripts/__init__.py".to_string(),
            r#"
@skill_command(name="init_tool")
def init_tool():
    '''This should be skipped.'''
    pass
"#
            .to_string(),
        ),
        (
            "/virtual/skill/scripts/real_tool.py".to_string(),
            r#"
@skill_command(name="real_tool")
def real_tool():
    '''This should be included.'''
    pass
"#
            .to_string(),
        ),
    ];

    let tools = scanner.scan_paths(&files, "test_skill", &[], &[])?;

    // Only one tool (skipping __init__.py)
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "test_skill.real_tool");

    Ok(())
}

#[test]
fn test_scan_paths_skips_private_files() -> TestResult {
    let scanner = ToolsScanner::new();
    let files = vec![
        (
            "/virtual/skill/scripts/_private.py".to_string(),
            r#"
@skill_command(name="private_tool")
def private_tool():
    '''This should be skipped.'''
    pass
"#
            .to_string(),
        ),
        (
            "/virtual/skill/scripts/public.py".to_string(),
            r#"
@skill_command(name="public_tool")
def public_tool():
    '''This should be included.'''
    pass
"#
            .to_string(),
        ),
    ];

    let tools = scanner.scan_paths(&files, "test_skill", &[], &[])?;

    // Only one tool (skipping _private.py)
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "test_skill.public_tool");

    Ok(())
}

#[test]
fn test_scan_paths_skips_non_python() -> TestResult {
    let scanner = ToolsScanner::new();
    let files = vec![
        (
            "/virtual/skill/scripts/readme.md".to_string(),
            "# This is not Python".to_string(),
        ),
        (
            "/virtual/skill/scripts/real_tool.py".to_string(),
            r#"
@skill_command(name="real_tool")
def real_tool():
    pass
"#
            .to_string(),
        ),
    ];

    let tools = scanner.scan_paths(&files, "test_skill", &[], &[])?;

    assert_eq!(tools.len(), 1);

    Ok(())
}

#[test]
fn test_scan_paths_empty_list() -> TestResult {
    let scanner = ToolsScanner::new();
    let files: Vec<(String, String)> = Vec::new();

    let tools = scanner.scan_paths(&files, "test_skill", &[], &[])?;

    assert!(tools.is_empty());

    Ok(())
}

#[test]
fn test_scan_paths_with_keywords_and_intents() -> TestResult {
    let scanner = ToolsScanner::new();
    let files = vec![(
        "/virtual/skill/scripts/tool.py".to_string(),
        r#"
@skill_command(name="test_tool")
def test_tool():
    '''A test tool.'''
    pass
"#
        .to_string(),
    )];

    let keywords = vec!["test".to_string(), "verify".to_string()];
    let intents = vec!["testing".to_string()];

    let tools = scanner.scan_paths(&files, "test_skill", &keywords, &intents)?;

    assert_eq!(tools.len(), 1);
    assert!(tools[0].keywords.contains(&"test".to_string()));
    assert!(tools[0].keywords.contains(&"verify".to_string()));
    assert!(tools[0].intents.contains(&"testing".to_string()));

    Ok(())
}

// Note: Comprehensive integration tests are in tests/tools_scanner.rs
// Category inference tests are in skill_command/category.rs
// These basic tests verify core functionality without complex setup.
