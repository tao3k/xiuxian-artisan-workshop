use crate::skills::tools::ToolsScanner;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_parse_content_single_tool() -> TestResult {
    let scanner = ToolsScanner::new();
    let content = r#"
from agent.skills.decorators import skill_command

@skill_command(name="write_text")
def write_text(content: str) -> str:
    '''Write text to a file.'''
    return "written"
"#;

    let tools = scanner.parse_content(
        content,
        "/virtual/path/scripts/tool.py",
        "writer",
        &["write".to_string()],
        &[],
    )?;

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "writer.write_text");
    assert_eq!(tools[0].function_name, "write_text");
    assert_eq!(tools[0].file_path, "/virtual/path/scripts/tool.py");
    assert!(tools[0].keywords.contains(&"write".to_string()));

    Ok(())
}

#[test]
fn test_parse_content_multiple_tools() -> TestResult {
    let scanner = ToolsScanner::new();
    let content = r#"
from agent.skills.decorators import skill_command

@skill_command(name="commit")
def commit(message: str) -> str:
    '''Create a commit.'''
    return f"Committed: {message}"

@skill_command(name="status")
def status() -> str:
    '''Show working tree status.'''
    return "status output"
"#;

    let tools = scanner.parse_content(
        content,
        "/virtual/path/scripts/main.py",
        "git",
        &["git".to_string()],
        &[],
    )?;

    assert_eq!(tools.len(), 2);
    assert!(tools.iter().any(|t| t.tool_name == "git.commit"));
    assert!(tools.iter().any(|t| t.tool_name == "git.status"));

    Ok(())
}

#[test]
fn test_parse_content_no_decorators() -> TestResult {
    let scanner = ToolsScanner::new();
    let content = r#"
def regular_function():
    '''This function has no decorator.'''
    return "no tool here"
"#;

    let tools =
        scanner.parse_content(content, "/virtual/path/scripts/tool.py", "test", &[], &[])?;

    assert!(tools.is_empty());

    Ok(())
}

#[test]
fn test_parse_content_skips_init() -> TestResult {
    let scanner = ToolsScanner::new();
    let content = r#"
from agent.skills.decorators import skill_command

@skill_command(name="init_tool")
def init_tool():
    '''This should be skipped.'''
    pass
"#;

    // parse_content doesn't skip __init__.py - that's handled in scan_paths
    let tools = scanner.parse_content(
        content,
        "/virtual/path/scripts/__init__.py",
        "test",
        &[],
        &[],
    )?;

    // This should find the tool since we're calling parse_content directly
    assert_eq!(tools.len(), 1);

    Ok(())
}

#[test]
fn test_parse_content_with_category() -> TestResult {
    let scanner = ToolsScanner::new();
    let content = r#"
@skill_command(name="test_tool", category="testing")
def test_tool():
    '''A test tool.'''
    pass
"#;

    let tools =
        scanner.parse_content(content, "/virtual/path/scripts/tool.py", "test", &[], &[])?;

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].category, "testing");

    Ok(())
}

#[test]
fn test_parse_content_with_intents() -> TestResult {
    let scanner = ToolsScanner::new();
    let content = r#"
@skill_command(name="test_tool")
def test_tool():
    '''A test tool.'''
    pass
"#;

    let intents = vec!["test".to_string(), "verify".to_string()];
    let tools = scanner.parse_content(
        content,
        "/virtual/path/scripts/tool.py",
        "test",
        &[],
        &intents,
    )?;

    assert_eq!(tools.len(), 1);
    assert!(tools[0].intents.contains(&"test".to_string()));
    assert!(tools[0].intents.contains(&"verify".to_string()));

    Ok(())
}

#[test]
fn test_parse_content_file_hash() -> TestResult {
    let scanner = ToolsScanner::new();
    let content = r#"
@skill_command(name="tool")
def tool():
    pass
"#;

    let tools1 =
        scanner.parse_content(content, "/virtual/path/scripts/tool.py", "test", &[], &[])?;

    // Same content should produce same hash
    let tools2 =
        scanner.parse_content(content, "/virtual/path/scripts/tool.py", "test", &[], &[])?;

    assert_eq!(tools1[0].file_hash, tools2[0].file_hash);

    // Different content should produce different hash
    let content2 = r#"
@skill_command(name="tool")
def tool():
    pass
# different
"#;

    let tools3 =
        scanner.parse_content(content2, "/virtual/path/scripts/tool.py", "test", &[], &[])?;

    assert_ne!(tools1[0].file_hash, tools3[0].file_hash);

    Ok(())
}
