//! Integration harness for skill command annotation unit tests.

mod skills {
    pub use xiuxian_skills::skills::*;
}

mod skill_command_annotations_module {
    pub use xiuxian_skills::skills::skill_command::annotations::build_annotations;

    mod tests {
        include!("unit/skills/skill_command/annotations/tests.rs");
    }
}
