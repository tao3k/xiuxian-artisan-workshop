#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo_bin="${CARGO_BIN:-${script_dir}/cargo_exec.sh}"
ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "${ROOT_DIR}"

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required to resolve libpython for omni-core-rs tests." >&2
  exit 1
fi

PYLIB_PATH="$(python3 scripts/rust/resolve_libpython_path.py)"

if [[ -z ${PYLIB_PATH} || ! -f ${PYLIB_PATH} ]]; then
  echo "failed to resolve libpython path from active python3." >&2
  exit 1
fi

TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"

if [[ $# -eq 0 ]]; then
  set -- --no-fail-fast
fi

echo "Running omni-core-rs tests with CARGO_TARGET_DIR=${TARGET_DIR}"
echo "Resolved libpython: ${PYLIB_PATH}"

case "$(uname -s)" in
Darwin)
  DYLD_INSERT_LIBRARIES="${PYLIB_PATH}" \
    CARGO_TARGET_DIR="${TARGET_DIR}" \
    "${cargo_bin}" test -p omni-core-rs "$@"
  ;;
*)
  CARGO_TARGET_DIR="${TARGET_DIR}" "${cargo_bin}" test -p omni-core-rs "$@"
  ;;
esac
