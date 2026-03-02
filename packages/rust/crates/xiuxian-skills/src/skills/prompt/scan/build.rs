use omni_ast::{DecoratedFunction, TreeSitterPythonParser};

use crate::skills::metadata::PromptRecord;

pub(super) fn build_prompt_records(
    content: &str,
    file_path: &str,
    skill_name: &str,
    file_hash: &str,
) -> Vec<PromptRecord> {
    let mut parser = TreeSitterPythonParser::new();
    let decorated_funcs: Vec<DecoratedFunction> =
        parser.find_decorated_functions(content, "prompt");

    let mut prompts = Vec::new();
    for func in &decorated_funcs {
        let decorator_args = func
            .decorator
            .as_ref()
            .map(|decorator| &decorator.arguments);
        let name = decorator_args
            .and_then(|arguments| arguments.name.clone())
            .unwrap_or_else(|| func.name.clone());

        let description = decorator_args
            .and_then(|arguments| arguments.description.clone())
            .or_else(|| {
                if func.docstring.is_empty() {
                    None
                } else {
                    Some(func.docstring.clone())
                }
            })
            .unwrap_or_else(|| format!("Prompt {skill_name}.{name}"));

        let parameters: Vec<String> = func
            .parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect();

        prompts.push(PromptRecord::new(
            name,
            description,
            skill_name.to_string(),
            file_path.to_string(),
            func.name.clone(),
            file_hash.to_string(),
            parameters,
        ));
    }

    prompts
}
