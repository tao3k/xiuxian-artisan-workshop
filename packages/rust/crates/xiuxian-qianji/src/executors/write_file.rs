//! Native file writing mechanism.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use crate::scheduler::preflight::resolve_semantic_content;
use async_trait::async_trait;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

/// Mechanism responsible for writing content to a local file path.
pub struct WriteFileMechanism {
    /// Destination path template (supports semantic placeholders and `{{key}}` interpolation).
    pub path: String,
    /// File content template (supports semantic placeholders and `{{key}}` interpolation).
    pub content: String,
    /// Context key used to store write metadata.
    pub output_key: String,
}

#[async_trait]
impl QianjiMechanism for WriteFileMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let resolved_path = render_template(&self.path, context)?;
        let resolved_content = render_template(&self.content, context)?;

        if resolved_path.trim().is_empty() {
            return Err("write_file path resolved to an empty value".to_string());
        }
        if resolved_path.contains("{{") || resolved_path.contains("}}") {
            return Err(format!(
                "write_file path contains unresolved template tokens: `{resolved_path}`"
            ));
        }

        let destination =
            resolve_destination_path(Path::new(resolved_path.as_str()), resolve_root_dir(context))?;

        fs::write(&destination, resolved_content.as_bytes()).map_err(|error| {
            format!(
                "write_file failed to write `{}`: {error}",
                destination.display()
            )
        })?;

        Ok(QianjiOutput {
            data: json!({
                self.output_key.clone(): {
                    "path": destination.display().to_string(),
                    "bytes_written": resolved_content.len()
                }
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

fn render_template(raw: &str, context: &serde_json::Value) -> Result<String, String> {
    let semantic = resolve_semantic_content(raw, context)?;
    Ok(interpolate_braced_placeholders(semantic.as_str(), context))
}

fn interpolate_braced_placeholders(raw: &str, context: &serde_json::Value) -> String {
    let mut rendered = String::with_capacity(raw.len());
    let mut remaining = raw;

    while let Some(open_index) = remaining.find("{{") {
        rendered.push_str(&remaining[..open_index]);
        let after_open = &remaining[open_index + 2..];
        let Some(close_index) = after_open.find("}}") else {
            rendered.push_str(&remaining[open_index..]);
            return rendered;
        };

        let token = after_open[..close_index].trim();
        if token.is_empty() {
            rendered.push_str("{{}}");
        } else if let Some(value) = lookup_context_value(context, token) {
            rendered.push_str(context_value_to_text(value).as_str());
        } else {
            rendered.push_str("{{");
            rendered.push_str(token);
            rendered.push_str("}}");
        }

        remaining = &after_open[close_index + 2..];
    }

    rendered.push_str(remaining);
    rendered
}

fn lookup_context_value<'a>(
    context: &'a serde_json::Value,
    key_path: &str,
) -> Option<&'a serde_json::Value> {
    let mut current = context;
    for segment in key_path.split('.') {
        let key = segment.trim();
        if key.is_empty() {
            continue;
        }
        current = current.get(key)?;
    }
    Some(current)
}

fn context_value_to_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Bool(flag) => flag.to_string(),
        serde_json::Value::Number(number) => number.to_string(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => value.to_string(),
    }
}

fn resolve_root_dir(context: &serde_json::Value) -> Option<PathBuf> {
    for key in ["project_root", "repo_root", "notebook_root"] {
        if let Some(text) = context.get(key).and_then(serde_json::Value::as_str) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        }
    }
    None
}

fn resolve_destination_path(
    destination: &Path,
    root_dir: Option<PathBuf>,
) -> Result<PathBuf, String> {
    let resolved = if let Some(root) = root_dir.as_ref() {
        if destination.is_absolute() {
            destination.to_path_buf()
        } else {
            root.join(destination)
        }
    } else {
        destination.to_path_buf()
    };

    let Some(parent) = resolved.parent() else {
        return Ok(resolved);
    };

    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "write_file failed to create parent directory `{}`: {error}",
            parent.display()
        )
    })?;

    let Some(root) = root_dir else {
        return Ok(resolved);
    };

    let canonical_root = fs::canonicalize(&root).map_err(|error| {
        format!(
            "write_file failed to canonicalize root directory `{}`: {error}",
            root.display()
        )
    })?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| {
        format!(
            "write_file failed to canonicalize parent directory `{}`: {error}",
            parent.display()
        )
    })?;

    if !canonical_parent.starts_with(&canonical_root) {
        return Err(format!(
            "write_file path escapes root directory: destination=`{}`, root=`{}`",
            resolved.display(),
            canonical_root.display()
        ));
    }

    Ok(resolved)
}
