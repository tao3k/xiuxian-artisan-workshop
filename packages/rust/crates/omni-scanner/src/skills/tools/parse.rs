use std::collections::HashMap;
use std::fs;
use std::path::Path;

use omni_ast::{DecoratedFunction, TreeSitterPythonParser};
use sha2::{Digest, Sha256};

use crate::skills::metadata::{DecoratorArgs, ToolRecord};
use crate::skills::skill_command::annotations::build_annotations;
use crate::skills::skill_command::category::infer_category_from_skill;
use crate::skills::skill_command::parser::ParsedParameter;

use super::ToolsScanner;
use super::schema::generate_input_schema;

fn read_script_content(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) => match fs::read(path) {
            Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
            Err(_) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to read file as UTF-8: {e}"),
            ))),
        },
    }
}

fn content_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

fn collect_docstrings(decorated_funcs: &[DecoratedFunction]) -> HashMap<String, String> {
    let mut func_docstrings = HashMap::new();
    for func in decorated_funcs {
        if !func.docstring.is_empty() {
            func_docstrings.insert(func.name.clone(), func.docstring.clone());
        }
    }
    func_docstrings
}

fn parse_decorated_tools(
    content: &str,
    file_path: &str,
    skill_name: &str,
    skill_keywords: &[String],
    skill_intents: &[String],
) -> Vec<ToolRecord> {
    let mut parser = TreeSitterPythonParser::new();
    let decorated_funcs: Vec<DecoratedFunction> =
        parser.find_decorated_functions(content, "skill_command");

    if !decorated_funcs.is_empty() {
        log::debug!(
            "ToolsScanner: Found {} @skill_command decorated functions in {}",
            decorated_funcs.len(),
            file_path
        );
    }

    let file_hash = content_sha256(content);
    let func_docstrings = collect_docstrings(&decorated_funcs);
    let mut tools = Vec::new();

    for func in &decorated_funcs {
        let decorator_args = func.decorator.as_ref().map(|d| &d.arguments);
        let tool_name = decorator_args
            .and_then(|a| a.name.clone())
            .unwrap_or_else(|| func.name.clone());

        let docstring = func_docstrings.get(&func.name).cloned().unwrap_or_default();

        let description = match decorator_args.and_then(|a| a.description.clone()) {
            Some(desc) => desc,
            None if !docstring.is_empty() => docstring.clone(),
            _ => format!("Execute {skill_name}.{tool_name}"),
        };

        let category = decorator_args
            .and_then(|a| a.category.clone())
            .unwrap_or_else(|| infer_category_from_skill(skill_name));

        let parameters: Vec<ParsedParameter> = func
            .parameters
            .iter()
            .map(|p| ParsedParameter {
                name: p.name.clone(),
                type_annotation: p.type_annotation.clone(),
                has_default: p.default_value.is_some(),
                default_value: p.default_value.clone(),
            })
            .collect();

        let resource_uri = decorator_args.and_then(|a| a.resource_uri.clone());
        let decorator_args = match decorator_args {
            Some(ts_args) => DecoratorArgs {
                name: ts_args.name.clone(),
                description: ts_args.description.clone(),
                category: ts_args.category.clone(),
                destructive: ts_args.destructive,
                read_only: ts_args.read_only,
                resource_uri: ts_args.resource_uri.clone(),
            },
            None => DecoratorArgs::default(),
        };
        let annotations = build_annotations(&decorator_args, &func.name, &parameters);

        let input_schema = generate_input_schema(&parameters, &description);

        let mut combined_keywords = vec![skill_name.to_string(), tool_name.clone()];
        combined_keywords.extend(skill_keywords.iter().cloned());

        tools.push(ToolRecord::with_enrichment(
            format!("{skill_name}.{tool_name}"),
            description,
            skill_name.to_string(),
            file_path.to_string(),
            func.name.clone(),
            "script".to_string(),
            combined_keywords,
            skill_intents.to_vec(),
            file_hash.clone(),
            docstring,
            category,
            annotations,
            &parameters,
            input_schema,
            Vec::new(),
            resource_uri.unwrap_or_default(),
        ));
    }

    tools
}

impl ToolsScanner {
    /// Parse a single script file for tool definitions.
    ///
    /// Uses tree-sitter for robust parsing of @`skill_command` decorated functions
    /// with proper handling of triple-quoted strings in decorator arguments.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Python script file
    /// * `skill_name` - Name of the parent skill
    /// * `skill_keywords` - Routing keywords from SKILL.md
    /// * `skill_intents` - Intents from SKILL.md
    ///
    /// # Returns
    ///
    /// A vector of `ToolRecord` objects with enriched metadata.
    pub(super) fn parse_script(
        &self,
        path: &Path,
        skill_name: &str,
        skill_keywords: &[String],
        skill_intents: &[String],
    ) -> Result<Vec<ToolRecord>, Box<dyn std::error::Error>> {
        let _ = self;
        let content = read_script_content(path)?;
        let file_path = path.to_string_lossy().to_string();
        Ok(parse_decorated_tools(
            &content,
            &file_path,
            skill_name,
            skill_keywords,
            skill_intents,
        ))
    }

    /// Parse script content directly without reading from disk.
    ///
    /// Uses tree-sitter for robust parsing of @`skill_command` decorated functions
    /// with proper handling of triple-quoted strings in decorator arguments.
    ///
    /// # Arguments
    ///
    /// * `content` - The Python script content as a string
    /// * `file_path` - Virtual file path (for metadata/logging only)
    /// * `skill_name` - Name of the parent skill
    /// * `skill_keywords` - Routing keywords from SKILL.md
    /// * `skill_intents` - Intents from SKILL.md
    ///
    /// # Returns
    ///
    /// A vector of `ToolRecord` objects with enriched metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when `file_path` is empty.
    pub fn parse_content(
        &self,
        content: &str,
        file_path: &str,
        skill_name: &str,
        skill_keywords: &[String],
        skill_intents: &[String],
    ) -> Result<Vec<ToolRecord>, Box<dyn std::error::Error>> {
        let _ = self;
        if file_path.trim().is_empty() {
            return Err("file_path cannot be empty".into());
        }

        Ok(parse_decorated_tools(
            content,
            file_path,
            skill_name,
            skill_keywords,
            skill_intents,
        ))
    }
}
