//! Knowledge Scanner - Scans knowledge documents with YAML frontmatter.
//!
//! This module provides the `KnowledgeScanner` struct which scans knowledge
//! directories and extracts metadata from markdown files with YAML frontmatter.
//!
//! # Architecture
//!
//! Knowledge documents follow a simple structure:
//! - `*.md` files in knowledge directories
//! - YAML frontmatter for metadata (category, tags, title, etc.)
//!
//! # Example
//!
//! ```ignore
//! use xiuxian_skills::knowledge::KnowledgeScanner;
//!
//! let scanner = KnowledgeScanner::new();
//! let entries = scanner.scan_all(PathBuf::from("assets/knowledge")).unwrap();
//!
//! for entry in entries {
//!     println!("Knowledge: {} - {:?}", entry.title, entry.category);
//! }
//! ```

mod document;
mod metadata;
mod scan;

/// Knowledge Scanner - Scans and indexes knowledge documents.
///
/// Scans knowledge directories to extract:
/// - Title, description, category from frontmatter
/// - Tags for semantic search
/// - File hashes for incremental indexing
///
/// # Usage
///
/// ```ignore
/// use xiuxian_skills::knowledge::KnowledgeScanner;
///
/// let scanner = KnowledgeScanner::new();
///
/// // Scan single directory
/// let entries = scanner.scan_all(PathBuf::from("assets/knowledge")).unwrap();
///
/// // Scan with filtering
/// let patterns = scanner.scan_category(PathBuf::from("assets/knowledge"), "pattern").unwrap();
/// ```
#[derive(Debug)]
pub struct KnowledgeScanner;
