//! Test coverage for omni-agent behavior.

use omni_agent::{
    TELEGRAM_MAX_MESSAGE_LENGTH, chunk_marker_reserve_chars, decorate_chunk_for_telegram,
    split_message_for_telegram,
};

fn max_chunk_chars() -> usize {
    TELEGRAM_MAX_MESSAGE_LENGTH - chunk_marker_reserve_chars()
}

fn assert_decorated_chunks_within_limit(chunks: &[String]) {
    for (index, chunk) in chunks.iter().enumerate() {
        let decorated = decorate_chunk_for_telegram(chunk, index, chunks.len());
        assert!(
            decorated.chars().count() <= TELEGRAM_MAX_MESSAGE_LENGTH,
            "decorated chunk {index} exceeds limit: {} > {}",
            decorated.chars().count(),
            TELEGRAM_MAX_MESSAGE_LENGTH
        );
    }
}

#[test]
fn split_message_handles_multibyte_char_at_chunk_boundary() {
    let max_chunk = max_chunk_chars();
    let message = format!("{}{}{}", "a".repeat(max_chunk - 1), "：", "z".repeat(64));
    let chunks = split_message_for_telegram(&message);
    assert!(chunks.len() > 1);
    assert!(chunks[0].ends_with('：'));
    assert_eq!(chunks.concat(), message);
    assert_decorated_chunks_within_limit(&chunks);
}

#[test]
fn split_message_preserves_cjk_content_without_panicking() {
    let message = "说".repeat(TELEGRAM_MAX_MESSAGE_LENGTH + 128);
    let chunks = split_message_for_telegram(&message);
    assert!(chunks.len() > 1);
    assert!(chunks.iter().all(|chunk| !chunk.is_empty()));
    assert_eq!(chunks.concat(), message);
    assert_decorated_chunks_within_limit(&chunks);
}

#[test]
fn split_message_prefers_nearby_newline_breaks() {
    let max_chunk = max_chunk_chars();
    let message = format!("{}\n{}", "a".repeat(max_chunk - 8), "b".repeat(80));
    let chunks = split_message_for_telegram(&message);
    assert!(chunks.len() > 1);
    assert!(chunks[0].ends_with('\n'));
    assert_eq!(chunks.concat(), message);
    assert_decorated_chunks_within_limit(&chunks);
}

#[test]
fn split_message_falls_back_to_space_when_newline_is_too_early() {
    let max_chunk = max_chunk_chars();
    let start = "head\n";
    let middle = "x".repeat(max_chunk - start.chars().count() - 1);
    let message = format!("{start}{middle} {}", "tail".repeat(32));
    let chunks = split_message_for_telegram(&message);
    assert!(chunks.len() > 1);
    assert!(chunks[0].ends_with(' '));
    assert_eq!(chunks.concat(), message);
    assert_decorated_chunks_within_limit(&chunks);
}

#[test]
fn split_message_prefers_markdown_ast_block_boundary_even_when_newline_is_early() {
    let max_chunk = max_chunk_chars();
    let first_paragraph = format!("{}\n\n", "intro".repeat(40));
    assert!(
        first_paragraph.chars().count() < max_chunk / 2,
        "precondition: first markdown block should be before halfway point"
    );
    let second_paragraph = "x".repeat(max_chunk + 200);
    let message = format!("{first_paragraph}{second_paragraph}");
    let chunks = split_message_for_telegram(&message);
    assert!(chunks.len() > 1);
    assert_eq!(
        chunks[0].trim_end_matches('\n'),
        first_paragraph.trim_end_matches('\n'),
        "AST-aware chunker should preserve top-level paragraph boundary"
    );
    assert!(
        !chunks[0].contains('x'),
        "first chunk should not consume characters from the next paragraph"
    );
    assert_eq!(chunks.concat(), message);
    assert_decorated_chunks_within_limit(&chunks);
}
