//! Native TOML orchestration integration tests for Qianji.

use serde_json::json;
use std::sync::Arc;
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_qianji::{QianjiCompiler, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

const DIAMOND_DAG_TOML: &str = include_str!("../resources/tests/diamond_dag.toml");

#[tokio::test]
async fn test_qianji_native_toml_orchestration_diamond()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let index = Arc::new(LinkGraphIndex::build(temp.path())?);
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Rules".to_string(), None));
    let registry = Arc::new(PersonaRegistry::with_builtins());

    // Fix: Inject None for llm_client
    let compiler = QianjiCompiler::new(index, orchestrator, registry, None);
    let engine = compiler.compile(DIAMOND_DAG_TOML)?;
    let scheduler = QianjiScheduler::new(engine);

    let result = scheduler.run(json!({})).await?;

    assert_eq!(result["A"], "done");
    assert_eq!(result["D"], "done");
    Ok(())
}
