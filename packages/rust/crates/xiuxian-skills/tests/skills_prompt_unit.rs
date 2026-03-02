//! Integration harness for prompt scanner unit tests.

mod prompt_module {
    pub use xiuxian_skills::skills::PromptScanner;

    mod tests {
        include!("unit/skills/prompt/tests.rs");
    }
}
