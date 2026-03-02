use super::DiscoveryOptions;
use std::collections::BTreeSet;
use std::path::Path;

/// Common extension policy for incremental sync routing.
///
/// This policy is intentionally domain-agnostic so consumers can drive
/// markdown/org/template/config synchronization using one shared filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalSyncPolicy {
    extensions: Vec<String>,
}

impl IncrementalSyncPolicy {
    /// Create policy from extension list.
    ///
    /// Empty input falls back to default sync extensions.
    #[must_use]
    pub fn new(extensions: &[String]) -> Self {
        let normalized = normalize_extensions(extensions);
        if normalized.is_empty() {
            return Self {
                extensions: DiscoveryOptions::default().extensions,
            };
        }
        Self {
            extensions: normalized,
        }
    }

    /// Build policy by extracting extension hints from glob patterns.
    ///
    /// Falls back to `fallback_extensions` when no extension can be extracted.
    #[must_use]
    pub fn from_glob_patterns(patterns: &[String], fallback_extensions: &[&str]) -> Self {
        let mut extensions = extract_extensions_from_glob_patterns(patterns);
        if extensions.is_empty() {
            extensions = fallback_extensions
                .iter()
                .filter_map(|value| normalize_extension(value))
                .collect();
        }
        Self::new(&extensions)
    }

    /// Build policy with explicit extension overrides.
    ///
    /// Precedence:
    /// 1) `explicit_extensions` (if not empty after normalization)
    /// 2) extensions extracted from `patterns`
    /// 3) `fallback_extensions`
    #[must_use]
    pub fn from_patterns_and_extensions(
        patterns: &[String],
        explicit_extensions: &[String],
        fallback_extensions: &[&str],
    ) -> Self {
        let explicit = normalize_extensions(explicit_extensions);
        if !explicit.is_empty() {
            return Self {
                extensions: explicit,
            };
        }
        Self::from_glob_patterns(patterns, fallback_extensions)
    }

    /// Returns normalized extension allowlist (lowercased, without leading dot).
    #[must_use]
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }

    /// Returns true when `path` has a supported extension.
    #[must_use]
    pub fn supports_path(&self, path: &Path) -> bool {
        let Some(extension) = extension_from_path(path) else {
            return false;
        };
        self.extensions.iter().any(|allowed| allowed == &extension)
    }
}

/// Extract lowercased extension from path (without dot).
#[must_use]
pub fn extension_from_path(path: &Path) -> Option<String> {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .and_then(normalize_extension)
}

/// Normalize extension token from user input.
#[must_use]
pub fn normalize_extension(raw: &str) -> Option<String> {
    let normalized = raw.trim().trim_start_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    if normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        Some(normalized)
    } else {
        None
    }
}

/// Extract extension hints from glob patterns (e.g. `"**/*.md"` -> `"md"`).
#[must_use]
pub fn extract_extensions_from_glob_patterns(patterns: &[String]) -> Vec<String> {
    let mut values = BTreeSet::new();
    for pattern in patterns {
        let normalized = pattern.trim();
        if normalized.is_empty() {
            continue;
        }
        if let Some(index) = normalized.rfind("*.") {
            let token = &normalized[index + 2..];
            if let Some(brace_payload) = token.strip_prefix('{')
                && let Some(end_index) = brace_payload.find('}')
            {
                for candidate in brace_payload[..end_index].split(',') {
                    if let Some(extension) = normalize_extension_token(candidate) {
                        values.insert(extension);
                    }
                }
                continue;
            }
            let simple_token = token
                .split(['/', '*', '?', '[', '{', ',', '}'])
                .next()
                .unwrap_or_default();
            if let Some(extension) = normalize_extension_token(simple_token) {
                values.insert(extension);
            }
        }
    }
    values.into_iter().collect()
}

fn normalize_extensions(extensions: &[String]) -> Vec<String> {
    let mut values = BTreeSet::new();
    for extension in extensions {
        if let Some(normalized) = normalize_extension(extension) {
            values.insert(normalized);
        }
    }
    values.into_iter().collect()
}

fn normalize_extension_token(raw: &str) -> Option<String> {
    let token = raw.trim();
    if token.is_empty() {
        return None;
    }
    let suffix = token.rsplit('.').next().unwrap_or(token);
    normalize_extension(suffix)
}
