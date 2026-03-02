//! Test coverage for omni-agent behavior.

use omni_agent::parse_react_shortcut;

#[test]
fn parse_react_shortcut_accepts_prefixed_command() {
    let Some(parsed) = parse_react_shortcut("!react summarize this issue") else {
        panic!("parsed");
    };
    assert_eq!(parsed, "summarize this issue");
}

#[test]
fn parse_react_shortcut_accepts_leading_and_trailing_spaces() {
    let Some(parsed) = parse_react_shortcut("   !react   analyze logs   ") else {
        panic!("parsed");
    };
    assert_eq!(parsed, "analyze logs");
}

#[test]
fn parse_react_shortcut_rejects_missing_payload() {
    assert!(parse_react_shortcut("!react").is_none());
    assert!(parse_react_shortcut("!react   ").is_none());
}

#[test]
fn parse_react_shortcut_rejects_non_shortcut_text() {
    assert!(parse_react_shortcut("react analyze").is_none());
    assert!(parse_react_shortcut("please !react analyze").is_none());
}
