//! Integration harness for reference record unit tests.

mod metadata_reference_record_module {
    pub use xiuxian_skills::skills::metadata::ReferenceRecord;

    mod tests {
        include!("unit/skills/metadata/records/reference/tests.rs");
    }
}
