const READ_ONLY_INDICATORS: &[&str] = &[
    "get", "fetch", "read", "query", "list", "show", "display", "view", "check", "validate",
    "exists", "find", "search", "lookup", "describe",
];

const DESTRUCTIVE_INDICATORS: &[&str] = &[
    "delete",
    "remove",
    "destroy",
    "drop",
    "truncate",
    "clear",
    "reset",
    "overwrite",
    "write",
    "create",
    "add",
    "insert",
    "update",
    "modify",
    "edit",
    "save",
    "commit",
    "push",
    "deploy",
];

const OPEN_WORLD_INDICATORS: &[&str] =
    &["fetch", "http", "request", "api", "url", "web", "network"];

pub(super) fn is_read_only_function(name_lower: &str) -> bool {
    READ_ONLY_INDICATORS
        .iter()
        .any(|indicator| name_lower.starts_with(indicator))
}

pub(super) fn is_destructive_function(name_lower: &str) -> bool {
    DESTRUCTIVE_INDICATORS
        .iter()
        .any(|indicator| name_lower.starts_with(indicator))
}

pub(super) fn is_open_world_function(name_lower: &str) -> bool {
    OPEN_WORLD_INDICATORS
        .iter()
        .any(|indicator| name_lower.contains(indicator))
}
