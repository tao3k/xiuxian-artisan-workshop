pub(super) use super::settings;

#[path = "../../src/config/xiuxian.rs"]
mod inner;

pub(crate) fn load_xiuxian_config_from_bases(
    system_base: &std::path::Path,
    user_base: &std::path::Path,
) -> inner::XiuxianConfig {
    inner::load_xiuxian_config_from_bases(system_base, user_base)
}

fn lint_symbol_probe() {
    let _ = inner::load_xiuxian_config as fn() -> inner::XiuxianConfig;
}

const _: fn() = lint_symbol_probe;
