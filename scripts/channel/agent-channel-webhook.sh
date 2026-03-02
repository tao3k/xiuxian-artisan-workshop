#!/usr/bin/env bash
# Run Telegram channel in webhook mode: ensure valkey, start ngrok, set webhook, run agent.
# By default this also starts Discord ingress runtime unless DISCORD_INGRESS_ENABLED=0.
# Usage: TELEGRAM_BOT_TOKEN=xxx ./scripts/channel/agent-channel-webhook.sh [valkey_port]
# Requires: ngrok installed, ngrok authtoken (NGROK_AUTHTOKEN env or ngrok config add-authtoken)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CARGO_BIN="${CARGO_BIN:-${PROJECT_ROOT}/scripts/rust/cargo_exec.sh}"
cd "${PROJECT_ROOT}"

resolve_valkey_field() {
  python3 "${PROJECT_ROOT}/scripts/channel/resolve_valkey_endpoint.py" --field "$1"
}

resolve_mcp_field() {
  python3 "${PROJECT_ROOT}/scripts/channel/resolve_mcp_endpoint.py" --field "$1"
}

LOCAL_HOST_DEFAULT="${XIUXIAN_WENDAO_LOCAL_HOST:-${LOCAL_HOST_DEFAULT:-$(resolve_mcp_field host)}}"
DEFAULT_TELEGRAM_WEBHOOK_PORT="${XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_PORT:-18081}"
DEFAULT_DISCORD_INGRESS_PORT="${XIUXIAN_WENDAO_DISCORD_INGRESS_PORT:-18082}"
DEFAULT_DISCORD_INGRESS_PATH="${XIUXIAN_WENDAO_DISCORD_INGRESS_PATH:-/discord/ingress}"
DEFAULT_DISCORD_INGRESS_STARTUP_TIMEOUT_SECS="${XIUXIAN_WENDAO_DISCORD_INGRESS_STARTUP_TIMEOUT_SECS:-120}"
DEFAULT_GATEWAY_STARTUP_TIMEOUT_SECS="${XIUXIAN_WENDAO_GATEWAY_STARTUP_TIMEOUT_SECS:-180}"
NGROK_API_BASE_URL="${XIUXIAN_WENDAO_NGROK_API_BASE_URL:-http://${LOCAL_HOST_DEFAULT}:4040}"

LOG_FILE="${OMNI_CHANNEL_LOG_FILE:-.run/logs/omni-agent-webhook.log}"
mkdir -p "$(dirname "${LOG_FILE}")"

# Source .env if present (TELEGRAM_BOT_TOKEN, TELEGRAM_WEBHOOK_SECRET, etc.)
if [ -f .env ]; then
  set -a
  # shellcheck source=/dev/null
  source .env
  set +a
fi

VALKEY_PORT="${VALKEY_PORT:-$(resolve_valkey_field port)}"
if [ $# -gt 0 ]; then
  VALKEY_PORT="$1"
  shift
fi

bash "${SCRIPT_DIR}/valkey-start.sh" "${VALKEY_PORT}"
VALKEY_HOST="${VALKEY_HOST:-$(resolve_valkey_field host)}"
VALKEY_DB="${VALKEY_DB:-$(resolve_valkey_field db)}"
VALKEY_RESOLVED_URL="redis://${VALKEY_HOST}:${VALKEY_PORT}/${VALKEY_DB}"
export XIUXIAN_WENDAO_VALKEY_URL="${XIUXIAN_WENDAO_VALKEY_URL:-${VALKEY_RESOLVED_URL}}"

if [ -z "${TELEGRAM_BOT_TOKEN:-}" ]; then
  echo "Error: TELEGRAM_BOT_TOKEN is required. Set it in env or .env" >&2
  echo "  export TELEGRAM_BOT_TOKEN=your_bot_token" >&2
  exit 1
fi

if [ -n "${OMNI_AGENT_NOTIFICATION_RECIPIENT:-}" ]; then
  export OMNI_AGENT_NOTIFICATION_RECIPIENT
  echo "Proactive notifications override enabled via env: OMNI_AGENT_NOTIFICATION_RECIPIENT='${OMNI_AGENT_NOTIFICATION_RECIPIENT}'"
else
  echo "Proactive notifications recipient is runtime-managed (task metadata / xiuxian.toml)."
fi

# Resolve webhook secret token:
#   1) TELEGRAM_WEBHOOK_SECRET env / .env
#   2) telegram.webhook_secret_token from xiuxian.toml
#   3) auto-generate ephemeral secret (local dev fallback)
if [ -z "${TELEGRAM_WEBHOOK_SECRET:-}" ]; then
  TELEGRAM_WEBHOOK_SECRET="$(uv run python scripts/channel/read_telegram_setting.py --key webhook_secret_token 2>/dev/null)" || true
fi
if [ -z "${TELEGRAM_WEBHOOK_SECRET:-}" ]; then
  TELEGRAM_WEBHOOK_SECRET="$(python3 scripts/channel/generate_secret_token.py --length 32)"
  echo "Warning: TELEGRAM_WEBHOOK_SECRET not set; generated ephemeral local secret token."
fi
export TELEGRAM_WEBHOOK_SECRET

if ! command -v ngrok >/dev/null 2>&1; then
  echo "Error: ngrok is required. Install: https://ngrok.com/download" >&2
  exit 1
fi

SETTINGS_WEBHOOK_BIND=""
SETTINGS_WEBHOOK_BIND="$(uv run python scripts/channel/read_telegram_setting.py --key webhook_bind 2>/dev/null)" || true

WEBHOOK_BIND="${XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_BIND:-${WEBHOOK_BIND:-}}"
webhook_host_source="default:${LOCAL_HOST_DEFAULT}"
webhook_port_source="default:${DEFAULT_TELEGRAM_WEBHOOK_PORT}"
if [ -n "${WEBHOOK_BIND}" ]; then
  if [ -n "${XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_BIND:-}" ]; then
    webhook_host_source="env:XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_BIND"
    webhook_port_source="env:XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_BIND"
  else
    webhook_host_source="env:WEBHOOK_BIND"
    webhook_port_source="env:WEBHOOK_BIND"
  fi
fi
if [ -z "${WEBHOOK_BIND}" ] && [ -n "${SETTINGS_WEBHOOK_BIND}" ]; then
  WEBHOOK_BIND="${SETTINGS_WEBHOOK_BIND}"
  webhook_host_source="config:telegram.webhook_bind"
  webhook_port_source="config:telegram.webhook_bind"
fi

resolved_webhook_host=""
resolved_webhook_port=""

if [ -n "${WEBHOOK_BIND}" ]; then
  resolved_webhook_host="${WEBHOOK_BIND%:*}"
  resolved_webhook_port="${WEBHOOK_BIND##*:}"
fi

if [ -n "${XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_PORT:-}" ]; then
  resolved_webhook_port="${XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_PORT}"
  webhook_port_source="env:XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_PORT"
elif [ -n "${WEBHOOK_PORT:-}" ]; then
  resolved_webhook_port="${WEBHOOK_PORT}"
  webhook_port_source="env:WEBHOOK_PORT"
fi

if [ -n "${XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_HOST:-}" ]; then
  resolved_webhook_host="${XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_HOST}"
  webhook_host_source="env:XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_HOST"
elif [ -n "${WEBHOOK_HOST:-}" ]; then
  resolved_webhook_host="${WEBHOOK_HOST}"
  webhook_host_source="env:WEBHOOK_HOST"
fi

if [ -z "${resolved_webhook_host}" ]; then
  resolved_webhook_host="${LOCAL_HOST_DEFAULT}"
fi
if [ -z "${resolved_webhook_port}" ]; then
  resolved_webhook_port="${DEFAULT_TELEGRAM_WEBHOOK_PORT}"
fi

if ! [[ ${resolved_webhook_port} =~ ^[0-9]+$ ]] || [ "${resolved_webhook_port}" -le 0 ] || [ "${resolved_webhook_port}" -gt 65535 ]; then
  echo "Error: invalid webhook port '${resolved_webhook_port}'. Set WEBHOOK_PORT or telegram.webhook_bind in config." >&2
  exit 1
fi

WEBHOOK_PORT="${resolved_webhook_port}"
WEBHOOK_BIND="${resolved_webhook_host}:${WEBHOOK_PORT}"
export WEBHOOK_PORT
export WEBHOOK_BIND

# Reclaim stale webhook listeners started by previous omni-agent webhook runs.
if lsof -nP -iTCP:"${WEBHOOK_PORT}" -sTCP:LISTEN >/dev/null 2>&1; then
  existing_webhook_pid="$(lsof -nP -iTCP:"${WEBHOOK_PORT}" -sTCP:LISTEN -t 2>/dev/null | head -n 1)"
  existing_webhook_cmd="$(ps -o command= -p "${existing_webhook_pid}" 2>/dev/null || true)"

  if [[ ${existing_webhook_cmd} == *"omni-agent"* ]] && [[ ${existing_webhook_cmd} == *"--mode webhook"* ]]; then
    echo "Warning: webhook port ${WEBHOOK_PORT} is occupied by an existing omni-agent webhook process (pid=${existing_webhook_pid}); reclaiming it."
    kill "${existing_webhook_pid}" 2>/dev/null || true

    webhook_port_released="false"
    for _ in $(seq 1 20); do
      if ! lsof -nP -iTCP:"${WEBHOOK_PORT}" -sTCP:LISTEN >/dev/null 2>&1; then
        webhook_port_released="true"
        break
      fi
      sleep 1
    done
    if [ "${webhook_port_released}" != "true" ]; then
      echo "Error: failed to reclaim webhook port ${WEBHOOK_PORT} from existing omni-agent process." >&2
      lsof -nP -iTCP:"${WEBHOOK_PORT}" -sTCP:LISTEN >&2 || true
      echo "Hint: stop the existing listener or run with WEBHOOK_PORT=<free_port>." >&2
      exit 1
    fi
    echo "  Reclaimed webhook port ${WEBHOOK_PORT}."
  else
    echo "Error: webhook port ${WEBHOOK_PORT} is already in use; cannot start webhook channel." >&2
    lsof -nP -iTCP:"${WEBHOOK_PORT}" -sTCP:LISTEN >&2 || true
    echo "Hint: stop the existing listener or run with WEBHOOK_PORT=<free_port>." >&2
    exit 1
  fi
fi

NGROK_PID=""
GATEWAY_PID=""
DISCORD_INGRESS_PID=""

cleanup() {
  if [ -n "$NGROK_PID" ]; then
    echo ""
    echo "Stopping ngrok (PID $NGROK_PID)..."
    kill "$NGROK_PID" 2>/dev/null || true
  fi
  if [ -n "$GATEWAY_PID" ]; then
    echo "Stopping gateway (PID $GATEWAY_PID)..."
    kill "$GATEWAY_PID" 2>/dev/null || true
  fi
  if [ -n "$DISCORD_INGRESS_PID" ]; then
    echo "Stopping Discord ingress (PID $DISCORD_INGRESS_PID)..."
    kill "$DISCORD_INGRESS_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

ts_utc() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

normalize_local_bind_host() {
  local raw_host="${1:-}"
  local host="${raw_host#[}"
  host="${host%]}"
  if [ -z "${host}" ] || [ "${host}" = "0.0.0.0" ] || [ "${host}" = "::" ]; then
    printf '%s' "${LOCAL_HOST_DEFAULT}"
    return 0
  fi
  printf '%s' "${host}"
  return 0
}

probe_discord_ingress_listener() {
  local bind_addr="$1"
  local ingress_path="$2"
  local secret_token="$3"
  local probe_host_raw="${bind_addr%:*}"
  local probe_port="${bind_addr##*:}"
  local probe_host
  probe_host="$(normalize_local_bind_host "${probe_host_raw}")"
  local probe_url="http://${probe_host}:${probe_port}${ingress_path}"
  local probe_status=""

  if [ -n "${secret_token}" ]; then
    probe_status="$(curl -sS -o /dev/null -w "%{http_code}" \
      -H "content-type: application/json" \
      -H "x-omni-discord-ingress-token: ${secret_token}" \
      -X POST \
      --data '{}' \
      --max-time 2 \
      "${probe_url}" || true)"
  else
    probe_status="$(curl -sS -o /dev/null -w "%{http_code}" \
      -H "content-type: application/json" \
      -X POST \
      --data '{}' \
      --max-time 2 \
      "${probe_url}" || true)"
  fi

  if [ "${probe_status}" = "200" ]; then
    return 0
  fi
  echo "Error: existing Discord ingress listener probe failed (status=${probe_status:-000}, url=${probe_url})." >&2
  echo "Hint: ensure bind/path/secret match this launcher or stop the existing listener." >&2
  return 1
}

is_truthy() {
  local raw="${1:-}"
  local normalized
  normalized="$(printf '%s' "$raw" | tr '[:upper:]' '[:lower:]')"
  case "${normalized}" in
  1 | true | yes | on)
    return 0
    ;;
  *)
    return 1
    ;;
  esac
}

wait_http_health() {
  local url="$1"
  local timeout_secs="$2"
  local pid_hint="${3:-}"
  local start_ts
  local now_ts
  start_ts="$(date +%s)"

  while true; do
    if curl -fsS --max-time 1 "${url}" >/dev/null 2>&1; then
      return 0
    fi

    if [ -n "${pid_hint}" ] && ! kill -0 "${pid_hint}" 2>/dev/null; then
      return 1
    fi

    now_ts="$(date +%s)"
    if [ $((now_ts - start_ts)) -ge "${timeout_secs}" ]; then
      return 1
    fi
    sleep 1
  done
}

on_bootstrap_error() {
  local exit_code="$1"
  local line_no="$2"
  local failed_cmd="$3"
  {
    echo "[$(ts_utc)] [agent-channel-webhook] bootstrap_failed exit_code=${exit_code} line=${line_no}"
    echo "[$(ts_utc)] [agent-channel-webhook] failed_command=${failed_cmd}"
  } | tee -a "${LOG_FILE}" >&2
}

trap 'on_bootstrap_error $? $LINENO "$BASH_COMMAND"' ERR
echo "Launcher: CARGO_RUN='${CARGO_BIN} run -p omni-agent -- ...'"
if [ -n "${OMNI_AGENT_MISTRAL_SDK_HF_CACHE_PATH:-}" ]; then
  echo "Embedding model cache override: OMNI_AGENT_MISTRAL_SDK_HF_CACHE_PATH='${OMNI_AGENT_MISTRAL_SDK_HF_CACHE_PATH}' (source=env)"
fi

echo "Step 1: Valkey ready at ${XIUXIAN_WENDAO_VALKEY_URL}"
echo "  Config read (xiuxian.toml): telegram.webhook_bind='${SETTINGS_WEBHOOK_BIND:-<empty>}'"
echo "  Config resolved: webhook_host='${resolved_webhook_host}' (source=${webhook_host_source}), webhook_port='${resolved_webhook_port}' (source=${webhook_port_source})"
echo "Step 2: Starting ngrok tunnel on port $WEBHOOK_PORT..."
ngrok http "$WEBHOOK_PORT" >/tmp/ngrok.log 2>&1 &
NGROK_PID=$!
echo "  Waiting for ngrok to be ready..."
sleep 8

echo "Step 3: Fetching public URL from ngrok..."
NGROK_URL=""
for _ in $(seq 1 15); do
  # Try ngrok local API first (port 4040)
  NGROK_URL="$(curl -s --connect-timeout 2 "${NGROK_API_BASE_URL}/api/tunnels" 2>/dev/null | python3 scripts/channel/extract_ngrok_public_url.py 2>/dev/null)" || true
  if [ -n "$NGROK_URL" ]; then
    break
  fi
  # Fallback: parse ngrok log for tunnel URL (exclude dashboard/signup pages)
  if [ -f /tmp/ngrok.log ]; then
    NGROK_URL=$(grep -oE 'https://[a-zA-Z0-9][-a-zA-Z0-9]*\.(ngrok-free\.app|ngrok\.io)\b' /tmp/ngrok.log 2>/dev/null | grep -v dashboard | head -1) || true
  fi
  if [ -n "$NGROK_URL" ]; then
    break
  fi
  sleep 1
done

# Reject invalid URLs (e.g. dashboard/signup when ngrok needs auth)
if [ -n "$NGROK_URL" ] && echo "$NGROK_URL" | grep -qE 'dashboard|signup'; then
  echo "Error: ngrok returned a signup URL (not authenticated)." >&2
  echo "  Set NGROK_AUTHTOKEN or run: ngrok config add-authtoken <your_token>" >&2
  echo "  Get token: https://dashboard.ngrok.com/get-started/your-authtoken" >&2
  kill "$NGROK_PID" 2>/dev/null || true
  exit 1
fi

if [ -z "$NGROK_URL" ]; then
  echo "Error: Could not get ngrok tunnel URL." >&2
  if [ -f /tmp/ngrok.log ] && grep -q -E 'signup|authtoken|dashboard\.ngrok' /tmp/ngrok.log 2>/dev/null; then
    echo "  ngrok requires authentication. Use either:" >&2
    echo "    export NGROK_AUTHTOKEN=<your_token>" >&2
    echo "    or: ngrok config add-authtoken <your_token>" >&2
    echo "  Get your token at: https://dashboard.ngrok.com/get-started/your-authtoken" >&2
  else
    echo "  Check /tmp/ngrok.log. Common causes:" >&2
    echo "  - ngrok needs auth: ngrok config add-authtoken <token>" >&2
    echo "  - ngrok inspector unavailable at ${NGROK_API_BASE_URL}" >&2
  fi
  if [ -f /tmp/ngrok.log ]; then
    echo "" >&2
    echo "  Last 10 lines of /tmp/ngrok.log:" >&2
    tail -10 /tmp/ngrok.log | sed 's/^/    /' >&2
  fi
  kill "$NGROK_PID" 2>/dev/null || true
  exit 1
fi

WEBHOOK_URL="${NGROK_URL}/telegram/webhook"
echo "  Public URL: $WEBHOOK_URL"

echo "  Setting Telegram webhook..."
SET_RESULT=$(curl -s -X POST "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/setWebhook" \
  --data-urlencode "url=${WEBHOOK_URL}" \
  --data-urlencode "secret_token=${TELEGRAM_WEBHOOK_SECRET}")
if echo "$SET_RESULT" | grep -q '"ok":true'; then
  echo "  Webhook set successfully."
else
  echo "  Webhook response: $SET_RESULT" >&2
fi

echo ""
SETTINGS_GATEWAY_BIND=""
SETTINGS_GATEWAY_BIND="$(uv run python scripts/channel/read_setting.py --key gateway.bind 2>/dev/null)" || true
SETTINGS_GATEWAY_MAX_CONCURRENT=""
SETTINGS_GATEWAY_MAX_CONCURRENT="$(uv run python scripts/channel/read_setting.py --key gateway.max_concurrent 2>/dev/null)" || true

GATEWAY_BIND="${GATEWAY_BIND:-}"
gateway_bind_source="disabled"
if [ -n "${GATEWAY_BIND}" ]; then
  gateway_bind_source="env:GATEWAY_BIND"
fi
if [ -z "${GATEWAY_BIND}" ] && [ -n "${SETTINGS_GATEWAY_BIND}" ]; then
  GATEWAY_BIND="${SETTINGS_GATEWAY_BIND}"
  gateway_bind_source="config:gateway.bind"
fi

if [ -n "${GATEWAY_PORT:-}" ]; then
  gateway_host="${GATEWAY_HOST:-${LOCAL_HOST_DEFAULT}}"
  GATEWAY_BIND="${gateway_host}:${GATEWAY_PORT}"
  gateway_bind_source="env:GATEWAY_PORT"
fi

GATEWAY_MAX_CONCURRENT="${GATEWAY_MAX_CONCURRENT:-}"
gateway_max_concurrent_source="default:1"
if [ -n "${GATEWAY_MAX_CONCURRENT}" ]; then
  gateway_max_concurrent_source="env:GATEWAY_MAX_CONCURRENT"
fi
if [ -z "${GATEWAY_MAX_CONCURRENT}" ] && [ -n "${SETTINGS_GATEWAY_MAX_CONCURRENT}" ]; then
  GATEWAY_MAX_CONCURRENT="${SETTINGS_GATEWAY_MAX_CONCURRENT}"
  gateway_max_concurrent_source="config:gateway.max_concurrent"
fi
if [ -z "${GATEWAY_MAX_CONCURRENT}" ]; then
  GATEWAY_MAX_CONCURRENT="1"
fi
if ! [[ ${GATEWAY_MAX_CONCURRENT} =~ ^[0-9]+$ ]] || [ "${GATEWAY_MAX_CONCURRENT}" -le 0 ]; then
  echo "Error: invalid gateway max concurrency '${GATEWAY_MAX_CONCURRENT}'. Set GATEWAY_MAX_CONCURRENT or gateway.max_concurrent to a positive integer." >&2
  exit 1
fi

GATEWAY_HEALTH_URL=""
if [ -n "${GATEWAY_BIND}" ]; then
  echo "Step 4: Gateway enabled with bind ${GATEWAY_BIND} (source=${gateway_bind_source}), max_concurrent=${GATEWAY_MAX_CONCURRENT} (source=${gateway_max_concurrent_source})"
  gateway_port="${GATEWAY_BIND##*:}"
  if ! [[ ${gateway_port} =~ ^[0-9]+$ ]] || [ "${gateway_port}" -le 0 ] || [ "${gateway_port}" -gt 65535 ]; then
    echo "Error: invalid gateway bind '${GATEWAY_BIND}'. Set GATEWAY_BIND, GATEWAY_PORT, or gateway.bind." >&2
    exit 1
  fi
  GATEWAY_HEALTH_URL="http://${GATEWAY_BIND}/health"
  gateway_startup_timeout_secs="${GATEWAY_STARTUP_TIMEOUT_SECS:-${DEFAULT_GATEWAY_STARTUP_TIMEOUT_SECS}}"
  if ! [[ ${gateway_startup_timeout_secs} =~ ^[0-9]+$ ]] || [ "${gateway_startup_timeout_secs}" -le 0 ]; then
    echo "Error: invalid gateway startup timeout '${gateway_startup_timeout_secs}'. Set GATEWAY_STARTUP_TIMEOUT_SECS or XIUXIAN_WENDAO_GATEWAY_STARTUP_TIMEOUT_SECS to a positive integer." >&2
    exit 1
  fi

  if ! lsof -nP -iTCP:"${gateway_port}" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "  Starting omni-agent gateway on ${GATEWAY_BIND}..."
    "${CARGO_BIN}" run -p omni-agent -- gateway --bind "${GATEWAY_BIND}" --max-concurrent "${GATEWAY_MAX_CONCURRENT}" >>"${LOG_FILE}" 2>&1 &
    GATEWAY_PID=$!

    if wait_http_health "${GATEWAY_HEALTH_URL}" "${gateway_startup_timeout_secs}" "${GATEWAY_PID}"; then
      echo "  Gateway healthy at ${GATEWAY_HEALTH_URL}"
    else
      echo "Error: gateway health probe timed out at ${GATEWAY_HEALTH_URL} after ${gateway_startup_timeout_secs}s." >&2
      if kill -0 "${GATEWAY_PID}" 2>/dev/null; then
        echo "  Stopping unready gateway process (pid=${GATEWAY_PID})." >&2
        kill "${GATEWAY_PID}" 2>/dev/null || true
      fi
      if [ -f "${LOG_FILE}" ]; then
        echo "  Last 30 lines from ${LOG_FILE}:" >&2
        tail -30 "${LOG_FILE}" | sed 's/^/    /' >&2 || true
      fi
      exit 1
    fi
  else
    existing_gateway_pid="$(lsof -nP -iTCP:"${gateway_port}" -sTCP:LISTEN -t 2>/dev/null | head -n 1)"
    existing_gateway_cmd="$(ps -o command= -p "${existing_gateway_pid}" 2>/dev/null || true)"
    if wait_http_health "${GATEWAY_HEALTH_URL}" 3 ""; then
      echo "  Gateway already listening on ${GATEWAY_BIND}; existing process is healthy and will be reused."
    else
      if [[ ${existing_gateway_cmd} == *"omni-agent"* ]] && [[ ${existing_gateway_cmd} == *"gateway"* ]]; then
        echo "Warning: existing gateway listener on ${GATEWAY_BIND} is unhealthy (pid=${existing_gateway_pid}); reclaiming it."
        kill "${existing_gateway_pid}" 2>/dev/null || true

        gateway_port_released="false"
        for _ in $(seq 1 20); do
          if ! lsof -nP -iTCP:"${gateway_port}" -sTCP:LISTEN >/dev/null 2>&1; then
            gateway_port_released="true"
            break
          fi
          sleep 1
        done
        if [ "${gateway_port_released}" != "true" ]; then
          echo "Error: failed to reclaim gateway port ${gateway_port} from unhealthy process." >&2
          lsof -nP -iTCP:"${gateway_port}" -sTCP:LISTEN >&2 || true
          exit 1
        fi

        echo "  Restarting omni-agent gateway on ${GATEWAY_BIND}..."
        "${CARGO_BIN}" run -p omni-agent -- gateway --bind "${GATEWAY_BIND}" --max-concurrent "${GATEWAY_MAX_CONCURRENT}" >>"${LOG_FILE}" 2>&1 &
        GATEWAY_PID=$!
        if wait_http_health "${GATEWAY_HEALTH_URL}" "${gateway_startup_timeout_secs}" "${GATEWAY_PID}"; then
          echo "  Gateway healthy at ${GATEWAY_HEALTH_URL}"
        else
          echo "Error: gateway remained unhealthy after restart at ${GATEWAY_HEALTH_URL}." >&2
          if [ -f "${LOG_FILE}" ]; then
            echo "  Last 30 lines from ${LOG_FILE}:" >&2
            tail -30 "${LOG_FILE}" | sed 's/^/    /' >&2 || true
          fi
          exit 1
        fi
      else
        echo "Error: gateway port ${gateway_port} is occupied by a non-gateway process or unhealthy listener." >&2
        echo "  pid='${existing_gateway_pid:-unknown}' cmd='${existing_gateway_cmd:-unknown}'" >&2
        echo "  health='${GATEWAY_HEALTH_URL}' is not reachable." >&2
        exit 1
      fi
    fi
  fi
  export GATEWAY_BIND
else
  echo "Step 4: Gateway disabled (gateway.bind='${SETTINGS_GATEWAY_BIND:-<empty>}', GATEWAY_BIND='${GATEWAY_BIND:-<empty>}', GATEWAY_PORT='${GATEWAY_PORT:-<empty>}')"
fi

echo ""
SETTINGS_DISCORD_INGRESS_BIND=""
SETTINGS_DISCORD_INGRESS_BIND="$(uv run python scripts/channel/read_setting.py --key discord.ingress_bind 2>/dev/null)" || true
SETTINGS_DISCORD_INGRESS_PATH=""
SETTINGS_DISCORD_INGRESS_PATH="$(uv run python scripts/channel/read_setting.py --key discord.ingress_path 2>/dev/null)" || true
SETTINGS_DISCORD_INGRESS_SECRET_TOKEN=""
SETTINGS_DISCORD_INGRESS_SECRET_TOKEN="$(uv run python scripts/channel/read_setting.py --key discord.ingress_secret_token 2>/dev/null)" || true

DISCORD_INGRESS_ENABLED="${DISCORD_INGRESS_ENABLED:-1}"
if is_truthy "${DISCORD_INGRESS_ENABLED}"; then
  DISCORD_INGRESS_BIND="${DISCORD_INGRESS_BIND:-}"
  discord_ingress_bind_source="default:${LOCAL_HOST_DEFAULT}:${DEFAULT_DISCORD_INGRESS_PORT}"
  if [ -n "${DISCORD_INGRESS_BIND}" ]; then
    discord_ingress_bind_source="env:DISCORD_INGRESS_BIND"
  fi
  if [ -z "${DISCORD_INGRESS_BIND}" ] && [ -n "${SETTINGS_DISCORD_INGRESS_BIND}" ]; then
    DISCORD_INGRESS_BIND="${SETTINGS_DISCORD_INGRESS_BIND}"
    discord_ingress_bind_source="config:discord.ingress_bind"
  fi
  if [ -z "${DISCORD_INGRESS_BIND}" ]; then
    DISCORD_INGRESS_BIND="${LOCAL_HOST_DEFAULT}:${DEFAULT_DISCORD_INGRESS_PORT}"
  fi

  if [ -n "${DISCORD_INGRESS_PORT:-}" ]; then
    discord_ingress_host="${DISCORD_INGRESS_HOST:-${LOCAL_HOST_DEFAULT}}"
    DISCORD_INGRESS_BIND="${discord_ingress_host}:${DISCORD_INGRESS_PORT}"
    discord_ingress_bind_source="env:DISCORD_INGRESS_PORT"
  fi

  DISCORD_INGRESS_PATH="${DISCORD_INGRESS_PATH:-}"
  discord_ingress_path_source="default:${DEFAULT_DISCORD_INGRESS_PATH}"
  if [ -n "${DISCORD_INGRESS_PATH}" ]; then
    discord_ingress_path_source="env:DISCORD_INGRESS_PATH"
  fi
  if [ -z "${DISCORD_INGRESS_PATH}" ] && [ -n "${SETTINGS_DISCORD_INGRESS_PATH}" ]; then
    DISCORD_INGRESS_PATH="${SETTINGS_DISCORD_INGRESS_PATH}"
    discord_ingress_path_source="config:discord.ingress_path"
  fi
  if [ -z "${DISCORD_INGRESS_PATH}" ]; then
    DISCORD_INGRESS_PATH="${DEFAULT_DISCORD_INGRESS_PATH}"
  fi
  if [[ ${DISCORD_INGRESS_PATH} != /* ]]; then
    DISCORD_INGRESS_PATH="/${DISCORD_INGRESS_PATH}"
  fi

  DISCORD_INGRESS_SECRET_TOKEN_RESOLVED="${DISCORD_INGRESS_SECRET_TOKEN:-}"
  discord_ingress_secret_source="disabled"
  if [ -n "${DISCORD_INGRESS_SECRET_TOKEN_RESOLVED}" ]; then
    discord_ingress_secret_source="env:DISCORD_INGRESS_SECRET_TOKEN"
  fi
  if [ -z "${DISCORD_INGRESS_SECRET_TOKEN_RESOLVED}" ] && [ -n "${SETTINGS_DISCORD_INGRESS_SECRET_TOKEN}" ]; then
    DISCORD_INGRESS_SECRET_TOKEN_RESOLVED="${SETTINGS_DISCORD_INGRESS_SECRET_TOKEN}"
    discord_ingress_secret_source="config:discord.ingress_secret_token"
  fi

  discord_ingress_port="${DISCORD_INGRESS_BIND##*:}"
  if ! [[ ${discord_ingress_port} =~ ^[0-9]+$ ]] || [ "${discord_ingress_port}" -le 0 ] || [ "${discord_ingress_port}" -gt 65535 ]; then
    echo "Error: invalid discord ingress bind '${DISCORD_INGRESS_BIND}'. Set DISCORD_INGRESS_BIND, DISCORD_INGRESS_PORT, or discord.ingress_bind." >&2
    exit 1
  fi

  DISCORD_INGRESS_BOT_TOKEN_RESOLVED="${DISCORD_BOT_TOKEN:-${DISCORD_INGRESS_BOT_TOKEN:-local-discord-ingress-token}}"
  if [ -z "${DISCORD_BOT_TOKEN:-}" ]; then
    echo "Warning: DISCORD_BOT_TOKEN is not set; using local placeholder token for Discord ingress runtime."
  fi

  echo "Step 5: Discord ingress enabled with bind ${DISCORD_INGRESS_BIND} (source=${discord_ingress_bind_source}), path='${DISCORD_INGRESS_PATH}' (source=${discord_ingress_path_source})"
  if [ -n "${DISCORD_INGRESS_SECRET_TOKEN_RESOLVED}" ]; then
    echo "  Discord ingress secret token source=${discord_ingress_secret_source} value='***${DISCORD_INGRESS_SECRET_TOKEN_RESOLVED: -6}'"
  fi

  if ! lsof -nP -iTCP:"${discord_ingress_port}" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "  Starting omni-agent discord ingress on ${DISCORD_INGRESS_BIND}${DISCORD_INGRESS_PATH}..."
    DISCORD_BOT_TOKEN="${DISCORD_INGRESS_BOT_TOKEN_RESOLVED}" \
      OMNI_AGENT_DISCORD_INGRESS_BIND="${DISCORD_INGRESS_BIND}" \
      OMNI_AGENT_DISCORD_INGRESS_PATH="${DISCORD_INGRESS_PATH}" \
      OMNI_AGENT_DISCORD_INGRESS_SECRET_TOKEN="${DISCORD_INGRESS_SECRET_TOKEN_RESOLVED}" \
      "${CARGO_BIN}" run -p omni-agent -- channel --provider discord --discord-runtime-mode ingress --verbose >>"${LOG_FILE}" 2>&1 &
    DISCORD_INGRESS_PID=$!

    discord_ingress_ready="false"
    for _ in $(seq 1 "${DEFAULT_DISCORD_INGRESS_STARTUP_TIMEOUT_SECS}"); do
      if ! kill -0 "${DISCORD_INGRESS_PID}" 2>/dev/null; then
        echo "Error: discord ingress process exited before listener became ready (pid=${DISCORD_INGRESS_PID})." >&2
        if [ -f "${LOG_FILE}" ]; then
          echo "  Last 30 lines from ${LOG_FILE}:" >&2
          tail -30 "${LOG_FILE}" | sed 's/^/    /' >&2 || true
        fi
        exit 1
      fi
      if lsof -nP -iTCP:"${discord_ingress_port}" -sTCP:LISTEN >/dev/null 2>&1; then
        discord_ingress_ready="true"
        break
      fi
      sleep 1
    done
    if [ "${discord_ingress_ready}" = "true" ]; then
      echo "  Discord ingress listening on ${DISCORD_INGRESS_BIND}${DISCORD_INGRESS_PATH}"
    else
      echo "Error: discord ingress startup probe timed out on ${DISCORD_INGRESS_BIND} after ${DEFAULT_DISCORD_INGRESS_STARTUP_TIMEOUT_SECS}s." >&2
      echo "Hint: set XIUXIAN_WENDAO_DISCORD_INGRESS_STARTUP_TIMEOUT_SECS to a larger value on cold builds." >&2
      exit 1
    fi
  else
    existing_ingress_pid="$(lsof -nP -iTCP:"${discord_ingress_port}" -sTCP:LISTEN -t 2>/dev/null | head -n 1)"
    existing_ingress_cmd="$(ps -o command= -p "${existing_ingress_pid}" 2>/dev/null || true)"
    if [[ ${existing_ingress_cmd} != *"omni-agent"* ]] || [[ ${existing_ingress_cmd} != *"--provider discord"* ]] || [[ ${existing_ingress_cmd} != *"--discord-runtime-mode ingress"* ]]; then
      echo "Error: port ${discord_ingress_port} is listening but not an omni-agent Discord ingress process." >&2
      echo "  pid='${existing_ingress_pid:-unknown}' cmd='${existing_ingress_cmd:-unknown}'" >&2
      echo "Hint: stop that process or choose a different DISCORD_INGRESS_BIND." >&2
      exit 1
    fi
    if ! probe_discord_ingress_listener "${DISCORD_INGRESS_BIND}" "${DISCORD_INGRESS_PATH}" "${DISCORD_INGRESS_SECRET_TOKEN_RESOLVED}"; then
      exit 1
    fi
    echo "  Discord ingress already listening on ${DISCORD_INGRESS_BIND}; existing process probe passed and will be reused."
  fi
  export OMNI_AGENT_DISCORD_INGRESS_BIND="${DISCORD_INGRESS_BIND}"
  export OMNI_AGENT_DISCORD_INGRESS_PATH="${DISCORD_INGRESS_PATH}"
  if [ -n "${DISCORD_INGRESS_SECRET_TOKEN_RESOLVED}" ]; then
    export OMNI_AGENT_DISCORD_INGRESS_SECRET_TOKEN="${DISCORD_INGRESS_SECRET_TOKEN_RESOLVED}"
  fi
else
  echo "Step 5: Discord ingress disabled (DISCORD_INGRESS_ENABLED='${DISCORD_INGRESS_ENABLED}')"
fi

echo ""
echo "Step 5.5: Initializing Wendao Knowledge Graph index..."
WENDAO_BIN="${PROJECT_ROOT}/target/debug/wendao"
export WENDAO_BIN
if [ -x "${WENDAO_BIN}" ]; then
  echo "  Running initial wendao sync..."
  "${WENDAO_BIN}" sync >>"${LOG_FILE}" 2>&1 || echo "Warning: initial wendao sync failed."
else
  echo "  Note: wendao binary not found at ${WENDAO_BIN}, skipping initial sync."
fi

echo ""
echo "Step 6: Warming up embedding runtime (binary-driven config)..."
echo "  Running foreground warmup before webhook channel starts..."
"${CARGO_BIN}" run -p omni-agent -- embedding-warmup --mistral-sdk-only --text "webhook embedding warmup" 2>&1 | tee -a "${LOG_FILE}"
echo "  Warmup check completed."
echo ""

echo "Step 7: Starting omni-agent channel (webhook mode)..."
echo "  XIUXIAN_WENDAO_VALKEY_URL='${XIUXIAN_WENDAO_VALKEY_URL}'"
echo "  WEBHOOK_BIND='${WEBHOOK_BIND}'"
if [ -n "${GATEWAY_BIND:-}" ]; then
  echo "  GATEWAY_BIND='${GATEWAY_BIND}'"
  echo "  GATEWAY_HEALTH='${GATEWAY_HEALTH_URL}'"
fi
if [ -n "${OMNI_AGENT_DISCORD_INGRESS_BIND:-}" ]; then
  echo "  DISCORD_INGRESS_BIND='${OMNI_AGENT_DISCORD_INGRESS_BIND}'"
  echo "  DISCORD_INGRESS_PATH='${OMNI_AGENT_DISCORD_INGRESS_PATH:-${DEFAULT_DISCORD_INGRESS_PATH}}'"
fi
echo "  Telegram ACL source='xiuxian.toml'"
echo "  TELEGRAM_WEBHOOK_SECRET='***${TELEGRAM_WEBHOOK_SECRET: -6}'"
export RUST_LOG="${RUST_LOG:-omni_agent=debug}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
REPORT_FILE="${OMNI_CHANNEL_EXIT_REPORT_FILE:-.run/logs/omni-agent-webhook.exit.json}"
REPORT_JSONL="${OMNI_CHANNEL_EXIT_REPORT_JSONL:-.run/logs/omni-agent-webhook.exit.jsonl}"
echo "  RUST_LOG='${RUST_LOG}'"
echo "  RUST_BACKTRACE='${RUST_BACKTRACE}'"
echo "  VERBOSE='true'"
echo "  LOG_FILE='${LOG_FILE}' (tee)"
echo "  EXIT_REPORT='${REPORT_FILE}'"
echo "  EXIT_REPORT_JSONL='${REPORT_JSONL}'"
echo "  Press Ctrl+C to stop (ngrok will be stopped automatically)."
echo ""

# Bootstrap succeeded; from here on, process exit is handled by explicit channel exit reporting.
trap - ERR

python3 scripts/channel/agent_channel_runtime_monitor.py \
  --log-file "${LOG_FILE}" \
  --report-file "${REPORT_FILE}" \
  --report-jsonl "${REPORT_JSONL}" \
  -- \
  "${CARGO_BIN}" run -p omni-agent -- channel \
  --mode webhook \
  --verbose \
  --webhook-bind "${WEBHOOK_BIND}" \
  --webhook-secret-token "${TELEGRAM_WEBHOOK_SECRET}" \
  "$@"
