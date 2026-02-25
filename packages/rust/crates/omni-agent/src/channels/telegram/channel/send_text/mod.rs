mod dispatch;
mod helpers;

use std::time::Duration;

use super::TelegramChannel;
use super::identity::parse_recipient_target;
use super::media::{parse_attachment_markers, parse_path_only_attachment};
use super::outbound_text::normalize_telegram_outbound_text;
use super::send_types::PreparedCaption;
use helpers::{
    PreparedChunk, assert_chunk_limit, prepare_chunks, split_chunks_with_guard,
    warn_chunk_payload_fallbacks,
};

impl TelegramChannel {
    pub(super) async fn send_text(&self, message: &str, recipient: &str) -> anyhow::Result<()> {
        let (chat_id, thread_id) = parse_recipient_target(recipient);
        let normalized_message = normalize_telegram_outbound_text(message);

        let (text_without_markers, attachments, has_invalid_attachment_marker) =
            parse_attachment_markers(&normalized_message);
        if !attachments.is_empty() {
            let first_attachment_caption =
                Self::select_first_attachment_caption(&text_without_markers, &attachments)
                    .map(|caption| PreparedCaption::from_plain(caption.as_str()));

            if first_attachment_caption.is_none() && !text_without_markers.is_empty() {
                self.send_text_chunks(&text_without_markers, chat_id, thread_id, false)
                    .await?;
            }
            self.send_attachments(
                chat_id,
                thread_id,
                &attachments,
                first_attachment_caption.as_ref(),
            )
            .await?;
            return Ok(());
        }

        if has_invalid_attachment_marker {
            return self
                .send_text_chunks(&text_without_markers, chat_id, thread_id, true)
                .await;
        }

        if let Some(attachment) = parse_path_only_attachment(&normalized_message) {
            self.send_attachments(chat_id, thread_id, &[attachment], None)
                .await?;
            return Ok(());
        }

        self.send_text_chunks(&normalized_message, chat_id, thread_id, false)
            .await
    }

    async fn send_text_chunks(
        &self,
        message: &str,
        chat_id: &str,
        thread_id: Option<&str>,
        force_plain: bool,
    ) -> anyhow::Result<()> {
        let (chunks, truncated) = split_chunks_with_guard(message);
        let prepared_chunks = prepare_chunks(&chunks);
        warn_chunk_payload_fallbacks(&prepared_chunks);

        self.send_prepared_chunks(chat_id, thread_id, &prepared_chunks, force_plain)
            .await?;
        self.send_truncation_notice(chat_id, thread_id, truncated)
            .await?;
        Ok(())
    }

    async fn send_prepared_chunks(
        &self,
        chat_id: &str,
        thread_id: Option<&str>,
        prepared_chunks: &[PreparedChunk],
        force_plain: bool,
    ) -> anyhow::Result<()> {
        for (index, chunk) in prepared_chunks.iter().enumerate() {
            self.send_single_chunk(chat_id, thread_id, chunk, force_plain)
                .await?;

            assert_chunk_limit(chunk, index);
            if index < prepared_chunks.len() - 1 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        Ok(())
    }

    async fn send_truncation_notice(
        &self,
        chat_id: &str,
        thread_id: Option<&str>,
        truncated: Option<(usize, usize)>,
    ) -> anyhow::Result<()> {
        if let Some((total_chunks, kept_chunks)) = truncated {
            let notice = format!(
                "Output truncated after {kept_chunks} of {total_chunks} chunks to prevent flood. Narrow the query or request paginated output."
            );
            self.send_message_with_mode(chat_id, thread_id, &notice, None)
                .await
                .map_err(|error| {
                    anyhow::anyhow!("Telegram sendMessage failed (truncation notice): {error}")
                })?;
        }
        Ok(())
    }
}
