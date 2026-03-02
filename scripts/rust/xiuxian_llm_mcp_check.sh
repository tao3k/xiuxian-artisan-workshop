#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo_bin="${CARGO_BIN:-${script_dir}/cargo_exec.sh}"
target_dir="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"

CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" check -p xiuxian-llm
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-llm --test mcp_facade
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-llm --test mcp_pool
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-llm --test mcp_pool_core
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-llm --test mcp_pool_retry
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-llm --test mcp_pool_runtime
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-llm --test mcp_pool_hard_timeout
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-llm --test mcp_pool_reconnect
