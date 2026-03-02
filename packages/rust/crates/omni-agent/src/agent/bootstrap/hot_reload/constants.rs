pub(super) const XIUXIAN_HOT_RELOAD_ENABLED_ENV: &str = "XIUXIAN_HOT_RELOAD_ENABLED";
pub(super) const XIUXIAN_HOT_RELOAD_DEBOUNCE_MS_ENV: &str = "XIUXIAN_HOT_RELOAD_DEBOUNCE_MS";
pub(super) const XIUXIAN_HOT_RELOAD_SYNC_INTERVAL_MS_ENV: &str =
    "XIUXIAN_HOT_RELOAD_SYNC_INTERVAL_MS";
pub(super) const XIUXIAN_HOT_RELOAD_VALKEY_URL_ENV: &str = "XIUXIAN_HOT_RELOAD_VALKEY_URL";
pub(super) const XIUXIAN_HOT_RELOAD_VALKEY_KEY_PREFIX_ENV: &str =
    "XIUXIAN_HOT_RELOAD_VALKEY_KEY_PREFIX";

pub(super) const DEFAULT_HOT_RELOAD_DEBOUNCE_MS: u64 = 150;
pub(super) const DEFAULT_HOT_RELOAD_SYNC_INTERVAL_MS: u64 = 1500;
pub(super) const DEFAULT_WENDAO_INCREMENTAL_EXTENSIONS: &[&str] =
    &["md", "markdown", "org", "orgm", "j2", "toml"];
pub(super) const DEFAULT_WENDAO_WATCH_PATTERNS: &[&str] = &[
    "**/*.md",
    "**/*.markdown",
    "**/*.org",
    "**/*.orgm",
    "**/*.j2",
    "**/*.toml",
];

pub(super) const HOT_RELOAD_DOMAIN: &str = "hot_reload";
pub(super) const HOT_RELOAD_TARGET_QIANHUAN_MANIFESTATION: &str =
    "hot_reload.target.qianhuan.manifestation";
pub(super) const HOT_RELOAD_TARGET_WENDAO_INDEX: &str = "hot_reload.target.wendao.index";
pub(super) const TARGET_ID_QIANHUAN_MANIFESTATION: &str =
    "xiuxian_qianhuan.manifestation.templates";
pub(super) const TARGET_ID_WENDAO_INDEX: &str = "xiuxian_wendao.link_graph.index";
