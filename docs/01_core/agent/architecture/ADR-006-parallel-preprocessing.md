---
type: knowledge
title: "ADR-006: Parallel Turn Preprocessing Pipeline"
status: "Accepted"
date: "2026-02-28"
category: "architecture"
tags:
  - agent
  - adr
  - performance
  - concurrency
metadata:
  title: "ADR-006: Parallel Turn Preprocessing Pipeline"
---

# ADR-006: Parallel Turn Preprocessing Pipeline

## 1. Context and Problem Statement

The `run_react_loop` in `omni-agent` is the primary entry point for turn execution. Currently, it follows a strict sequential execution model:

1. `prepare_react_decision` (await)
2. `prepare_react_messages` (await) - Fetches history from Redis.
3. `apply_agenda_validation_if_needed` (await) - Potential multi-LLM call.
4. `apply_memory_recall_if_enabled` (await) - Vector search.

This serial chain results in high **Time To First Token (TTFT)**. Even if each step is fast, the cumulative latency is noticeable, especially on IM platforms like Telegram. Furthermore, heavy validation tasks (like `agenda_validation`) are executed even for queries where they are irrelevant.

## 2. Decision

We will refactor the turn execution entry point into a **Parallel Preprocessing Pipeline**.

1.  **Concurrency via `tokio::join!`**: Tasks that do not have data dependencies (e.g., fetching history, performing vector search, and running the agenda validation intent-check) will be executed concurrently.
2.  **Stateless Decomposition**: Refactor "In-place modification" functions (those taking `&mut Vec<ChatMessage>`) into "Pure Producers" that return results to be merged.
3.  **Adaptive Validation Policy**: Introduce an `agenda_validation_policy` switch ("always", "never", "auto"). In "auto" mode, the full validation pipeline is only triggered if an initial intent-check (performed in parallel) confirms a scheduling-related query.

## 3. Technical Design

### 3.1 The Unified Outcome Structure

```rust
struct PreprocessingOutcome {
    decision: (OmegaDecision, Option<PolicyHintDirective>),
    messages_result: Result<ReactPreparedMessages>,
    validation_hint: Option<ChatMessage>,
    memory_results: Vec<RecallCandidate>,
}
```

### 3.2 Parallel Execution Flow

The orchestrator will use `tokio::try_join!` or `tokio::join!` to resolve independent branches:

- **Branch A**: Decision Logic + History Retrieval.
- **Branch B**: Agenda Validation (Intent Check + Critique).
- **Branch C**: Memory Recall.

## 4. Consequences

### Positive

- **Reduced TTFT**: Cumulative latency is reduced to the duration of the longest single step (usually the LLM validation).
- **Resource Efficiency**: In "auto" mode, heavy LLM-based audit nodes are skipped for simple queries.
- **Improved UX**: Users receive responses significantly faster.

### Negative

- **Complex Merging**: Care must be taken to maintain the correct ordering of `system` messages (Persona vs. Summary vs. Validation Hints).
- **Concurrency Overhead**: Slight increase in CPU/Connection usage due to simultaneous requests to Redis and LLM providers.

## 5. Implementation Status

- `tokio::join!` parallel branches are active in `omni-agent` turn execution (`decision/messages`, `agenda_validation/memory_recall`).
- Runtime switch `agent.agenda_validation_policy` (`always` / `never` / `auto`) is wired through merged `xiuxian.toml` + env override `OMNI_AGENT_AGENDA_VALIDATION_POLICY`.
- `auto` mode now uses a lightweight LLM gate (`RUN`/`SKIP`) before invoking the full agenda-validation pipeline.
