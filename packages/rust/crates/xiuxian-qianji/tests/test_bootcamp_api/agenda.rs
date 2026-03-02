#[cfg(feature = "llm")]
use crate::common::{
    AGENDA_FACTS, AGENDA_FLOW_URI_FROM_ALIAS, bootcamp_context_from_env, mock_llm_options,
    runtime_default_llm_options, zhixing_mount,
};
#[cfg(feature = "llm")]
use serde_json::json;
#[cfg(feature = "llm")]
use xiuxian_qianji::run_scenario;

#[cfg(feature = "llm")]
#[tokio::test]
async fn bootcamp_runs_real_adversarial_flow() {
    let mounts = zhixing_mount("zhixing", "zhixing/skills/agenda-management/references");

    let Some(initial_context) = bootcamp_context_from_env() else {
        return;
    };
    let options = runtime_default_llm_options();

    let flow_uri = "wendao://skills/zhixing/references/agenda_flow.toml";
    let report = run_scenario(flow_uri, initial_context, &mounts, options)
        .await
        .unwrap_or_else(|error| panic!("Real-world adversarial flow failed: {error}"));

    if let Some(final_report) = report.final_context.get("final_synaptic_report") {
        println!("Final Synaptic Report:\n{final_report}");
    }
    println!(
        "Reasoning Trace: Node Count = {node_count}",
        node_count = report.node_count
    );
}

#[cfg(feature = "llm")]
#[tokio::test]
async fn bootcamp_runs_embedded_agenda_flow_with_mock_llm() {
    let mounts = zhixing_mount("agenda-lab", "zhixing/skills/agenda-management/references");
    let options = mock_llm_options(
        "<agenda_critique_report><score>0.95</score><reason>approved</reason></agenda_critique_report>",
    );

    let report = run_scenario(
        AGENDA_FLOW_URI_FROM_ALIAS,
        json!({
            "request": "Generate today's agenda and then critique it.",
            "raw_facts": AGENDA_FACTS
        }),
        &mounts,
        options,
    )
    .await
    .unwrap_or_else(|error| panic!("bootcamp should execute agenda flow with mock llm: {error}"));

    assert!(
        report.manifest_name.contains("Agenda_Governance_Flow"),
        "unexpected agenda manifest name: {}",
        report.manifest_name
    );
    assert!(
        report.node_count >= 4,
        "agenda flow should have at least 4 nodes, got {}",
        report.node_count
    );
}
