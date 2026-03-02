//! Qianji (千机) - The automated execution engine binary.
//!
//! This binary provides the entrypoint for compiling manifests and executing
//! long-running agentic workflows within the Xiuxian ecosystem.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::sync::Arc;
use xiuxian_llm::llm::OpenAIClient;
use xiuxian_qianhuan::{
    orchestrator::ThousandFacesOrchestrator,
    persona::{PersonaProfile, PersonaRegistry},
};
use xiuxian_qianji::manifest_requires_llm;
use xiuxian_qianji::runtime_config::resolve_qianji_runtime_llm_config;
use xiuxian_qianji::{QianjiCompiler, QianjiLlmClient, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

/// Main entry point for the Qianji execution engine.
///
/// # Errors
/// Returns an error if environment resolution, compilation, or execution fails.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: qianji <repo_path> <manifest_path> <context_json> [session_id]");
        std::process::exit(1);
    }

    let repo_path = &args[1];
    let manifest_path = &args[2];
    let context_json = &args[3];
    let session_id = args.get(4).cloned();

    let manifest_toml = fs::read_to_string(manifest_path).map_err(|e| {
        io::Error::other(format!(
            "Failed to read manifest file at {manifest_path}: {e}"
        ))
    })?;

    let mut context: serde_json::Value = serde_json::from_str(context_json).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Failed to parse context_json as valid JSON: {e}"),
        )
    })?;

    let requires_llm = manifest_requires_llm(&manifest_toml).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to inspect manifest for llm requirements: {e}"),
        )
    })?;
    let llm_runtime = if requires_llm {
        let resolved = resolve_qianji_runtime_llm_config().map_err(|e| {
            io::Error::other(format!(
                "Failed to resolve Qianji runtime config from qianji.toml: {e}"
            ))
        })?;
        inject_llm_model_fallback_if_missing(&mut context, &resolved.model);
        Some(resolved)
    } else {
        None
    };

    let redis_url = env::var("VALKEY_URL")
        .ok()
        .unwrap_or_else(|| "redis://localhost:6379/0".to_string());

    println!("Initializing Qianji Engine on: {repo_path}");
    if let Some(runtime) = llm_runtime.as_ref() {
        println!(
            "Resolved Qianji LLM runtime config: model='{}', base_url='{}', api_key_env='{}'",
            runtime.model, runtime.base_url, runtime.api_key_env
        );
    } else {
        println!("Manifest has no llm nodes; skipping Qianji LLM runtime initialization.");
    }

    // Some basic dummy index for workflows that don't need a real graph
    let index = Arc::new(
        match LinkGraphIndex::build(std::path::Path::new(repo_path)) {
            Ok(index) => index,
            Err(primary_error) => {
                LinkGraphIndex::build(std::env::temp_dir().as_path()).map_err(|fallback_error| {
                    io::Error::other(format!(
                        "Failed to build LinkGraph index at repo path ({primary_error}); \
fallback temp index also failed ({fallback_error})"
                    ))
                })?
            }
        },
    );

    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));

    let registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "artisan-engineer".to_string(),
        name: "Artisan".to_string(),
        background: None,
        voice_tone: "Precise".to_string(),
        guidelines: Vec::new(),
        style_anchors: vec![],
        cot_template: "1. Audit requirements -> 2. Verify constraints -> 3. Execute precision change -> 4. Trace feedback.".to_string(),
        forbidden_words: vec!["maybe".to_string(), "ignore".to_string()],
        metadata: HashMap::new(),
    });

    let llm_client: Option<Arc<QianjiLlmClient>> = llm_runtime.as_ref().map(|runtime| {
        Arc::new(OpenAIClient {
            api_key: runtime.api_key.clone(),
            base_url: runtime.base_url.clone(),
            http: reqwest::Client::new(),
        }) as Arc<QianjiLlmClient>
    });

    let compiler = QianjiCompiler::new(index, orchestrator, Arc::new(registry), llm_client);
    let engine = compiler.compile(&manifest_toml)?;
    let scheduler = QianjiScheduler::new(engine);

    println!("Executing Context: {context_json}");

    let result = scheduler
        .run_with_checkpoint(context, session_id, Some(redis_url))
        .await?;

    println!("\n=== Final Qianji Execution Result ===");
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

fn inject_llm_model_fallback_if_missing(context: &mut serde_json::Value, default_model: &str) {
    let Some(map) = context.as_object_mut() else {
        return;
    };

    let has_explicit_model = map
        .get("llm_model")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let has_fallback_model = map
        .get("llm_model_fallback")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    if has_explicit_model || has_fallback_model {
        return;
    }

    map.insert(
        "llm_model_fallback".to_string(),
        serde_json::Value::String(default_model.to_string()),
    );
}
