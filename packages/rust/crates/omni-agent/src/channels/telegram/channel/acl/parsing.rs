use crate::env_parse::parse_bool_from_env;

pub(super) fn resolve_string_env_or_setting(
    env_name: &str,
    setting_value: Option<String>,
    default: &str,
) -> String {
    if let Ok(value) = std::env::var(env_name)
        && !value.trim().is_empty()
    {
        return value;
    }
    setting_value.unwrap_or_else(|| default.to_string())
}

pub(super) fn resolve_optional_env_or_setting(
    env_name: &str,
    setting_value: Option<String>,
) -> Option<String> {
    if let Ok(value) = std::env::var(env_name) {
        return Some(value);
    }
    setting_value
}

pub(super) fn resolve_bool_env_or_setting(
    env_name: &str,
    setting_value: Option<bool>,
    default: bool,
) -> bool {
    if std::env::var_os(env_name).is_some() {
        return parse_bool_from_env(env_name).unwrap_or(default);
    }
    setting_value.unwrap_or(default)
}

pub(super) fn parse_comma_entries(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
        .collect()
}

pub(super) fn parse_optional_comma_entries(raw: Option<String>) -> Option<Vec<String>> {
    raw.map(|value| parse_comma_entries(value.as_str()))
}
