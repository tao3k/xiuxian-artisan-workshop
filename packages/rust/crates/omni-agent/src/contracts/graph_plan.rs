use serde::{Deserialize, Serialize};

use super::{OmegaFallbackPolicy, OmegaRoute};

/// Explicit workflow bridge mode used by graph shortcut execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphWorkflowMode {
    Graph,
    Omega,
}

impl GraphWorkflowMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Graph => "graph",
            Self::Omega => "omega",
        }
    }
}

/// Deterministic graph-plan step type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphPlanStepKind {
    PrepareInjectionContext,
    InvokeGraphTool,
    EvaluateFallback,
}

/// One graph-plan step in execution order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphPlanStep {
    pub index: u8,
    pub id: String,
    pub kind: GraphPlanStepKind,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_action: Option<String>,
}

/// Deterministic plan contract produced by Rust graph planner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphExecutionPlan {
    /// Stable plan identifier for traceability and replay.
    pub plan_id: String,
    /// Contract version.
    pub plan_version: String,
    /// Route selected by governance for this plan.
    pub route: OmegaRoute,
    /// Workflow mode (`graph` / `omega`) that generated this plan.
    pub workflow_mode: GraphWorkflowMode,
    /// Primary MCP bridge tool to execute.
    pub tool_name: String,
    /// Fallback policy tied to the route decision.
    pub fallback_policy: OmegaFallbackPolicy,
    /// Ordered deterministic execution steps.
    pub steps: Vec<GraphPlanStep>,
}

impl GraphExecutionPlan {
    /// Validate deterministic shortcut-plan contract invariants.
    ///
    /// This is shared by planner/executor so generation and consumption follow
    /// the same schema-level guarantees.
    ///
    /// # Errors
    /// Returns an error string when any graph-plan contract invariant is violated.
    pub fn validate_shortcut_contract(&self) -> std::result::Result<(), String> {
        self.validate_plan_metadata()?;
        let ordered = self.collect_ordered_steps()?;
        self.validate_step_kinds(&ordered)?;
        self.validate_prepare_step(ordered[0])?;
        self.validate_invoke_step(ordered[1])?;
        self.validate_fallback_step(ordered[2])
    }

    fn validate_plan_metadata(&self) -> std::result::Result<(), String> {
        if self.plan_id.trim().is_empty() {
            return Err("graph plan has empty plan_id".to_string());
        }
        if self.plan_version != "v1" {
            return Err(format!(
                "graph plan `{}` has unsupported plan_version `{}` (expected `v1`)",
                self.plan_id, self.plan_version
            ));
        }
        if self.tool_name.trim().is_empty() {
            return Err(format!("graph plan `{}` has empty tool_name", self.plan_id));
        }
        if self.steps.len() != 3 {
            return Err(format!(
                "graph plan `{}` must contain exactly 3 steps, found {}",
                self.plan_id,
                self.steps.len()
            ));
        }
        Ok(())
    }

    fn collect_ordered_steps(&self) -> std::result::Result<[&GraphPlanStep; 3], String> {
        let mut ordered: Vec<&GraphPlanStep> = self.steps.iter().collect();
        ordered.sort_by_key(|step| step.index);
        for (idx, step) in ordered.iter().enumerate() {
            let expected = u8::try_from(idx.saturating_add(1)).map_err(|_| {
                format!(
                    "graph plan `{}` contains step index overflow at position {}",
                    self.plan_id,
                    idx.saturating_add(1)
                )
            })?;
            if step.index != expected {
                return Err(format!(
                    "graph plan `{}` step ordering is invalid: expected index {}, found {}",
                    self.plan_id, expected, step.index
                ));
            }
            if step.id.trim().is_empty() {
                return Err(format!(
                    "graph plan `{}` step {} has empty id",
                    self.plan_id, step.index
                ));
            }
            if step.description.trim().is_empty() {
                return Err(format!(
                    "graph plan `{}` step {} has empty description",
                    self.plan_id, step.index
                ));
            }
        }

        let [prepare, invoke, fallback] = ordered
            .try_into()
            .map_err(|_| format!("graph plan `{}` must contain exactly 3 steps", self.plan_id))?;
        Ok([prepare, invoke, fallback])
    }

    fn validate_step_kinds(
        &self,
        ordered: &[&GraphPlanStep; 3],
    ) -> std::result::Result<(), String> {
        let expected_kinds = [
            GraphPlanStepKind::PrepareInjectionContext,
            GraphPlanStepKind::InvokeGraphTool,
            GraphPlanStepKind::EvaluateFallback,
        ];
        for (idx, step) in ordered.iter().enumerate() {
            let expected_kind = expected_kinds[idx];
            if step.kind != expected_kind {
                return Err(format!(
                    "graph plan `{}` step {} kind is invalid: expected `{}`, found `{}`",
                    self.plan_id,
                    step.index,
                    expected_kind.as_str(),
                    step.kind.as_str()
                ));
            }
        }
        Ok(())
    }

    fn validate_prepare_step(&self, step: &GraphPlanStep) -> std::result::Result<(), String> {
        if step.tool_name.is_some() {
            return Err(format!(
                "graph plan `{}` prepare step must not set tool_name",
                self.plan_id
            ));
        }
        if step.fallback_action.is_some() {
            return Err(format!(
                "graph plan `{}` prepare step must not set fallback_action",
                self.plan_id
            ));
        }
        Ok(())
    }

    fn validate_invoke_step(&self, step: &GraphPlanStep) -> std::result::Result<(), String> {
        let invoke_tool = step
            .tool_name
            .as_deref()
            .unwrap_or(self.tool_name.as_str())
            .trim();
        if invoke_tool.is_empty() {
            return Err(format!(
                "graph plan `{}` invoke step has empty tool_name",
                self.plan_id
            ));
        }
        if step.fallback_action.is_some() {
            return Err(format!(
                "graph plan `{}` invoke step must not set fallback_action",
                self.plan_id
            ));
        }
        Ok(())
    }

    fn validate_fallback_step(&self, step: &GraphPlanStep) -> std::result::Result<(), String> {
        if step.tool_name.is_some() {
            return Err(format!(
                "graph plan `{}` fallback step must not set tool_name",
                self.plan_id
            ));
        }
        let fallback_action = step.fallback_action.as_deref().map(str::trim);
        let Some(fallback_action) = fallback_action else {
            return Err(format!(
                "graph plan `{}` fallback step missing fallback_action",
                self.plan_id
            ));
        };
        if fallback_action.is_empty() {
            return Err(format!(
                "graph plan `{}` fallback step has empty fallback_action",
                self.plan_id
            ));
        }
        if !is_supported_fallback_action(fallback_action) {
            return Err(format!(
                "graph plan `{}` fallback step has unsupported fallback_action `{}`",
                self.plan_id, fallback_action
            ));
        }
        Ok(())
    }
}

impl GraphPlanStepKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PrepareInjectionContext => "prepare_injection_context",
            Self::InvokeGraphTool => "invoke_graph_tool",
            Self::EvaluateFallback => "evaluate_fallback",
        }
    }
}

fn is_supported_fallback_action(action: &str) -> bool {
    matches!(
        action,
        "abort" | "retry_bridge_without_metadata" | "route_to_react"
    )
}
