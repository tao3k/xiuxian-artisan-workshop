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
  if [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
    echo "Valkey is running on ${PORT} (pid $(cat "$PIDFILE"))."
  else
    echo "Valkey is reachable on ${PORT} (pidfile not managed by just)."
  fi
  echo "PONG"
  exit 0
fi

echo "Valkey is not running on ${PORT}."
exit 1
