//! AST-based Security Scanning Mechanism.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use omni_ast::SecurityScanner;
use serde_json::json;
use std::fs;
use std::path::Path;

/// Mechanism responsible for statically analyzing code files for security violations.
pub struct SecurityScanMechanism {
    /// Context key containing a list of file paths to scan.
    pub files_key: String,
    /// Context key to output the list of violations.
    pub output_key: String,
    /// Whether to abort execution if any violation is found.
    pub abort_on_violation: bool,
    /// Context key for the working directory to resolve relative paths against.
    pub cwd_key: Option<String>,
}

#[async_trait]
impl QianjiMechanism for SecurityScanMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let mut file_paths = Vec::new();

        let files_val = context
            .get(&self.files_key)
            .ok_or_else(|| format!("Missing context key: {}", self.files_key))?;

        if let Some(arr) = files_val.as_array() {
            for v in arr {
                if let Some(s) = v.as_str() {
                    file_paths.push(s.to_string());
                }
            }
        } else if let Some(s) = files_val.as_str() {
            for line in s.split('\n') {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    file_paths.push(trimmed.to_string());
                }
            }
        } else {
            return Err(format!(
                "Context key {} must be a string or array",
                self.files_key
            ));
        }

        let base_dir = if let Some(cwd_key) = &self.cwd_key {
            context.get(cwd_key).and_then(|v| v.as_str()).map(Path::new)
        } else {
            None
        };

        let mut all_violations = Vec::new();
        let scanner = SecurityScanner::new();

        for file_str in file_paths {
            let mut path_buf = std::path::PathBuf::from(&file_str);
            if let Some(base) = base_dir {
                if path_buf.is_relative() {
                    path_buf = base.join(path_buf);
                }
            }

            // Read file if it exists (it might be a deleted staged file, so we skip reading errors softly)
            if path_buf.exists() && path_buf.is_file() {
                if let Ok(content) = fs::read_to_string(&path_buf) {
                    let file_violations = scanner.scan_all(&content);
                    for v in file_violations {
                        all_violations.push(json!({
                            "file": file_str,
                            "rule_id": v.rule_id,
                            "description": v.description,
                            "line": v.line,
                            "snippet": v.snippet,
                        }));
                    }
                }
            }
        }

        if !all_violations.is_empty() && self.abort_on_violation {
            return Ok(QianjiOutput {
                data: json!({ self.output_key.clone(): all_violations }),
                instruction: FlowInstruction::Abort("security_violation".to_string()),
            });
        }

        Ok(QianjiOutput {
            data: json!({ self.output_key.clone(): all_violations }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}
