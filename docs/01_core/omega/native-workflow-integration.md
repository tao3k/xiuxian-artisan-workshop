---
type: knowledge
title: "Workflow Runtime Architecture - Omni-Dev-Fusion"
category: "architecture"
tags:
  - architecture
  - workflow_runtime
saliency_base: 7.0
decay_rate: 0.03
metadata:
  title: "Workflow Runtime Architecture - Omni-Dev-Fusion"
---

# Workflow Runtime Architecture - Omni-Dev-Fusion

> Cognitive state-machine runtime for agent workflows.
> Last Updated: 2026-02-25

## Overview

The workflow runtime provides the deterministic control plane for Omni Agent execution.
It powers ReAct-style loops, multi-step plans, and skill orchestration with native runtime
components instead of external graph-runtime dependencies.

Core goals:

- Deterministic state transitions
- Native async execution
- Checkpoint-capable compilation
- Modular node composition
- Clear separation between runtime, tracing, and domain logic

## Runtime Layers

```text
User Query
    |
    v
+-------------------------------+
| Agent Workflow Entrypoints    |
| - robust_task.graph           |
| - memory.graph                |
| - core/omni/graph/workflow    |
+---------------+---------------+
                |
                v
+-------------------------------+
| Native Workflow Engine        |
| - NativeStateGraph            |
| - END_NODE routing            |
| - NativeCompiledWorkflow      |
+---------------+---------------+
                |
                v
+-------------------------------+
| Pipeline Runtime + Tracer     |
| - create_workflow_from_*      |
| - compile_workflow            |
| - graphflow runtime/tracer    |
+---------------+---------------+
                |
                v
+-------------------------------+
| State + Persistence           |
| - workflow_state checkpointer |
| - vector/memory subsystems    |
+-------------------------------+
```

## Key Modules

| Component                 | Purpose                                 | Location                                                            |
| ------------------------- | --------------------------------------- | ------------------------------------------------------------------- |
| `NativeStateGraph`        | Graph construction API                  | `packages/python/foundation/src/omni/tracer/workflow_engine.py`     |
| `NativeCompiledWorkflow`  | Async execution engine                  | `packages/python/foundation/src/omni/tracer/workflow_engine.py`     |
| `compile_workflow`        | Checkpointer-aware compilation          | `packages/python/foundation/src/omni/tracer/pipeline_checkpoint.py` |
| `create_workflow_from_*`  | Pipeline factory APIs                   | `packages/python/foundation/src/omni/tracer/pipeline_runtime.py`    |
| `PipelineWorkflowBuilder` | Declarative pipeline-to-graph builder   | `packages/python/foundation/src/omni/tracer/pipeline_builder.py`    |
| `run_graphflow_pipeline`  | Feature runtime for graphflow scenarios | `packages/python/foundation/src/omni/tracer/graphflow/runtime.py`   |

## Agent Entrypoints

Agent-side workflow entrypoints are now native:

- `packages/python/agent/src/omni/agent/workflows/robust_task/graph.py`
- `packages/python/agent/src/omni/agent/workflows/memory/graph.py`
- `packages/python/agent/src/omni/agent/core/omni/graph/workflow.py`

These modules define node logic and routing policies while delegating runtime semantics to
native graph execution primitives.

## Checkpoint and Resume

Checkpoint integration is handled by `compile_workflow(...)` and the project-wide
workflow-state API. Runtime callers can inject a custom checkpointer or enable
in-memory default behavior when appropriate.

Guidelines:

- Keep checkpoint contracts runtime-agnostic.
- Do not couple retrieval/factory layers to tracer pipeline internals.
- Prefer explicit state schema types for multi-step workflows.

## Design Principles

1. Runtime-first: execution semantics belong to native runtime modules.
2. Domain isolation: business logic stays in nodes, not engine internals.
3. Typed boundaries: use typed state contracts for non-trivial flows.
4. Explicit orchestration: routing conditions are first-class, testable functions.
5. Minimal wrappers: skill commands delegate to package runtime APIs.

## Migration Note

This document supersedes legacy graph-runtime architecture references.
Use `omni.tracer` native workflow APIs and agent workflow modules listed above as the
current source of truth.
