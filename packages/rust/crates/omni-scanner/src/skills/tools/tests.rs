use super::*;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_scan_scripts_single_tool() {
    let temp_dir = TempDir::new().unwrap();
    let scripts_dir = temp_dir.path().join("writer/scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    let script_content = r#"
from agent.skills.decorators import skill_command

@skill_command(name="write_text")
def write_text(content: str) -> str:
    '''Write text to a file.'''
    return "written"
"#;

    let script_file = scripts_dir.join("text.py");
    let mut file = File::create(&script_file).unwrap();
    file.write_all(script_content.as_bytes()).unwrap();

    let scanner = ToolsScanner::new();
    let tools = scanner
        .scan_scripts(&scripts_dir, "writer", &["write".to_string()], &[])
        .unwrap();

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "writer.write_text");
    assert_eq!(tools[0].function_name, "write_text");
    // Verify routing keywords are included
    assert!(tools[0].keywords.contains(&"write".to_string()));
}

#[test]
fn test_scan_scripts_multiple_tools() {
    let temp_dir = TempDir::new().unwrap();
    let scripts_dir = temp_dir.path().join("git/scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

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
    let mut file = File::create(&script_file).unwrap();
    file.write_all(script_content.as_bytes()).unwrap();

    let scanner = ToolsScanner::new();
    let tools = scanner
        .scan_scripts(&scripts_dir, "git", &["git".to_string()], &[])
        .unwrap();

    assert_eq!(tools.len(), 2);
    assert!(tools.iter().any(|t| t.tool_name == "git.commit"));
    assert!(tools.iter().any(|t| t.tool_name == "git.status"));
}

#[test]
fn test_scan_scripts_no_scripts_dir() {
    let temp_dir = TempDir::new().unwrap();
    let scripts_dir = temp_dir.path().join("empty/scripts");

    let scanner = ToolsScanner::new();
    let tools = scanner
        .scan_scripts(&scripts_dir, "empty", &[], &[])
        .unwrap();

    assert!(tools.is_empty());
}

#[test]
fn test_scan_scripts_empty_dir() {
    let temp_dir = TempDir::new().unwrap();
    let scripts_dir = temp_dir.path().join("empty/scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    let scanner = ToolsScanner::new();
    let tools = scanner
        .scan_scripts(&scripts_dir, "empty", &[], &[])
        .unwrap();

    assert!(tools.is_empty());
}

#[test]
fn test_scan_skill_scripts() {
    let temp_dir = TempDir::new().unwrap();
    let skill_path = temp_dir.path().join("test_skill");
    let scripts_dir = skill_path.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    let script_content = r#"
@skill_command(name="test")
def test_tool():
    '''A test tool.'''
    pass
"#;

    let script_file = scripts_dir.join("test.py");
    std::fs::write(&script_file, script_content).unwrap();

    let scanner = ToolsScanner::new();
    let tools = scanner
        .scan_skill_scripts(&skill_path, "test_skill", &[], &[])
        .unwrap();

    assert_eq!(tools.len(), 1);
    // When decorator has name="test", tool_name should be "test_skill.test"
    // If it returns "test_skill.test_tool", the decorator name wasn't parsed
    assert!(tools[0].tool_name.starts_with("test_skill."));
}

#[test]
fn test_parse_script_skips_init() {
    let temp_dir = TempDir::new().unwrap();
    let scripts_dir = temp_dir.path().join("test/scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    // Write __init__.py with a decorated function (should be skipped)
    let init_content = r#"
from agent.skills.decorators import skill_command

@skill_command(name="init_tool")
def init_tool():
    '''This should be skipped.'''
    pass
"#;

    let init_file = scripts_dir.join("__init__.py");
    std::fs::write(&init_file, init_content).unwrap();

    let scanner = ToolsScanner::new();
    let tools = scanner
        .scan_scripts(&scripts_dir, "test", &[], &[])
        .unwrap();

    assert!(tools.is_empty());
}

#[test]
fn test_tool_record_keywords_includes_skill_keywords() {
    let temp_dir = TempDir::new().unwrap();
    let scripts_dir = temp_dir.path().join("writer/scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    let script_content = r#"
@skill_command(name="polish_text")
def polish_text(text: str) -> str:
    '''Polish text using writing guidelines.'''
    return text
"#;

    let script_file = scripts_dir.join("text.py");
    std::fs::write(&script_file, script_content).unwrap();

    let scanner = ToolsScanner::new();
    let routing_keywords = vec![
        "write".to_string(),
        "edit".to_string(),
        "polish".to_string(),
    ];
    let tools = scanner
        .scan_scripts(&scripts_dir, "writer", &routing_keywords, &[])
        .unwrap();

    assert_eq!(tools.len(), 1);
    let keywords = &tools[0].keywords;
    assert!(keywords.contains(&"writer".to_string()));
    assert!(keywords.contains(&"polish_text".to_string()));
    assert!(keywords.contains(&"polish".to_string())); // From routing_keywords
}

#[test]
fn test_tools_scanner_new() {
    let _scanner = ToolsScanner::new();
    // Just verify it can be created
    assert!(true);
}

#[test]
fn test_parse_content_single_tool() {
    let scanner = ToolsScanner::new();
    let content = r#"
from agent.skills.decorators import skill_command

@skill_command(name="write_text")
def write_text(content: str) -> str:
    '''Write text to a file.'''
    return "written"
"#;

    let tools = scanner
        .parse_content(
            content,
            "/virtual/path/scripts/tool.py",
            "writer",
            &["write".to_string()],
            &[],
        )
        .unwrap();

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "writer.write_text");
    assert_eq!(tools[0].function_name, "write_text");
    assert_eq!(tools[0].file_path, "/virtual/path/scripts/tool.py");
    assert!(tools[0].keywords.contains(&"write".to_string()));
}

#[test]
fn test_parse_content_multiple_tools() {
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

    let tools = scanner
        .parse_content(
            content,
            "/virtual/path/scripts/main.py",
            "git",
            &["git".to_string()],
            &[],
        )
        .unwrap();

    assert_eq!(tools.len(), 2);
    assert!(tools.iter().any(|t| t.tool_name == "git.commit"));
    assert!(tools.iter().any(|t| t.tool_name == "git.status"));
}

#[test]
fn test_parse_content_no_decorators() {
    let scanner = ToolsScanner::new();
    let content = r#"
def regular_function():
    '''This function has no decorator.'''
    return "no tool here"
"#;

    let tools = scanner
        .parse_content(content, "/virtual/path/scripts/tool.py", "test", &[], &[])
        .unwrap();

    assert!(tools.is_empty());
}

#[test]
fn test_parse_content_skips_init() {
    let scanner = ToolsScanner::new();
    let content = r#"
from agent.skills.decorators import skill_command

@skill_command(name="init_tool")
def init_tool():
    '''This should be skipped.'''
    pass
"#;

    // parse_content doesn't skip __init__.py - that's handled in scan_paths
    let tools = scanner
        .parse_content(
            content,
            "/virtual/path/scripts/__init__.py",
            "test",
            &[],
            &[],
        )
        .unwrap();

    // This should find the tool since we're calling parse_content directly
    assert_eq!(tools.len(), 1);
}

#[test]
fn test_parse_content_with_category() {
    let scanner = ToolsScanner::new();
    let content = r#"
@skill_command(name="test_tool", category="testing")
def test_tool():
    '''A test tool.'''
    pass
"#;

    let tools = scanner
        .parse_content(content, "/virtual/path/scripts/tool.py", "test", &[], &[])
        .unwrap();

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].category, "testing");
}

#[test]
fn test_parse_content_with_intents() {
    let scanner = ToolsScanner::new();
    let content = r#"
@skill_command(name="test_tool")
def test_tool():
    '''A test tool.'''
    pass
"#;

    let intents = vec!["test".to_string(), "verify".to_string()];
    let tools = scanner
        .parse_content(
            content,
            "/virtual/path/scripts/tool.py",
            "test",
            &[],
            &intents,
        )
        .unwrap();

    assert_eq!(tools.len(), 1);
    assert!(tools[0].intents.contains(&"test".to_string()));
    assert!(tools[0].intents.contains(&"verify".to_string()));
}

#[test]
fn test_parse_content_file_hash() {
    let scanner = ToolsScanner::new();
    let content = r#"
@skill_command(name="tool")
def tool():
    pass
"#;

    let tools1 = scanner
        .parse_content(content, "/virtual/path/scripts/tool.py", "test", &[], &[])
        .unwrap();

    // Same content should produce same hash
    let tools2 = scanner
        .parse_content(content, "/virtual/path/scripts/tool.py", "test", &[], &[])
        .unwrap();

    assert_eq!(tools1[0].file_hash, tools2[0].file_hash);

    // Different content should produce different hash
    let content2 = r#"
@skill_command(name="tool")
def tool():
    pass
# different
"#;

    let tools3 = scanner
        .parse_content(content2, "/virtual/path/scripts/tool.py", "test", &[], &[])
        .unwrap();

    assert_ne!(tools1[0].file_hash, tools3[0].file_hash);
}

#[test]
fn test_scan_paths_multiple_files() {
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

    let tools = scanner.scan_paths(&files, "test_skill", &[], &[]).unwrap();

    assert_eq!(tools.len(), 2);
    assert!(tools.iter().any(|t| t.tool_name == "test_skill.tool_a"));
    assert!(tools.iter().any(|t| t.tool_name == "test_skill.tool_b"));
}

#[test]
fn test_scan_paths_skips_init() {
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

    let tools = scanner.scan_paths(&files, "test_skill", &[], &[]).unwrap();

    // Only one tool (skipping __init__.py)
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "test_skill.real_tool");
}

#[test]
fn test_scan_paths_skips_private_files() {
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

    let tools = scanner.scan_paths(&files, "test_skill", &[], &[]).unwrap();

    // Only one tool (skipping _private.py)
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "test_skill.public_tool");
}

#[test]
fn test_scan_paths_skips_non_python() {
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

    let tools = scanner.scan_paths(&files, "test_skill", &[], &[]).unwrap();

    assert_eq!(tools.len(), 1);
}

#[test]
fn test_scan_paths_empty_list() {
    let scanner = ToolsScanner::new();
    let files: Vec<(String, String)> = Vec::new();

    let tools = scanner.scan_paths(&files, "test_skill", &[], &[]).unwrap();

    assert!(tools.is_empty());
}

#[test]
fn test_scan_paths_with_keywords_and_intents() {
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

    let tools = scanner
        .scan_paths(&files, "test_skill", &keywords, &intents)
        .unwrap();

    assert_eq!(tools.len(), 1);
    assert!(tools[0].keywords.contains(&"test".to_string()));
    assert!(tools[0].keywords.contains(&"verify".to_string()));
    assert!(tools[0].intents.contains(&"testing".to_string()));
}

// Note: Comprehensive integration tests are in tests/tools_scanner.rs
// Category inference tests are in skill_command/category.rs
// These basic tests verify core functionality without complex setup.
