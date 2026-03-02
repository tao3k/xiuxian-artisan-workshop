//! Tests for `TreeSitter` Python Parser.
//!
//! Tests for decorator extraction including @`skill_command`, @`skill_resource`,
//! and @`prompt`.

use omni_ast::TreeSitterPythonParser;

fn some_ref_or_panic<'a, T>(value: Option<&'a T>, label: &str) -> &'a T {
    match value {
        Some(value) => value,
        None => panic!("expected {label} to be present"),
    }
}

#[test]
fn test_find_decorated_functions_skill_command() {
    let mut parser = TreeSitterPythonParser::new();
    let code = r#"
@skill_command(name="test_tool", description="A test tool")
def test_tool():
    '''This is a test tool.'''
    pass

@skill_command(name="another_tool")
def another_tool():
    pass
"#;

    let funcs = parser.find_decorated_functions(code, "skill_command");
    assert_eq!(funcs.len(), 2);
    assert_eq!(funcs[0].name, "test_tool");
    assert_eq!(funcs[1].name, "another_tool");
}

#[test]
fn test_find_decorated_functions_skill_resource() {
    let mut parser = TreeSitterPythonParser::new();
    let code = r#"
@skill_resource(
    name="status",
    description="Get system status",
    resource_uri="omni://skill/test/status"
)
def status_resource():
    '''Returns system status.'''
    return {"status": "ok"}
"#;

    let funcs = parser.find_decorated_functions(code, "skill_resource");
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "status_resource");

    let decorator = some_ref_or_panic(funcs[0].decorator.as_ref(), "decorator");
    let args = &decorator.arguments;
    assert_eq!(
        some_ref_or_panic(args.name.as_ref(), "decorator name"),
        "status"
    );
    assert_eq!(
        some_ref_or_panic(args.resource_uri.as_ref(), "resource_uri"),
        "omni://skill/test/status"
    );
}

#[test]
fn test_find_decorated_functions_prompt() {
    let mut parser = TreeSitterPythonParser::new();
    let code = r#"
@prompt(
    name="analyze_code",
    description="Analyze code structure"
)
def analyze_code(file_path: str):
    '''Analyze the given code file.'''
    return f"Please analyze {file_path}"
"#;

    let funcs = parser.find_decorated_functions(code, "prompt");
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "analyze_code");

    let decorator = some_ref_or_panic(funcs[0].decorator.as_ref(), "decorator");
    let args = &decorator.arguments;
    assert_eq!(
        some_ref_or_panic(args.name.as_ref(), "decorator name"),
        "analyze_code"
    );
}

#[test]
fn test_find_decorated_functions_any() {
    let mut parser = TreeSitterPythonParser::new();
    let code = r#"
@skill_command(name="cmd_tool")
def cmd_tool():
    pass

@skill_resource(name="res_tool", resource_uri="omni://skill/test/res")
def res_tool():
    pass

@prompt(name="prompt_tool")
def prompt_tool():
    pass

def regular_function():
    pass
"#;

    // Find all three decorator types at once
    let funcs =
        parser.find_decorated_functions_any(code, &["skill_command", "skill_resource", "prompt"]);
    assert_eq!(funcs.len(), 3);
    assert_eq!(funcs[0].name, "cmd_tool");
    assert_eq!(funcs[1].name, "res_tool");
    assert_eq!(funcs[2].name, "prompt_tool");

    // Check decorator names
    assert_eq!(
        some_ref_or_panic(funcs[0].decorator.as_ref(), "skill_command decorator").name,
        "skill_command"
    );
    assert_eq!(
        some_ref_or_panic(funcs[1].decorator.as_ref(), "skill_resource decorator").name,
        "skill_resource"
    );
    assert_eq!(
        some_ref_or_panic(funcs[2].decorator.as_ref(), "prompt decorator").name,
        "prompt"
    );
}

#[test]
fn test_find_decorated_functions_any_only_matches() {
    let mut parser = TreeSitterPythonParser::new();
    let code = r#"
@skill_command(name="cmd_tool")
def cmd_tool():
    pass

@unknown_decorator(name="unknown")
def unknown_tool():
    pass
"#;

    // Should only find skill_command, not unknown_decorator
    let funcs =
        parser.find_decorated_functions_any(code, &["skill_command", "skill_resource", "prompt"]);
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "cmd_tool");
}

#[test]
fn test_find_decorated_functions_empty() {
    let mut parser = TreeSitterPythonParser::new();
    let code = r"
def regular_function():
    pass
";

    let funcs = parser.find_decorated_functions(code, "skill_command");
    assert!(funcs.is_empty());
}

#[test]
fn test_skill_resource_decorator_args() {
    let mut parser = TreeSitterPythonParser::new();
    let code = r#"
@skill_resource(
    name="stats",
    description="Get statistics",
    resource_uri="omni://skill/myapp/stats"
)
def get_stats():
    return {}
"#;

    let funcs = parser.find_decorated_functions(code, "skill_resource");
    assert_eq!(funcs.len(), 1);

    let decorator = some_ref_or_panic(funcs[0].decorator.as_ref(), "decorator");
    assert_eq!(decorator.name, "skill_resource");

    let args = &decorator.arguments;
    assert_eq!(
        some_ref_or_panic(args.name.as_ref(), "decorator name"),
        "stats"
    );
    assert_eq!(
        some_ref_or_panic(args.description.as_ref(), "decorator description"),
        "Get statistics"
    );
    assert_eq!(
        some_ref_or_panic(args.resource_uri.as_ref(), "resource_uri"),
        "omni://skill/myapp/stats"
    );
}

#[test]
fn test_prompt_decorator_args() {
    let mut parser = TreeSitterPythonParser::new();
    let code = r#"
@prompt(
    name="code_review",
    description="Generate code review"
)
def review_code(file_path: str, style: str = "google"):
    '''Provide a detailed code review.'''
    return f"Reviewing {file_path}"
"#;

    let funcs = parser.find_decorated_functions(code, "prompt");
    assert_eq!(funcs.len(), 1);

    let decorator = some_ref_or_panic(funcs[0].decorator.as_ref(), "decorator");
    assert_eq!(decorator.name, "prompt");

    let args = &decorator.arguments;
    assert_eq!(
        some_ref_or_panic(args.name.as_ref(), "decorator name"),
        "code_review"
    );
    assert_eq!(
        some_ref_or_panic(args.description.as_ref(), "decorator description"),
        "Generate code review"
    );
}

#[test]
fn test_decorator_without_args() {
    let mut parser = TreeSitterPythonParser::new();
    let code = r"
@skill_command
def simple_tool():
    pass
";

    let funcs = parser.find_decorated_functions(code, "skill_command");
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "simple_tool");

    // Should still have decorator with default/empty args
    let decorator = some_ref_or_panic(funcs[0].decorator.as_ref(), "decorator");
    assert_eq!(decorator.name, "skill_command");
    assert!(decorator.arguments.name.is_none());
}

#[test]
fn test_multiple_decorators_same_function() {
    let mut parser = TreeSitterPythonParser::new();
    let code = r#"
@decorator_a
@skill_command(name="main_tool")
@decorator_b
def complex_tool():
    pass
"#;

    // Should still find the skill_command
    let funcs = parser.find_decorated_functions(code, "skill_command");
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "complex_tool");
}
