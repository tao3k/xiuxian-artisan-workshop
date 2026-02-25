use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct EmbedBatchResponse {
    pub(crate) vectors: Option<Vec<Vec<f32>>>,
}
