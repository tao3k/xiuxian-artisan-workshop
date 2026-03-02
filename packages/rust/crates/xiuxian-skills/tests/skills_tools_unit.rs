//! Integration harness for tools scanner unit tests.

mod skills {
    pub use xiuxian_skills::skills::*;
}

#[path = "unit/skills/tools/tests/mod.rs"]
mod skills_tools_tests;
