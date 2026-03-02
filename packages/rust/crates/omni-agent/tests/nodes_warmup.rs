//! Top-level integration harness for `nodes::warmup`.

mod resolve {
    fn parse_positive_from_env<T, F>(name: &str, parser: F) -> Option<T>
    where
        F: FnOnce(&str) -> Option<T>,
    {
        std::env::var(name).ok().as_deref().and_then(parser)
    }

    pub(crate) fn parse_positive_u64_from_env(name: &str) -> Option<u64> {
        parse_positive_from_env(name, |raw| {
            raw.parse::<u64>().ok().filter(|value| *value > 0)
        })
    }

    pub(crate) fn parse_positive_usize_from_env(name: &str) -> Option<usize> {
        parse_positive_from_env(name, |raw| {
            raw.parse::<usize>().ok().filter(|value| *value > 0)
        })
    }
}

mod nodes {
    mod warmup_impl {
        include!("../src/nodes/warmup.rs");

        mod tests {
            include!("nodes/warmup.rs");
        }

        fn lint_symbol_probe() {
            let _ = run_embedding_warmup;
            let _ = resolve_warmup_options;
            let _ = first_non_empty::<1>;
            let _ = non_empty_env as fn(&str) -> Option<String>;
            let _ = trim_non_empty as fn(Option<&str>) -> Option<String>;
            let _ = std::mem::size_of::<WarmupOptions>();
            let _ = std::mem::size_of::<WarmupEnvOverrides>();
        }

        const _: fn() = lint_symbol_probe;
    }
}
