//! Valkey-backed storage operations for knowledge entries.

mod crud;
mod keyspace;
mod query;

use std::collections::HashSet;
use std::path::PathBuf;

use crate::types::{KnowledgeCategory, KnowledgeEntry};

const KNOWLEDGE_VALKEY_URL_ENV: &str = "XIUXIAN_WENDAO_KNOWLEDGE_VALKEY_URL";
const KNOWLEDGE_VALKEY_KEY_PREFIX_ENV: &str = "XIUXIAN_WENDAO_KNOWLEDGE_VALKEY_KEY_PREFIX";
const DEFAULT_KNOWLEDGE_VALKEY_KEY_PREFIX: &str = "xiuxian_wendao:knowledge";

/// Knowledge storage using Valkey.
#[derive(Debug)]
pub struct KnowledgeStorage {
    /// Base storage path (used as namespace scope, not filesystem persistence).
    path: PathBuf,
    /// Logical table name.
    table_name: String,
    /// Storage vector dimension.
    dimension: usize,
}

fn saturating_usize_to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

impl KnowledgeStorage {
    /// Create a new `KnowledgeStorage` instance.
    #[must_use]
    pub fn new(path: &str, table_name: &str) -> Self {
        Self {
            path: PathBuf::from(path),
            table_name: table_name.to_string(),
            dimension: 128,
        }
    }

    /// Get the dataset path.
    #[must_use]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get the table name.
    #[must_use]
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub(super) fn category_to_str(category: &KnowledgeCategory) -> &'static str {
        match category {
            KnowledgeCategory::Pattern => "patterns",
            KnowledgeCategory::Solution => "solutions",
            KnowledgeCategory::Error => "errors",
            KnowledgeCategory::Technique => "techniques",
            KnowledgeCategory::Note => "notes",
            KnowledgeCategory::Reference => "references",
            KnowledgeCategory::Architecture => "architecture",
            KnowledgeCategory::Workflow => "workflows",
        }
    }

    pub(super) fn normalize_vector(&self, input: &[f32]) -> Vec<f32> {
        if input.len() == self.dimension {
            return input.to_vec();
        }
        let mut out = vec![0.0_f32; self.dimension];
        let copy_len = input.len().min(self.dimension);
        out[..copy_len].copy_from_slice(&input[..copy_len]);
        out
    }

    pub(super) fn text_to_vector(&self, text: &str) -> Vec<f32> {
        let mut vec = vec![0.0_f32; self.dimension];
        for (idx, byte) in text.as_bytes().iter().enumerate() {
            let bucket = idx % self.dimension;
            vec[bucket] += f32::from(*byte) / 255.0;
        }

        let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        vec
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|token| !token.is_empty())
            .map(ToString::to_string)
            .collect()
    }

    pub(super) fn text_score(query: &str, entry: &KnowledgeEntry) -> f32 {
        let query_tokens = Self::tokenize(query);
        if query_tokens.is_empty() {
            return 0.0;
        }

        let mut doc_tokens = Self::tokenize(&entry.title);
        doc_tokens.extend(Self::tokenize(&entry.content));
        for tag in &entry.tags {
            doc_tokens.extend(Self::tokenize(tag));
        }

        if doc_tokens.is_empty() {
            return 0.0;
        }

        let doc_set: HashSet<String> = doc_tokens.iter().cloned().collect();
        let overlap_count = query_tokens.iter().filter(|t| doc_set.contains(*t)).count();
        let overlap = u16::try_from(overlap_count).unwrap_or(u16::MAX);
        let token_count = u16::try_from(query_tokens.len()).unwrap_or(u16::MAX);
        f32::from(overlap) / f32::from(token_count)
    }

    pub(super) fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }
        let mut dot = 0.0_f32;
        let mut norm_a = 0.0_f32;
        let mut norm_b = 0.0_f32;
        for idx in 0..a.len() {
            dot += a[idx] * b[idx];
            norm_a += a[idx] * a[idx];
            norm_b += b[idx] * b[idx];
        }
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a.sqrt() * norm_b.sqrt())
        }
    }
}
