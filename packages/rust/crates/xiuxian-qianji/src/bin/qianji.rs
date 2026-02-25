use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Arc;
use xiuxian_llm::llm::OpenAIClient;
use xiuxian_qianhuan::{PersonaProfile, PersonaRegistry, ThousandFacesOrchestrator};
use xiuxian_qianji::{QianjiCompiler, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

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

    let manifest_toml = fs::read_to_string(manifest_path)
        .unwrap_or_else(|_| panic!("Failed to read manifest file at {}", manifest_path));

    let context: serde_json::Value =
        serde_json::from_str(context_json).expect("Failed to parse context_json as valid JSON");

    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| "dummy_key".to_string());
    let base_url =
        env::var("OPENAI_API_BASE").unwrap_or_else(|_| "http://localhost:11434/v1".to_string());
    let redis_url = env::var("VALKEY_URL").ok().unwrap_or_else(|| "redis://127.0.0.1:6379/0".to_string());

    println!("Initializing Qianji Engine on: {}", repo_path);

    // Some basic dummy index for workflows that don't need a real graph
    let index = Arc::new(
        LinkGraphIndex::build(std::path::Path::new(&repo_path))
            .unwrap_or_else(|_| LinkGraphIndex::build(std::env::temp_dir().as_path()).unwrap()),
    );

    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));

    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "artisan-engineer".to_string(),
        name: "Artisan".to_string(),
        voice_tone: "Precise".to_string(),
        style_anchors: vec![],
        cot_template: "1. Audit requirements -> 2. Verify constraints -> 3. Execute precision change -> 4. Trace feedback.".to_string(),
        forbidden_words: vec!["maybe".to_string(), "ignore".to_string()],
        metadata: HashMap::new(),
    });

    let llm_client = Arc::new(OpenAIClient {
        api_key,
        base_url,
        http: reqwest::Client::new(),
    });

    let compiler = QianjiCompiler::new(index, orchestrator, Arc::new(registry), Some(llm_client));
    let engine = compiler.compile(&manifest_toml)?;
    let scheduler = QianjiScheduler::new(engine);

    println!("Executing Context: {}", context_json);

    let result = scheduler
        .run_with_checkpoint(context, session_id, Some(redis_url))
        .await?;

    println!("\n=== Final Qianji Execution Result ===");
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}
