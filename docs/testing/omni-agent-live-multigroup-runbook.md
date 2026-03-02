---
type: knowledge
title: "Omni-Agent Live Multi-Group Runbook"
category: "testing"
tags:
  - testing
  - omni
saliency_base: 6.5
decay_rate: 0.04
metadata:
  title: "Omni-Agent Live Multi-Group Runbook"
---

# Omni-Agent Live Multi-Group Runbook

This runbook standardizes live black-box validation on three real Telegram groups (`Test1`, `Test2`, `Test3`).

## 1. Preconditions

1. `omni-agent` webhook runtime is already running.
2. Runtime logs are written to `.run/logs/omni-agent-webhook.log` (or a known path).
3. You have posted at least one message (for example `/help`) in each target group so the runtime log contains `chat_id` + `chat_title`.

## 2. Capture Group Profile

```bash
python3 scripts/channel/capture_telegram_group_profile.py \
  --titles Test1,Test2,Test3 \
  --log-file .run/logs/omni-agent-webhook.log \
  --output-json .run/config/agent-channel-groups.json \
  --output-env .run/config/agent-channel-groups.env
```

Load the generated profile:

```bash
set -a
source .run/config/agent-channel-groups.env
set +a
```

## 3. Run Live Session Isolation Matrix

```bash
python3 scripts/channel/test_omni_agent_session_matrix.py \
  --chat-id "$OMNI_TEST_CHAT_ID" \
  --chat-b "$OMNI_TEST_CHAT_B" \
  --chat-c "$OMNI_TEST_CHAT_C" \
  --user-a "$OMNI_TEST_USER_ID" \
  --user-b "$OMNI_TEST_USER_B" \
  --user-c "$OMNI_TEST_USER_C" \
  --max-wait 45 \
  --max-idle-secs 30 \
  --output-json .run/reports/agent-channel-session-matrix-live.json \
  --output-markdown .run/reports/agent-channel-session-matrix-live.md
```

Pass condition:

- `overall_passed=true`
- all steps pass
- three distinct group IDs are present.

## 3.5 Run Live Command Event Core Suite

Load group profile env first, then run core command-event probes:

```bash
set -a
source .run/config/agent-channel-groups.env
set +a

python3 scripts/channel/test_omni_agent_command_events.py \
  --suite core \
  --allow-chat-id "$OMNI_TEST_CHAT_ID" \
  --username tao3k \
  --max-wait 35 \
  --max-idle-secs 25 \
  --output-json .run/reports/agent-channel-command-events-core-live.json \
  --output-markdown .run/reports/agent-channel-command-events-core-live.md
```

Pass condition:

- all selected core probes pass (`session_status_json`, `session_budget_json`, `session_memory_json`, `session_feedback_up_json`)
- no forbidden MCP error regex is matched in runtime logs.

## 4. Run Live Memory Evolution DAG

```bash
python3 scripts/channel/test_omni_agent_complex_scenarios.py \
  --dataset scripts/channel/fixtures/memory_evolution_complex_scenarios.json \
  --scenario memory_self_correction_high_complexity_dag \
  --chat-a "$OMNI_TEST_CHAT_ID" \
  --chat-b "$OMNI_TEST_CHAT_B" \
  --chat-c "$OMNI_TEST_CHAT_C" \
  --user-a "$OMNI_TEST_USER_ID" \
  --user-b "$OMNI_TEST_USER_B" \
  --user-c "$OMNI_TEST_USER_C" \
  --max-wait 90 \
  --max-idle-secs 40 \
  --max-parallel 1 \
  --output-json .run/reports/omni-agent-memory-evolution-live.json \
  --output-markdown .run/reports/omni-agent-memory-evolution-live.md
```

Pass condition:

- scenario passes
- quality gates meet the dataset thresholds.

## 5. Run Live Trace Reconstruction

```bash
python3 scripts/channel/reconstruct_omni_agent_trace.py \
  .run/logs/omni-agent-webhook.log \
  --session-id "telegram:${OMNI_TEST_CHAT_ID}" \
  --max-events 4000 \
  --required-stage route \
  --required-stage injection \
  --required-stage reflection \
  --required-stage memory \
  --json-out .run/reports/omni-agent-trace-reconstruction-live.json \
  --markdown-out .run/reports/omni-agent-trace-reconstruction-live.md
```

Pass condition:

- required stages present
- quality score is `100.0`
- no reconstruction errors.

## 6. Live ReAct Resilience & Memory Evolution Scenarios (Manual Validation)

To prove that the features `R-01`, `R-02`, and `R-03` work in a real-world environment (like Telegram), execute the following manual test scenarios in your test groups.

### Scenario A: Reflection-Driven Correction (R-01)

1. **Trigger Error:** Send a message to the bot asking it to execute a tool with intentionally invalid or malformed parameters (e.g., asking it to read a file that doesn't exist using `file_read` but formatting the JSON terribly).
2. **Observe Failure:** Wait for the bot to reply with a failure message or a hallucinated response.
3. **Trigger Correction:** Immediately reply with: `"Try that again, but pay attention to the error you just made."`
4. **Validation:** In the `.run/logs/omni-agent-webhook.log`, search for `agent.next_turn_hint`. You should see that `omni-memory` caught the previous failure, generated a reflection hint, and `xiuxian-qianhuan` successfully injected it into this second turn's prompt, causing the bot to correct its behavior.

### Scenario B: ReAct Budget Pressure & Anchor Survival (R-02)

1. **Trigger Token Explosion:** Send a message asking the bot to run a command that produces a massive output (e.g., `"Run the 'ls -R /' command or read the entire cargo.lock file and print it."`).
2. **Observe Truncation:** The bot should reply, but it shouldn't crash or "forget who it is".
3. **Validation:** Check the logs for `session.injection.snapshot_created`. Look at the `dropped_blocks` or `truncated_blocks` metrics. You should see that the massive tool output was truncated by `omni-window` to respect the `max_chars` budget, but the core XML tags like `<genesis_rules>` were preserved.

### Scenario C: Dynamic Role-Mix Switching (R-03)

1. **Trigger Recovery Mode:** Induce a severe failure (similar to Scenario A) that causes the tool loop to fail completely.
2. **Observe Tone Shift:** In the very next interaction, ask the bot `"What happened?"`.
3. **Validation:** The bot's response should shift from its normal helpful tone to a strict, analytical "Recovery/Debug" persona. In the logs, verify that Omega selected the `role_mix_profile=recovery` and Qianhuan injected it successfully.

## 7. Release Artifact Checklist

Attach these files to the release/test evidence set:

1. `.run/config/agent-channel-groups.json`
2. `.run/reports/agent-channel-session-matrix-live.json`
3. `.run/reports/omni-agent-memory-evolution-live.json`
4. `.run/reports/omni-agent-trace-reconstruction-live.json`
5. `.run/reports/agent-channel-command-events-core-live.json`
