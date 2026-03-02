use omni_ast::{DecoratedFunction, DecoratorArguments};

use crate::skills::metadata::DecoratorArgs;
use crate::skills::skill_command::parser::ParsedParameter;

pub(super) fn build_parameters(function: &DecoratedFunction) -> Vec<ParsedParameter> {
    function
        .parameters
        .iter()
        .map(|parameter| ParsedParameter {
            name: parameter.name.clone(),
            type_annotation: parameter.type_annotation.clone(),
            has_default: parameter.default_value.is_some(),
            default_value: parameter.default_value.clone(),
        })
        .collect()
}

pub(super) fn parameter_names(parameters: &[ParsedParameter]) -> Vec<String> {
    parameters
        .iter()
        .map(|parameter| parameter.name.clone())
        .collect()
}

pub(super) fn to_decorator_args(arguments: Option<&DecoratorArguments>) -> DecoratorArgs {
    match arguments {
        Some(arguments) => DecoratorArgs {
            name: arguments.name.clone(),
            description: arguments.description.clone(),
            category: arguments.category.clone(),
            destructive: arguments.destructive,
            read_only: arguments.read_only,
            resource_uri: arguments.resource_uri.clone(),
        },
        None => DecoratorArgs::default(),
    }
}
