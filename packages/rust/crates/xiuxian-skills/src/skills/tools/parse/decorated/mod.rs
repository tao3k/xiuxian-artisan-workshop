use omni_ast::{DecoratedFunction, TreeSitterPythonParser};

use crate::skills::metadata::ToolRecord;

use super::hashing::content_sha256;

mod build;
mod collect;

use build::build_tool_record;
use collect::collect_docstrings;

pub(super) fn parse_decorated_tools(
    content: &str,
    file_path: &str,
    skill_name: &str,
    skill_keywords: &[String],
    skill_intents: &[String],
) -> Vec<ToolRecord> {
    let mut parser = TreeSitterPythonParser::new();
    let decorated_funcs: Vec<DecoratedFunction> =
        parser.find_decorated_functions(content, "skill_command");

    if !decorated_funcs.is_empty() {
        log::debug!(
            "ToolsScanner: Found {} @skill_command decorated functions in {}",
            decorated_funcs.len(),
            file_path
        );
    }

    let file_hash = content_sha256(content);
    let docstrings = collect_docstrings(&decorated_funcs);

    decorated_funcs
        .iter()
        .map(|function| {
            build_tool_record(
                function,
                file_path,
                skill_name,
                skill_keywords,
                skill_intents,
                file_hash.as_str(),
                &docstrings,
            )
        })
        .collect()
}
