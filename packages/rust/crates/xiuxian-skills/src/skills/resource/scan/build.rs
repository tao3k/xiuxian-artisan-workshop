use omni_ast::{DecoratedFunction, TreeSitterPythonParser};

use crate::skills::metadata::ResourceRecord;

pub(super) fn build_resource_records(
    content: &str,
    file_path: &str,
    skill_name: &str,
    file_hash: &str,
) -> Vec<ResourceRecord> {
    let mut parser = TreeSitterPythonParser::new();
    let decorated_funcs: Vec<DecoratedFunction> =
        parser.find_decorated_functions(content, "skill_resource");

    let mut resources = Vec::new();
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
            .unwrap_or_else(|| format!("Resource {skill_name}.{name}"));

        let resource_uri = decorator_args
            .and_then(|arguments| arguments.resource_uri.clone())
            .unwrap_or_else(|| format!("omni://skill/{skill_name}/{name}"));

        resources.push(ResourceRecord::new(
            name,
            description,
            resource_uri,
            skill_name.to_string(),
            file_path.to_string(),
            func.name.clone(),
            file_hash.to_string(),
        ));
    }

    resources
}
