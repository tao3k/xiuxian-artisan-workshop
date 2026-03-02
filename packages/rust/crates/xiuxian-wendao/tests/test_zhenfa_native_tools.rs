//! Integration tests for Wendao native zhenfa tool implementations.
#![cfg(feature = "zhenfa-router")]

use std::fs;
use std::sync::Arc;

use serde_json::json;
use tempfile::TempDir;
use xiuxian_wendao::{
    LinkGraphIndex,
    zhenfa_router::{WendaoContextExt, WendaoSearchTool},
};
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, ZhenfaTool};

fn build_notebook_fixture() -> TempDir {
    let temp_dir = TempDir::new().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let alpha_note = temp_dir.path().join("alpha.md");
    fs::write(
        &alpha_note,
        "# Native Tool\n\nWendao native zhenfa tool should search this document.\n",
    )
    .unwrap_or_else(|error| panic!("write alpha note: {error}"));
    temp_dir
}

fn context_with_index(root: &std::path::Path) -> ZhenfaContext {
    let index = Arc::new(
        LinkGraphIndex::build(root)
            .unwrap_or_else(|error| panic!("build link graph index: {error}")),
    );
    let mut ctx = ZhenfaContext::default();
    let _ = ctx.insert_shared_extension(Arc::clone(&index));
    ctx
}

#[tokio::test]
async fn wendao_search_tool_executes_native_dispatch() {
    let notebook = build_notebook_fixture();
    let tool = WendaoSearchTool;
    let ctx = context_with_index(notebook.path());

    let output = tool
        .call_native(
            &ctx,
            json!({
                "query": "native zhenfa tool",
                "root_dir": notebook.path().to_string_lossy().to_string(),
                "limit": 5
            }),
        )
        .await
        .unwrap_or_else(|error| panic!("native dispatch should succeed: {error}"));

    assert!(output.contains("<hit id=\"alpha.md\""));
    assert!(output.contains("Native Tool"));
}

#[tokio::test]
async fn wendao_search_tool_requires_link_graph_index_extension() {
    let tool = WendaoSearchTool;
    let error = tool
        .call_native(
            &ZhenfaContext::default(),
            json!({
                "query": "native zhenfa tool",
                "limit": 5
            }),
        )
        .await;
    let error = match error {
        Ok(output) => panic!("missing index extension should fail, received output: {output}"),
        Err(error) => error,
    };
    assert!(matches!(error, ZhenfaError::Execution { .. }));
}

#[tokio::test]
async fn wendao_search_tool_reuses_injected_index_after_source_removed() {
    let notebook = build_notebook_fixture();
    let tool = WendaoSearchTool;
    let ctx = context_with_index(notebook.path());

    let first = tool
        .call_native(
            &ctx,
            json!({
                "query": "search this document",
                "limit": 5
            }),
        )
        .await
        .unwrap_or_else(|error| panic!("first native dispatch should succeed: {error}"));
    assert!(first.contains("<hit id=\"alpha.md\""));
    assert!(first.contains("Native Tool"));

    fs::remove_file(notebook.path().join("alpha.md"))
        .unwrap_or_else(|error| panic!("remove fixture note: {error}"));

    let second = tool
        .call_native(
            &ctx,
            json!({
                "query": "search this document",
                "limit": 5
            }),
        )
        .await
        .unwrap_or_else(|error| {
            panic!("second native dispatch should use injected index: {error}")
        });

    assert!(second.contains("<hit id=\"alpha.md\""));
    assert!(second.contains("Native Tool"));
}

#[tokio::test]
async fn wendao_search_tool_emits_semantic_hit_type_for_journal_paths() {
    let notebook = TempDir::new().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let journal_dir = notebook.path().join("journal");
    fs::create_dir_all(&journal_dir).unwrap_or_else(|error| panic!("create journal dir: {error}"));
    fs::write(
        journal_dir.join("daily.md"),
        "# Daily Journal\n\njournal semantic type marker unique-native-tool.\n",
    )
    .unwrap_or_else(|error| panic!("write journal note: {error}"));

    let tool = WendaoSearchTool;
    let ctx = context_with_index(notebook.path());
    let output = tool
        .call_native(
            &ctx,
            json!({
                "query": "unique-native-tool",
                "limit": 5
            }),
        )
        .await
        .unwrap_or_else(|error| panic!("native dispatch should classify journal path: {error}"));

    assert!(output.contains("<hit id=\"journal/daily.md\""));
    assert!(output.contains("type=\"journal\""));
}

#[tokio::test]
async fn wendao_search_tool_prefers_tag_driven_hit_type_for_ambiguous_paths() {
    let notebook = TempDir::new().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let notes_dir = notebook.path().join("notes");
    fs::create_dir_all(&notes_dir).unwrap_or_else(|error| panic!("create notes dir: {error}"));
    fs::write(
        notes_dir.join("entry.md"),
        "---\ntags:\n  - journal\n---\n# Entry\n\ntag-driven-classification-marker.\n",
    )
    .unwrap_or_else(|error| panic!("write tagged note: {error}"));

    let tool = WendaoSearchTool;
    let ctx = context_with_index(notebook.path());
    let output = tool
        .call_native(
            &ctx,
            json!({
                "query": "tag-driven-classification-marker",
                "limit": 5
            }),
        )
        .await
        .unwrap_or_else(|error| panic!("native dispatch should classify from tags: {error}"));

    assert!(output.contains("<hit id=\"notes/entry.md\""));
    assert!(output.contains("type=\"journal\""));
}

#[tokio::test]
async fn wendao_search_tool_prefers_frontmatter_type_over_path_and_tags() {
    let notebook = TempDir::new().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let journal_dir = notebook.path().join("journal");
    fs::create_dir_all(&journal_dir).unwrap_or_else(|error| panic!("create journal dir: {error}"));
    fs::write(
        journal_dir.join("override.md"),
        "---\ntype: agenda\ntags:\n  - journal\n---\n# Override\n\ndoc-type-precedence-marker.\n",
    )
    .unwrap_or_else(|error| panic!("write typed note: {error}"));

    let tool = WendaoSearchTool;
    let ctx = context_with_index(notebook.path());
    let output = tool
        .call_native(
            &ctx,
            json!({
                "query": "doc-type-precedence-marker",
                "limit": 5
            }),
        )
        .await
        .unwrap_or_else(|error| {
            panic!("native dispatch should classify from frontmatter type: {error}")
        });

    assert!(output.contains("<hit id=\"journal/override.md\""));
    assert!(output.contains("type=\"agenda\""));
}

#[test]
fn wendao_search_tool_cache_key_is_stable_for_equivalent_option_objects() {
    let notebook = build_notebook_fixture();
    let tool = WendaoSearchTool;
    let ctx = context_with_index(notebook.path());

    let args_one = json!({
        "query": "native zhenfa tool",
        "limit": 5,
        "options": {
            "match_strategy": "fts",
            "case_sensitive": false,
            "sort_terms": [{ "field": "score", "order": "desc" }],
            "filters": {},
            "created_after": null,
            "created_before": null,
            "modified_after": null,
            "modified_before": null
        }
    });
    let args_two = json!({
        "limit": 5,
        "query": "native zhenfa tool",
        "options": {
            "created_before": null,
            "created_after": null,
            "match_strategy": "fts",
            "sort_terms": [{ "order": "desc", "field": "score" }],
            "case_sensitive": false,
            "filters": {},
            "modified_before": null,
            "modified_after": null
        }
    });

    let key_one = tool
        .cache_key(&ctx, &args_one)
        .unwrap_or_else(|| panic!("cache key should be generated"));
    let key_two = tool
        .cache_key(&ctx, &args_two)
        .unwrap_or_else(|| panic!("cache key should be generated"));
    assert_eq!(key_one, key_two);
}

#[test]
fn wendao_context_ext_builds_skill_asset_request() {
    let request = ZhenfaContext::default()
        .skill_asset("agenda-management", "teacher.md")
        .unwrap_or_else(|error| panic!("skill asset request should build: {error}"));
    assert_eq!(
        request.uri(),
        "wendao://skills/agenda-management/references/teacher.md"
    );
}
