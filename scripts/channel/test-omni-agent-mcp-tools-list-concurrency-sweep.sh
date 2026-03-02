#!/usr/bin/env bash
# Compatibility wrapper: use Python tools/list concurrency sweep probe.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec uv run python "${SCRIPT_DIR}/test_omni_agent_mcp_tools_list_concurrency_sweep.py" "$@"
