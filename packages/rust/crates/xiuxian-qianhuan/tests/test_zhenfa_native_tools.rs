//! Integration tests for Qianhuan native zhenfa tool implementations.
#![cfg(feature = "zhenfa-router")]

use std::sync::Arc;

use serde_json::json;
use tempfile::TempDir;
use xiuxian_qianhuan::ManifestationManager;
use xiuxian_qianhuan::zhenfa_router::{QianhuanReloadTool, QianhuanRenderTool};
use xiuxian_wendao::SkillVfsResolver;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaTool};

fn test_manager() -> ManifestationManager {
    ManifestationManager::new_with_embedded_templates(
        &[],
        &[(
            "daily_agenda.md",
            "Task: {{ task }}\nState: {{ qianhuan.state_context | default(value=\"\") }}",
        )],
    )
    .unwrap_or_else(|error| panic!("create manifestation manager for tests: {error}"))
}

fn semantic_skill_fixture() -> (TempDir, Arc<SkillVfsResolver>) {
    let temp =
        TempDir::new().unwrap_or_else(|error| panic!("create semantic fixture root: {error}"));
    let skill_root = temp.path().join("skills").join("zhixing");
    std::fs::create_dir_all(skill_root.join("references"))
        .unwrap_or_else(|error| panic!("create semantic references directory: {error}"));
    std::fs::write(
        skill_root.join("SKILL.md"),
        r#"---
name: agenda-management
description: "Agenda skill fixture"
---

# Agenda Management
"#,
    )
    .unwrap_or_else(|error| panic!("write semantic fixture skill descriptor: {error}"));
    std::fs::write(
        skill_root.join("references").join("draft_agenda.j2"),
        "Semantic Task: {{ task }}",
    )
    .unwrap_or_else(|error| panic!("write semantic fixture template: {error}"));
    let resolver = SkillVfsResolver::from_roots(&[temp.path().to_path_buf()])
        .unwrap_or_else(|error| panic!("build semantic fixture resolver: {error}"));
    (temp, Arc::new(resolver))
}

#[tokio::test]
async fn qianhuan_render_tool_executes_native_dispatch() {
    let tool = QianhuanRenderTool;
    let manager = Arc::new(test_manager());
    let mut ctx = ZhenfaContext::default();
    let _ = ctx.insert_shared_extension(Arc::clone(&manager));
    let output = tool
        .call_native(
            &ctx,
            json!({
                "target": "daily_agenda",
                "data": { "task": "refactor native bridge" },
                "runtime": { "state_context": "SUCCESS_STREAK" }
            }),
        )
        .await
        .unwrap_or_else(|error| panic!("qianhuan render should succeed: {error}"));

    assert!(output.contains("Task: refactor native bridge"));
    assert!(output.contains("State: SUCCESS_STREAK"));
}

#[tokio::test]
async fn qianhuan_render_tool_loads_semantic_template_from_skill_vfs() {
    let tool = QianhuanRenderTool;
    let manager = Arc::new(ManifestationManager::new_empty());
    let (_fixture, resolver) = semantic_skill_fixture();
    let mut ctx = ZhenfaContext::default();
    let _ = ctx.insert_shared_extension(Arc::clone(&manager));
    let _ = ctx.insert_shared_extension(Arc::clone(&resolver));

    let output = tool
        .call_native(
            &ctx,
            json!({
                "target": "wendao://skills/agenda-management/references/draft_agenda.j2",
                "data": { "task": "bridge semantic resources" },
                "runtime": {}
            }),
        )
        .await
        .unwrap_or_else(|error| panic!("semantic qianhuan render should succeed: {error}"));

    assert!(output.contains("Semantic Task: bridge semantic resources"));
}

#[tokio::test]
async fn qianhuan_render_tool_rejects_semantic_target_without_vfs_extension() {
    let tool = QianhuanRenderTool;
    let manager = Arc::new(ManifestationManager::new_empty());
    let mut ctx = ZhenfaContext::default();
    let _ = ctx.insert_shared_extension(Arc::clone(&manager));
    let error = tool
        .call_native(
            &ctx,
            json!({
                "target": "wendao://skills/agenda-management/references/draft_agenda.j2",
                "data": { "task": "bridge semantic resources" },
                "runtime": {}
            }),
        )
        .await
        .expect_err("semantic render without SkillVfsResolver should fail");

    assert!(error.to_string().contains("missing SkillVfsResolver"));
}

#[tokio::test]
async fn qianhuan_reload_tool_executes_native_dispatch() {
    let tool = QianhuanReloadTool;
    let manager = Arc::new(test_manager());
    let mut ctx = ZhenfaContext::default();
    let _ = ctx.insert_shared_extension(Arc::clone(&manager));
    let output = tool
        .call_native(&ctx, json!({}))
        .await
        .unwrap_or_else(|error| panic!("qianhuan reload should succeed: {error}"));
    assert!(output.contains("<qianhuan_reload"));
}

#[test]
fn qianhuan_reload_tool_declares_mutation_scope() {
    let tool = QianhuanReloadTool;
    let scope = tool.mutation_scope(&ZhenfaContext::default(), &json!({}));
    assert_eq!(scope.as_deref(), Some("qianhuan.reload.templates"));
}
