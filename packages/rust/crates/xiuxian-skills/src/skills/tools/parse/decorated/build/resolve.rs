use omni_ast::{DecoratedFunction, DecoratorArguments};

use crate::skills::skill_command::category::infer_category_from_skill;

pub(super) fn resolve_tool_name(
    decorator_arguments: Option<&DecoratorArguments>,
    function: &DecoratedFunction,
) -> String {
    decorator_arguments
        .and_then(|arguments| arguments.name.clone())
        .unwrap_or_else(|| function.name.clone())
}

pub(super) fn resolve_tool_description(
    decorator_arguments: Option<&DecoratorArguments>,
    docstring: &str,
    skill_name: &str,
    tool_name: &str,
) -> String {
    match decorator_arguments.and_then(|arguments| arguments.description.clone()) {
        Some(description) => description,
        None if !docstring.is_empty() => docstring.to_string(),
        None => format!("Execute {skill_name}.{tool_name}"),
    }
}

pub(super) fn resolve_tool_category(
    decorator_arguments: Option<&DecoratorArguments>,
    skill_name: &str,
) -> String {
    decorator_arguments
        .and_then(|arguments| arguments.category.clone())
        .unwrap_or_else(|| infer_category_from_skill(skill_name))
}

pub(super) fn merged_keywords(
    skill_name: &str,
    tool_name: &str,
    skill_keywords: &[String],
) -> Vec<String> {
    let mut keywords = vec![skill_name.to_string(), tool_name.to_string()];
    keywords.extend(skill_keywords.iter().cloned());
    keywords
}
