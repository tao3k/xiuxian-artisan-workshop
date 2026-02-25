//! Script parsing utilities for @`skill_command` decorator extraction.
//!
//! Provides functions to parse Python scripts and extract:
//! - Decorator positions and arguments
//! - Function docstrings
//! - Parameter names and schema hints

mod decorator;
mod docstring;
mod parameters;

pub use decorator::{find_skill_command_decorators, parse_decorator_args};
pub use docstring::{extract_docstring_from_text, extract_param_descriptions};
pub use parameters::{
    ParsedParameter, extract_parameters_from_text, extract_parsed_parameters, parse_parameters,
};

#[cfg(test)]
mod tests;
