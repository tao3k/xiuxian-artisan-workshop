//! End-to-end adversarial agenda validation and reward-evolution scenario.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use num_traits::ToPrimitive;
use omni_memory::{EpisodeStore, StoreConfig};
use serde_json::{Value, json};
use tokio::time::sleep;
use xiuxian_llm::llm::{ChatRequest, LlmClient};
use xiuxian_qianhuan::{PersonaRegistry, orchestrator::ThousandFacesOrchestrator};
use xiuxian_qianji::{QianjiApp, QianjiLlmClient};
use xiuxian_wendao::embedded_resource_text_from_wendao_uri;
use xiuxian_wendao::{LinkGraphIndex, zhenfa_router::WendaoSearchTool};
use xiuxian_zhenfa::{
    ZhenfaContext, ZhenfaError, ZhenfaOrchestrator, ZhenfaOrchestratorHooks, ZhenfaRegistry,
    ZhenfaSignal, ZhenfaSignalSink, ZhenfaTool,
};

struct RewardRelayTool;

const AGENDA_VALIDATION_WORKFLOW_URI: &str =
    "wendao://skills/agenda-management/references/agenda_flow.toml";

#[async_trait]
impl ZhenfaTool for RewardRelayTool {
    fn id(&self) -> &'static str {
        "reward.relay"
    }

    fn definition(&self) -> Value {
        json!({
            "name": "reward.relay",
            "description": "Emit a reward signal into zhenfa signal bus.",
            "parameters": {
                "type": "object",
                "properties": {
                    "episode_id": { "type": "string" },
                    "value": { "type": "number" }
                }
            }
        })
    }

    async fn call_native(&self, ctx: &ZhenfaContext, args: Value) -> Result<String, ZhenfaError> {
        let episode_id = args
            .get("episode_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let value = args
            .get("value")
            .and_then(Value::as_f64)
            .and_then(|raw| raw.to_f32())
            .unwrap_or(0.0);
        ctx.emit_signal(ZhenfaSignal::Reward {
            episode_id,
            value,
            source: "scenario.adversarial_evolution".to_string(),
        });
        Ok("<ok/>".to_string())
    }
}

#[derive(Clone)]
struct EpisodeStoreRewardSink {
    store: Arc<EpisodeStore>,
}

#[async_trait]
impl ZhenfaSignalSink for EpisodeStoreRewardSink {
    async fn emit(&self, _ctx: &ZhenfaContext, signal: ZhenfaSignal) -> Result<(), ZhenfaError> {
        if let ZhenfaSignal::Reward {
            episode_id, value, ..
        } = signal
        {
            if episode_id.trim().is_empty() {
                return Ok(());
            }
            self.store.update_q(&episode_id, value.clamp(0.0, 1.0));
        }
        Ok(())
    }
}

#[derive(Default)]
struct ScriptedAgendaAuditLlm {
    call_count: AtomicUsize,
    seen_models: Mutex<Vec<String>>,
}

impl ScriptedAgendaAuditLlm {
    fn calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }

    fn models(&self) -> Vec<String> {
        self.seen_models
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }
}

#[async_trait]
impl LlmClient for ScriptedAgendaAuditLlm {
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<String> {
        if let Ok(mut models) = self.seen_models.lock() {
            models.push(request.model);
        }
        let round = self.call_count.fetch_add(1, Ordering::SeqCst);
        if round == 0 {
            return Ok(
                "<agenda_critique_report><score>0.2</score><critique>Overload risk from carryover history.</critique></agenda_critique_report>"
                    .to_string(),
            );
        }
        Ok(
            "<agenda_critique_report><score>0.92</score><critique>Reduced scope is feasible.</critique></agenda_critique_report>"
                .to_string(),
        )
    }
}

fn build_black_history_notebook() -> Result<(tempfile::TempDir, Arc<LinkGraphIndex>)> {
    let notebook = tempfile::tempdir()?;
    std::fs::write(
        notebook.path().join("history.md"),
        r"# 2026-02-26 Agenda
- Task: Refactor compiler (carryover=3)
- Task: Refactor compiler tests (carryover=2)
- Task: Write review report (carryover=1)
",
    )?;
    let index = LinkGraphIndex::build(notebook.path())
        .map_err(|error| anyhow::anyhow!("build link graph index: {error}"))?;
    Ok((notebook, Arc::new(index)))
}

async fn query_wendao_black_history(index: Arc<LinkGraphIndex>) -> Result<String> {
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(WendaoSearchTool));
    let orchestrator = ZhenfaOrchestrator::new(registry);
    let mut ctx = ZhenfaContext::default();
    let _ = ctx.insert_shared_extension::<LinkGraphIndex>(index);
    let xml = orchestrator
        .dispatch(
            "wendao.search",
            &ctx,
            json!({
                "query": "refactor compiler carryover",
                "limit": 8
            }),
        )
        .await?;
    Ok(xml)
}

fn build_agenda_validation_context(
    user_request: &str,
    wendao_search_results: &str,
    grounding_anchors: &[String],
) -> Value {
    json!({
        "request": user_request,
        "user_request": user_request,
        "raw_facts": format!(
            "user_request: {user_request}\nagenda_steward_capability_contract: {}",
            grounding_anchors.join(", ")
        ),
        "wendao_search_results": wendao_search_results,
        "analysis_trace": [
            {
                "predicate": "AgendaPlanningIntent",
                "has_grounding": true,
                "confidence": 0.97
            }
        ]
    })
}

async fn dispatch_reward_signal(
    orchestrator: &ZhenfaOrchestrator,
    episode_id: &str,
    reward: f32,
) -> Result<()> {
    let result = orchestrator
        .dispatch(
            "reward.relay",
            &ZhenfaContext::default(),
            json!({
                "episode_id": episode_id,
                "value": reward
            }),
        )
        .await?;
    assert_eq!(result, "<ok/>");
    Ok(())
}

async fn wait_q_value(store: &EpisodeStore, episode_id: &str, expected: f32) {
    for _ in 0..40 {
        let q = store.q_table.get_q(episode_id);
        if (q - expected).abs() < 1e-4 {
            break;
        }
        sleep(Duration::from_millis(10)).await;
    }
}

async fn assert_reward_reinforcement(output: &Value) -> Result<()> {
    let store = Arc::new(EpisodeStore::new(StoreConfig {
        path: tempfile::tempdir()?.path().to_string_lossy().to_string(),
        ..StoreConfig::default()
    }));
    let reward_sink = Arc::new(EpisodeStoreRewardSink {
        store: Arc::clone(&store),
    });
    let mut reward_registry = ZhenfaRegistry::new();
    reward_registry.register(Arc::new(RewardRelayTool));
    let reward_orchestrator = ZhenfaOrchestrator::with_hooks(
        reward_registry,
        ZhenfaOrchestratorHooks {
            cache: None,
            mutation_lock: None,
            audit_sink: None,
            signal_sink: Some(reward_sink),
        },
    );

    let episode_id = "episode:agenda:adversarial";
    dispatch_reward_signal(&reward_orchestrator, episode_id, 0.2).await?;
    wait_q_value(&store, episode_id, 0.44).await;
    let q_after_penalty = store.q_table.get_q(episode_id);
    assert!(
        (q_after_penalty - 0.44).abs() < 1e-4,
        "expected q after penalty around 0.44, got {q_after_penalty}"
    );

    let recovered_reward = output["memrl_reward"]
        .as_f64()
        .and_then(|value| value.to_f32())
        .unwrap_or(0.92_f32);
    dispatch_reward_signal(&reward_orchestrator, episode_id, recovered_reward).await?;
    wait_q_value(&store, episode_id, 0.536).await;
    let q_after_recovery = store.q_table.get_q(episode_id);
    assert!(
        q_after_recovery > q_after_penalty,
        "expected recovery q to be higher than penalty q (penalty={q_after_penalty}, recovery={q_after_recovery})"
    );
    assert!(
        (q_after_recovery - 0.536).abs() < 1e-4,
        "expected q after recovery around 0.536, got {q_after_recovery}"
    );

    Ok(())
}

#[tokio::test]
async fn scenario_adversarial_evolution_runs_retry_loop_and_updates_memory_reward() -> Result<()> {
    let (_notebook, index) = build_black_history_notebook()?;
    let wendao_xml = query_wendao_black_history(Arc::clone(&index)).await?;
    assert!(
        wendao_xml.contains("<hit "),
        "expected xml-lite hits from wendao.search, got: {wendao_xml}"
    );

    let persona_registry = Arc::new(PersonaRegistry::with_builtins());
    let agenda_style_anchors = persona_registry
        .get("agenda_steward")
        .map_or_else(Vec::new, |persona| persona.style_anchors.clone());
    assert!(
        !agenda_style_anchors.is_empty(),
        "agenda_steward style anchors should be available for context grounding"
    );
    let strict_teacher_anchors = persona_registry
        .get("strict_teacher")
        .map_or_else(Vec::new, |persona| persona.style_anchors.clone());
    assert!(
        !strict_teacher_anchors.is_empty(),
        "strict_teacher style anchors should be available for context grounding"
    );
    let mut grounding_anchors = Vec::new();
    for anchor in agenda_style_anchors
        .iter()
        .chain(strict_teacher_anchors.iter())
    {
        if !grounding_anchors.iter().any(|existing| existing == anchor) {
            grounding_anchors.push(anchor.clone());
        }
    }

    let qianhuan_orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Agenda validation must be grounded in historical execution evidence.".to_string(),
        None,
    ));
    let scripted_llm = Arc::new(ScriptedAgendaAuditLlm::default());
    let manifest_toml = embedded_resource_text_from_wendao_uri(AGENDA_VALIDATION_WORKFLOW_URI)
        .unwrap_or_else(|| {
            panic!(
                "expected embedded agenda validation workflow at {AGENDA_VALIDATION_WORKFLOW_URI}"
            )
        });
    let scheduler = QianjiApp::create_pipeline_from_manifest(
        manifest_toml,
        Arc::clone(&index),
        qianhuan_orchestrator,
        Arc::clone(&persona_registry),
        Some(Arc::clone(&scripted_llm) as Arc<QianjiLlmClient>),
    )?;

    let output = scheduler
        .run(build_agenda_validation_context(
            "Today plan: refactor compiler, write ten tests, and social meetup.",
            &wendao_xml,
            &grounding_anchors,
        ))
        .await?;

    assert_eq!(output["audit_status"], "passed");
    assert_eq!(output["agenda_commit_status"], "agenda_validated");
    assert!(
        output
            .get("agenda_steward_propose")
            .and_then(Value::as_str)
            .is_some(),
        "Agenda_Steward_Proposer output should be projected into context"
    );
    assert_eq!(output["audit_retry_count"].as_u64(), Some(1));
    let teacher_score = output["teacher_score"]
        .as_f64()
        .unwrap_or_else(|| panic!("teacher_score should be numeric"));
    assert!(
        teacher_score >= 0.9,
        "expected converged high score after retry, got {teacher_score}"
    );
    assert_eq!(scripted_llm.calls(), 2, "expected one retry loop");
    assert_eq!(
        scripted_llm.models().len(),
        2,
        "expected two llm requests in retry cycle"
    );

    assert_reward_reinforcement(&output).await?;

    Ok(())
}
