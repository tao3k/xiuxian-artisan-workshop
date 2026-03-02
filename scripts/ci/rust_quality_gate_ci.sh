#!/usr/bin/env bash
set -euo pipefail

timeout_secs="${1:-3600}"

just rust-lint-inheritance-check
just rust-test-layout
just rust-check "${timeout_secs}"
just rust-clippy
just rust-nextest
just rust-xiuxian-llm-mcp
just rust-omni-agent-mcp-facade-smoke
just rust-omni-agent-backend-role-contracts
if [[ ${OMNI_ENABLE_EMBED_ROLE_PERF_GATE:-0} == "1" ]]; then
  profile="${OMNI_EMBED_ROLE_PERF_GATE_PROFILE:-medium}"
  case "${profile}" in
  medium)
    just rust-omni-agent-embedding-role-perf-medium-gate
    ;;
  heavy)
    just rust-omni-agent-embedding-role-perf-heavy-gate
    ;;
  *)
    echo "Unsupported OMNI_EMBED_ROLE_PERF_GATE_PROFILE='${profile}' (expected: medium|heavy)." >&2
    exit 1
    ;;
  esac
fi
just rust-test-omni-core-rs
just rust-security-gate
