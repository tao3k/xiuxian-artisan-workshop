use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use mistralrs::{EmbeddingModelBuilder, EmbeddingRequest, Model};
use tokio::sync::Mutex;

type SharedEmbeddingModel = Arc<Model>;
const MAX_MISTRAL_SDK_EMBED_MAX_NUM_SEQS: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EmbeddingModelCacheKey {
    model_id: String,
    hf_cache_path: Option<PathBuf>,
    hf_revision: Option<String>,
    max_num_seqs: Option<usize>,
}

type EmbeddingModelCache = HashMap<EmbeddingModelCacheKey, SharedEmbeddingModel>;

static EMBEDDING_MODEL_CACHE: OnceLock<Mutex<EmbeddingModelCache>> = OnceLock::new();

fn embedding_model_cache() -> &'static Mutex<EmbeddingModelCache> {
    EMBEDDING_MODEL_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Normalize SDK model id.
#[must_use]
pub fn normalize_mistral_sdk_model(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

/// Normalize optional Hugging Face cache path used by `mistralrs`.
#[must_use]
pub fn normalize_mistral_sdk_hf_cache_path(raw: Option<&str>) -> Option<PathBuf> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// Normalize optional Hugging Face revision hint used by `mistralrs`.
#[must_use]
pub fn normalize_mistral_sdk_hf_revision(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

/// Normalize optional maximum in-flight sequence count used by `mistralrs` embedding runtime.
#[must_use]
pub fn normalize_mistral_sdk_max_num_seqs(raw: Option<usize>) -> Option<usize> {
    raw.filter(|value| *value > 0)
        .map(|value| value.min(MAX_MISTRAL_SDK_EMBED_MAX_NUM_SEQS))
}

async fn get_or_load_embedding_model(
    cache_key: &EmbeddingModelCacheKey,
) -> Option<SharedEmbeddingModel> {
    if let Some(existing) = {
        let cache = embedding_model_cache().lock().await;
        cache.get(cache_key).cloned()
    } {
        return Some(existing);
    }

    let hf_cache_path_for_log = cache_key
        .hf_cache_path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    tracing::info!(
        event = "xiuxian.llm.embedding.mistral_sdk.model_loading",
        model = %cache_key.model_id,
        hf_cache_path = hf_cache_path_for_log,
        hf_revision = cache_key.hf_revision.as_deref().unwrap_or(""),
        max_num_seqs = cache_key.max_num_seqs,
        "loading mistralrs embedding model in-process"
    );

    let mut builder = EmbeddingModelBuilder::new(cache_key.model_id.clone()).with_logging();
    if let Some(hf_cache_path) = cache_key.hf_cache_path.clone() {
        builder = builder.from_hf_cache_path(hf_cache_path);
    }
    if let Some(hf_revision) = cache_key.hf_revision.as_deref() {
        builder = builder.with_hf_revision(hf_revision);
    }
    if let Some(max_num_seqs) = cache_key.max_num_seqs {
        builder = builder.with_max_num_seqs(max_num_seqs);
    }

    let built = match builder.build().await {
        Ok(model) => model,
        Err(error) => {
            tracing::warn!(
                event = "xiuxian.llm.embedding.mistral_sdk.model_load_failed",
                model = %cache_key.model_id,
                error = %error,
                "failed to load mistralrs embedding model"
            );
            return None;
        }
    };

    let shared = Arc::new(built);
    let mut cache = embedding_model_cache().lock().await;
    if let Some(existing) = cache.get(cache_key) {
        return Some(existing.clone());
    }
    cache.insert(cache_key.clone(), shared.clone());
    Some(shared)
}

/// Generate embeddings through in-process `mistralrs` SDK.
pub async fn embed_with_mistral_sdk(
    texts: &[String],
    model: Option<&str>,
    hf_cache_path: Option<&str>,
    hf_revision: Option<&str>,
    max_num_seqs: Option<usize>,
) -> Option<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Some(vec![]);
    }
    let model_id = normalize_mistral_sdk_model(model)?;
    let cache_key = EmbeddingModelCacheKey {
        model_id: model_id.clone(),
        hf_cache_path: normalize_mistral_sdk_hf_cache_path(hf_cache_path),
        hf_revision: normalize_mistral_sdk_hf_revision(hf_revision),
        max_num_seqs: normalize_mistral_sdk_max_num_seqs(max_num_seqs),
    };
    let model = get_or_load_embedding_model(&cache_key).await?;

    let mut request = EmbeddingRequest::builder();
    for text in texts {
        request = request.add_prompt(text.as_str());
    }

    let embeddings = match model.generate_embeddings(request).await {
        Ok(vectors) => vectors,
        Err(error) => {
            tracing::warn!(
                event = "xiuxian.llm.embedding.mistral_sdk.request_failed",
                model = model_id,
                batch_size = texts.len(),
                error = %error,
                "mistralrs embedding request failed"
            );
            return None;
        }
    };

    if embeddings.len() != texts.len() {
        tracing::warn!(
            event = "xiuxian.llm.embedding.mistral_sdk.invalid_vector_count",
            model = model_id,
            expected_vectors = texts.len(),
            actual_vectors = embeddings.len(),
            "mistralrs embedding result vector count mismatch"
        );
        return None;
    }
    Some(embeddings)
}
