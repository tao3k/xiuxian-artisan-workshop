//! Integration harness for category inference unit tests.

mod skill_command_category_module {
    pub use xiuxian_skills::skills::skill_command::category::infer_category_from_skill;

    mod tests {
        include!("unit/skills/skill_command/category/tests.rs");
    }
}
