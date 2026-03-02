//! Integration harness for parser parameter model unit tests.

mod skill_command_parser_parameters_model_module {
    pub use xiuxian_skills::skills::skill_command::parser::ParsedParameter;

    mod tests {
        include!("unit/skills/skill_command/parser/parameters/model/tests.rs");
    }
}
