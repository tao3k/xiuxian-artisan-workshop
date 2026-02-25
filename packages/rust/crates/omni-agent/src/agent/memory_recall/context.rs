use omni_memory::Episode;

/// Build one bounded memory context block for system prompt injection.
pub(crate) fn build_memory_context_message(
    recalled: &[(Episode, f32)],
    max_context_chars: usize,
) -> Option<String> {
    if recalled.is_empty() || max_context_chars == 0 {
        return None;
    }

    let header = "Relevant past experiences (use to inform your response):";
    let mut lines = vec![header.to_string()];
    let mut remaining_chars = max_context_chars.saturating_sub(header.chars().count() + 1);

    for (index, (episode, score)) in recalled.iter().enumerate() {
        if remaining_chars < 80 {
            break;
        }

        let intent = clip_to_chars(&episode.intent, 72);
        let outcome = clip_to_chars(&episode.outcome, 56);
        let prefix = format!(
            "- [{}] score={:.3} intent={} outcome={} experience=",
            index + 1,
            score,
            intent,
            outcome
        );

        let prefix_chars = prefix.chars().count();
        if prefix_chars >= remaining_chars {
            break;
        }

        let experience_budget = remaining_chars.saturating_sub(prefix_chars).clamp(48, 260);
        let experience = clip_to_chars(&episode.experience, experience_budget);
        let line = format!("{prefix}{experience}");
        remaining_chars = remaining_chars.saturating_sub(line.chars().count() + 1);
        lines.push(line);
    }

    if lines.len() <= 1 {
        return None;
    }

    Some(lines.join("\n"))
}

fn clip_to_chars(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let keep = max_chars.saturating_sub(3);
    let mut out = input.chars().take(keep).collect::<String>();
    out.push_str("...");
    out
}
