---
type: knowledge
title: "Omega + Graph + Loop/ReAct: Rust Unification Blueprint"
category: "plans"
tags:
  - plan
  - omega
saliency_base: 7.2
decay_rate: 0.03
metadata:
  title: "Omega + Graph + Loop/ReAct: Rust Unification Blueprint"
---

# Omega + Graph + Loop/ReAct: Rust Unification Blueprint

> Goal: converge execution into a single Rust runtime (`omni-agent`) by fusing Omega reasoning, Graph planning, ReAct tool execution, and authoritative Xiuxian-Qianhuan injection, then progressively remove Python runtime paths.
>
> Detailed companion: [Xiuxian-Qianhuan Injection + Memory Self-Evolution + Reflection](../memory/injection-evolution.md)
>
> LinkGraph execution companion (primary core track): [LinkGraph PPR Algorithm Spec](../wendao/ppr-algorithm.md)
>
> Execution sequence companion: [Integrated Architecture Audit Checklist (2026)](../../03_features/qianhuan-audit-closure.md)

## 1. Scope and Boundaries

- In scope:
  - Unify Omega, Graph, and ReAct under one Rust execution kernel.
  - Keep Python as MCP tool provider during transition.
  - Move session windowing, compression, and memory self-evolution to Rust-first execution path.
- Out of scope:
  - Rewriting every tool/skill from Python to Rust immediately.
  - Breaking existing MCP contracts during migration.

## 2. Target Architecture

```mermaid
flowchart LR
  U[User / Channel] --> G[omni-agent gateway/repl]
  G --> R[Unified Rust Runtime Kernel]

  R --> O[Omega Deliberation Engine]
  O --> I[Xiuxian-Qianhuan Assembler]
  R --> P[Graph Planning Engine]
  R --> X[ReAct Execution Engine]

  X --> M[MCP Client Pool]
  M --> PY[Python MCP Tool Servers (transition)]
  M --> RS[Rust-native MCP servers]

  R --> W[omni-window]
  R --> MM[omni-memory]
  R --> KG[xiuxian-wendao / link-graph]
  R --> SPI[Session Prompt Injection XML]

  W --> I
  MM --> I
  KG --> I
  SPI --> I
  I --> P
  I --> X
  W --> MM
  MM --> R
  KG --> R
```

## 3. Unified Runtime Workflow

1. Intake:
   - Receive request and resolve `session_id` (channel/chat/thread aware).
   - Load bounded context from `omni-window`.
2. Omega deliberation:
   - Evaluate complexity and choose execution route (`react` direct vs `graph` first).
   - Produce context policy (what to inject, max size, ordering, role-mix profile, injection mode).
3. Xiuxian-Qianhuan context assembly (knowledge inject role):
   - Assemble typed context blocks from:
     - session prompt injection XML (operator/session scoped),
     - memory recall context (`omni-memory`, MemRL-style),
     - bounded summaries/window state (`omni-window`),
     - knowledge context (`xiuxian-wendao`, link-graph).
   - Compose scenario-specific mixed-role prompts (for example debug/recovery/architecture reflection packs).
   - Apply deterministic ordering and token budget before execution.
4. Execution routing:
   - Fast/simple request goes to ReAct execution with assembled context.
   - Complex request triggers Graph plan synthesis first, then ReAct/tool execution.
5. Omega quality gating:
   - Evaluate plan quality, risk, and tool ordering.
   - Repair plan before execution when quality checks fail.
6. ReAct execution:
   - Execute tool loop with budget, retries, and structured error taxonomy.
   - Call tools through MCP client pool only.
7. Self-evolution update:
   - Store episode outcome and feedback in `omni-memory`.
   - Persist session window snapshots and summary segments.
8. Response:
   - Emit user-facing answer plus structured observability events.

## 3.1 Xiuxian-Qianhuan: Architectural Role

- Ownership:
  - Owned by Rust runtime, policy decided by Omega.
  - Not owned by Python runtime loop.
- Responsibility:
  - Deliver high-signal context to Graph/ReAct without changing model weights.
  - Provide flexible injection modes (`single`, `classified`, `hybrid`) and mixed-role composition.
  - Keep context bounded, session-scoped, and auditable.
- Non-goals:
  - No free-form hidden prompt mutation in random call sites.
  - No bypass of context policy on deterministic execution paths.
- Contract direction:
  - Introduce typed `PromptContextBlock` and `InjectionPolicy` contracts.
  - Keep tool payload contracts stable; pass injected context through explicit fields only when schema supports it.

## 4. Feature-Name Roadmap (Backlog-Aligned)

Project progress must be tracked by feature name (not phase/stage labels). Recommended feature names:

| Feature name                             | Definition of done                                                                        |
| ---------------------------------------- | ----------------------------------------------------------------------------------------- |
| **Unified Rust Execution Kernel**        | One Rust entry for channel/repl/gateway execution; no Python runtime loop on hot path.    |
| **Graph Planning Engine (Rust)**         | Graph planning API runs inside Rust runtime and produces stable, testable plan contracts. |
| **Omega Deliberation Engine (Rust)**     | Quality gates and plan-repair logic run in Rust with structured outputs.                  |
| **ReAct Tool Runtime (Rust)**            | Tool-call loop, retry, budget, and failure policy consolidated in Rust.                   |
| **Session Window Compression (Rust)**    | Predictable context compression and restore strategy backed by `omni-window`.             |
| **Memory Self-Evolution Runtime (Rust)** | Outcome feedback and recall adaptation persisted via DB-backed `omni-memory`.             |
| **Python Runtime Decommissioning**       | Python side is MCP tool service only; no duplicated runtime loop entrypoints.             |

## 5. Migration Rules

- Single authority:
  - Runtime orchestration authority is Rust.
  - Python authority is tool implementation behind MCP.
- Thin orchestrator rule:
  - `omni-agent` remains orchestration-only.
  - Memory lifecycle/revalidation/promotion core logic must live in Rust memory package(s), not inside agent runtime modules.
- MCP interoperability rule:
  - Keep `skill memory` as MCP-facing tool surface for external clients.
  - `skill memory` acts as a thin facade to Rust memory core (via bindings/bridge), without duplicating policy logic.
- Prompt/context authority:
  - Prompt/knowledge injection authority is Rust `Xiuxian-Qianhuan Assembler`.
  - Python side must not inject hidden runtime prompt context.
- No dual-loop fallback:
  - Do not keep long-term “Rust loop + Python loop” behavior parity mode.
  - Keep one execution contract and migrate callers to it.
- Contract-first evolution:
  - Keep MCP `tools/list` and `tools/call` behavior stable while internals move.
  - Version schemas when changing output shape.
- Isolation by default:
  - Session partition key is mandatory (`channel:chat_id:thread_id` when applicable).
  - Window snapshots and memory feedback must be session-scoped.

## 5.1 Boundary Corrections (Roadmap Clarification)

- `memory`:
  - short-term operational runtime memory (Rust core owned)
  - exposed through MCP memory skill facade for interoperability
- `knowledge`:
  - long-term durable knowledge interface (MCP knowledge skill)
- `omni-agent`:
  - orchestration only; no embedding of memory lifecycle policy logic

## 5.2 Data Plane Standard (Valkey + LanceDB + Arrow)

- `Valkey`:
  - hot runtime state, dedup/idempotency, stream events, and high-concurrency caches.
- `LanceDB`:
  - durable retrieval state, tool/knowledge indexes, episodic memory persistence, replay analytics.
- `Arrow`:
  - canonical inter-stage schema for ranking/gate traces with zero-copy batch movement.

Rule:

- no hot-path JSON file state source;
- read-through/write-through flows must follow `Valkey -> LanceDB` boundaries with Arrow contracts.

## 5.3 Discover Confidence Contract

`skill.discover` and route selection must preserve calibrated ranking metadata end-to-end:

- `score`
- `final_score`
- `confidence` (`high` | `medium` | `low`)
- `ranking_reason`
- `usage_template`

Policy:

- `high`: direct recommendation allowed
- `medium`: top-k + clarification
- `low`: refine intent before execution

## 6. Quality Gates

- Correctness:
  - Cross-session isolation matrix (multi-group, multi-thread, mixed `/reset` and `/resume` concurrency).
  - Deterministic parser and command routing tests in dedicated `tests/` modules.
  - Prompt injection determinism tests (same inputs => same ordered context blocks).
- Reliability:
  - MCP startup and reconnect resilience under slow-start and transient failures.
  - No silent exits; structured startup/shutdown diagnostics.
- Performance:
  - Baseline and regression benchmark for p50/p95 latency, failure rate, and memory peak.
  - Concurrent-session load tests for gateway mode.
- Observability:
  - Structured events for session lifecycle, snapshot operations, memory recall/update, MCP call duration, and tool failures.

## 7. Python Runtime Removal End-State

- End-state contract:
  - `omni-agent` is the only runtime orchestrator.
  - Python process provides MCP tools and supporting services only.
- Cleanup targets:
  - Remove Python runtime loop command paths after Rust parity is proven.
  - Keep compatibility wrappers only where they map directly to Rust commands.
- Acceptance rule:
  - Removal is complete only after black-box suites pass on multi-session, multi-channel, and memory self-evolution scenarios.

## 8. Contract Seeds (Next)

- `OmegaDecision`:
  - route (`react` | `graph`)
  - risk level
  - injection policy (enabled blocks, max chars/tokens, ordering strategy)
- `PromptContextBlock`:
  - source (`memory_recall` | `session_xml` | `window_summary` | `knowledge`)
  - priority, size, session scope
  - payload (rendered text/XML)
- `TurnTrace`:
  - selected route, tool chain, retries, latency, failure taxonomy
  - injection stats (`blocks_used`, `chars_injected`, dropped-by-budget)
- `ReflectionRecord`:
  - outcome, failure category, corrective action
  - memory credit update and next-turn strategy hint
- `DiscoverMatch`:
  - tool id, usage template, score/final_score/confidence, ranking_reason, schema digest

## 9. Post-A7 P0 Execution Queue

After A0-A7 closure, the next implementation queue is feature-driven (not gate-driven):

| Priority | Feature                                     | Scope                                                                                            | Exit criteria                                                                              | Primary verification                                                           |
| -------- | ------------------------------------------- | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------ |
| P0-1     | Graph Planning Engine (Rust)                | Move planning contract and graph execution entry to Rust runtime with deterministic plan schema. | Rust graph plan contract is generated and consumed without Python runtime loop dependency. | `cargo test -p omni-agent --test contracts` + graph planning integration tests |
| P0-2     | Omega Deliberation Engine (Rust)            | Expand policy routing into explicit plan-repair/quality-gate path in Rust.                       | Route policy can enforce repair or fallback with auditable reason fields.                  | `cargo test -p omni-agent --test agent_injection` + reflection threshold tests |
| P0-3     | Role-Mix Injection Profiles                 | Add `single/classified/hybrid` profile selection with deterministic assembly.                    | Role-mix profile is selected by policy and recorded in injection snapshot traces.          | `cargo test -p omni-agent --lib injection::tests` + trace reconstruction gate  |
| P0-4     | Python Runtime Decommissioning (Loop paths) | Remove duplicated Python runtime loop entrypoints while preserving MCP tool plane.               | Runtime orchestration entry remains Rust-only (`omni-agent`).                              | `python3 scripts/channel/test_omni_agent_memory_ci_gate.py --profile nightly`  |
| P0-5     | Adversarial Sub-graph Routing               | Deprecate regex-based triggers for Qianji workflows; elevate to Omega routing policy.            | Omega natively outputs `route: graph` + `workflow_mode: agenda_validation` via LLM JSON.   | `cargo test -p omni-agent --test agent_omega_routing`                          |

### P0-1 Status Update (2026-02-23)

Completed in current branch:

- Added `GraphExecutionPlan::validate_shortcut_contract()` as the shared deterministic-schema validator for graph shortcut plans.
- Wired graph plan consumption (`agent/graph/executor.rs`) to enforce the shared validator before execution.
- Added planner-side debug assertion so generated shortcut plans are checked against the same validator.
- Added planner-focused tests and contract-focused tests:
  - `tests/agent/graph_planner.rs`
  - `tests/contracts/test_runtime_contracts.rs`
  - `tests/agent/graph_executor.rs` (invalid fallback action / step ordering validation)

Verification commands:

- `cargo test -p omni-agent --lib graph_ -- --nocapture`
- `cargo test -p omni-agent --test contracts graph_execution_plan_contract -- --nocapture`

### P0-2 Status Update (2026-02-23)

Completed in current branch:

- Added explicit quality-gate repair path in `agent/omega/decision.rs`:
  - `apply_quality_gate(decision)` enforces high-risk graph safeguards.
  - When `route=graph` and risk is `high|critical`, fallback policy is repaired from
    `switch_to_graph` to `retry_react`.
  - High-risk `tool_trust_class=evidence` is upgraded to `verification`.
- Added auditable reason markers for deterministic traceability:
  - `quality_gate=graph_retry_loop_guard;repair=fallback_policy:retry_react`
  - `quality_gate=graph_high_risk_trust_upgrade;repair=tool_trust_class:verification`
- Wired quality-gate enforcement into runtime execution paths:
  - `agent/turn_execution/shortcut.rs`
  - `agent/turn_execution/react_loop.rs`
- Added unit tests for quality-gate behavior:
  - `tests/agent/omega_decision.rs`

Verification commands:

- `cargo test -p omni-agent --lib apply_quality_gate_ -- --nocapture`
- `cargo test -p omni-agent --test agent_injection omega_shortcut_ -- --nocapture`

### P0-3 Status Update (2026-02-23)

Completed in current branch:

- Added deterministic injection policy resolver in Rust runtime:
  - New module: `agent/injection/policy.rs`
  - Adaptive rule from `classified` baseline:
    - single block => `single`
    - multi-domain blocks => `hybrid`
    - otherwise => `classified`
  - `single` mode is compact by construction (`max_blocks <= 1`, priority-first ordering).
- Wired effective policy resolution into both injection paths:
  - `normalize_messages_with_snapshot(...)`
  - `build_snapshot_from_messages(...)`
- Upgraded role-mix profile selection to align with policy mode and always produce auditable profile metadata:
  - `role_mix.single.v1`
  - `role_mix.classified.v1`
  - `role_mix.hybrid.v1`
- Extended injection trace observability:
  - `session.injection.snapshot_created` now logs `injection_mode`
  - shortcut `_omni.session_context` now includes `injection_mode`

Verification commands:

- `cargo test -p omni-agent --lib injection::tests -- --nocapture`
- `cargo test -p omni-agent --test agent_injection graph_shortcut_includes_typed_injection_snapshot_metadata -- --nocapture`

### P0-4 Status Update (2026-02-22)

Completed in current branch:

- `omni.agent.cli.commands.gateway_agent` now dispatches to Rust runtime only; Python loop helpers removed.
- `omni.agent.gateway.webhook.create_webhook_app()` is explicitly decommissioned and fails fast with a migration message.
- Added Rust-orchestrator startup guard in Python CLI (`agent.runtime_orchestrator` must stay `rust`).
- `omni.agent.workflows.run_entry` is removed from the package.
- `omni.agent.core.omni` public API no longer exports Python runtime orchestrators (`OmniLoop`, `OmegaRunner`, `MissionConfig`).
- Python modules `core/omni/loop.py` and `core/omni/omega.py` are fully removed from the package.
- Python runtime modules `omni.agent.main` and `omni.agent.cli.omni_loop` are fully removed from the package.
- MCP tool-plane behavior is preserved; no compatibility fallback to Python runtime loops was added.

Verification evidence:

- `uv run pytest -n0 packages/python/agent/tests/unit/runtime/test_python_runtime_decommission.py -q`
- `uv run pytest -n0 packages/python/agent/tests/contracts/test_runtime_decommission_contract.py -q`
- `uv run pytest -n0 packages/python/agent/tests/contracts/test_data_interface_services.py -q`
- `uv run pytest -n0 packages/python/agent/tests/unit -q -W error::RuntimeWarning`

### P0-5 Status Update (Planned: Adversarial Sub-graph Routing)

**Goal:** Eradicate the regex-based `should_run_agenda_validation` placeholder.

**Action Plan:**

1. Extend `OmegaDecision` in `packages/rust/crates/omni-agent/src/contracts/omega.rs` to support `workflow_mode = "agenda_validation"`.
2. Update the Omega system prompt (or tool schema) so the LLM explicitly selects this mode when asked to schedule tasks.
3. Remove `apply_agenda_validation_if_needed` from `agent/turn_execution/react_loop/mod.rs`.
4. Intercept the request in `agent/turn_execution/shortcut.rs`. When `route == graph` and `workflow_mode == agenda_validation`, execute the Qianji `agenda_validation_pipeline.toml`.
5. Ensure the final result of the Qianji execution is returned directly as the user-facing response, avoiding the secondary ReAct loop entirely.

Execution rule:

- Land each P0 item with tests in the same change.
- Keep `omni-agent` orchestration-only; do not move memory lifecycle policy into channel/runtime handlers.
