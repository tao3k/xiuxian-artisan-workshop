#!/usr/bin/env bash
set -euo pipefail

timeout_secs="${1:-3600}"

just rust-lint-inheritance-check
just rust-check "${timeout_secs}"
just rust-clippy
just rust-nextest
just rust-xiuxian-llm-mcp
just rust-omni-agent-mcp-facade-smoke
just rust-omni-agent-backend-role-contracts
just rust-test-omni-core-rs
just rust-security-gate
