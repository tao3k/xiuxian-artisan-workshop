use crate::common::{
    FORGE_FLOW_URI_CANONICAL, bootcamp_context_from_env, ensure_runtime_forge_context_defaults,
    mock_llm_options, runtime_default_llm_options, zhixing_mount,
};
use serde_json::json;
use std::fs;
use tempfile::tempdir;
use xiuxian_qianji::run_scenario;

#[tokio::test]
async fn bootcamp_runs_embedded_forge_flow_with_mock_llm() {
    let mounts = zhixing_mount(
        "forge-evolution",
        "zhixing/skills/forge-evolution/references",
    );
    let options = mock_llm_options(
        "<forge_audit_report><score>0.91</score><critique>Blueprint has measurable guardrails.</critique><verdict>pass</verdict></forge_audit_report>",
    );
    let project_root = tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let target_persona_dir = project_root.path().join("personas");
    let expected_manifested_path = target_persona_dir.join("soul_forger_v2.md");

    let report = run_scenario(
        FORGE_FLOW_URI_CANONICAL,
        json!({
            "failure_trace": "retry loop exceeded threshold in agenda validation for three consecutive sessions",
            "failure_cluster": "planning-consistency-regression",
            "target_domain": "agenda-management",
            "raw_facts": "three repeated low scores below 0.5; stale carryover accumulation; unstable prioritization output",
            "wendao_search_results": "<hit id=\"audit:1\" type=\"audit\" score=\"0.42\">Repeated overload planning failure</hit>",
            "project_root": project_root.path().display().to_string(),
            "target_persona_dir": target_persona_dir.display().to_string(),
            "role_id": "soul_forger_v2",
            "forge_changed_paths": [expected_manifested_path.display().to_string()]
        }),
        &mounts,
        options,
    )
    .await
    .unwrap_or_else(|error| panic!("bootcamp should execute forge flow with mock llm: {error}"));

    assert_eq!(report.manifest_name, "Evolution_Trinity_Soul_Forge_Flow");
    assert_eq!(report.node_count, 4);
    assert!(report.final_context["forge_candidate_blueprint"].is_string());
    let manifested_path = report.final_context["manifestation_result"]["path"]
        .as_str()
        .unwrap_or_else(|| panic!("manifestation_result.path should be present"));
    let manifested_content = fs::read_to_string(manifested_path)
        .unwrap_or_else(|error| panic!("manifested file should be readable: {error}"));
    assert_eq!(
        manifested_content,
        report.final_context["forge_candidate_blueprint"]
            .as_str()
            .unwrap_or_else(|| panic!("forge_candidate_blueprint should be string"))
    );
    assert_eq!(
        report.final_context["forge_index_refresh"]["changed_paths"][0]
            .as_str()
            .unwrap_or_else(|| panic!("forge_index_refresh.changed_paths[0] should be string")),
        manifested_path
    );
    assert_eq!(report.final_context["forge_index_refresh"]["mode"], "delta");
    assert_eq!(
        report.final_context["forge_index_refresh"]["changed_count"],
        1
    );
    assert_eq!(
        report.final_context["forge_index_refresh"]["fallback"],
        false
    );
}

#[tokio::test]
async fn bootcamp_runs_real_forge_flow() {
    let mounts = zhixing_mount(
        "forge-evolution",
        "zhixing/skills/forge-evolution/references",
    );

    let Some(mut context) = bootcamp_context_from_env() else {
        return;
    };
    ensure_runtime_forge_context_defaults(&mut context);
    let options = runtime_default_llm_options();

    let report = run_scenario(FORGE_FLOW_URI_CANONICAL, context, &mounts, options)
        .await
        .unwrap_or_else(|error| panic!("Real-world forge flow failed: {error}"));

    println!("Forge Output:\n{:#?}", report.final_context);
}
