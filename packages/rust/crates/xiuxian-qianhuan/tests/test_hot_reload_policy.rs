//! Integration tests for shared hot-reload policy normalization helpers.

use xiuxian_qianhuan::{resolve_hot_reload_watch_extensions, resolve_hot_reload_watch_patterns};

#[test]
fn resolve_hot_reload_watch_extensions_prefers_configured_values() {
    let configured = vec![
        " .ORG ".to_string(),
        "J2".to_string(),
        "invalid^token".to_string(),
        "org".to_string(),
    ];
    let resolved = resolve_hot_reload_watch_extensions(Some(&configured), &["md", "markdown"]);
    assert_eq!(resolved, vec!["org".to_string(), "j2".to_string()]);
}

#[test]
fn resolve_hot_reload_watch_extensions_falls_back_to_defaults() {
    let configured = vec![String::new(), "bad^token".to_string()];
    let resolved = resolve_hot_reload_watch_extensions(Some(&configured), &["md", "org", "j2"]);
    assert_eq!(
        resolved,
        vec!["md".to_string(), "org".to_string(), "j2".to_string()]
    );
}

#[test]
fn resolve_hot_reload_watch_patterns_prefers_configured_patterns() {
    let configured = vec!["docs/**/*.md".to_string(), "**/SKILL.md".to_string()];
    let resolved = resolve_hot_reload_watch_patterns(Some(&configured), None, &["**/*.md"]);
    assert_eq!(resolved, configured);
}

#[test]
fn resolve_hot_reload_watch_patterns_derives_from_extensions() {
    let configured_extensions = vec!["org".to_string(), "j2".to_string(), "toml".to_string()];
    let resolved =
        resolve_hot_reload_watch_patterns(None, Some(&configured_extensions), &["**/*.md"]);
    assert_eq!(
        resolved,
        vec![
            "**/*.org".to_string(),
            "**/*.j2".to_string(),
            "**/*.toml".to_string()
        ]
    );
}

#[test]
fn resolve_hot_reload_watch_patterns_falls_back_to_defaults() {
    let resolved = resolve_hot_reload_watch_patterns(None, None, &["**/*.md", "**/*.markdown"]);
    assert_eq!(
        resolved,
        vec!["**/*.md".to_string(), "**/*.markdown".to_string()]
    );
}
