/// Normalized LLM backend kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmBackendKind {
    /// OpenAI-compatible HTTP endpoint.
    OpenAiCompatibleHttp,
    /// Local mistral runtime (`mistralrs-server`) over OpenAI-compatible endpoint.
    MistralLocal,
    /// `litellm-rs` provider path.
    LiteLlmRs,
}

impl LlmBackendKind {
    /// Stable config value for this backend.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAiCompatibleHttp => "http",
            Self::MistralLocal => "mistral_local",
            Self::LiteLlmRs => "litellm_rs",
        }
    }
}

/// Parse a backend hint into normalized LLM backend kind.
///
/// Returns `None` when the input is empty or not recognized.
#[must_use]
pub fn parse_llm_backend_kind(raw: Option<&str>) -> Option<LlmBackendKind> {
    let normalized = raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)?;

    if normalized == "http"
        || normalized == "openai_compatible"
        || normalized == "openai-compatible"
        || normalized == "openai_http"
        || normalized == "openai-http"
        || normalized == "openai"
    {
        return Some(LlmBackendKind::OpenAiCompatibleHttp);
    }

    if normalized == "mistral_local"
        || normalized == "mistral-local"
        || normalized == "mistral_server"
        || normalized == "mistral-server"
        || normalized == "mistral_http"
        || normalized == "mistral-http"
        || normalized == "mistral_rs"
        || normalized == "mistral-rs"
        || normalized == "mistral"
        || normalized == "mistralrs"
    {
        return Some(LlmBackendKind::MistralLocal);
    }

    if normalized == "litellm_rs" || normalized == "litellm-rs" {
        return Some(LlmBackendKind::LiteLlmRs);
    }

    None
}
