#!/usr/bin/env bash
set -euo pipefail

REPORT_DIR="${1:-.run/reports/skills-tools-benchmark}"
DETERMINISTIC_RUNS="${2:-3}"
NETWORK_RUNS="${3:-5}"
CLI_SUMMARY_BASELINE_INPUT="${4:-}"
CLI_SUMMARY_ARTIFACT="${REPORT_DIR}/cli_runner_summary.json"
CLI_SUMMARY_DIFF_REPORT="${REPORT_DIR}/cli_runner_summary_diff.json"
CLI_SUMMARY_REMOTE_FETCH_REPORT="${REPORT_DIR}/cli_runner_summary_remote_fetch.json"
PREVIOUS_STATUS_JSON="${REPORT_DIR}/skills_tools_ci_status.previous.json"
PREVIOUS_STATUS_FETCH_REPORT="${REPORT_DIR}/skills_tools_ci_status_previous_fetch.json"
CI_STATUS_JSON="${REPORT_DIR}/skills_tools_ci_status.json"
CI_STATUS_MARKDOWN="${REPORT_DIR}/skills_tools_ci_status.md"
DEFAULT_CLI_SUMMARY_BASELINE="${REPORT_DIR}/cli_runner_summary.base.json"
CLI_SUMMARY_BASELINE="${CLI_SUMMARY_BASELINE_INPUT:-${OMNI_SKILLS_TOOLS_CLI_SUMMARY_BASELINE:-${DEFAULT_CLI_SUMMARY_BASELINE}}}"
CLI_SUMMARY_DIFF_MAX_MS="${OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_MAX_MS:-70}"
CLI_SUMMARY_DIFF_MAX_RATIO="${OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_MAX_RATIO:-1.2}"
CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_MS="${OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_MS:-60}"
CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_RATIO="${OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_RATIO:-1.7}"
CLI_SUMMARY_PROMOTE_BASELINE="${OMNI_SKILLS_TOOLS_CLI_SUMMARY_PROMOTE_BASELINE:-1}"
FETCH_REMOTE_BASELINE="${OMNI_SKILLS_TOOLS_FETCH_REMOTE_BASELINE:-1}"
REMOTE_WORKFLOW_FILE="${OMNI_SKILLS_TOOLS_REMOTE_WORKFLOW_FILE:-ci.yaml}"
REMOTE_ARTIFACT_NAME="${OMNI_SKILLS_TOOLS_REMOTE_ARTIFACT_NAME:-}"
TREND_MAX_OVERALL_STREAK="${OMNI_SKILLS_TOOLS_TREND_MAX_OVERALL_STREAK:-0}"
TREND_MAX_COMPONENT_STREAK="${OMNI_SKILLS_TOOLS_TREND_MAX_COMPONENT_STREAK:-0}"
TREND_STRICT="${OMNI_SKILLS_TOOLS_TREND_STRICT:-0}"

is_truthy() {
  case "${1:-}" in
  1 | true | TRUE | yes | YES | on | ON)
    return 0
    ;;
  *)
    return 1
    ;;
  esac
}

deterministic_cmd=(
  bash scripts/benchmark_skills_tools_gate.sh deterministic "${DETERMINISTIC_RUNS}"
  --cli-summary-file "${CLI_SUMMARY_ARTIFACT}"
)
network_cmd=(
  bash scripts/benchmark_skills_tools_gate.sh network "${NETWORK_RUNS}"
)
compare_cmd=(
  uv run python scripts/compare_cli_runner_summary.py
  "${CLI_SUMMARY_BASELINE}"
  "${CLI_SUMMARY_ARTIFACT}"
  --max-regression-ms "${CLI_SUMMARY_DIFF_MAX_MS}"
  --max-regression-ratio "${CLI_SUMMARY_DIFF_MAX_RATIO}"
)
if [ -n "${CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_MS}" ]; then
  compare_cmd+=(--bootstrap-max-regression-ms "${CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_MS}")
fi
if [ -n "${CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_RATIO}" ]; then
  compare_cmd+=(--bootstrap-max-regression-ratio "${CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_RATIO}")
fi
render_cmd=(
  uv run python scripts/render_skills_tools_ci_summary.py
  --deterministic-report "${REPORT_DIR}/deterministic_gate.json"
  --cli-diff-report "${CLI_SUMMARY_DIFF_REPORT}"
  --remote-fetch-report "${CLI_SUMMARY_REMOTE_FETCH_REPORT}"
  --network-report "${REPORT_DIR}/crawl4ai_network_observability.json"
  --baseline-file "${CLI_SUMMARY_BASELINE}"
  --artifact-file "${CLI_SUMMARY_ARTIFACT}"
  --previous-status-json "${PREVIOUS_STATUS_JSON}"
  --output-json "${CI_STATUS_JSON}"
  --output-markdown "${CI_STATUS_MARKDOWN}"
  --max-overall-regression-streak "${TREND_MAX_OVERALL_STREAK}"
  --max-component-regression-streak "${TREND_MAX_COMPONENT_STREAK}"
)
if is_truthy "${TREND_STRICT}"; then
  render_cmd+=(--strict-trend-alert)
fi

if [[ ${OMNI_SKILLS_TOOLS_CI_DRY_RUN:-} == "1" || ${OMNI_SKILLS_TOOLS_CI_DRY_RUN:-} == "true" ]]; then
  printf 'mkdir -p %q\n' "${REPORT_DIR}"
  printf '%q ' "${deterministic_cmd[@]}"
  printf '> %q\n' "${REPORT_DIR}/deterministic_gate.json"
  printf '# artifact: %q\n' "${CLI_SUMMARY_ARTIFACT}"
  printf '# baseline: %q\n' "${CLI_SUMMARY_BASELINE}"
  printf '# diff_report: %q\n' "${CLI_SUMMARY_DIFF_REPORT}"
  printf '# remote_fetch_report: %q\n' "${CLI_SUMMARY_REMOTE_FETCH_REPORT}"
  printf '# previous_status_file: %q\n' "${PREVIOUS_STATUS_JSON}"
  printf '# previous_status_fetch_report: %q\n' "${PREVIOUS_STATUS_FETCH_REPORT}"
  printf '# ci_status_json: %q\n' "${CI_STATUS_JSON}"
  printf '# ci_status_markdown: %q\n' "${CI_STATUS_MARKDOWN}"
  printf '# trend_thresholds: overall=%q component=%q strict=%q\n' \
    "${TREND_MAX_OVERALL_STREAK}" \
    "${TREND_MAX_COMPONENT_STREAK}" \
    "${TREND_STRICT}"
  printf '# cli_diff_thresholds: max_ms=%q max_ratio=%q bootstrap_max_ms=%q bootstrap_max_ratio=%q\n' \
    "${CLI_SUMMARY_DIFF_MAX_MS}" \
    "${CLI_SUMMARY_DIFF_MAX_RATIO}" \
    "${CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_MS}" \
    "${CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_RATIO}"
  if [ ! -f "${CLI_SUMMARY_BASELINE}" ] && is_truthy "${FETCH_REMOTE_BASELINE}" && [ -n "${REMOTE_ARTIFACT_NAME}" ]; then
    printf 'uv run python scripts/fetch_previous_skills_benchmark_artifact.py --artifact-name %q --workflow-file %q --output %q > %q\n' \
      "${REMOTE_ARTIFACT_NAME}" \
      "${REMOTE_WORKFLOW_FILE}" \
      "${CLI_SUMMARY_BASELINE}" \
      "${CLI_SUMMARY_REMOTE_FETCH_REPORT}"
  elif [ ! -f "${CLI_SUMMARY_BASELINE}" ] && is_truthy "${FETCH_REMOTE_BASELINE}" && [ -z "${REMOTE_ARTIFACT_NAME}" ]; then
    printf '# remote baseline fetch skipped: OMNI_SKILLS_TOOLS_REMOTE_ARTIFACT_NAME is empty\n'
  fi
  if [ ! -f "${PREVIOUS_STATUS_JSON}" ] && is_truthy "${FETCH_REMOTE_BASELINE}" && [ -n "${REMOTE_ARTIFACT_NAME}" ]; then
    printf 'uv run python scripts/fetch_previous_skills_benchmark_artifact.py --artifact-name %q --workflow-file %q --preferred-member %q --fallback-member %q --output %q > %q\n' \
      "${REMOTE_ARTIFACT_NAME}" \
      "${REMOTE_WORKFLOW_FILE}" \
      "skills_tools_ci_status.json" \
      "skills_tools_ci_status.json" \
      "${PREVIOUS_STATUS_JSON}" \
      "${PREVIOUS_STATUS_FETCH_REPORT}"
  fi
  if [ -f "${CLI_SUMMARY_BASELINE}" ]; then
    printf '%q ' "${compare_cmd[@]}"
    printf '> %q\n' "${CLI_SUMMARY_DIFF_REPORT}"
  else
    printf '# compare skipped: baseline file not found\n'
  fi
  if is_truthy "${CLI_SUMMARY_PROMOTE_BASELINE}"; then
    printf '# baseline promotion occurs only when cli_diff.regression_count == 0\n'
    printf 'if [ "$(jq -r '\''.regression_count // 0'\'' %q)" -eq 0 ]; then mkdir -p %q && cp -f %q %q; else echo "Baseline promotion skipped: regressions>0" >&2; fi\n' \
      "${CLI_SUMMARY_DIFF_REPORT}" \
      "$(dirname "${CLI_SUMMARY_BASELINE}")" \
      "${CLI_SUMMARY_ARTIFACT}" \
      "${CLI_SUMMARY_BASELINE}"
  else
    printf '# baseline promotion disabled (OMNI_SKILLS_TOOLS_CLI_SUMMARY_PROMOTE_BASELINE=%q)\n' "${CLI_SUMMARY_PROMOTE_BASELINE}"
  fi
  printf '%q ' "${render_cmd[@]}"
  printf '> /dev/null\n'
  printf '%q ' "${network_cmd[@]}"
  printf '> %q\n' "${REPORT_DIR}/crawl4ai_network_observability.json"
  exit 0
fi

mkdir -p "${REPORT_DIR}"

if [ ! -f "${CLI_SUMMARY_BASELINE}" ] && is_truthy "${FETCH_REMOTE_BASELINE}" && [ -n "${REMOTE_ARTIFACT_NAME}" ]; then
  uv run python scripts/fetch_previous_skills_benchmark_artifact.py \
    --artifact-name "${REMOTE_ARTIFACT_NAME}" \
    --workflow-file "${REMOTE_WORKFLOW_FILE}" \
    --output "${CLI_SUMMARY_BASELINE}" \
    >"${CLI_SUMMARY_REMOTE_FETCH_REPORT}"
fi

if [ ! -f "${PREVIOUS_STATUS_JSON}" ] && is_truthy "${FETCH_REMOTE_BASELINE}" && [ -n "${REMOTE_ARTIFACT_NAME}" ]; then
  uv run python scripts/fetch_previous_skills_benchmark_artifact.py \
    --artifact-name "${REMOTE_ARTIFACT_NAME}" \
    --workflow-file "${REMOTE_WORKFLOW_FILE}" \
    --preferred-member "skills_tools_ci_status.json" \
    --fallback-member "skills_tools_ci_status.json" \
    --output "${PREVIOUS_STATUS_JSON}" \
    >"${PREVIOUS_STATUS_FETCH_REPORT}"
fi

"${deterministic_cmd[@]}" >"${REPORT_DIR}/deterministic_gate.json"

cli_diff_regression_count=0
if [ -f "${CLI_SUMMARY_BASELINE}" ]; then
  "${compare_cmd[@]}" >"${CLI_SUMMARY_DIFF_REPORT}"
  cli_diff_regression_count="$(jq -r '.regression_count // 0' "${CLI_SUMMARY_DIFF_REPORT}" 2>/dev/null || echo 0)"
  case "${cli_diff_regression_count}" in
  '' | *[!0-9]*)
    cli_diff_regression_count=0
    ;;
  esac
else
  cat >"${CLI_SUMMARY_DIFF_REPORT}" <<EOF
{
  "schema": "omni.skills.cli_runner_summary.diff.v1",
  "status": "skipped",
  "reason": "baseline_missing",
  "baseline_file": "${CLI_SUMMARY_BASELINE}",
  "target_file": "${CLI_SUMMARY_ARTIFACT}"
}
EOF
fi

if is_truthy "${CLI_SUMMARY_PROMOTE_BASELINE}"; then
  if [ "${cli_diff_regression_count}" -eq 0 ]; then
    mkdir -p "$(dirname "${CLI_SUMMARY_BASELINE}")"
    cp -f "${CLI_SUMMARY_ARTIFACT}" "${CLI_SUMMARY_BASELINE}"
  else
    echo "Baseline promotion skipped: cli_diff regressions=${cli_diff_regression_count}" >&2
  fi
fi

network_status=0
if ! "${network_cmd[@]}" >"${REPORT_DIR}/crawl4ai_network_observability.json"; then
  network_status=$?
  cat >"${REPORT_DIR}/crawl4ai_network_observability.json" <<EOF
{
  "success": false,
  "status": "network_observability_failed",
  "exit_code": ${network_status},
  "message": "network observability command failed",
  "report_path": "${REPORT_DIR}/crawl4ai_network_observability.json"
}
EOF
fi

if [ "${network_status}" -ne 0 ]; then
  echo "Warning: network observability step failed (exit=${network_status}); deterministic gate already enforced." >&2
fi

"${render_cmd[@]}" >/dev/null
