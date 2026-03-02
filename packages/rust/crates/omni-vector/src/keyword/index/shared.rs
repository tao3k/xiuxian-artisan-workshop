// `KeywordIndex` - Tantivy wrapper for keyword search with `BM25`.

use std::cell::RefCell;
use std::path::Path;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, IndexRecordOption, Schema, TextOptions, Value};
use tantivy::tokenizer::{
    AsciiFoldingFilter, LowerCaser, RemoveLongFilter, SimpleTokenizer, TextAnalyzer,
};
use tantivy::{
    Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, TantivyError, Term, doc,
};

use crate::ToolSearchResult;
use crate::error::VectorStoreError;

/// `KeywordIndex` - Tantivy wrapper for keyword search with `BM25`.
/// Caches a single `IndexWriter` and reuses it across
/// `bulk_upsert` / `upsert_document` / `index_batch`
/// to avoid repeated writer creation and teardown.
pub struct KeywordIndex {
    /// Tantivy index for full-text search
    index: Index,
    /// Index reader for search operations
    reader: IndexReader,
    /// Cached writer reused across writes; one writer per index (Tantivy allows only one).
    writer_cache: RefCell<Option<IndexWriter>>,
    /// Field handle for tool name (used for exact matching and boosting)
    pub tool_name: Field,
    /// Field handle for tool description (used for relevance scoring)
    pub description: Field,
    /// Field handle for skill category (used for filtering)
    pub category: Field,
    /// Field handle for routing keywords (used for keyword matching)
    pub keywords: Field,
    /// Field handle for intents (used for semantic alignment)
    pub intents: Field,
}
