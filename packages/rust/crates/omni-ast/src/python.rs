//! Python-specific AST utilities.
//!
//! Provides functions for Python-specific pattern matching and extraction.

use crate::item::Match;
use crate::lang::Lang;
use crate::scan::scan;

/// Find all function definitions in Python code
#[must_use]
pub fn find_python_functions(content: &str) -> Vec<Match> {
    // Note: Using simpler pattern because ast-grep has issues with complex body matching
    scan(content, "def $NAME", Lang::Python).unwrap_or_default()
}

/// Find all async function definitions in Python code
#[must_use]
pub fn find_python_async_functions(content: &str) -> Vec<Match> {
    scan(content, "async def $NAME", Lang::Python).unwrap_or_default()
}

/// Find all class definitions in Python code
#[must_use]
pub fn find_python_classes(content: &str) -> Vec<Match> {
    scan(content, "class $NAME", Lang::Python).unwrap_or_default()
}

/// Find all decorated functions in Python code
#[must_use]
pub fn find_python_decorated_functions(content: &str, decorator: &str) -> Vec<Match> {
    let pattern = format!("@{decorator}($A)\ndef $NAME($ARGS): $BODY");
    scan(content, &pattern, Lang::Python).unwrap_or_default()
}

/// Find all function definitions with a specific decorator
#[must_use]
pub fn find_python_decorated_by_any(content: &str) -> Vec<Match> {
    scan(content, "@$DECORATOR\ndef $NAME($ARGS)", Lang::Python).unwrap_or_default()
}

/// Extract docstring from Python function body
#[must_use]
pub fn extract_python_docstring(body: &str) -> String {
    if let Some(start) = body.find("\"\"\"")
        && let Some(end) = body[start + 3..].find("\"\"\"")
    {
        return body[start + 3..start + 3 + end].trim().to_string();
    }
    if let Some(start) = body.find("'''")
        && let Some(end) = body[start + 3..].find("'''")
    {
        return body[start + 3..start + 3 + end].trim().to_string();
    }
    String::new()
}

/// Extract docstring from a match
#[must_use]
pub fn extract_docstring_from_match(m: &Match) -> String {
    extract_python_docstring(&m.text)
}
