pub(crate) fn recipient_target_for<'a>(recipient: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    let (prefix, target) = parse_prefixed_recipient(recipient)?;
    prefixes
        .iter()
        .any(|candidate| prefix.eq_ignore_ascii_case(candidate))
        .then_some(target)
}

pub(crate) fn parse_prefixed_recipient(recipient: &str) -> Option<(&str, &str)> {
    let (prefix, target) = recipient.split_once(':')?;
    let prefix = prefix.trim();
    let target = target.trim();
    (!prefix.is_empty() && !target.is_empty()).then_some((prefix, target))
}

pub(crate) fn recipient_is_telegram_chat_id(recipient: &str) -> bool {
    let trimmed = recipient.trim();
    if trimmed.is_empty() {
        return false;
    }
    let digits = trimmed.strip_prefix('-').unwrap_or(trimmed);
    !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit())
}
