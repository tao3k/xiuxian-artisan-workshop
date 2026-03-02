#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo_bin="${CARGO_BIN:-${script_dir}/cargo_exec.sh}"
target_dir="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"

CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" check -p omni-agent
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" check -p omni-agent --no-default-features
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --no-run
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --no-run --no-default-features
