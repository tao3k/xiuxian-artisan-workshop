use xiuxian_llm::embedding::sdk::embed_with_mistral_sdk;

pub(super) async fn embed_mistral_sdk(
    texts: &[String],
    model: Option<&str>,
    hf_cache_path: Option<&str>,
    hf_revision: Option<&str>,
    max_num_seqs: Option<usize>,
) -> Option<Vec<Vec<f32>>> {
    embed_with_mistral_sdk(texts, model, hf_cache_path, hf_revision, max_num_seqs).await
}
