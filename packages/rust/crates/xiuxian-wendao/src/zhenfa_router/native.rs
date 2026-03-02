use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use crate::{
    AssetRequest, LinkGraphIndex, LinkGraphSearchOptions, SkillVfsResolver, WendaoAssetHandle,
};

mod xml_lite;

const DEFAULT_SEARCH_LIMIT: usize = 20;
const MAX_SEARCH_LIMIT: usize = 200;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub(crate) struct WendaoSearchArgs {
    query: String,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    root_dir: Option<String>,
    #[serde(default)]
    options: Option<LinkGraphSearchOptions>,
    #[serde(default)]
    include_provisional: Option<bool>,
    #[serde(default)]
    provisional_limit: Option<usize>,
}

/// Typed extension accessors for Wendao native tools.
pub trait WendaoContextExt {
    /// Resolve the injected immutable `LinkGraph` index from zhenfa context.
    ///
    /// # Errors
    /// Returns execution error when the index is not present in context.
    fn link_graph_index(&self) -> Result<std::sync::Arc<LinkGraphIndex>, ZhenfaError>;

    /// Resolve the injected semantic skill VFS resolver from zhenfa context.
    ///
    /// # Errors
    /// Returns execution error when resolver is not present in context.
    fn vfs(&self) -> Result<std::sync::Arc<SkillVfsResolver>, ZhenfaError>;

    /// Builds one skill-scoped asset request.
    ///
    /// # Errors
    /// Returns execution error when semantic URI mapping arguments are invalid.
    fn skill_asset(
        &self,
        semantic_name: &str,
        relative_path: &str,
    ) -> Result<AssetRequest, ZhenfaError>;
}

impl WendaoContextExt for ZhenfaContext {
    fn link_graph_index(&self) -> Result<std::sync::Arc<LinkGraphIndex>, ZhenfaError> {
        self.get_extension::<LinkGraphIndex>().ok_or_else(|| {
            ZhenfaError::execution("missing LinkGraphIndex in zhenfa context extensions")
        })
    }

    fn vfs(&self) -> Result<std::sync::Arc<SkillVfsResolver>, ZhenfaError> {
        self.get_extension::<SkillVfsResolver>().ok_or_else(|| {
            ZhenfaError::execution("missing SkillVfsResolver in zhenfa context extensions")
        })
    }

    fn skill_asset(
        &self,
        semantic_name: &str,
        relative_path: &str,
    ) -> Result<AssetRequest, ZhenfaError> {
        WendaoAssetHandle::skill_reference_asset(semantic_name, relative_path).map_err(|error| {
            ZhenfaError::invalid_arguments(format!(
                "invalid skill asset mapping (`{semantic_name}`, `{relative_path}`): {error}"
            ))
        })
    }
}

/// Search the Wendao graph index and return stripped XML-Lite `<hit>` records.
#[zhenfa_tool(
    name = "wendao.search",
    description = "Search the Wendao graph index and return stripped XML-Lite <hit> records.",
    tool_struct = "WendaoSearchTool",
    cache_key = "wendao_search_cache_key"
)]
pub fn wendao_search(ctx: &ZhenfaContext, args: WendaoSearchArgs) -> Result<String, ZhenfaError> {
    let query = args.query.trim();
    if query.is_empty() {
        return Err(ZhenfaError::invalid_arguments(
            "`query` must be a non-empty string",
        ));
    }

    validate_root_dir_argument(args.root_dir.as_deref())?;
    let options = args.options.unwrap_or_default();
    let index = ctx.link_graph_index()?;
    let payload = index.search_planned_payload_with_agentic(
        query,
        normalize_limit(args.limit),
        options,
        args.include_provisional,
        args.provisional_limit,
    );
    Ok(xml_lite::render_xml_lite(&payload))
}

fn wendao_search_cache_key(ctx: &ZhenfaContext, args: &WendaoSearchArgs) -> Option<String> {
    let index = ctx.link_graph_index().ok()?;
    let query = args.query.trim();
    if query.is_empty() {
        return None;
    }
    let options = args.options.clone().unwrap_or_default();
    let canonical_payload = json!({
        "root": index.root().to_string_lossy(),
        "query": query,
        "limit": normalize_limit(args.limit),
        "include_provisional": args.include_provisional,
        "provisional_limit": args.provisional_limit,
        "options": options
    });
    Some(format!(
        "wendao.search::{}",
        canonical_json_string(canonical_payload)
    ))
}

fn normalize_limit(raw: Option<usize>) -> usize {
    raw.unwrap_or(DEFAULT_SEARCH_LIMIT)
        .clamp(1, MAX_SEARCH_LIMIT)
}

fn validate_root_dir_argument(root_dir: Option<&str>) -> Result<(), ZhenfaError> {
    if let Some(value) = root_dir {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ZhenfaError::invalid_arguments(
                "`root_dir` must be non-empty when provided",
            ));
        }
    }
    Ok(())
}

fn canonical_json_string(value: Value) -> String {
    match serde_json::to_string(&canonicalize_json(value)) {
        Ok(serialized) => serialized,
        Err(_error) => "{}".to_string(),
    }
}

fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map.into_iter().collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut canonical = serde_json::Map::new();
            for (key, nested) in entries {
                canonical.insert(key, canonicalize_json(nested));
            }
            Value::Object(canonical)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(canonicalize_json).collect()),
        other => other,
    }
}
