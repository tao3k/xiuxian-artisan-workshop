#[allow(clippy::wildcard_imports)]
use super::*;

use crate::agent::memory_recall::MemoryRecallPlan;

pub(super) struct ReactPreparedMessages {
    pub(super) messages: Vec<ChatMessage>,
    pub(super) summary_segment_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MemoryRecallTuning {
    pub(super) k1: usize,
    pub(super) k2: usize,
    pub(super) lambda: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MemoryRecallPlanContext {
    pub(super) recall_started: Instant,
    pub(super) active_turns_estimate: usize,
    pub(super) query_tokens: usize,
    pub(super) recall_plan: MemoryRecallPlan,
    pub(super) recall_feedback_bias: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MemoryRecallExecutionContext<'a> {
    pub(super) session_id: &'a str,
    pub(super) summary_segment_count: usize,
    pub(super) embedding_source: &'static str,
    pub(super) recall_credit_enabled: bool,
    pub(super) recall_credit_max_candidates: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MemoryRecallResultStats {
    pub(super) recalled_count: usize,
    pub(super) selected_count: usize,
    pub(super) injected_count: usize,
    pub(super) context_chars_injected: usize,
    pub(super) best_score: Option<f32>,
    pub(super) weakest_score: Option<f32>,
    pub(super) pipeline_duration_ms: u64,
}

pub(super) struct ReactConversationState {
    pub(super) messages: Vec<ChatMessage>,
    pub(super) tools_json: Option<Vec<serde_json::Value>>,
    pub(super) round: u32,
    pub(super) total_tool_calls_this_turn: u32,
    pub(super) last_tool_names: Vec<String>,
    pub(super) tool_summary: ToolExecutionSummary,
}

impl ReactConversationState {
    pub(super) fn new(
        messages: Vec<ChatMessage>,
        tools_json: Option<Vec<serde_json::Value>>,
    ) -> Self {
        Self {
            messages,
            tools_json,
            round: 0,
            total_tool_calls_this_turn: 0,
            last_tool_names: Vec::new(),
            tool_summary: ToolExecutionSummary::default(),
        }
    }
}

pub(super) struct TurnRuntimeContext<'a> {
    pub(super) session_id: &'a str,
    pub(super) user_message: &'a str,
    pub(super) turn_id: u64,
    pub(super) route: OmegaRoute,
    pub(super) recall_credit_candidates: &'a [RecalledEpisodeCandidate],
}

pub(super) struct ContextRepairResult {
    pub(super) response: crate::llm::AssistantMessage,
    pub(super) messages: Vec<ChatMessage>,
    pub(super) tools_json: Option<Vec<serde_json::Value>>,
}
