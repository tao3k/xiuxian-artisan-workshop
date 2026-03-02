//! Core system overrides only.
//! Legacy shortcuts for external tools (crawl, graph, etc.) have been removed.
//! External capabilities are now strictly managed via the `ReAct` tool-call loop.

/// Forces the agent to use the `ReAct` loop even if quality gates suggest otherwise.
/// Used primarily for debugging: `!react <message>`.
#[must_use]
pub fn parse_react_shortcut(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if !trimmed.starts_with("!react") {
        return None;
    }
    let mut head_tail = trimmed.splitn(2, char::is_whitespace);
    let _verb = head_tail.next()?;
    let message = head_tail.next()?.trim();
    if message.is_empty() {
        return None;
    }
    Some(message.to_string())
}
