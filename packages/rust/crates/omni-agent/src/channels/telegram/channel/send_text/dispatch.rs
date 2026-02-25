use super::super::TelegramChannel;
use super::super::constants::TELEGRAM_MAX_MESSAGE_LENGTH;
use super::helpers::{PreparedChunk, should_prefer_html_chunk};

impl TelegramChannel {
    pub(super) async fn send_single_chunk(
        &self,
        chat_id: &str,
        thread_id: Option<&str>,
        chunk: &PreparedChunk,
        force_plain: bool,
    ) -> anyhow::Result<()> {
        if force_plain {
            return self
                .send_forced_plain_chunk(chat_id, thread_id, chunk)
                .await;
        }

        if chunk.markdown_chars > TELEGRAM_MAX_MESSAGE_LENGTH {
            return self
                .send_oversized_markdown_chunk(chat_id, thread_id, chunk)
                .await;
        }

        if should_prefer_html_chunk(&chunk.plain_text, chunk.markdown_chars, chunk.html_chars)
            && chunk.html_chars <= TELEGRAM_MAX_MESSAGE_LENGTH
        {
            return self
                .send_preferred_html_chunk(chat_id, thread_id, chunk)
                .await;
        }

        self.send_markdown_chunk_with_fallback(chat_id, thread_id, chunk)
            .await
    }

    async fn send_forced_plain_chunk(
        &self,
        chat_id: &str,
        thread_id: Option<&str>,
        chunk: &PreparedChunk,
    ) -> anyhow::Result<()> {
        self.send_message_with_mode(chat_id, thread_id, &chunk.plain_text, None)
            .await
            .map_err(|plain_error| {
                anyhow::anyhow!("Telegram sendMessage failed (forced plain mode: {plain_error})")
            })
    }

    async fn send_oversized_markdown_chunk(
        &self,
        chat_id: &str,
        thread_id: Option<&str>,
        chunk: &PreparedChunk,
    ) -> anyhow::Result<()> {
        if chunk.html_chars <= TELEGRAM_MAX_MESSAGE_LENGTH {
            return self
                .send_html_then_plain_retry(
                    chat_id,
                    thread_id,
                    chunk,
                    "Telegram HTML send failed with parse-mode error for oversized markdown chunk; retrying without parse_mode",
                    |html_error, plain_error| {
                        anyhow::anyhow!(
                            "Telegram sendMessage failed (markdown exceeded limit; html fallback failed: {html_error}; plain fallback: {plain_error})"
                        )
                    },
                    |error| {
                        anyhow::anyhow!(
                            "Telegram sendMessage failed (markdown exceeded limit; html fallback failed: {error})"
                        )
                    },
                )
                .await;
        }

        self.send_message_with_mode(chat_id, thread_id, &chunk.plain_text, None)
            .await
            .map_err(|plain_error| {
                anyhow::anyhow!(
                    "Telegram sendMessage failed (markdown/html exceeded size limits; plain fallback: {plain_error})"
                )
            })
    }

    async fn send_preferred_html_chunk(
        &self,
        chat_id: &str,
        thread_id: Option<&str>,
        chunk: &PreparedChunk,
    ) -> anyhow::Result<()> {
        self.send_html_then_plain_retry(
            chat_id,
            thread_id,
            chunk,
            "Telegram preferred HTML send failed with parse-mode error; retrying without parse_mode",
            |html_error, plain_error| {
                anyhow::anyhow!(
                    "Telegram sendMessage failed (preferred html fallback failed: {html_error}; plain fallback: {plain_error})"
                )
            },
            |error| anyhow::anyhow!("Telegram sendMessage failed (preferred html send failed: {error})"),
        )
        .await
    }

    async fn send_markdown_chunk_with_fallback(
        &self,
        chat_id: &str,
        thread_id: Option<&str>,
        chunk: &PreparedChunk,
    ) -> anyhow::Result<()> {
        let send_result = self
            .send_message_with_mode(
                chat_id,
                thread_id,
                &chunk.markdown_v2_text,
                Some("MarkdownV2"),
            )
            .await;

        match send_result {
            Ok(()) => Ok(()),
            Err(markdown_error) if markdown_error.should_retry_without_parse_mode() => {
                tracing::warn!(
                    error = %markdown_error,
                    "Telegram MarkdownV2 send failed with parse-mode error; retrying with HTML parse mode"
                );
                self.send_markdown_html_plain_fallback(chat_id, thread_id, chunk, markdown_error)
                    .await
            }
            Err(error) => Err(anyhow::anyhow!("Telegram sendMessage failed: {error}")),
        }
    }

    async fn send_markdown_html_plain_fallback(
        &self,
        chat_id: &str,
        thread_id: Option<&str>,
        chunk: &PreparedChunk,
        markdown_error: impl std::fmt::Display,
    ) -> anyhow::Result<()> {
        if chunk.html_chars <= TELEGRAM_MAX_MESSAGE_LENGTH {
            return self
                .send_html_then_plain_retry(
                    chat_id,
                    thread_id,
                    chunk,
                    "Telegram HTML send failed with parse-mode error; retrying without parse_mode",
                    |html_error, plain_error| {
                        anyhow::anyhow!(
                            "Telegram sendMessage failed (markdown request failed: {markdown_error}; html fallback failed: {html_error}; plain fallback: {plain_error})"
                        )
                    },
                    |error| {
                        anyhow::anyhow!(
                            "Telegram sendMessage failed (markdown request failed: {markdown_error}; html fallback failed: {error})"
                        )
                    },
                )
                .await;
        }

        tracing::warn!(
            html_chars = chunk.html_chars,
            "Telegram HTML fallback chunk exceeds message limit; sending plain text"
        );
        self.send_message_with_mode(chat_id, thread_id, &chunk.plain_text, None)
            .await
            .map_err(|plain_error| {
                anyhow::anyhow!(
                    "Telegram sendMessage failed (markdown request failed: {markdown_error}; plain fallback: {plain_error})"
                )
            })
    }

    async fn send_html_then_plain_retry<OnRetryError, OnFatalError>(
        &self,
        chat_id: &str,
        thread_id: Option<&str>,
        chunk: &PreparedChunk,
        retry_log: &str,
        on_retry_error: OnRetryError,
        on_fatal_error: OnFatalError,
    ) -> anyhow::Result<()>
    where
        OnRetryError: FnOnce(&dyn std::fmt::Display, &dyn std::fmt::Display) -> anyhow::Error,
        OnFatalError: FnOnce(&dyn std::fmt::Display) -> anyhow::Error,
    {
        let html_result = self
            .send_message_with_mode(chat_id, thread_id, &chunk.html_text, Some("HTML"))
            .await;

        match html_result {
            Ok(()) => Ok(()),
            Err(html_error) if html_error.should_retry_without_parse_mode() => {
                tracing::warn!(error = %html_error, "{retry_log}");
                self.send_message_with_mode(chat_id, thread_id, &chunk.plain_text, None)
                    .await
                    .map_err(|plain_error| on_retry_error(&html_error, &plain_error))
            }
            Err(error) => Err(on_fatal_error(&error)),
        }
    }
}
