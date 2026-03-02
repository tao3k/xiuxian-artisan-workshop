use std::collections::HashMap;

use omni_ast::DecoratedFunction;

pub(super) fn collect_docstrings(decorated_funcs: &[DecoratedFunction]) -> HashMap<String, String> {
    let mut docstrings = HashMap::new();
    for function in decorated_funcs {
        if !function.docstring.is_empty() {
            docstrings.insert(function.name.clone(), function.docstring.clone());
        }
    }
    docstrings
}
