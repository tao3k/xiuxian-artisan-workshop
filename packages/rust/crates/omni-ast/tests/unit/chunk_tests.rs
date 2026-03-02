use super::*;

fn chunk_or_panic(
    content: &str,
    path: &str,
    lang: Lang,
    patterns: &[&str],
    min_lines: usize,
    max_lines: usize,
) -> Vec<CodeChunk> {
    match chunk_code(content, path, lang, patterns, min_lines, max_lines) {
        Ok(chunks) => chunks,
        Err(error) => panic!("chunk_code failed: {error}"),
    }
}

#[test]
fn test_chunk_python_functions() {
    let content = r#"
def hello(name: str) -> str:
    """Greet someone."""
    return f"Hello, {name}!"

def goodbye():
    """Say goodbye."""
    pass

class Greeter:
    """A greeting class."""
    def __init__(self, name: str):
        self.name = name
"#;

    let chunks = chunk_or_panic(
        content,
        "test.py",
        Lang::Python,
        &["def $NAME", "class $NAME"],
        2, // min_lines=2 to exclude 1-line functions
        0,
    );

    // Should find: hello (4 lines), goodbye (3 lines), Greeter (5 lines)
    assert_eq!(chunks.len(), 3);

    // Check that we found functions and classes
    let funcs: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == "function")
        .collect();
    let classes: Vec<_> = chunks.iter().filter(|c| c.chunk_type == "class").collect();
    assert_eq!(funcs.len(), 2);
    assert_eq!(classes.len(), 1);

    // Check docstrings
    let Some(docstring) = funcs[0].docstring.as_ref() else {
        panic!("expected hello docstring to be present");
    };
    assert_eq!(docstring, "Greet someone.");
}

#[test]
fn test_chunk_id_generation() {
    let content = r"
def my_function():
    pass
";

    let chunks = chunk_or_panic(content, "test.py", Lang::Python, &["def $NAME"], 1, 0);

    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].id.contains("my_function"));
    // file_stem gives "test" not "test.py"
    assert!(chunks[0].id.contains("test"));
    assert!(chunks[0].id.contains("function"));
}

#[test]
fn test_min_lines_filter() {
    let content = r"
def short():
    x = 1
def normal():
    x = 1
    y = 2
    z = 3
";

    let chunks = chunk_or_panic(content, "test.py", Lang::Python, &["def $NAME"], 3, 0);

    // Only the normal function (4 lines) should be included
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].content.contains("normal"));
}

#[test]
fn test_max_lines_split() {
    // Create a single large function that spans 25 lines
    let mut lines: Vec<String> = vec!["def large_function():".to_string()];
    for i in 0..24 {
        lines.push(format!("    x_{i} = {i}"));
    }
    let content = lines.join("\n");

    let chunks = chunk_or_panic(&content, "test.py", Lang::Python, &["def $NAME"], 1, 10);

    // 25 lines should be split into 3 chunks
    assert_eq!(chunks.len(), 3);
    for (i, chunk) in chunks.iter().enumerate() {
        assert!(chunk.id.contains("large_function"));
        assert!(chunk.id.contains(&format!("_part_{i}")));
    }
}

#[test]
fn test_chunk_rust_functions() {
    let content = r#"
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn goodbye() {
    println!("Goodbye");
}

struct Greeter {
    name: String,
}

impl Greeter {
    fn new(name: String) -> Self {
        Self { name }
    }
}
"#;

    let chunks = chunk_or_panic(
        content,
        "lib.rs",
        Lang::Rust,
        &["fn $NAME", "struct $NAME"],
        1,
        0,
    );

    // Should find: hello, goodbye, Greeter, new
    assert_eq!(chunks.len(), 4);

    // Check chunk types
    let funcs: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == "function")
        .collect();
    let structs: Vec<_> = chunks.iter().filter(|c| c.chunk_type == "struct").collect();
    assert_eq!(funcs.len(), 3);
    assert_eq!(structs.len(), 1);
}

#[test]
fn test_chunk_javascript_functions() {
    let content = r#"
function hello(name) {
    return `Hello, ${name}!`;
}

const goodbye = () => {
    console.log("Goodbye");
};

class Greeter {
    constructor(name) {
        this.name = name;
    }
}
"#;

    let chunks = chunk_or_panic(
        content,
        "app.js",
        Lang::JavaScript,
        &["function $NAME", "const $NAME"],
        1,
        0,
    );

    assert_eq!(chunks.len(), 2);
}

#[test]
fn test_chunk_python_async_functions() {
    let content = r#"
async def fetch_data(url: str) -> dict:
    """Fetch data from URL."""
    response = await http_get(url)
    return response.json()

async def process_items():
    """Process all items concurrently."""
    results = []
    for item in items:
        result = await process(item)
        results.append(result)
    return results
"#;

    let chunks = chunk_or_panic(content, "api.py", Lang::Python, &["async def $NAME"], 1, 0);

    assert_eq!(chunks.len(), 2);
    assert!(chunks[0].content.contains("fetch_data"));
    assert!(chunks[1].content.contains("process_items"));
}

#[test]
fn test_chunk_empty_content() {
    let chunks = chunk_or_panic("", "empty.py", Lang::Python, &["def $NAME"], 1, 0);
    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_chunk_no_matches() {
    let content = r"
x = 1
y = 2
";

    let chunks = chunk_or_panic(
        content,
        "test.py",
        Lang::Python,
        &["def $NAME", "class $NAME"],
        1,
        0,
    );
    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_chunk_preserves_order() {
    let content = r"
class First:
    pass

def second():
    pass

class Third:
    pass

def fourth():
    pass
";

    let chunks = chunk_or_panic(
        content,
        "test.py",
        Lang::Python,
        &["def $NAME", "class $NAME"],
        1,
        0,
    );

    assert_eq!(chunks.len(), 4);

    // Verify order is preserved
    assert!(chunks[0].id.contains("First"));
    assert!(chunks[1].id.contains("second"));
    assert!(chunks[2].id.contains("Third"));
    assert!(chunks[3].id.contains("fourth"));
}

#[test]
fn test_chunk_metadata_extraction() {
    let content = r#"
def process_user_data(user_id: int, name: str, email: str) -> bool:
    """Process user data."""
    return True
"#;

    let chunks = chunk_or_panic(content, "test.py", Lang::Python, &["def $NAME"], 1, 0);

    assert_eq!(chunks.len(), 1);
    let chunk = &chunks[0];

    // Check metadata contains NAME capture
    assert!(chunk.metadata.contains_key("NAME"));
    assert_eq!(chunk.metadata["NAME"], "process_user_data");
}

#[test]
fn test_chunk_with_single_quoted_docstring() {
    let content = r"
def hello():
    '''Single quoted docstring.'''
    pass
";

    let chunks = chunk_or_panic(content, "test.py", Lang::Python, &["def $NAME"], 1, 0);

    assert_eq!(chunks.len(), 1);
    assert_eq!(
        chunks[0].docstring,
        Some("Single quoted docstring.".to_string())
    );
}

#[test]
fn test_chunk_multiple_patterns_same_file() {
    let content = r"
def foo():
    pass

class Bar:
    pass

def baz():
    pass
";

    // Match both functions and classes in a single call
    let chunks = chunk_or_panic(
        content,
        "test.py",
        Lang::Python,
        &["def $NAME", "class $NAME"],
        1,
        0,
    );

    assert_eq!(chunks.len(), 3);
}

#[test]
fn test_chunk_line_numbers_correct() {
    let content = r"
def first():
    line 2

def second():
    line 5
";

    let chunks = chunk_or_panic(content, "test.py", Lang::Python, &["def $NAME"], 1, 0);

    assert_eq!(chunks.len(), 2);

    // Check line numbers are correct (1-indexed)
    assert_eq!(chunks[0].line_start, 2);
    assert_eq!(chunks[0].line_end, 3);

    assert_eq!(chunks[1].line_start, 5);
    assert_eq!(chunks[1].line_end, 6);
}
