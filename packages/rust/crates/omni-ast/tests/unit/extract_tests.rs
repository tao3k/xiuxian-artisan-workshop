use super::*;

#[test]
fn test_extract_python_functions() {
    let content = r#"
def hello(name: str) -> str:
    '''Greet someone.'''
    return f"Hello, {name}!"

def goodbye():
    pass
"#;

    let results = extract_items(content, "def $NAME", Lang::Python, None);
    // Should find 2 top-level functions (not method inside class)
    assert_eq!(results.len(), 2);

    // Check first function
    let hello = &results[0];
    assert!(hello.text.starts_with("def hello"));
    assert!(hello.captures.contains_key("NAME"));
    assert_eq!(hello.captures["NAME"], "hello");
    assert!(hello.line_start >= 2);
    assert!(hello.line_end >= hello.line_start);
}

#[test]
fn test_extract_with_capture_filter() {
    let content = r#"
def hello(name: str) -> str:
    return f"Hello, {name}!"

def goodbye():
    pass
"#;

    // Use simple pattern without ARGS to match both functions
    let results = extract_items(content, "def $NAME", Lang::Python, Some(vec!["NAME"]));
    assert_eq!(results.len(), 2);

    for r in &results {
        assert!(r.captures.contains_key("NAME"));
    }
}

#[test]
fn test_extract_rust_functions() {
    let content = r#"
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn goodbye() {
    println!("Goodbye");
}
"#;

    let results = extract_items(content, "fn $NAME", Lang::Rust, None);
    assert_eq!(results.len(), 2);

    let hello = &results[0];
    assert!(hello.text.starts_with("fn hello"));
    assert_eq!(hello.captures["NAME"], "hello");
}

#[test]
fn test_extract_classes() {
    let content = r"
class MyClass:
    def method(self):
        pass

class AnotherClass:
    pass
";

    let results = extract_items(content, "class $NAME", Lang::Python, None);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_extract_empty_result() {
    let content = "let x = 42;";
    let results = extract_items(content, "def $NAME", Lang::Python, None);
    assert!(results.is_empty());
}

#[test]
fn test_line_numbers() {
    let content = "x = 1\ny = 2\nz = 3\n";
    let results = extract_items(content, "$NAME = $VALUE", Lang::Python, Some(vec!["NAME"]));

    // Should find 3 matches
    assert_eq!(results.len(), 3);

    // Check line numbers (1-indexed)
    assert_eq!(results[0].line_start, 1);
    assert_eq!(results[1].line_start, 2);
    assert_eq!(results[2].line_start, 3);
}

#[test]
fn test_invalid_pattern() {
    let content = "def hello(): pass";
    let results = extract_items(content, "invalid[pattern", Lang::Python, None);
    assert!(results.is_empty());
}

#[test]
fn test_extract_variables() {
    let content = r#"
x = 1
y = 2
name = "hello"
"#;

    let results = extract_items(content, "$NAME = $VALUE", Lang::Python, None);
    assert_eq!(results.len(), 3);

    for r in &results {
        assert!(r.captures.contains_key("NAME"));
        assert!(r.captures.contains_key("VALUE"));
    }
}

#[test]
fn test_extract_skeleton_python() {
    let content = r#"
def hello(name: str) -> str:
    """Greet someone by name."""
    return f"Hello, {name}!"

def goodbye():
    """Say goodbye."""
    print("Goodbye")

class MyClass:
    """A sample class."""
    def method(self):
        """A method."""
        pass
"#;

    let skeleton = extract_skeleton(content, Lang::Python);

    // Should contain function/class names
    assert!(
        skeleton.contains("hello"),
        "Should contain 'hello' function"
    );
    assert!(
        skeleton.contains("goodbye"),
        "Should contain 'goodbye' function"
    );
    assert!(skeleton.contains("MyClass"), "Should contain 'MyClass'");

    // Should NOT contain implementation details
    assert!(
        !skeleton.contains("return f"),
        "Should not contain 'return f'"
    );
    assert!(!skeleton.contains("print("), "Should not contain 'print('");
    assert!(!skeleton.contains("pass"), "Should not contain 'pass'");

    // Each signature should be on its own line
    let lines: Vec<&str> = skeleton.lines().collect();
    assert!(lines.len() >= 3, "Should have at least 3 signatures");
}

#[test]
fn test_extract_skeleton_rust() {
    let content = r#"
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub struct User {
    id: u32,
    name: String,
}

impl User {
    pub fn new(id: u32, name: String) -> Self {
        User { id, name }
    }
}
"#;

    let skeleton = extract_skeleton(content, Lang::Rust);

    // Should contain signatures (truncated at '{')
    assert!(skeleton.contains("fn hello"), "Should contain 'fn hello'");
    assert!(
        skeleton.contains("pub struct User"),
        "Should contain 'pub struct User'"
    );
    assert!(skeleton.contains("impl User"), "Should contain 'impl User'");

    // Should NOT contain implementation
    assert!(
        !skeleton.contains("format!"),
        "Should not contain 'format!'"
    );
    assert!(
        !skeleton.contains("User { id, name }"),
        "Should not contain struct init"
    );
}

#[test]
fn test_get_skeleton_patterns() {
    let py_patterns = get_skeleton_patterns(Lang::Python);
    assert!(py_patterns.contains(&"def $NAME"));
    assert!(py_patterns.contains(&"class $NAME"));

    let rs_patterns = get_skeleton_patterns(Lang::Rust);
    assert!(rs_patterns.contains(&"fn $NAME"));
    assert!(rs_patterns.contains(&"pub fn $NAME"));
}
