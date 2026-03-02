use std::path::Path;

use sha2::{Digest, Sha256};

pub(super) fn content_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(super) fn knowledge_id_from_relative_path(relative_path: &Path) -> String {
    let path_str = relative_path.to_string_lossy();
    let mut path_hasher = Sha256::new();
    path_hasher.update(path_str.as_bytes());
    let hash_result = path_hasher.finalize();
    hex::encode(&hash_result[..16])
}
