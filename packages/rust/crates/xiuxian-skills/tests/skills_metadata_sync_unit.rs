//! Integration harness for metadata sync unit tests.

mod skills {
    pub use xiuxian_skills::skills::*;
}

mod metadata_sync_module {
    pub use xiuxian_skills::skills::metadata::{ScanConfig, SyncReport, calculate_sync_ops};

    mod tests {
        include!("unit/skills/metadata/sync/tests.rs");
    }
}
