//! Integration harness for knowledge scanner unit tests.

mod knowledge {
    pub use xiuxian_skills::knowledge::*;
}

mod knowledge_scanner_module {
    pub use xiuxian_skills::KnowledgeScanner;

    mod tests {
        include!("unit/knowledge/scanner/tests.rs");
    }
}
