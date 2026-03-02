//! Integration harness for `search_impl` IPC conversion unit tests.

mod skill {
    pub use omni_vector::skill::*;
}

mod search_impl_module {
    mod confidence {
        include!("../src/search/search_impl/confidence.rs");
    }

    const _: f32 = confidence::KEYWORD_BOOST;

    mod ipc {
        include!("../src/search/search_impl/ipc.rs");
    }

    mod tests {
        include!("unit/search/search_impl/tests.rs");
    }
}
