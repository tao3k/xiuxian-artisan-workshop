#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: start-omni-agent-memory-ci.sh --profile <quick|nightly> [options] [-- <gate-args...>]

Launch memory CI gate in background via nohup.
After completion, the runner updates:
  - latest run status JSON
  - latest failure JSON/Markdown snapshots (when run fails)

Options:
  --profile <quick|nightly>      Gate profile (required for direct invocation)
  --python-bin <path>            Python interpreter (default: .devenv/state/venv/bin/python3 or python3)
  --agent-bin <path>             Prebuilt omni-agent binary path (default: auto-detect target/debug/omni-agent)
  --no-agent-bin-default         Do not auto-attach --agent-bin even when target/debug/omni-agent exists
  --ensure-mcp                   Ensure configured MCP SSE server is healthy before launching gate
  --mcp-host <host>              MCP SSE host for health/restart checks (default: resolved from settings)
  --mcp-port <port>              MCP SSE port for health/restart checks (default: resolve from settings)
  --foreground                   Run in foreground (block until gate exits)
  --log-file <path>              Background log file path
  --latest-failure-json <path>   Aggregated latest failure JSON path
  --latest-failure-md <path>     Aggregated latest failure Markdown path
  --latest-run-json <path>       Latest run status JSON path
  --pid-file <path>              PID file path for background runner
  --help                         Show this help

All unknown options after "--" are passed to:
  test_omni_agent_memory_ci_gate.py --profile <profile> ...
EOF
}

resolve_path() {
  local path="$1"
  if [[ $path == /* ]]; then
    printf '%s' "$path"
  else
    printf '%s/%s' "$PROJECT_ROOT" "$path"
  fi
}

resolve_mcp_port_from_settings() {
  "$PYTHON_BIN" scripts/channel/resolve_mcp_port_from_settings.py
}

resolve_mcp_host_from_settings() {
  "$PYTHON_BIN" scripts/channel/resolve_mcp_endpoint.py --field host
}

mcp_health_ok() {
  "$PYTHON_BIN" scripts/channel/check_mcp_health.py --host "$1" --port "$2" --timeout-secs 2.0
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
RUNTIME_DIR_RAW="${PRJ_RUNTIME_DIR:-.run}"
if [[ $RUNTIME_DIR_RAW == /* ]]; then
  RUNTIME_DIR="$RUNTIME_DIR_RAW"
else
  RUNTIME_DIR="${PROJECT_ROOT}/${RUNTIME_DIR_RAW}"
fi

REPORTS_DIR="${RUNTIME_DIR}/reports"
LOGS_DIR="${RUNTIME_DIR}/logs"
STATE_DIR="${RUNTIME_DIR}/state"

PYTHON_BIN_DEFAULT="${PROJECT_ROOT}/.devenv/state/venv/bin/python3"
if [[ -x $PYTHON_BIN_DEFAULT ]]; then
  PYTHON_BIN="$PYTHON_BIN_DEFAULT"
else
  PYTHON_BIN="python3"
fi

AUTO_AGENT_BIN="true"
AGENT_BIN=""
ENSURE_MCP="false"
MCP_HOST=""
MCP_PORT=""
PROFILE=""
PROFILE_TITLE=""
RUN_FOREGROUND="false"
if [[ -x "${PROJECT_ROOT}/target/debug/omni-agent" ]]; then
  AGENT_BIN="${PROJECT_ROOT}/target/debug/omni-agent"
fi

RUN_STAMP="$("$PYTHON_BIN" scripts/channel/epoch_millis.py)"
LOG_FILE=""
LATEST_FAILURE_JSON="${REPORTS_DIR}/omni-agent-memory-ci-latest-failure.json"
LATEST_FAILURE_MD="${REPORTS_DIR}/omni-agent-memory-ci-latest-failure.md"
LATEST_RUN_JSON="${REPORTS_DIR}/omni-agent-memory-ci-latest-run.json"
PID_FILE=""

GATE_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
  --python-bin)
    PYTHON_BIN="$2"
    shift 2
    ;;
  --profile)
    PROFILE="$2"
    shift 2
    ;;
  --agent-bin)
    AGENT_BIN="$(resolve_path "$2")"
    AUTO_AGENT_BIN="false"
    shift 2
    ;;
  --no-agent-bin-default)
    AUTO_AGENT_BIN="false"
    AGENT_BIN=""
    shift
    ;;
  --ensure-mcp)
    ENSURE_MCP="true"
    shift
    ;;
  --mcp-host)
    MCP_HOST="$2"
    shift 2
    ;;
  --mcp-port)
    MCP_PORT="$2"
    shift 2
    ;;
  --foreground)
    RUN_FOREGROUND="true"
    shift
    ;;
  --log-file)
    LOG_FILE="$(resolve_path "$2")"
    shift 2
    ;;
  --latest-failure-json)
    LATEST_FAILURE_JSON="$(resolve_path "$2")"
    shift 2
    ;;
  --latest-failure-md)
    LATEST_FAILURE_MD="$(resolve_path "$2")"
    shift 2
    ;;
  --latest-run-json)
    LATEST_RUN_JSON="$(resolve_path "$2")"
    shift 2
    ;;
  --pid-file)
    PID_FILE="$(resolve_path "$2")"
    shift 2
    ;;
  --help)
    usage
    exit 0
    ;;
  --)
    shift
    while [[ $# -gt 0 ]]; do
      GATE_ARGS+=("$1")
      shift
    done
    ;;
  *)
    GATE_ARGS+=("$1")
    shift
    ;;
  esac
done

case "$PROFILE" in
quick)
  PROFILE_TITLE="Quick"
  ;;
nightly)
  PROFILE_TITLE="Nightly"
  ;;
*)
  echo "Invalid --profile: ${PROFILE:-<empty>} (expected quick or nightly)" >&2
  exit 2
  ;;
esac

if [[ -z $LOG_FILE ]]; then
  LOG_FILE="${LOGS_DIR}/omni-agent-memory-ci-${PROFILE}-${RUN_STAMP}.log"
fi
if [[ -z $PID_FILE ]]; then
  PID_FILE="${STATE_DIR}/omni-agent-memory-ci-${PROFILE}.pid"
fi

if ! command -v "$PYTHON_BIN" >/dev/null 2>&1 && [[ ! -x $PYTHON_BIN ]]; then
  echo "Python interpreter not found: ${PYTHON_BIN}" >&2
  exit 2
fi

if [[ $AUTO_AGENT_BIN == "true" && -z $AGENT_BIN && -x "${PROJECT_ROOT}/target/debug/omni-agent" ]]; then
  AGENT_BIN="${PROJECT_ROOT}/target/debug/omni-agent"
fi
if [[ -n $AGENT_BIN && ! -x $AGENT_BIN ]]; then
  echo "Agent binary is not executable: ${AGENT_BIN}" >&2
  exit 2
fi

mkdir -p "$REPORTS_DIR" "$LOGS_DIR" "$STATE_DIR" "$(dirname "$LATEST_FAILURE_JSON")" "$(dirname "$LATEST_FAILURE_MD")" "$(dirname "$LATEST_RUN_JSON")" "$(dirname "$PID_FILE")" "$(dirname "$LOG_FILE")"

if [[ -z $MCP_PORT ]]; then
  MCP_PORT="$(resolve_mcp_port_from_settings)"
fi
if [[ -z $MCP_HOST ]]; then
  MCP_HOST="$(resolve_mcp_host_from_settings)"
fi
if [[ -n $MCP_PORT ]]; then
  if ! [[ $MCP_PORT =~ ^[0-9]+$ ]] || ((MCP_PORT < 1 || MCP_PORT > 65535)); then
    echo "Invalid --mcp-port: ${MCP_PORT}" >&2
    exit 2
  fi
fi
if [[ $ENSURE_MCP == "true" ]]; then
  if [[ -z $MCP_PORT ]]; then
    echo "Cannot resolve MCP port from settings. Set mcp.preferred_embed_port or embedding.client_url, or pass --mcp-port." >&2
    exit 2
  fi
  if mcp_health_ok "$MCP_HOST" "$MCP_PORT"; then
    echo "MCP already healthy at http://${MCP_HOST}:${MCP_PORT}/health"
  else
    MCP_PID_FILE="${RUNTIME_DIR}/omni-mcp-sse-${MCP_PORT}.pid"
    MCP_LOG_FILE="${LOGS_DIR}/omni-mcp-sse-${MCP_PORT}.log"
    "${SCRIPT_DIR}/restart-omni-mcp.sh" \
      --host "$MCP_HOST" \
      --port "$MCP_PORT" \
      --pid-file "$MCP_PID_FILE" \
      --log-file "$MCP_LOG_FILE"
  fi
fi

GATE_CMD=(
  "$PYTHON_BIN"
  "${SCRIPT_DIR}/test_omni_agent_memory_ci_gate.py"
  "--profile"
  "$PROFILE"
)
if [[ -n $AGENT_BIN ]]; then
  GATE_CMD+=("--agent-bin" "$AGENT_BIN")
fi
if [[ ${#GATE_ARGS[@]} -gt 0 ]]; then
  GATE_CMD+=("${GATE_ARGS[@]}")
fi

printf -v GATE_CMD_STR '%q ' "${GATE_CMD[@]}"
printf -v Q_RUN_STAMP '%q' "$RUN_STAMP"
printf -v Q_REPORTS_DIR '%q' "$REPORTS_DIR"
printf -v Q_LATEST_FAILURE_JSON '%q' "$LATEST_FAILURE_JSON"
printf -v Q_LATEST_FAILURE_MD '%q' "$LATEST_FAILURE_MD"
printf -v Q_LATEST_RUN_JSON '%q' "$LATEST_RUN_JSON"
printf -v Q_LOG_FILE '%q' "$LOG_FILE"
printf -v Q_PYTHON_BIN '%q' "$PYTHON_BIN"
printf -v Q_SCRIPT_DIR '%q' "$SCRIPT_DIR"
printf -v Q_PROFILE '%q' "$PROFILE"
printf -v Q_PROFILE_TITLE '%q' "$PROFILE_TITLE"

RUNNER_SCRIPT="${STATE_DIR}/omni-agent-memory-ci-${PROFILE}-runner-${RUN_STAMP}.sh"
cat >"$RUNNER_SCRIPT" <<EOF
#!/usr/bin/env bash
set -euo pipefail

START_STAMP=${Q_RUN_STAMP}
REPORTS_DIR=${Q_REPORTS_DIR}
PROFILE=${Q_PROFILE}
PROFILE_TITLE=${Q_PROFILE_TITLE}
LATEST_FAILURE_JSON=${Q_LATEST_FAILURE_JSON}
LATEST_FAILURE_MD=${Q_LATEST_FAILURE_MD}
LATEST_RUN_JSON=${Q_LATEST_RUN_JSON}
LOG_FILE=${Q_LOG_FILE}
PYTHON_BIN=${Q_PYTHON_BIN}
SCRIPT_DIR=${Q_SCRIPT_DIR}

set +e
${GATE_CMD_STR}
EXIT_CODE=\$?
set -e

FINISH_STAMP="\$("\$PYTHON_BIN" "\$SCRIPT_DIR/epoch_millis.py")"
"\$PYTHON_BIN" "\$SCRIPT_DIR/memory_ci_finalize.py" \
  --reports-dir "\$REPORTS_DIR" \
  --profile "\$PROFILE" \
  --start-stamp "\$START_STAMP" \
  --exit-code "\$EXIT_CODE" \
  --latest-failure-json "\$LATEST_FAILURE_JSON" \
  --latest-failure-md "\$LATEST_FAILURE_MD" \
  --latest-run-json "\$LATEST_RUN_JSON" \
  --log-file "\$LOG_FILE" \
  --finish-stamp "\$FINISH_STAMP"

if [[ "\$EXIT_CODE" -eq 0 ]]; then
  echo "\${PROFILE_TITLE} memory CI gate completed successfully."
else
  echo "\${PROFILE_TITLE} memory CI gate failed with exit code \$EXIT_CODE."
fi

exit "\$EXIT_CODE"
EOF

chmod +x "$RUNNER_SCRIPT"

{
  echo "[$(date -u +"%Y-%m-%dT%H:%M:%SZ")] launch ${PROFILE} memory ci gate"
  echo "+ ${GATE_CMD_STR}"
  echo "runner_script=${RUNNER_SCRIPT}"
} >>"$LOG_FILE"

if [[ $RUN_FOREGROUND == "true" ]]; then
  echo "${PROFILE_TITLE} memory CI gate running in foreground."
  echo "  log_file: ${LOG_FILE}"
  echo "  latest_run_json: ${LATEST_RUN_JSON}"
  echo "  latest_failure_json: ${LATEST_FAILURE_JSON}"
  echo "  latest_failure_md: ${LATEST_FAILURE_MD}"
  echo "  runner_script: ${RUNNER_SCRIPT}"
  set +e
  "$RUNNER_SCRIPT" 2>&1 | tee -a "$LOG_FILE"
  RUNNER_RC=${PIPESTATUS[0]}
  set -e
  exit "$RUNNER_RC"
fi

nohup "$RUNNER_SCRIPT" >>"$LOG_FILE" 2>&1 &
BG_PID=$!
echo "$BG_PID" >"$PID_FILE"

echo "${PROFILE_TITLE} memory CI gate started."
echo "  pid: ${BG_PID}"
echo "  pid_file: ${PID_FILE}"
echo "  log_file: ${LOG_FILE}"
echo "  latest_run_json: ${LATEST_RUN_JSON}"
echo "  latest_failure_json: ${LATEST_FAILURE_JSON}"
echo "  latest_failure_md: ${LATEST_FAILURE_MD}"
echo "  runner_script: ${RUNNER_SCRIPT}"
