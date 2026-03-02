//! Package-top harness for xml-lite unit tests.

#![cfg(feature = "zhenfa-router")]

mod link_graph {
    pub use xiuxian_wendao::link_graph::{LinkGraphDisplayHit, LinkGraphPlannedSearchPayload};
}

mod xml_lite_harness {
    include!("../src/zhenfa_router/native/xml_lite.rs");

    mod tests {
        include!("unit/zhenfa_router/native/xml_lite_tests.rs");
    }
}
