//! Top-level integration harness for `session::redis_backend::message_store`.

mod observability {
    /// Minimal event shim used by redis backend message-store tests.
    #[derive(Debug, Clone, Copy)]
    pub(crate) enum SessionEvent {
        SessionMessagesAppended,
        SessionMessagesReplaced,
        SessionMessagesLoaded,
        SessionMessagesCleared,
        ContextBackupCaptured,
    }

    impl SessionEvent {
        pub(crate) const fn as_str(self) -> &'static str {
            match self {
                Self::SessionMessagesAppended => "session.messages.appended",
                Self::SessionMessagesReplaced => "session.messages.replaced",
                Self::SessionMessagesLoaded => "session.messages.loaded",
                Self::SessionMessagesCleared => "session.messages.cleared",
                Self::ContextBackupCaptured => "session.context_backup.captured",
            }
        }
    }
}

mod session {
    pub(crate) use message::{ChatMessage, FunctionCall, ToolCallOut};

    pub(crate) mod message {
        pub(crate) use omni_agent::{ChatMessage, FunctionCall, ToolCallOut};
    }

    pub(crate) mod redis_backend {
        /// Minimal redis-backend shim required for compiling message-store helpers in isolation.
        #[derive(Debug, Clone, Default)]
        pub(crate) struct RedisSessionBackend {
            pub(crate) message_content_max_chars: Option<usize>,
            pub(crate) ttl_secs: Option<u64>,
        }

        impl RedisSessionBackend {
            pub(crate) fn messages_key(&self, _session_id: &str) -> String {
                String::new()
            }

            pub(crate) async fn run_pipeline<T, F>(
                &self,
                _op: &str,
                _builder: F,
            ) -> anyhow::Result<T>
            where
                T: Default,
                F: FnOnce() -> redis::Pipeline,
            {
                Ok(T::default())
            }

            pub(crate) async fn run_command<T, F>(
                &self,
                _op: &str,
                _builder: F,
            ) -> anyhow::Result<T>
            where
                T: Default,
                F: FnOnce() -> redis::Cmd,
            {
                Ok(T::default())
            }
        }

        pub(crate) mod message_store {
            include!("../src/session/redis_backend/message_store.rs");
        }

        fn lint_symbol_probe() {
            let _ = (
                crate::observability::SessionEvent::SessionMessagesAppended,
                crate::observability::SessionEvent::SessionMessagesReplaced,
                crate::observability::SessionEvent::SessionMessagesLoaded,
                crate::observability::SessionEvent::SessionMessagesCleared,
                crate::observability::SessionEvent::ContextBackupCaptured,
            );
            let backend = RedisSessionBackend {
                message_content_max_chars: Some(1024),
                ttl_secs: Some(60),
            };
            let _ = (
                &backend.message_content_max_chars,
                &backend.ttl_secs,
                backend.messages_key("probe"),
            );
            let _ = RedisSessionBackend::run_pipeline::<(), fn() -> redis::Pipeline>;
            let _ = RedisSessionBackend::run_command::<(), fn() -> redis::Cmd>;
            let _ = RedisSessionBackend::append_messages;
            let _ = RedisSessionBackend::replace_messages;
            let _ = RedisSessionBackend::get_messages;
            let _ = RedisSessionBackend::get_messages_len;
            let _ = RedisSessionBackend::clear_messages;
            let _ = message_store::encode_chat_message_payload
                as fn(
                    &crate::session::ChatMessage,
                    Option<usize>,
                ) -> anyhow::Result<message_store::EncodedChatMessagePayload>;
            let _ = message_store::decode_chat_message_payload
                as fn(
                    &str,
                    &str,
                )
                    -> std::result::Result<crate::session::ChatMessage, serde_json::Error>;
        }

        const _: fn() = lint_symbol_probe;

        mod tests {
            include!("unit/session/redis_backend_tests.rs");
        }
    }
}
