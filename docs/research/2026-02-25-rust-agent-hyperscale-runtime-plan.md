# Rust Agent Hyperscale Runtime Plan (10k Bot Readiness)

## Scope

This document defines a practical scaling plan for Rust-first omni-agent runtime under high fan-in traffic (for example, many Discord/Telegram bots with concurrent active sessions).

Goals:

- Keep session isolation correct under burst traffic.
- Prevent unbounded memory growth from in-process task accumulation.
- Maintain predictable latency through explicit backpressure and queue budgets.
- Keep Python embedding fallback out of the steady-state data plane.

## Current Runtime Baseline

### Channel ingress and foreground

- Telegram runtime already uses two bounded queues:
  - inbound queue (`telegram.inbound_queue_capacity`)
  - foreground queue (`telegram.foreground_queue_capacity`)
- Discord runtime uses bounded inbound queue and in-flight semaphore for foreground turns.

### Embedding/LLM role boundary

- `litellm_rs`: provider-routing path.
- `mistral_local`: explicit local runtime path.
- Rust embedding/LLM path is the primary runtime path; Python MCP embedding fallback is removed from agent steady-state flow.

## Hardening Added in This Iteration

### 1) Discord foreground scheduling backpressure

Discord foreground scheduling now acquires the foreground semaphore **before** spawning the turn task.

Impact:

- Prevents unbounded accumulation of spawned tasks waiting on semaphore permits.
- Converts saturation into bounded ingress wait (controlled by inbound queue + semaphore budget).

### 2) Structured queue-wait saturation signals

Added structured warning events when queue send waits indicate pressure:

- `discord.ingress.inbound_queue_wait`
- `discord.gateway.inbound_queue_wait`
- `telegram.webhook.inbound_queue_wait`
- `discord.foreground.gate_wait`

These events provide direct observability for saturation onset without requiring full tracing pipelines.

## Target Architecture for 10k Bot Scenario

### Plane split

- Control plane:
  - Config distribution
  - Bot/token lifecycle
  - ACL and policy mutation
  - Runtime health aggregation
- Data plane:
  - Message ingress
  - Session routing and partitioning
  - Foreground turn execution
  - Background job execution
  - Memory persistence/recall

### Isolation hierarchy

- Keep session key partitioning explicit per channel:
  - Discord default: `guild_channel_user`
  - Telegram default: chat+topic+user compatible partitioning
- All foreground generation, interruption, and memory writes must be keyed by session id.
- ACL resolution must remain recipient-scoped to avoid cross-guild/group bleed-through.

### Backpressure chain (must remain bounded)

1. Ingress endpoint/gateway receive
2. Inbound queue (bounded)
3. Foreground scheduling gate (bounded by `max_in_flight`)
4. Background jobs queue (bounded)
5. Embedding batch gate (`max_in_flight`)
6. LLM request gate (`max_in_flight`)

If any stage saturates, wait/backoff should happen at the nearest upstream stage, not by unbounded task creation.

## Capacity Model (Operational)

### Concurrency budget equation

For each runtime process:

- `foreground_active <= foreground_max_in_flight`
- `foreground_queued <= inbound_queue_capacity`
- `embedding_active <= embedding.max_in_flight`
- `llm_active <= inference.max_in_flight`

Recommended initial guardrail:

- `foreground_max_in_flight <= min(embedding.max_in_flight, inference.max_in_flight)`

This prevents foreground workers from generating more downstream load than embedding/LLM layers can drain.

### SLO and alert thresholds

Track and alert on:

- queue-wait events (`*_inbound_queue_wait`, `discord.foreground.gate_wait`) sustained over rolling windows
- embedding gate wait p95 / p99
- embedding request success rate
- turn timeout rate
- background queue stalled state (`oldest_queued_age_secs > threshold`)

Suggested initial targets:

- foreground queue/gate wait p95 < 100 ms under normal load
- embedding single p95 < 220 ms, batch(8) p95 < 320 ms
- timeout rate < 1% (excluding forced chaos tests)

## Rollout Plan

### Phase 1: Saturation visibility and boundedness (done)

- Add queue/gate wait structured events.
- Ensure Discord foreground scheduling is bounded before spawn.

### Phase 2: Multi-instance pressure validation

- Run concurrent session stress on webhook + gateway paths.
- Validate no unbounded task growth and stable memory footprint over sustained load.

### Phase 3: Policy-driven autoscaling inputs

- Convert queue-wait and gate-wait metrics into autoscaling signals.
- Define scale-out policy by sustained wait + timeout + throughput drop.

### Phase 4: Fault-injection hardening

- Inject provider/network failures on embedding/LLM downstream.
- Verify bounded degradation (no cascade or runaway memory/task growth).

## Verification Commands

- Backend role contracts:
  - `just rust-omni-agent-backend-role-contracts`
- Embedding role perf smoke:
  - `just rust-omni-agent-embedding-role-perf-smoke`
- Focused channel regressions:
  - `cargo test -p omni-agent --test channels_discord_ingress`
  - `cargo test -p omni-agent --test channels_webhook`
- Discord ingress stress harness:
  - `just agent-channel-discord-ingress-stress`
  - `just agent-channel-discord-ingress-stress 6 1 8 20 10 0.2 "" "2001" "1001" "3001"`
  - Output reports:
    - `.run/reports/omni-agent-discord-ingress-stress.json`
    - `.run/reports/omni-agent-discord-ingress-stress.md`

## Next Implementation Candidates

1. Add periodic runtime snapshots (queue depth, in-flight, gate wait histogram buckets) to a single structured telemetry stream.
2. Add a discord ingress stress harness equivalent to current webhook stress script conventions.
   - Status: implemented (`scripts/channel/test_omni_agent_discord_ingress_stress.py`).
3. Add process-level admission control by channel/provider when downstream embedding/LLM saturation crosses thresholds.
