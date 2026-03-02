---
type: knowledge
metadata:
  title: "Scripts Directory"
---

# Scripts Directory

This directory contains utility scripts for the Omni-Dev Fusion project.

## Available Scripts

| Script                                          | Purpose                                                                                                                                                                            |
| ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `generate_llm_index.py`                         | Generate skill index for LLM context                                                                                                                                               |
| `verify_skill_descriptions.py`                  | Verify skill command descriptions                                                                                                                                                  |
| `verify_system.py`                              | End-to-end smoke test for kernel                                                                                                                                                   |
| `benchmark_wendao_search.py`                    | Benchmark wendao search latency                                                                                                                                                    |
| `evaluate_wendao_retrieval.py`                  | Evaluate wendao Top1/Top3/Top10 on fixed query matrix                                                                                                                              |
| `evaluate_wendao_retrieval.sh`                  | Thin shell wrapper for `evaluate_wendao_retrieval.py`                                                                                                                              |
| `benchmark_wendao_related.py`                   | Benchmark wendao related latency and PPR diagnostics                                                                                                                               |
| `gate_wendao_ppr.sh`                            | Unified WG2/WG3 gate: retrieval matrix quality + related PPR latency/diagnostics                                                                                                   |
| `benchmark_skills_tools.py`                     | Benchmark curated safe skill tools plus CLI runner cold/warm/no-reuse cases, then persist latency snapshot baselines (`crawl4ai` supports `network_http` + `local_file` scenarios) |
| `benchmark_skills_tools_gate.sh`                | Unified wrapper for strict deterministic gate and network observability                                                                                                            |
| `benchmark_skills_tools_ci.sh`                  | Unified CI/local runner for deterministic gate + CLI summary regression diff + optional baseline promotion + network observability + trend summary                                 |
| `compare_cli_runner_summary.py`                 | Compare two CLI summary artifacts and detect per-case/per-phase latency regressions                                                                                                |
| `fetch_previous_skills_benchmark_artifact.py`   | Fetch previous successful workflow artifact and extract `cli_runner_summary(.base).json` as baseline                                                                               |
| `render_skills_tools_ci_summary.py`             | Aggregate benchmark reports into one `skills_tools_ci_status.json/.md` status summary with `improved/regressed/unchanged` trends                                                   |
| `ci-local-recall-gates.sh`                      | Unified runner for `knowledge.recall` perf gates (`auto` + `graph_only`)                                                                                                           |
| `channel/test_omni_agent_discord_acl_events.py` | Live Discord ingress ACL black-box probe for managed command denial events                                                                                                         |
| `channel/start-omni-agent-memory-ci.sh`         | Unified launcher for quick/nightly memory CI gates with latest status/failure aggregation                                                                                          |
| `channel/start-omni-agent-memory-ci-quick.sh`   | Launch quick memory CI gate in background and aggregate latest failure reports                                                                                                     |
| `channel/start-omni-agent-memory-ci-nightly.sh` | Launch nightly memory CI gate in background and aggregate latest failure reports                                                                                                   |
| `channel/memory_ci_finalize.py`                 | Shared artifact finalizer for memory CI launcher (`latest-run`, failure JSON/Markdown)                                                                                             |

### Memory CI launchers

```bash
# unified launcher (direct profile selection)
bash scripts/channel/start-omni-agent-memory-ci.sh --profile quick --foreground --ensure-mcp

# quick gate (foreground) with MCP preflight
bash scripts/channel/start-omni-agent-memory-ci-quick.sh --foreground --ensure-mcp

# nightly gate (background) with MCP preflight
bash scripts/channel/start-omni-agent-memory-ci-nightly.sh --ensure-mcp
```

### `benchmark_skills_tools.py` quick patterns

```bash
# Strict deterministic gate (recommended for CI/local; enforces CLI warm < no_reuse < cold ordering with 50ms jitter tolerance)
bash scripts/benchmark_skills_tools_gate.sh deterministic 3

# Network observability only (advisory)
bash scripts/benchmark_skills_tools_gate.sh network 5

# Include embedding-dependent tools (opt-in)
uv run python scripts/benchmark_skills_tools.py --runs 3 --include-embedding-tools --json

# Disable CLI runner benchmark cases (opt-out)
uv run python scripts/benchmark_skills_tools.py --runs 3 --no-cli-runner-cases --json

# Benchmark only CLI runner cases (demo.hello + knowledge.search, cold/warm/no-reuse)
uv run python scripts/benchmark_skills_tools.py --runs 3 --tools cli.skill_run --json

# Inspect grouped CLI profile summary from JSON report
uv run python scripts/benchmark_skills_tools.py --runs 3 --tools cli.skill_run --json | jq '.cli_runner_summary'

# Write standalone CLI runner summary artifact
uv run python scripts/benchmark_skills_tools.py --runs 3 --tools cli.skill_run --json \
  --cli-summary-file .run/reports/skills-tools-benchmark/cli_runner_summary.json

# Compare two CLI summary artifacts and fail when regressions exceed thresholds
uv run python scripts/compare_cli_runner_summary.py \
  .run/reports/skills-tools-benchmark/cli_runner_summary.base.json \
  .run/reports/skills-tools-benchmark/cli_runner_summary.json \
  --fail-on-regression --max-regression-ms 20 --max-regression-ratio 1.05

# Print resolved command only (for debugging/tests)
OMNI_SKILLS_TOOLS_GATE_DRY_RUN=1 bash scripts/benchmark_skills_tools_gate.sh deterministic 3

# Same entry points via just
just benchmark-skills-tools-gate
just benchmark-skills-tools-network-observability
just benchmark-skills-tools-ci
just knowledge-recall-perf-ci

# One-shot unified CI/local runner
bash scripts/benchmark_skills_tools_ci.sh .run/reports/skills-tools-benchmark 3 5

# Enable trend-streak guardrails (fail only after sustained regressions)
OMNI_SKILLS_TOOLS_TREND_MAX_OVERALL_STREAK=2 \
OMNI_SKILLS_TOOLS_TREND_MAX_COMPONENT_STREAK=2 \
OMNI_SKILLS_TOOLS_TREND_STRICT=1 \
  bash scripts/benchmark_skills_tools_ci.sh .run/reports/skills-tools-benchmark 3 5

# CI runner with explicit baseline summary (4th arg)
bash scripts/benchmark_skills_tools_ci.sh .run/reports/skills-tools-benchmark 3 5 \
  .run/reports/skills-tools-benchmark/cli_runner_summary.base.json

# Disable baseline promotion (default: enabled)
OMNI_SKILLS_TOOLS_CLI_SUMMARY_PROMOTE_BASELINE=0 \
  bash scripts/benchmark_skills_tools_ci.sh .run/reports/skills-tools-benchmark 3 5

# Loosen bootstrap-only diff thresholds (keep non-bootstrap thresholds strict)
OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_MS=60 \
OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_RATIO=1.7 \
  bash scripts/benchmark_skills_tools_ci.sh .run/reports/skills-tools-benchmark 3 5

# Enable remote baseline inheritance from previous successful CI run artifact
OMNI_SKILLS_TOOLS_REMOTE_ARTIFACT_NAME=skills-tools-benchmark-ubuntu-latest \
  bash scripts/benchmark_skills_tools_ci.sh .run/reports/skills-tools-benchmark 3 5

# Disable remote baseline/status inheritance
OMNI_SKILLS_TOOLS_FETCH_REMOTE_BASELINE=0 \
  bash scripts/benchmark_skills_tools_ci.sh .run/reports/skills-tools-benchmark 3 5

# Render one-file CI status summary markdown/json from report artifacts
uv run python scripts/render_skills_tools_ci_summary.py \
  --deterministic-report .run/reports/skills-tools-benchmark/deterministic_gate.json \
  --cli-diff-report .run/reports/skills-tools-benchmark/cli_runner_summary_diff.json \
  --remote-fetch-report .run/reports/skills-tools-benchmark/cli_runner_summary_remote_fetch.json \
  --network-report .run/reports/skills-tools-benchmark/crawl4ai_network_observability.json \
  --baseline-file .run/reports/skills-tools-benchmark/cli_runner_summary.base.json \
  --artifact-file .run/reports/skills-tools-benchmark/cli_runner_summary.json \
  --previous-status-json .run/reports/skills-tools-benchmark/skills_tools_ci_status.previous.json \
  --max-overall-regression-streak 2 \
  --max-component-regression-streak 2 \
  --strict-trend-alert \
  --output-json .run/reports/skills-tools-benchmark/skills_tools_ci_status.json \
  --output-markdown .run/reports/skills-tools-benchmark/skills_tools_ci_status.md

# Print resolved commands only (for debugging/tests)
OMNI_SKILLS_TOOLS_CI_DRY_RUN=1 bash scripts/benchmark_skills_tools_ci.sh

# One-shot knowledge.recall perf gates (auto + graph_only)
bash scripts/ci-local-recall-gates.sh 3 1 x 2 .run/reports/knowledge-recall-perf

# Print resolved commands only (for debugging/tests)
OMNI_RECALL_GATES_DRY_RUN=1 bash scripts/ci-local-recall-gates.sh
```

## Running Scripts

All scripts should be run from the project root:

```bash
# Using uv (recommended)
uv run python scripts/script_name.py

# Or directly with python
python scripts/script_name.py
```

## Database Commands

Database operations are now available via the `omni db` CLI command:

```bash
# List all databases
omni db list

# Query knowledge base
omni db query "error handling"

# Show database statistics
omni db stats

# Count records in table
omni db count <table_name>
```
