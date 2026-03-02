//! Integration harness for `xiuxian-tui` main argument parsing unit tests.

#[path = "../src/cli_args.rs"]
mod cli_args_impl;

mod main_demo_module {
    pub(crate) use super::cli_args_impl::Args;
    pub(crate) use clap::Parser;

    mod tests {
        include!("unit/main_demo_tests.rs");
    }
}
