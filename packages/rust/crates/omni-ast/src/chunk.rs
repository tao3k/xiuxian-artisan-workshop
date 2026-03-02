//! AST-based code chunking for semantic partitioning.
//!
//! Provides functions to split source code into semantic chunks based on
//! AST patterns, enabling high-quality knowledge base construction.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::lang::Lang;
use crate::python::extract_python_docstring;
use crate::re_exports::{LanguageExt, MatcherExt, MetaVariable, Pattern, SupportLang};

/// Code chunk for semantic partitioning
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeChunk {
    /// Chunk identifier
    pub id: String,
    /// Chunk type (function, class, etc.)
    pub chunk_type: String,
    /// Raw code content
    pub content: String,
    /// Byte offset start
    pub start: usize,
    /// Byte offset end
    pub end: usize,
    /// Line number start (1-indexed)
    pub line_start: usize,
    /// Line number end (1-indexed)
    pub line_end: usize,
    /// Captured metadata (function name, class name, etc.)
    pub metadata: HashMap<String, String>,
    /// Docstring/comment content
    pub docstring: Option<String>,
}

/// Chunk a file into semantic units based on AST patterns.
///
/// # Arguments
/// * `content` - Source code content
/// * `file_path` - Path to the file (for ID generation)
/// * `lang` - Programming language
/// * `patterns` - AST patterns to match (e.g., `["def $NAME", "class $NAME"]`)
/// * `min_lines` - Minimum lines for a chunk to be included
/// * `max_lines` - Maximum lines for a chunk (splits large chunks, 0 = no limit)
///
/// # Returns
/// Vector of `CodeChunk` objects.
///
/// # Errors
/// Returns an error when language or pattern parsing fails.
pub fn chunk_code(
    content: &str,
    file_path: &str,
    lang: Lang,
    patterns: &[&str],
    min_lines: usize,
    max_lines: usize,
) -> Result<Vec<CodeChunk>> {
    let lang_str = lang.as_str();
    let support_lang: SupportLang = lang_str
        .parse()
        .with_context(|| format!("Failed to parse language: {lang_str}"))?;

    let grep_result = support_lang.ast_grep(content);
    let root_node = grep_result.root();

    let mut chunks = Vec::new();
    let file_name = Path::new(file_path)
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    for (chunk_idx, pattern) in patterns.iter().enumerate() {
        let search_pattern = Pattern::try_new(pattern, support_lang)
            .with_context(|| format!("Failed to parse pattern: {pattern}"))?;

        // Determine chunk type from pattern
        let chunk_type = detect_chunk_type(pattern, chunk_idx);

        for node in root_node.dfs() {
            if let Some(m) = search_pattern.match_node(node.clone()) {
                let range = m.range();
                let start = range.start;
                let end = range.end;

                // Calculate line numbers
                let line_start = content[..start].lines().count() + 1;
                let line_end = content[..end].lines().count();

                // Skip if too small
                if line_end - line_start + 1 < min_lines {
                    continue;
                }

                // Extract metadata from captures
                let mut metadata = HashMap::new();
                let env = m.get_env();
                for mv in env.get_matched_variables() {
                    if let MetaVariable::Capture(name, _) = mv
                        && let Some(captured) = env.get_match(&name)
                    {
                        metadata.insert(name.clone(), captured.text().to_string());
                    }
                }

                // Generate chunk ID
                let id = generate_chunk_id(file_name, &chunk_type, &metadata, chunk_idx);

                // Extract docstring from the matched text
                let matched_text = m.text();
                let docstring = if lang == Lang::Python {
                    let doc = extract_python_docstring(&matched_text);
                    if doc.is_empty() { None } else { Some(doc) }
                } else {
                    None
                };

                // Clone chunk_type for the chunk struct
                let chunk_type_for_chunk = chunk_type.clone();

                let chunk = CodeChunk {
                    id,
                    chunk_type: chunk_type_for_chunk,
                    content: m.text().to_string(),
                    start,
                    end,
                    line_start,
                    line_end,
                    metadata,
                    docstring,
                };

                chunks.push(chunk);
            }
        }
    }

    // Sort by line number
    chunks.sort_by_key(|a| a.line_start);

    // Handle max_lines by splitting large chunks
    if max_lines > 0 {
        chunks = split_large_chunks(chunks, max_lines);
    }

    Ok(chunks)
}

/// Detect chunk type from pattern string
fn detect_chunk_type(pattern: &str, idx: usize) -> String {
    if pattern.contains("def $NAME") || pattern.contains("function $NAME") {
        "function".to_string()
    } else if pattern.contains("class $NAME") {
        "class".to_string()
    } else if pattern.contains("interface $NAME") {
        "interface".to_string()
    } else if pattern.contains("struct $NAME") {
        "struct".to_string()
    } else if pattern.contains("const $NAME") || pattern.contains("let $NAME") {
        "variable".to_string()
    } else if pattern.contains("fn $NAME") {
        "function".to_string()
    } else {
        format!("chunk_{idx}")
    }
}

/// Generate unique chunk ID
fn generate_chunk_id(
    file_name: &str,
    chunk_type: &str,
    metadata: &HashMap<String, String>,
    idx: usize,
) -> String {
    if let Some(name) = metadata.get("NAME") {
        format!("{file_name}_{chunk_type}_{name}")
    } else {
        format!("{file_name}_{chunk_type}_{idx}")
    }
}

/// Split large chunks into smaller parts
fn split_large_chunks(chunks: Vec<CodeChunk>, max_lines: usize) -> Vec<CodeChunk> {
    let mut result = Vec::new();

    for chunk in chunks {
        if chunk.line_end - chunk.line_start + 1 > max_lines {
            result.extend(split_chunk(&chunk, max_lines));
        } else {
            result.push(chunk);
        }
    }

    result
}

/// Split a large chunk into smaller parts
fn split_chunk(chunk: &CodeChunk, max_lines: usize) -> Vec<CodeChunk> {
    let total_lines = chunk.line_end - chunk.line_start + 1;
    if total_lines <= max_lines {
        return vec![chunk.clone()];
    }

    let lines: Vec<&str> = chunk.content.lines().collect();
    let num_parts = total_lines.div_ceil(max_lines);
    let mut parts = Vec::new();

    for i in 0..num_parts {
        let start_line = i * max_lines;
        let end_line = std::cmp::min((i + 1) * max_lines, total_lines);

        if start_line >= lines.len() {
            break;
        }

        let part_content = lines[start_line..end_line].join("\n");
        let trimmed_content = part_content.trim_end();
        let trimmed_len = trimmed_content.len();

        let part_start = chunk.start + (part_content.len() - trimmed_len);
        let part_end = chunk.start + part_content.len();

        let part = CodeChunk {
            id: format!("{}_part_{}", chunk.id, i),
            chunk_type: chunk.chunk_type.clone(),
            content: trimmed_content.to_string(),
            start: part_start,
            end: part_end,
            line_start: chunk.line_start + start_line,
            line_end: chunk.line_start + end_line - 1,
            metadata: chunk.metadata.clone(),
            docstring: if i == 0 {
                chunk.docstring.clone()
            } else {
                None
            },
        };

        parts.push(part);
    }

    parts
}
