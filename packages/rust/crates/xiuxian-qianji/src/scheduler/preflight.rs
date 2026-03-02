//! Pre-execution context preflight for semantic placeholder resolution.

use include_dir::Dir;
use serde_json::{Map, Value};
use std::sync::{OnceLock, RwLock};
use xiuxian_wendao::{
    WendaoResourceUri, embedded_discover_canonical_uris, embedded_resource_text_from_wendao_uri,
};
use xiuxian_zhenfa::ZhenfaTransmuter;

static RUNTIME_WENDAO_MOUNTS: OnceLock<RwLock<Vec<RuntimeWendaoMount>>> = OnceLock::new();

/// Runtime mount descriptor used by semantic URI resolution hooks.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeWendaoMount {
    /// Semantic skill name (host segment in `wendao://skills/<name>/...`).
    pub(crate) semantic_name: &'static str,
    /// Relative references root inside mounted embedded directory.
    pub(crate) references_dir: &'static str,
    /// Embedded directory providing referenced resources.
    pub(crate) dir: &'static Dir<'static>,
}

/// RAII guard that restores previous runtime mount registry on drop.
pub(crate) struct RuntimeWendaoMountGuard {
    previous: Vec<RuntimeWendaoMount>,
}

impl Drop for RuntimeWendaoMountGuard {
    fn drop(&mut self) {
        if let Ok(mut slot) = runtime_wendao_mounts().write() {
            *slot = std::mem::take(&mut self.previous);
        }
    }
}

/// Installs runtime mounts for this execution scope.
pub(crate) fn install_runtime_wendao_mounts(
    mounts: Vec<RuntimeWendaoMount>,
) -> RuntimeWendaoMountGuard {
    if let Ok(mut slot) = runtime_wendao_mounts().write() {
        let previous = std::mem::replace(&mut *slot, mounts);
        return RuntimeWendaoMountGuard { previous };
    }
    RuntimeWendaoMountGuard {
        previous: Vec::new(),
    }
}

fn runtime_wendao_mounts() -> &'static RwLock<Vec<RuntimeWendaoMount>> {
    RUNTIME_WENDAO_MOUNTS.get_or_init(|| RwLock::new(Vec::new()))
}

/// Resolves `$wendao://...` placeholders recursively before node execution.
///
/// # Errors
///
/// Returns an error when a placeholder token is empty or when one semantic URI
/// cannot be resolved from embedded Wendao resources.
pub(crate) fn resolve_wendao_placeholders_in_context(context: &Value) -> Result<Value, String> {
    resolve_value(context, context)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SemanticResolutionMode {
    Content,
    Reference,
}

fn resolve_value(value: &Value, context: &Value) -> Result<Value, String> {
    match value {
        Value::String(raw) => {
            resolve_string(raw, context, SemanticResolutionMode::Content).map(Value::String)
        }
        Value::Array(items) => items
            .iter()
            .map(|item| resolve_value(item, context))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(object) => {
            let mut resolved = Map::with_capacity(object.len());
            for (key, item) in object {
                resolved.insert(key.clone(), resolve_value(item, context)?);
            }
            Ok(Value::Object(resolved))
        }
        _ => Ok(value.clone()),
    }
}

fn resolve_string(
    raw: &str,
    context: &Value,
    mode: SemanticResolutionMode,
) -> Result<String, String> {
    let trimmed = raw.trim();
    let Some(token) = trimmed.strip_prefix('$') else {
        return match mode {
            SemanticResolutionMode::Content => Ok(raw.to_string()),
            SemanticResolutionMode::Reference => Ok(trimmed.to_string()),
        };
    };
    let token = token.trim();
    if token.is_empty() {
        return Err("semantic placeholder must not be empty".to_string());
    }

    if token.starts_with("wendao://") {
        return match mode {
            SemanticResolutionMode::Content => resolve_wendao_uri_with_zhenfa(token),
            SemanticResolutionMode::Reference => Ok(token.to_string()),
        };
    }

    if let Some(value) = lookup_context_path(context, token)
        && let Some(text) = context_value_to_text(value)
    {
        return Ok(text);
    }

    match mode {
        SemanticResolutionMode::Content => {
            if let Some(expanded) = resolve_dynamic_query_with_uri_expansion(token)? {
                return Ok(expanded);
            }
            Ok(raw.to_string())
        }
        SemanticResolutionMode::Reference => Ok(token.to_string()),
    }
}

/// Resolves a semantic placeholder (`$...`) as runtime content.
///
/// Resolution order:
/// 1. `$wendao://...` -> embedded semantic resource payload.
/// 2. `$context.path` -> current context value text.
/// 3. `$<query>` -> dynamic Wendao URI expansion XML-Lite.
/// 4. unresolved -> original raw input.
///
/// # Errors
///
/// Returns an error when the placeholder token is empty or when semantic
/// resource/query resolution fails.
pub(crate) fn resolve_semantic_content(raw: &str, context: &Value) -> Result<String, String> {
    resolve_string(raw, context, SemanticResolutionMode::Content)
}

/// Resolves a semantic placeholder (`$...`) as one symbolic reference value.
///
/// Resolution order:
/// 1. `$context.path` -> current context value text.
/// 2. `$wendao://...` -> canonical URI string (no dereference).
/// 3. unresolved -> token text (without `$`).
///
/// # Errors
///
/// Returns an error when the placeholder token is empty.
pub(crate) fn resolve_semantic_reference(raw: &str, context: &Value) -> Result<String, String> {
    resolve_string(raw, context, SemanticResolutionMode::Reference)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ContextPathSegment {
    Key(String),
    Index(usize),
}

fn parse_context_path(path: &str) -> Option<Vec<ContextPathSegment>> {
    if path.is_empty() {
        return None;
    }

    let bytes = path.as_bytes();
    let mut cursor = 0usize;
    let mut segments = Vec::new();

    while cursor < bytes.len() {
        if bytes[cursor] == b'.' {
            cursor += 1;
            continue;
        }

        if bytes[cursor] == b'[' {
            cursor += 1;
            let index_start = cursor;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
            if index_start == cursor || cursor >= bytes.len() || bytes[cursor] != b']' {
                return None;
            }
            let index_text = &path[index_start..cursor];
            let index = index_text.parse::<usize>().ok()?;
            segments.push(ContextPathSegment::Index(index));
            cursor += 1;
            continue;
        }

        let key_start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b'.' && bytes[cursor] != b'[' {
            cursor += 1;
        }
        let key = path[key_start..cursor].trim();
        if key.is_empty() {
            return None;
        }
        segments.push(ContextPathSegment::Key(key.to_string()));
    }

    if segments.is_empty() {
        None
    } else {
        Some(segments)
    }
}

/// Looks up one context value using a dot/bracket semantic path.
///
/// Examples: `agenda_steward_propose.output`, `hits[0].content`.
#[must_use]
pub(crate) fn lookup_context_path<'a>(context: &'a Value, path: &str) -> Option<&'a Value> {
    let segments = parse_context_path(path)?;
    let mut current = context;

    for segment in segments {
        match segment {
            ContextPathSegment::Key(key) => match current {
                Value::Object(map) => {
                    current = map.get(&key)?;
                }
                _ => return None,
            },
            ContextPathSegment::Index(index) => match current {
                Value::Array(items) => {
                    current = items.get(index)?;
                }
                _ => return None,
            },
        }
    }
    Some(current)
}

/// Converts one context value to non-empty text for semantic placeholder use.
#[must_use]
pub(crate) fn context_value_to_text(value: &Value) -> Option<String> {
    let text = match value {
        Value::String(raw) => raw.trim().to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    };
    if text.is_empty() { None } else { Some(text) }
}

/// Resolve one `wendao://` URI and delegate validation/refinement to Zhenfa.
pub(crate) fn resolve_wendao_uri_with_zhenfa(uri: &str) -> Result<String, String> {
    ZhenfaTransmuter::resolve_and_wash(uri, resolve_wendao_uri_text)
        .map_err(|error| error.to_string())
}

fn normalize_relative_path(path: &str) -> String {
    path.trim().trim_start_matches("./").replace('\\', "/")
}

fn resolve_wendao_uri_from_runtime_mounts(uri: &str) -> Option<String> {
    let parsed = WendaoResourceUri::parse(uri).ok()?;
    let semantic_name = parsed.semantic_name();
    let entity_relative_path =
        normalize_relative_path(parsed.entity_relative_path().to_string_lossy().as_ref());
    let mounts = runtime_wendao_mounts().read().ok()?;
    for mount in mounts.iter() {
        if !semantic_name.eq_ignore_ascii_case(mount.semantic_name) {
            continue;
        }
        let references_dir = normalize_relative_path(mount.references_dir);
        if references_dir.is_empty() {
            continue;
        }
        let candidate = format!("{references_dir}/{entity_relative_path}");
        let Some(content) = mount
            .dir
            .get_file(candidate.as_str())
            .and_then(include_dir::File::contents_utf8)
        else {
            continue;
        };
        return Some(content.to_string());
    }
    None
}

fn resolve_wendao_uri_text(uri: &str) -> Option<String> {
    resolve_wendao_uri_from_runtime_mounts(uri)
        .or_else(|| embedded_resource_text_from_wendao_uri(uri).map(str::to_string))
}

/// Attempts to resolve one semantic query into URI hits and returns aggregated
/// XML-Lite payload when any hit is found.
///
/// Returns `Ok(None)` when no URI can be discovered from the query.
pub(crate) fn resolve_dynamic_query_with_uri_expansion(
    query_expression: &str,
) -> Result<Option<String>, String> {
    let mut uris = embedded_discover_canonical_uris(query_expression)
        .map_err(|error| format!("semantic discovery failed for `{query_expression}`: {error}"))?;
    if uris.is_empty() {
        return Ok(None);
    }
    uris.sort();
    uris.dedup();

    let mut resources = Vec::with_capacity(uris.len());
    for uri in uris {
        let content = resolve_wendao_uri_with_zhenfa(uri.as_str())?;
        resources.push((uri, content));
    }

    let mut xml = String::new();
    xml.push_str("<wendao_query_result>");
    xml.push_str("<query>");
    xml.push_str(escape_xml_lite(query_expression).as_str());
    xml.push_str("</query>");
    xml.push_str("<hit_count>");
    xml.push_str(resources.len().to_string().as_str());
    xml.push_str("</hit_count>");
    xml.push_str("<resources>");
    for (uri, content) in resources {
        xml.push_str("<resource>");
        xml.push_str("<uri>");
        xml.push_str(escape_xml_lite(uri.as_str()).as_str());
        xml.push_str("</uri>");
        xml.push_str("<content>");
        xml.push_str(escape_xml_lite(content.as_str()).as_str());
        xml.push_str("</content>");
        xml.push_str("</resource>");
    }
    xml.push_str("</resources>");
    xml.push_str("</wendao_query_result>");

    Ok(Some(xml))
}

fn escape_xml_lite(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
