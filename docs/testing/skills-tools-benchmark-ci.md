---
type: knowledge
title: "Skills Tools Benchmark CI Gate"
category: "testing"
tags:
  - benchmark
  - ci
  - skills
  - regression
saliency_base: 6.8
decay_rate: 0.04
metadata:
  title: "Skills Tools Benchmark CI Gate"
---

# Skills Tools Benchmark CI Gate

This document defines the canonical threshold policy and execution path for the
skills-tools benchmark gate.

## Scope

The gate validates:

- deterministic benchmark behavior (`benchmark_skills_tools_gate.sh deterministic`)
- CLI runner summary diff against baseline (`compare_cli_runner_summary.py`)
- optional remote baseline/status inheritance
- trend summary and streak alerts (`render_skills_tools_ci_summary.py`)
- network observability lane (`benchmark_skills_tools_gate.sh network`, advisory)

## Runtime Entry Point

Primary CI/local entry:

- `devenv tasks run ci:benchmark-skills-tools`

Task wiring:

- `nix/modules/tasks.nix` exports gate env defaults.
- `scripts/benchmark_skills_tools_ci.sh` executes deterministic gate, diff gate,
  baseline promotion, trend render, and network observability.
- `.github/workflows/ci.yaml` uses the same task path.

## Canonical Thresholds (Current)

Default thresholds used by CI/task wiring:

- `OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_MAX_MS=70`
- `OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_MAX_RATIO=1.2`
- `OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_MS=60`
- `OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_RATIO=1.7`
- `OMNI_SKILLS_TOOLS_TREND_MAX_OVERALL_STREAK=2`
- `OMNI_SKILLS_TOOLS_TREND_MAX_COMPONENT_STREAK=2`
- `OMNI_SKILLS_TOOLS_TREND_STRICT=1`

## Why These Thresholds

### 1) Non-bootstrap metrics: `70ms / 1.2`

`case.p50_ms`, `timing.daemon_connect.p50_ms`, and `timing.tool_exec.p50_ms`
represent the main user-impact path once process startup noise is excluded.

`20ms / 1.05` was too brittle in local/CI replay due to normal scheduler and
cache jitter. `70ms / 1.2` still catches meaningful latency regressions while
removing false positives from single-run noise.

### 2) Bootstrap metric: `60ms / 1.7`

`timing.bootstrap.p50_ms` is inherently noisier (process spawn, import/runtime
init, host load). It is not a stable proxy for runtime algorithm changes.

A dedicated bootstrap threshold prevents startup volatility from masking real
runtime regressions in `daemon_connect/tool_exec`.

`default_cold` daemon-spawn bootstrap is excluded from blocking diff comparison
because it is tightly coupled to process/runtime jitter in CI hosts.

### 3) Trend strict mode: streak-based enforcement

One-off regressions can be environmental. Trend strict mode fails only when
regression streaks persist (`>=2`) at overall or component level. This keeps the
gate strict while reducing flake-driven noise.

Execution policy:

- CLI diff is always generated (for visibility) and then consumed by trend logic.
- Blocking failure comes from trend-alert thresholds (not one-shot diff noise).
- Baseline auto-promotion is skipped whenever `cli_diff.regression_count > 0`.

## Expected Artifacts

Generated under `.run/reports/skills-tools-benchmark/`:

- `deterministic_gate.json`
- `cli_runner_summary.json`
- `cli_runner_summary.base.json`
- `cli_runner_summary_diff.json`
- `skills_tools_ci_status.json`
- `skills_tools_ci_status.md`
- `crawl4ai_network_observability.json`

## Tuning Rule

When adjusting thresholds, change all three together:

1. task defaults in `nix/modules/tasks.nix`
2. CI env in `.github/workflows/ci.yaml`
3. rationale section in this document

Do not tune only workflow envs without updating task defaults and docs.
