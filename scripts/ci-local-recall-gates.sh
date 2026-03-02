#!/usr/bin/env bash
set -euo pipefail

RUNS="${1:-3}"
WARM_RUNS="${2:-1}"
QUERY="${3:-x}"
LIMIT="${4:-2}"
REPORT_DIR="${5:-.run/reports/knowledge-recall-perf}"

mkdir -p "${REPORT_DIR}"

AUTO_P95_MS="${OMNI_KNOWLEDGE_RECALL_P95_MS:-2500}"
AUTO_RSS_MB="${OMNI_KNOWLEDGE_RECALL_RSS_PEAK_DELTA_MB:-320}"
AUTO_ROW_BUDGET_RSS_MB="${OMNI_KNOWLEDGE_RECALL_ROW_BUDGET_RSS_PEAK_DELTA_MB:-${AUTO_RSS_MB}}"
GRAPH_P95_MS="${OMNI_KNOWLEDGE_RECALL_GRAPH_P95_MS:-1800}"
GRAPH_RSS_MB="${OMNI_KNOWLEDGE_RECALL_GRAPH_RSS_PEAK_DELTA_MB:-280}"
GRAPH_ROW_BUDGET_RSS_MB="${OMNI_KNOWLEDGE_RECALL_GRAPH_ROW_BUDGET_RSS_PEAK_DELTA_MB:-${GRAPH_RSS_MB}}"
# Row-budget memory telemetry may be unavailable on some environments/configs.
# Keep default non-blocking; CI can raise this via env when telemetry is guaranteed.
MIN_ROW_BUDGET_OBSERVED="${OMNI_KNOWLEDGE_RECALL_ROW_BUDGET_MEMORY_OBSERVED_MIN:-0}"
MAX_FAILURES="${OMNI_KNOWLEDGE_RECALL_MAX_FAILURES:-0}"

auto_cmd=(
  uv run python scripts/knowledge_recall_perf_gate.py
  --query "${QUERY}"
  --limit "${LIMIT}"
  --runs "${RUNS}"
  --warm-runs "${WARM_RUNS}"
  --retrieval-mode auto
  --max-p95-ms "${AUTO_P95_MS}"
  --max-rss-peak-delta-mb "${AUTO_RSS_MB}"
  --max-row-budget-rss-peak-delta-mb "${AUTO_ROW_BUDGET_RSS_MB}"
  --min-row-budget-memory-observed-runs "${MIN_ROW_BUDGET_OBSERVED}"
  --max-failures "${MAX_FAILURES}"
  --json-output "${REPORT_DIR}/auto.json"
)

graph_cmd=(
  uv run python scripts/knowledge_recall_perf_gate.py
  --query "${QUERY}"
  --limit "${LIMIT}"
  --runs "${RUNS}"
  --warm-runs "${WARM_RUNS}"
  --retrieval-mode graph_only
  --max-p95-ms "${GRAPH_P95_MS}"
  --max-rss-peak-delta-mb "${GRAPH_RSS_MB}"
  --max-row-budget-rss-peak-delta-mb "${GRAPH_ROW_BUDGET_RSS_MB}"
  --min-row-budget-memory-observed-runs "${MIN_ROW_BUDGET_OBSERVED}"
  --max-failures "${MAX_FAILURES}"
  --json-output "${REPORT_DIR}/graph_only.json"
)

if [[ ${OMNI_RECALL_GATES_DRY_RUN:-} == "1" || ${OMNI_RECALL_GATES_DRY_RUN:-} == "true" ]]; then
  printf 'mkdir -p %q\n' "${REPORT_DIR}"
  printf '%q ' "${auto_cmd[@]}"
  printf '\n'
  printf '%q ' "${graph_cmd[@]}"
  printf '\n'
  exit 0
fi

echo "knowledge.recall perf gate (auto/hybrid)"
"${auto_cmd[@]}"

echo "knowledge.recall perf gate (graph_only)"
"${graph_cmd[@]}"

echo ""
echo "reports:"
echo "  ${REPORT_DIR}/auto.json"
echo "  ${REPORT_DIR}/graph_only.json"
