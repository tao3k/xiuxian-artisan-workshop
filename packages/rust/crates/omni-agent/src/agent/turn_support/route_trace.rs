use crate::contracts::{OmegaFallbackPolicy, RouteTrace};
use crate::observability::SessionEvent;

use super::super::Agent;

const ROUTE_TRACE_STREAM_NAME: &str = "route.events";

impl Agent {
    pub(in crate::agent) async fn record_route_trace(&self, trace: &RouteTrace) {
        let route_trace_json = serialize_route_trace(trace);
        let stream_fields = build_route_trace_stream_fields(trace, &route_trace_json);
        if let Err(error) = self
            .session
            .publish_stream_event(ROUTE_TRACE_STREAM_NAME, stream_fields)
            .await
        {
            tracing::warn!(
                event = SessionEvent::RouteTraceEmitted.as_str(),
                session_id = %trace.session_id,
                turn_id = trace.turn_id,
                stream_name = ROUTE_TRACE_STREAM_NAME,
                error = %error,
                "failed to publish route trace stream event"
            );
        }
        log_route_trace_emitted(trace, &route_trace_json);
    }
}

fn serialize_route_trace(trace: &RouteTrace) -> String {
    serde_json::to_string(trace)
        .unwrap_or_else(|_| "{\"error\":\"route_trace_serialize_failed\"}".to_string())
}

fn build_route_trace_stream_fields(
    trace: &RouteTrace,
    route_trace_json: &str,
) -> Vec<(String, String)> {
    let fallback_applied = trace.fallback_applied.unwrap_or(false);
    let fallback_policy = trace
        .fallback_policy
        .map_or("none", OmegaFallbackPolicy::as_str);
    let workflow_mode = trace
        .workflow_mode
        .map_or("none", crate::GraphWorkflowMode::as_str);
    let plan_id = trace.plan_id.as_deref().unwrap_or("");
    let failure_taxonomy_json =
        serde_json::to_string(&trace.failure_taxonomy).unwrap_or_else(|_| "[]".to_string());
    let graph_steps = trace.graph_steps.clone().unwrap_or_default();
    let graph_steps_json = serde_json::to_string(&graph_steps).unwrap_or_else(|_| "[]".to_string());
    let mut stream_fields = vec![
        (
            "kind".to_string(),
            SessionEvent::RouteTraceEmitted.as_str().to_string(),
        ),
        ("session_id".to_string(), trace.session_id.clone()),
        ("turn_id".to_string(), trace.turn_id.to_string()),
        (
            "selected_route".to_string(),
            trace.selected_route.as_str().to_string(),
        ),
        ("confidence".to_string(), format!("{:.6}", trace.confidence)),
        (
            "risk_level".to_string(),
            trace.risk_level.as_str().to_string(),
        ),
        (
            "tool_trust_class".to_string(),
            trace.tool_trust_class.as_str().to_string(),
        ),
        ("fallback_applied".to_string(), fallback_applied.to_string()),
        ("fallback_policy".to_string(), fallback_policy.to_string()),
        ("plan_id".to_string(), plan_id.to_string()),
        ("workflow_mode".to_string(), workflow_mode.to_string()),
        (
            "tool_chain_len".to_string(),
            trace.tool_chain.len().to_string(),
        ),
        (
            "failure_count".to_string(),
            trace.failure_taxonomy.len().to_string(),
        ),
        (
            "graph_steps_count".to_string(),
            trace.graph_steps.as_ref().map_or(0, Vec::len).to_string(),
        ),
        (
            "latency_ms".to_string(),
            format!("{:.3}", trace.latency_ms.unwrap_or(0.0)),
        ),
        ("failure_taxonomy_json".to_string(), failure_taxonomy_json),
        ("graph_steps_json".to_string(), graph_steps_json),
        ("route_trace_json".to_string(), route_trace_json.to_string()),
    ];
    if let Some(injection) = trace.injection.as_ref() {
        stream_fields.extend([
            (
                "injection_blocks_used".to_string(),
                injection.blocks_used.to_string(),
            ),
            (
                "injection_chars_injected".to_string(),
                injection.chars_injected.to_string(),
            ),
            (
                "injection_dropped_by_budget".to_string(),
                injection.dropped_by_budget.to_string(),
            ),
        ]);
    }
    stream_fields
}

fn log_route_trace_emitted(trace: &RouteTrace, route_trace_json: &str) {
    tracing::info!(
        event = SessionEvent::RouteTraceEmitted.as_str(),
        session_id = %trace.session_id,
        turn_id = trace.turn_id,
        selected_route = trace.selected_route.as_str(),
        confidence = trace.confidence,
        risk_level = trace.risk_level.as_str(),
        tool_trust_class = trace.tool_trust_class.as_str(),
        fallback_applied = trace.fallback_applied,
        fallback_policy = trace.fallback_policy.map(OmegaFallbackPolicy::as_str),
        plan_id = trace.plan_id.as_deref(),
        workflow_mode = trace.workflow_mode.map(crate::GraphWorkflowMode::as_str),
        tool_chain_len = trace.tool_chain.len(),
        failure_count = trace.failure_taxonomy.len(),
        graph_steps = trace.graph_steps.as_ref().map_or(0, Vec::len),
        latency_ms = trace.latency_ms,
        route_trace = %route_trace_json,
        "route trace emitted"
    );
}
