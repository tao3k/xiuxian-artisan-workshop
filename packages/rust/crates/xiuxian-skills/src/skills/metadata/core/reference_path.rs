use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

/// A validated relative path to a reference document (md, pdf, txt, html, json, yaml, yml).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemarsJsonSchema)]
#[serde(try_from = "String", into = "String")]
pub struct ReferencePath(String);

impl ReferencePath {
    const VALID_EXTENSIONS: &[&str] = &["md", "pdf", "txt", "html", "json", "yaml", "yml"];

    /// Creates a new `ReferencePath` after validating the path format.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is empty, absolute, contains `..`,
    /// or has an unsupported file extension.
    pub fn new(path: impl Into<String>) -> Result<Self, String> {
        let path = path.into();
        if path.trim().is_empty() {
            return Err("Reference path cannot be empty".to_string());
        }
        if path.starts_with('/') {
            return Err(format!("Reference path must be relative: {path}"));
        }
        if path.contains("..") {
            return Err(format!("Reference path cannot contain '..': {path}"));
        }
        let extension = path.rsplit('.').next().unwrap_or("");
        if !extension.is_empty() && !Self::VALID_EXTENSIONS.contains(&extension) {
            return Err(format!("Invalid reference extension '{extension}'"));
        }
        Ok(Self(path))
    }

    /// Returns the reference path as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ReferencePath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl From<ReferencePath> for String {
    fn from(value: ReferencePath) -> Self {
        value.0
    }
}

impl TryFrom<String> for ReferencePath {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
