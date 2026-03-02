#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

resolve_valkey_field() {
  python3 "${PROJECT_ROOT}/scripts/channel/resolve_valkey_endpoint.py" --field "$1"
}

DEFAULT_PORT="$(resolve_valkey_field port)"
DEFAULT_HOST="$(resolve_valkey_field host)"
DEFAULT_DB="$(resolve_valkey_field db)"

PORT="${1:-${VALKEY_PORT:-${DEFAULT_PORT}}}"
HOST="${VALKEY_HOST:-${DEFAULT_HOST}}"
DB="${VALKEY_DB:-${DEFAULT_DB}}"

if ! command -v valkey-server >/dev/null 2>&1; then
  echo "Error: valkey-server not found in PATH." >&2
  exit 1
fi
if ! command -v valkey-cli >/dev/null 2>&1; then
  echo "Error: valkey-cli not found in PATH." >&2
  exit 1
fi

RUNTIME_DIR="${PRJ_RUNTIME_DIR:-.run}/valkey"
mkdir -p "$RUNTIME_DIR"
PIDFILE="$RUNTIME_DIR/valkey-${PORT}.pid"
LOGFILE="$RUNTIME_DIR/valkey-${PORT}.log"
URL="redis://${HOST}:${PORT}/${DB}"

if valkey-cli -u "$URL" ping >/dev/null 2>&1; then
  echo "Valkey is already reachable at $URL."
  exit 0
fi

if [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
  echo "Valkey already running on ${PORT} (pid $(cat "$PIDFILE"))."
  valkey-cli -u "$URL" ping || true
  exit 0
fi

echo "Starting Valkey on port ${PORT}..."
valkey-server \
  --port "$PORT" \
  --bind "${HOST}" \
  --daemonize yes \
  --dir "$RUNTIME_DIR" \
  --pidfile "$PIDFILE" \
  --logfile "$LOGFILE"

sleep 0.2
valkey-cli -u "$URL" ping
echo "Valkey started. pidfile=$PIDFILE logfile=$LOGFILE"
