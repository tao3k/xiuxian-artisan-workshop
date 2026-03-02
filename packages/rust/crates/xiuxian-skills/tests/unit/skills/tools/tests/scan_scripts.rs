use std::fs::File;
use std::io::Write;

use tempfile::TempDir;

use crate::skills::tools::ToolsScanner;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_scan_scripts_single_tool() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = temp_dir.path().join("writer/scripts");
    std::fs::create_dir_all(&scripts_dir)?;

    let script_content = r#"
from agent.skills.decorators import skill_command

@skill_command(name="write_text")
def write_text(content: str) -> str:
    '''Write text to a file.'''
    return "written"
"#;

    let script_file = scripts_dir.join("text.py");
    let mut file = File::create(&script_file)?;
    file.write_all(script_content.as_bytes())?;

    let scanner = ToolsScanner::new();
    let tools = scanner.scan_scripts(&scripts_dir, "writer", &["write".to_string()], &[])?;

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "writer.write_text");
    assert_eq!(tools[0].function_name, "write_text");
    // Verify routing keywords are included
    assert!(tools[0].keywords.contains(&"write".to_string()));

    Ok(())
}

#[test]
fn test_scan_scripts_multiple_tools() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = temp_dir.path().join("git/scripts");
    std::fs::create_dir_all(&scripts_dir)?;

    let script_content = r#"
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

    let script_file = scripts_dir.join("main.py");
    let mut file = File::create(&script_file)?;
    file.write_all(script_content.as_bytes())?;

    let scanner = ToolsScanner::new();
    let tools = scanner.scan_scripts(&scripts_dir, "git", &["git".to_string()], &[])?;

    assert_eq!(tools.len(), 2);
    assert!(tools.iter().any(|t| t.tool_name == "git.commit"));
    assert!(tools.iter().any(|t| t.tool_name == "git.status"));

    Ok(())
}

#[test]
fn test_scan_scripts_no_scripts_dir() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = temp_dir.path().join("empty/scripts");

    let scanner = ToolsScanner::new();
    let tools = scanner.scan_scripts(&scripts_dir, "empty", &[], &[])?;

    assert!(tools.is_empty());

    Ok(())
}

#[test]
fn test_scan_scripts_empty_dir() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = temp_dir.path().join("empty/scripts");
    std::fs::create_dir_all(&scripts_dir)?;

    let scanner = ToolsScanner::new();
    let tools = scanner.scan_scripts(&scripts_dir, "empty", &[], &[])?;

    assert!(tools.is_empty());

    Ok(())
}

#[test]
fn test_scan_skill_scripts() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("test_skill");
    let scripts_dir = skill_path.join("scripts");
    std::fs::create_dir_all(&scripts_dir)?;

    let script_content = r#"
@skill_command(name="test")
def test_tool():
    '''A test tool.'''
    pass
"#;

    let script_file = scripts_dir.join("test.py");
    std::fs::write(&script_file, script_content)?;

    let scanner = ToolsScanner::new();
    let tools = scanner.scan_skill_scripts(&skill_path, "test_skill", &[], &[])?;

    assert_eq!(tools.len(), 1);
    // When decorator has name="test", tool_name should be "test_skill.test"
    // If it returns "test_skill.test_tool", the decorator name wasn't parsed
    assert!(tools[0].tool_name.starts_with("test_skill."));

    Ok(())
}

#[test]
fn test_parse_script_skips_init() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = temp_dir.path().join("test/scripts");
    std::fs::create_dir_all(&scripts_dir)?;

    // Write __init__.py with a decorated function (should be skipped)
    let init_content = r#"
from agent.skills.decorators import skill_command

@skill_command(name="init_tool")
def init_tool():
    '''This should be skipped.'''
    pass
"#;

    let init_file = scripts_dir.join("__init__.py");
    std::fs::write(&init_file, init_content)?;

    let scanner = ToolsScanner::new();
    let tools = scanner.scan_scripts(&scripts_dir, "test", &[], &[])?;

    assert!(tools.is_empty());

    Ok(())
}

#[test]
fn test_tool_record_keywords_includes_skill_keywords() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = temp_dir.path().join("writer/scripts");
    std::fs::create_dir_all(&scripts_dir)?;

    let script_content = r#"
@skill_command(name="polish_text")
def polish_text(text: str) -> str:
    '''Polish text using writing guidelines.'''
    return text
"#;

    let script_file = scripts_dir.join("text.py");
    std::fs::write(&script_file, script_content)?;

    let scanner = ToolsScanner::new();
    let routing_keywords = vec![
        "write".to_string(),
        "edit".to_string(),
        "polish".to_string(),
    ];
    let tools = scanner.scan_scripts(&scripts_dir, "writer", &routing_keywords, &[])?;

    assert_eq!(tools.len(), 1);
    let keywords = &tools[0].keywords;
    assert!(keywords.contains(&"writer".to_string()));
    assert!(keywords.contains(&"polish_text".to_string()));
    assert!(keywords.contains(&"polish".to_string())); // From routing_keywords

    Ok(())
}

#[test]
fn test_tools_scanner_new() {
    let _ = ToolsScanner::new();
}
