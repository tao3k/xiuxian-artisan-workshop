use anyhow::{Result, bail};

use crate::agent::omega::ShortcutFallbackAction;
use crate::contracts::{GraphExecutionPlan, GraphPlanStep, RouteTraceGraphStep};

pub(super) fn push_step_trace(
    traces: &mut Vec<RouteTraceGraphStep>,
    step: &GraphPlanStep,
    attempt: u32,
    started_at: std::time::Instant,
    status: &str,
    failure_reason: Option<String>,
) {
    traces.push(RouteTraceGraphStep {
        index: step.index,
        id: step.id.clone(),
        kind: step.kind,
        attempt,
        latency_ms: started_at.elapsed().as_secs_f64() * 1000.0,
        status: status.to_string(),
        failure_reason,
        tool_name: step.tool_name.clone(),
        fallback_action: step.fallback_action.clone(),
    });
}

pub(super) fn derive_tool_chain(plan: &GraphExecutionPlan) -> Vec<String> {
    let mut chain = Vec::<String>::new();
    for tool in plan
        .steps
        .iter()
        .filter_map(|step| step.tool_name.as_deref())
    {
        if !tool.trim().is_empty() && !chain.iter().any(|existing| existing == tool) {
            chain.push(tool.to_string());
        }
    }
    if chain.is_empty() {
        chain.push(plan.tool_name.clone());
    }
    chain
}

pub(super) fn classify_failure_taxonomy(reason: &str) -> String {
    let lower = reason.to_ascii_lowercase();
    if lower.contains("timeout") || lower.contains("timed out") {
        return "timeout".to_string();
    }
    if lower.contains("connection")
        || lower.contains("connect")
        || lower.contains("transport")
        || lower.contains("send")
        || lower.contains("broken pipe")
        || lower.contains("refused")
    {
        return "transport".to_string();
    }
    if lower.contains("schema")
        || lower.contains("invalid")
        || lower.contains("must")
        || lower.contains("unsupported")
        || lower.contains("graph plan")
    {
        return "validation".to_string();
    }
    if lower.contains("tool") && lower.contains("error") {
        return "tool_error_payload".to_string();
    }
    "execution_error".to_string()
}

pub(super) fn ordered_steps(plan: &GraphExecutionPlan) -> Result<Vec<&GraphPlanStep>> {
    if let Err(error) = plan.validate_shortcut_contract() {
        bail!("{error}");
    }

    let mut ordered: Vec<&GraphPlanStep> = plan.steps.iter().collect();
    ordered.sort_by_key(|step| step.index);

    Ok(ordered)
}

pub(super) fn fallback_action_from_step(step: &GraphPlanStep) -> Result<ShortcutFallbackAction> {
    match step.fallback_action.as_deref() {
        Some("abort") => Ok(ShortcutFallbackAction::Abort),
        Some("retry_bridge_without_metadata") => {
            Ok(ShortcutFallbackAction::RetryBridgeWithoutMetadata)
        }
        Some("route_to_react") => Ok(ShortcutFallbackAction::RouteToReact),
        Some(other) => bail!(
            "graph plan step `{}` contains unsupported fallback action `{}`",
            step.id,
            other
        ),
        None => bail!(
            "graph plan step `{}` missing fallback_action in evaluate_fallback",
            step.id
        ),
    }
}
