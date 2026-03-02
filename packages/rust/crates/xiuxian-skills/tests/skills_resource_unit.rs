//! Integration harness for resource scanner unit tests.

mod resource_module {
    pub use xiuxian_skills::skills::ResourceScanner;

    mod tests {
        include!("unit/skills/resource/tests.rs");
    }
}
