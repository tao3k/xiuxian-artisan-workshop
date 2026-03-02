/// Normalized embedding backend kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingBackendKind {
    /// Legacy `/embed/batch` HTTP endpoint.
    Http,
    /// OpenAI-compatible `/v1/embeddings` endpoint.
    OpenAiHttp,
    /// In-process `mistralrs` SDK embedding runtime.
    MistralSdk,
    /// `litellm-rs` in-process provider path.
    LiteLlmRs,
}

impl EmbeddingBackendKind {
    /// Stable config value for this backend.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::OpenAiHttp => "openai_http",
            Self::MistralSdk => "mistral_sdk",
            Self::LiteLlmRs => "litellm_rs",
        }
    }
}

/// Parse a backend hint into a normalized embedding backend kind.
///
/// Returns `None` when the input is empty or not recognized.
#[must_use]
pub fn parse_embedding_backend_kind(raw: Option<&str>) -> Option<EmbeddingBackendKind> {
    let normalized = raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)?;

    if normalized == "http" || normalized == "client" {
        return Some(EmbeddingBackendKind::Http);
    }

    if normalized == "openai_http"
        || normalized == "openai-http"
        || normalized == "openai_compat"
        || normalized == "openai-compatible"
        || normalized == "openai"
    {
        return Some(EmbeddingBackendKind::OpenAiHttp);
    }

    if normalized == "mistral_sdk"
        || normalized == "mistral-sdk"
        || normalized == "mistral_inproc"
        || normalized == "mistral-inproc"
    {
        return Some(EmbeddingBackendKind::MistralSdk);
    }

    if normalized == "litellm_rs"
        || normalized == "litellm-rs"
        || normalized == "litellm"
        || normalized == "provider"
    {
        return Some(EmbeddingBackendKind::LiteLlmRs);
    }

    None
}
