//! Probabilistic routing integration tests for Qianji manifests.

use serde_json::json;
use std::sync::Arc;
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_qianji::{QianjiCompiler, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

const BRANCH_TOML: &str = include_str!("../resources/tests/probabilistic_branch.toml");

#[tokio::test]
async fn test_probabilistic_routing_from_resource()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let index = Arc::new(LinkGraphIndex::build(temp.path())?);
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Rules".to_string(), None));
    let registry = Arc::new(PersonaRegistry::with_builtins());

    // Fix: Inject None for llm_client
    let compiler = QianjiCompiler::new(index, orchestrator, registry, None);
    let engine = compiler.compile(BRANCH_TOML)?;
    let scheduler = QianjiScheduler::new(engine);

    let result = scheduler.run(json!({})).await?;

    // Verified by resource: PathB has 0.0 weight
    assert_eq!(result["selected_route"], "PathA");
    assert_eq!(result["BranchA"], "done");
    Ok(())
}
