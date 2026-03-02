#!/usr/bin/env bash
# Run Telegram channel in polling mode with local Valkey bootstrapping.
# Usage: TELEGRAM_BOT_TOKEN=xxx ./scripts/channel/agent-channel-polling.sh [valkey_port]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CARGO_BIN="${CARGO_BIN:-${PROJECT_ROOT}/scripts/rust/cargo_exec.sh}"
cd "${PROJECT_ROOT}"

resolve_valkey_field() {
  python3 "${PROJECT_ROOT}/scripts/channel/resolve_valkey_endpoint.py" --field "$1"
}

resolve_prj_data_home() {
  if [ -n "${PRJ_DATA_HOME:-}" ]; then
    printf '%s' "${PRJ_DATA_HOME}"
    return 0
  fi
  printf '%s' "${PROJECT_ROOT}/.data"
  return 0
}

# Source .env if present
if [ -f .env ]; then
  set -a
  # shellcheck source=/dev/null
  source .env
  set +a
fi

VALKEY_PORT="${VALKEY_PORT:-$(resolve_valkey_field port)}"
if [ $# -gt 0 ] && [[ $1 =~ ^[0-9]+$ ]]; then
  VALKEY_PORT="$1"
  shift
fi

bash "${SCRIPT_DIR}/valkey-start.sh" "${VALKEY_PORT}"
VALKEY_HOST="${VALKEY_HOST:-$(resolve_valkey_field host)}"
VALKEY_DB="${VALKEY_DB:-$(resolve_valkey_field db)}"
VALKEY_RESOLVED_URL="redis://${VALKEY_HOST}:${VALKEY_PORT}/${VALKEY_DB}"
export XIUXIAN_WENDAO_VALKEY_URL="${XIUXIAN_WENDAO_VALKEY_URL:-${VALKEY_RESOLVED_URL}}"
PRJ_DATA_HOME_RESOLVED="$(resolve_prj_data_home)"
OLLAMA_MODELS_SOURCE="env:OLLAMA_MODELS"
if [ -z "${OLLAMA_MODELS:-}" ]; then
  OLLAMA_MODELS="${PRJ_DATA_HOME_RESOLVED}/models"
  OLLAMA_MODELS_SOURCE="default:${PRJ_DATA_HOME_RESOLVED}/models"
fi
export OLLAMA_MODELS

echo "Starting Telegram channel (polling mode)..."
echo "XIUXIAN_WENDAO_VALKEY_URL='${XIUXIAN_WENDAO_VALKEY_URL}'"
echo "OLLAMA_MODELS='${OLLAMA_MODELS}' (source=${OLLAMA_MODELS_SOURCE})"
echo "Telegram ACL source='xiuxian.toml'"

"${CARGO_BIN}" run -p omni-agent -- channel \
  --mode polling \
  "$@"
