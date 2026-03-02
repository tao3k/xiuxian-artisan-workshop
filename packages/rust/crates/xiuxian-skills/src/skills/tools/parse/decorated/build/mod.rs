use std::collections::HashMap;

use omni_ast::DecoratedFunction;

use crate::skills::metadata::{ToolEnrichment, ToolRecord};
use crate::skills::skill_command::annotations::build_annotations;

mod convert;
mod resolve;

use convert::{build_parameters, parameter_names, to_decorator_args};
use resolve::{
    merged_keywords, resolve_tool_category, resolve_tool_description, resolve_tool_name,
};

use super::super::super::schema::generate_input_schema;

pub(super) fn build_tool_record(
    function: &DecoratedFunction,
    file_path: &str,
    skill_name: &str,
    skill_keywords: &[String],
    skill_intents: &[String],
    file_hash: &str,
    docstrings: &HashMap<String, String>,
) -> ToolRecord {
    let decorator_arguments = function
        .decorator
        .as_ref()
        .map(|decorator| &decorator.arguments);
    let tool_name = resolve_tool_name(decorator_arguments, function);
    let docstring = docstrings.get(&function.name).cloned().unwrap_or_default();
    let description = resolve_tool_description(
        decorator_arguments,
        docstring.as_str(),
        skill_name,
        &tool_name,
    );
    let category = resolve_tool_category(decorator_arguments, skill_name);
    let parameters = build_parameters(function);
    let resource_uri = decorator_arguments.and_then(|arguments| arguments.resource_uri.clone());
    let decorator_args = to_decorator_args(decorator_arguments);

    let annotations = build_annotations(&decorator_args, &function.name, &parameters);
    let input_schema = generate_input_schema(&parameters, &description);
    let keywords = merged_keywords(skill_name, &tool_name, skill_keywords);

    let enrichment = ToolEnrichment {
        execution_mode: "script".to_string(),
        keywords,
        intents: skill_intents.to_vec(),
        file_hash: file_hash.to_string(),
        docstring,
        category,
        annotations,
        parameters: parameter_names(&parameters),
        input_schema,
        skill_tools_refers: Vec::new(),
        resource_uri: resource_uri.unwrap_or_default(),
    };

    ToolRecord::with_enrichment(
        format!("{skill_name}.{tool_name}"),
        description,
        skill_name.to_string(),
        file_path.to_string(),
        function.name.clone(),
        enrichment,
    )
}
