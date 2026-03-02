//! Integration harness for tool record unit tests.

mod skills {
    pub use xiuxian_skills::skills::*;
}

mod tool_record_module {
    pub use xiuxian_skills::skills::metadata::{ToolEnrichment, ToolRecord};

    mod tests {
        include!("unit/skills/metadata/core/tool_record/tests.rs");
    }
}
