//! Top-level integration harness for `config::xiuxian`.

mod config {
    mod settings {
        pub use omni_agent::RuntimeSettings;

        pub(crate) fn runtime_settings_paths() -> (std::path::PathBuf, std::path::PathBuf) {
            (
                std::path::PathBuf::from("packages/conf/xiuxian.toml"),
                std::path::PathBuf::from(".config/xiuxian-artisan-workshop/xiuxian.toml"),
            )
        }

        fn lint_symbol_probe() {
            let _ = runtime_settings_paths as fn() -> (std::path::PathBuf, std::path::PathBuf);
        }

        const _: fn() = lint_symbol_probe;
    }

    #[path = "xiuxian.rs"]
    mod xiuxian;

    mod tests {
        include!("unit/config/config_tests.rs");
    }
}
