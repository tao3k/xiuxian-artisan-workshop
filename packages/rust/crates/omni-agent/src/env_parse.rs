use xiuxian_macros::env_first_non_empty;

pub(crate) const XIUXIAN_WENDAO_VALKEY_URL_ENV: &str = "XIUXIAN_WENDAO_VALKEY_URL";
pub(crate) const LEGACY_VALKEY_URL_ENV: &str = "VALKEY_URL";

/// Parse an environment variable as positive `usize`.
#[must_use]
pub fn parse_positive_usize_from_env(name: &str) -> Option<usize> {
    parse_env_value(
        name,
        |raw| raw.parse::<usize>().ok().filter(|value| *value > 0),
        "invalid positive integer env value",
    )
}

/// Parse an environment variable as positive `u64`.
#[must_use]
pub fn parse_positive_u64_from_env(name: &str) -> Option<u64> {
    parse_env_value(
        name,
        |raw| raw.parse::<u64>().ok().filter(|value| *value > 0),
        "invalid positive integer env value",
    )
}

/// Parse an environment variable as boolean (`true/false`, `1/0`, `yes/no`, `on/off`).
#[must_use]
pub fn parse_bool_from_env(name: &str) -> Option<bool> {
    parse_env_value(
        name,
        |raw| match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        },
        "invalid boolean env value",
    )
}

#[must_use]
pub(crate) fn resolve_valkey_url_env() -> Option<String> {
    env_first_non_empty!(XIUXIAN_WENDAO_VALKEY_URL_ENV, LEGACY_VALKEY_URL_ENV)
}

fn parse_env_value<T>(
    name: &str,
    parser: impl FnOnce(&str) -> Option<T>,
    invalid_message: &'static str,
) -> Option<T> {
    let raw = std::env::var(name).ok()?;
    if let Some(value) = parser(raw.as_str()) {
        Some(value)
    } else {
        tracing::warn!(env_var = %name, value = %raw, "{invalid_message}");
        None
    }
}
