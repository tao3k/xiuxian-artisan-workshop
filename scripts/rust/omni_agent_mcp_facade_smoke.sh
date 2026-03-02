#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo_bin="${CARGO_BIN:-${script_dir}/cargo_exec.sh}"
target_dir="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"

CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --test mcp_connect_startup
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --test discover_cache_valkey_precedence
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --test mcp_pool_hard_timeout
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --test mcp_pool_reconnect
