//! Centralized transmutation and structural validation for LLM-bound text.

use crate::xml_lite::{extract_tag_f32, extract_tag_value};
use thiserror::Error;

/// Structural validation failures detected by the Zhenfa transmuter.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ZhenfaTransmuterError {
    /// Input contains null bytes and is rejected before model ingestion.
    #[error("input contains null bytes")]
    NullByteDetected,
    /// Closing tag did not match the latest opening tag.
    #[error("mismatched XML-Lite tag: expected </{expected}>, found </{found}>")]
    MismatchedClosingTag {
        /// The opening tag waiting to be closed.
        expected: String,
        /// The closing tag found in the payload.
        found: String,
    },
    /// Closing tag appeared without a corresponding opening tag.
    #[error("unexpected XML-Lite closing tag </{found}>")]
    UnexpectedClosingTag {
        /// The closing tag that could not be matched.
        found: String,
    },
    /// Input ended while some opening tags were still unclosed.
    #[error("unclosed XML-Lite tag <{tag}>")]
    UnclosedTag {
        /// The opening tag that remained on stack.
        tag: String,
    },
}

/// Failures for semantic URI resolution plus transmutation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ZhenfaResolveAndWashError {
    /// Semantic URI could not be resolved to any non-empty payload.
    #[error("semantic resource URI `{uri}` could not be resolved")]
    ResourceNotFound {
        /// Canonical semantic resource URI.
        uri: String,
    },
    /// Structural validation failed after semantic resolution.
    #[error(transparent)]
    Transmuter(#[from] ZhenfaTransmuterError),
}

impl ZhenfaTransmuterError {
    /// Returns one LLM-safe semantic summary for structural validation failures.
    #[must_use]
    pub fn llm_safe_message(&self) -> &'static str {
        match self {
            Self::NullByteDetected => {
                "content contains unsupported control characters; clean the payload and retry"
            }
            Self::MismatchedClosingTag { .. }
            | Self::UnexpectedClosingTag { .. }
            | Self::UnclosedTag { .. } => {
                "content has malformed XML-Lite structure; ensure all tags are balanced"
            }
        }
    }
}

/// Unified transmutation entry point used by Agent/Qianji before model feeding.
pub struct ZhenfaTransmuter;

impl ZhenfaTransmuter {
    /// Resolves one semantic URI with caller-provided resolver and applies
    /// structural validation plus LLM refinement.
    ///
    /// # Errors
    ///
    /// Returns [`ZhenfaResolveAndWashError::ResourceNotFound`] when the
    /// resolver cannot resolve a non-empty payload for `uri`.
    /// Returns [`ZhenfaResolveAndWashError::Transmuter`] when structural
    /// validation fails.
    pub fn resolve_and_wash<F>(uri: &str, resolver: F) -> Result<String, ZhenfaResolveAndWashError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let canonical_uri = uri.trim();
        let raw = resolver(canonical_uri)
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| ZhenfaResolveAndWashError::ResourceNotFound {
                uri: canonical_uri.to_string(),
            })?;
        let refined = Self::refine_for_llm(raw.as_str());
        if should_validate_xml_lite(canonical_uri) {
            Self::validate_structure(refined.as_str()).map_err(ZhenfaResolveAndWashError::from)?;
        }
        Ok(refined)
    }

    /// Extracts the first XML-Lite tag payload as owned text.
    #[must_use]
    pub fn get_tag_value(content: &str, tag: &str) -> Option<String> {
        extract_tag_value(content, tag).map(ToString::to_string)
    }

    /// Extracts the first XML-Lite tag payload and parses it as `f32`.
    #[must_use]
    pub fn get_tag_f32(content: &str, tag: &str) -> Option<f32> {
        extract_tag_f32(content, tag)
    }

    /// Applies lightweight normalization before data is sent to the model.
    ///
    /// This pass normalizes line endings, strips null bytes, trims trailing
    /// line whitespace, and collapses consecutive blank lines to at most two.
    #[must_use]
    pub fn refine_for_llm(content: &str) -> String {
        let normalized_line_endings = content.replace("\r\n", "\n").replace('\r', "\n");
        let sanitized = normalized_line_endings.replace('\0', "");

        let mut refined = String::with_capacity(sanitized.len());
        let mut blank_run = 0usize;
        for line in sanitized.lines() {
            let trimmed_end = line.trim_end();
            if trimmed_end.is_empty() {
                blank_run += 1;
                if blank_run > 2 {
                    continue;
                }
            } else {
                blank_run = 0;
            }

            if !refined.is_empty() {
                refined.push('\n');
            }
            refined.push_str(trimmed_end);
        }

        refined.trim().to_string()
    }

    /// Validates that XML-Lite-like tags are structurally balanced.
    ///
    /// Non-tag usages such as `1 < 2` are ignored by design.
    ///
    /// # Errors
    ///
    /// Returns [`ZhenfaTransmuterError`] when null bytes or malformed tag
    /// nesting is detected.
    pub fn validate_structure(content: &str) -> Result<(), ZhenfaTransmuterError> {
        if content.contains('\0') {
            return Err(ZhenfaTransmuterError::NullByteDetected);
        }

        let bytes = content.as_bytes();
        let mut cursor = 0usize;
        let mut stack: Vec<String> = Vec::new();

        while cursor < bytes.len() {
            if bytes[cursor] != b'<' {
                cursor += 1;
                continue;
            }

            if cursor + 1 >= bytes.len() {
                break;
            }

            if bytes[cursor + 1] == b'!' {
                if content[cursor..].starts_with("<!--") {
                    if let Some(offset) = content[cursor + 4..].find("-->") {
                        cursor = cursor + 4 + offset + 3;
                        continue;
                    }
                    return Err(ZhenfaTransmuterError::UnclosedTag {
                        tag: "!--".to_string(),
                    });
                }
                cursor += 1;
                continue;
            }

            if bytes[cursor + 1] == b'?' {
                if let Some(offset) = content[cursor + 2..].find("?>") {
                    cursor = cursor + 2 + offset + 2;
                    continue;
                }
                break;
            }

            let closing = bytes[cursor + 1] == b'/';
            let tag_start = if closing { cursor + 2 } else { cursor + 1 };
            if tag_start >= bytes.len() {
                break;
            }
            if !is_tag_name_start(bytes[tag_start]) {
                cursor += 1;
                continue;
            }

            let mut tag_end = tag_start + 1;
            while tag_end < bytes.len() && is_tag_name_char(bytes[tag_end]) {
                tag_end += 1;
            }
            let tag_name = &content[tag_start..tag_end];

            let mut angle_close = tag_end;
            while angle_close < bytes.len() && bytes[angle_close] != b'>' {
                angle_close += 1;
            }
            if angle_close >= bytes.len() {
                return Err(ZhenfaTransmuterError::UnclosedTag {
                    tag: tag_name.to_string(),
                });
            }

            let self_closing = !closing && angle_close > cursor && bytes[angle_close - 1] == b'/';
            if closing {
                match stack.pop() {
                    Some(expected) if expected == tag_name => {}
                    Some(expected) => {
                        return Err(ZhenfaTransmuterError::MismatchedClosingTag {
                            expected,
                            found: tag_name.to_string(),
                        });
                    }
                    None => {
                        return Err(ZhenfaTransmuterError::UnexpectedClosingTag {
                            found: tag_name.to_string(),
                        });
                    }
                }
            } else if !self_closing {
                stack.push(tag_name.to_string());
            }

            cursor = angle_close + 1;
        }

        if let Some(tag) = stack.pop() {
            return Err(ZhenfaTransmuterError::UnclosedTag { tag });
        }
        Ok(())
    }

    /// Performs light semantic checks for markdown assets passed through Zhenfa.
    ///
    /// Current checks enforce balanced `WikiLink` delimiters and mandatory semantic
    /// suffixes for `references/*` links.
    #[must_use]
    pub fn check_semantic_integrity(md: &str) -> bool {
        if md.contains('\0') {
            return false;
        }
        let open = md.match_indices("[[").count();
        let close = md.match_indices("]]").count();
        if open != close {
            return false;
        }

        let mut cursor = 0usize;
        while let Some(start) = md[cursor..].find("[[") {
            let absolute_start = cursor + start + 2;
            let Some(end_offset) = md[absolute_start..].find("]]") else {
                return false;
            };
            let absolute_end = absolute_start + end_offset;
            let body = md[absolute_start..absolute_end].trim();
            if body.starts_with("references/") && !body.contains('#') {
                return false;
            }
            cursor = absolute_end + 2;
        }
        true
    }

    /// Refines a payload and validates its structure before model ingestion.
    ///
    /// # Errors
    ///
    /// Returns [`ZhenfaTransmuterError`] if the refined payload fails structural
    /// validation.
    pub fn validate_and_refine(content: &str) -> Result<String, ZhenfaTransmuterError> {
        let refined = Self::refine_for_llm(content);
        Self::validate_structure(refined.as_str())?;
        Ok(refined)
    }
}

fn is_tag_name_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_tag_name_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':')
}

fn should_validate_xml_lite(uri: &str) -> bool {
    let extension = uri
        .rsplit('.')
        .next()
        .map(str::trim)
        .map(str::to_ascii_lowercase);
    matches!(extension.as_deref(), Some("xml" | "xml-lite" | "xlite"))
}
