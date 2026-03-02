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

RUNTIME_DIR="${PRJ_RUNTIME_DIR:-.run}/valkey"
PIDFILE="$RUNTIME_DIR/valkey-${PORT}.pid"
URL="redis://${HOST}:${PORT}/${DB}"

if valkey-cli -u "$URL" ping >/dev/null 2>&1; then
  valkey-cli -u "$URL" shutdown nosave >/dev/null 2>&1 || true
  sleep 0.2
fi

if [ -f "$PIDFILE" ]; then
  PID="$(cat "$PIDFILE")"
  if kill -0 "$PID" 2>/dev/null; then
    kill "$PID" >/dev/null 2>&1 || true
  fi
  rm -f "$PIDFILE"
fi

if valkey-cli -u "$URL" ping >/dev/null 2>&1; then
  echo "Warning: Valkey is still reachable at $URL (managed by another process)." >&2
  exit 1
fi

echo "Valkey stopped on port ${PORT}."
