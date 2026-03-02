//! Integration harness for skill command parser unit tests.

mod skill_command_parser_module {
    pub use xiuxian_skills::skills::skill_command::parser::*;

    mod tests {
        include!("unit/skills/skill_command/parser/tests.rs");
    }
}
