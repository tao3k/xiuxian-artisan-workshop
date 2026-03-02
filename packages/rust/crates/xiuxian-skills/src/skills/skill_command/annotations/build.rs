use super::heuristics::{is_destructive_function, is_open_world_function, is_read_only_function};
use crate::skills::metadata::{DecoratorArgs, ToolAnnotations};
use crate::skills::skill_command::parser::ParsedParameter;

/// Build `ToolAnnotations` from decorator args and naming heuristics.
#[must_use]
pub fn build_annotations(
    args: &DecoratorArgs,
    func_name: &str,
    _parameters: &[ParsedParameter],
) -> ToolAnnotations {
    let mut annotations = ToolAnnotations::default();
    let name_lower = func_name.to_lowercase();

    apply_explicit_overrides(&mut annotations, args);

    if args.read_only.is_none() && is_read_only_function(name_lower.as_str()) {
        annotations.read_only = true;
        annotations.set_idempotent(true);
    }

    if args.destructive.is_none() && is_destructive_function(name_lower.as_str()) {
        annotations.destructive = true;
        annotations.set_idempotent(false);
    }

    if is_open_world_function(name_lower.as_str()) {
        annotations.set_open_world(true);
    }

    if annotations.destructive {
        annotations.set_idempotent(false);
    }

    annotations
}

fn apply_explicit_overrides(annotations: &mut ToolAnnotations, args: &DecoratorArgs) {
    if let Some(read_only) = args.read_only {
        annotations.read_only = read_only;
        if read_only {
            annotations.set_idempotent(true);
        }
    }

    if let Some(destructive) = args.destructive {
        annotations.destructive = destructive;
        if destructive {
            annotations.set_idempotent(false);
        }
    }
}
