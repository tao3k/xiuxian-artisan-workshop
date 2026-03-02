//! Integration harness for skill metadata index unit tests.

mod metadata_index_module {
    pub use xiuxian_skills::skills::metadata::{DocsAvailable, IndexToolEntry, SkillIndexEntry};

    mod tests {
        include!("unit/skills/metadata/index/tests.rs");
    }
}
