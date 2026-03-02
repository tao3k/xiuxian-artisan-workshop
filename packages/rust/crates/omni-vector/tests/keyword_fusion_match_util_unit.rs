//! Integration harness for keyword fusion `match_util` unit tests.

#[path = "../src/keyword/fusion/match_util.rs"]
mod match_util_impl;

mod match_util_module {
    pub(crate) use super::match_util_impl::*;
    pub(crate) use aho_corasick::PatternID;
    pub(crate) use lance::deps::arrow_array::StringArray;

    mod tests {
        include!("unit/keyword/fusion/match_util_tests.rs");
    }
}
