//! Integration harness for virtual path filter unit tests.

mod virtual_paths_module {
    mod filter {
        include!("../src/skills/tools/scan/virtual_paths/filter.rs");
    }

    mod tests {
        include!("unit/skills/tools/scan/virtual_paths/tests.rs");
    }
}
