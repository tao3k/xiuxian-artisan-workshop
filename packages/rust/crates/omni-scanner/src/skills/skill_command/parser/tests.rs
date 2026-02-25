use super::*;

#[test]
fn test_parse_decorator_simple() {
    let decorator = r#"@skill_command(name="test", description="A test")"#;
    let args = parse_decorator_args(decorator);
    assert_eq!(args.name, Some("test".to_string()));
    assert_eq!(args.description, Some("A test".to_string()));
}

#[test]
fn test_parse_decorator_triple_quote() {
    let decorator = r#"@skill_command(name="test", description="""A multi-line
description""")"#;
    let args = parse_decorator_args(decorator);
    assert_eq!(args.name, Some("test".to_string()));
    assert!(args.description.unwrap().contains("multi-line"));
}

#[test]
fn test_extract_docstring() {
    let text = r#"def test():
    """This is a docstring."""
    pass"#;
    let doc = extract_docstring_from_text(text);
    assert_eq!(doc, "This is a docstring.");
}

#[test]
fn test_parse_parameters() {
    let params = parse_parameters("a: str, b: int, c = None");
    assert_eq!(params, vec!["a", "b", "c"]);
}

#[test]
fn test_parse_parameters_with_types() {
    let params = parse_parameters("content: str, mode: WriteMode = WriteMode::default()");
    assert_eq!(params, vec!["content", "mode"]);
}

#[test]
fn test_extract_parameters_from_text() {
    let text = "def test(a: str, b: int) -> str: pass";
    let params = extract_parameters_from_text(text);
    assert_eq!(params, vec!["a", "b"]);
}

#[test]
fn test_extract_param_descriptions() {
    let description = r#"
Tool description.

Args:
    - query: str - The search query to use (required)
    - limit: int - Maximum number of results (optional)
    - session_id: str - Optional session ID

Returns:
    List of results
"#;
    let params = extract_param_descriptions(description);

    assert_eq!(
        params.get("query"),
        Some(&"The search query to use (required)".to_string())
    );
    assert_eq!(
        params.get("limit"),
        Some(&"Maximum number of results (optional)".to_string())
    );
    assert_eq!(
        params.get("session_id"),
        Some(&"Optional session ID".to_string())
    );
    assert!(!params.contains_key("Returns")); // Not a parameter
}

#[test]
fn test_extract_param_descriptions_no_args() {
    let description = "Tool description without Args section.";
    let params = extract_param_descriptions(description);
    assert!(params.is_empty());
}

#[test]
fn test_find_skill_command_decorators() {
    let content = r#"
@skill_command(name="test1")
def foo():
    pass

@skill_command(name="test2")
def bar():
    pass
"#;
    let decs = find_skill_command_decorators(content);
    assert_eq!(decs.len(), 2);
}

#[test]
fn test_extract_parameters_with_nested_parens() {
    // Test with type annotations containing parentheses like str | None
    let text = r#"def save_memory(content: str | None, metadata: dict[str, Any] | None) -> bool:"#;
    let params = extract_parameters_from_text(text);
    assert_eq!(params, vec!["content", "metadata"]);
}

#[test]
fn test_extract_param_descriptions_real_decorator_format() {
    // This is the actual format from commit.py decorator
    let description = r#"
    Commit staged changes with a message.

    Args:
        - message: str - The commit message for the changes (required)

    Returns:
        Success or failure message with commit hash.
    "#;

    let params = extract_param_descriptions(description);

    println!("Extracted params: {:?}", params);

    assert_eq!(
        params.get("message"),
        Some(&"The commit message for the changes (required)".to_string())
    );
}

#[test]
fn test_parse_decorator_skill_discover_real() {
    // This tests the actual skill.discover decorator from assets/skills/skill/scripts/discovery.py
    // The decorator has a complex multi-line description with commas
    let decorator = r#"@skill_command(
    name="discover",
    category="system",
    description="""
    [CRITICAL] Capability Discovery & Intent Resolver - The Agent's PRIMARY Entry Point.

    MANDATORY WORKFLOW: This tool is the EXCLUSIVE gateway for solving any task. It maps high-level natural language goals to specific, executable @omni commands.

    CORE RESPONSIBILITIES:
    1. INTENT MAPPING: Converts vague requests (e.g., "debug network", "optimize rust") into concrete tool sequences.
    2. GLOBAL REGISTRY ACCESS: Searches the entire Skill Registry (Active + Inactive). If a tool is found but not loaded, it provides `jit_install` instructions.
    3. SYNTAX ENFORCEMENT: Resolves the EXACT @omni(...) invocation template. Direct @omni calls are FORBIDDEN without first retrieving the template from discovery.
    4. ARCHITECTURAL ORIENTATION: Use this at the START of every session or new sub-task to identify available "superpowers" before planning.

    WHEN TO USE:
    - To find out *how* to perform a task (e.g., "how to analyze a pcap").
    - To check if a specific capability (e.g., "image processing") exists.
    - To get the correct parameter schema for a tool.
    - Whenever you encounter a new domain you haven't worked with in the current session.

    Args:
        - intent: str - The natural language goal or action (required).
        - limit: int = 5 - Max results to return (increase for complex/ambiguous tasks).

    Returns:
        A structured map containing:
        - 'quick_guide': Direct usage templates to copy and paste.
        - 'details': Full metadata, descriptions, and scores for each tool.
    """,
)"#;

    let args = parse_decorator_args(decorator);
    assert_eq!(args.name, Some("discover".to_string()));
    assert_eq!(args.category, Some("system".to_string()));
    assert!(args.description.is_some(), "description should be Some");
    let desc = args.description.unwrap();
    assert!(
        desc.contains("CRITICAL"),
        "description should contain CRITICAL"
    );
    assert!(
        desc.contains("INTENT MAPPING"),
        "description should contain INTENT MAPPING"
    );
    assert!(
        desc.contains("CORE RESPONSIBILITIES"),
        "description should contain CORE RESPONSIBILITIES"
    );
}
