#!/usr/bin/env bash
set -euo pipefail

bash -n \
  scripts/ci/rust_quality_gate_ci.sh \
  scripts/ci/test_quick.sh \
  scripts/rust/check_lint_inheritance.sh \
  scripts/rust/omni_agent_profiles_check.sh \
  scripts/rust/omni_agent_dependency_assertions.sh \
  scripts/rust/omni_agent_mcp_facade_smoke.sh \
  scripts/rust/omni_agent_backend_role_contracts.sh \
  scripts/rust/xiuxian_llm_mcp_check.sh \
  scripts/rust/wendao_retrieval_audits.sh \
  scripts/rust/telegram_session_isolation_rust.sh \
  scripts/channel/valkey_live_gate.sh \
  scripts/benchmark_skills_tools_ci.sh \
  scripts/gate_wendao_ppr.sh \
  scripts/wendao_ppr_rollout_ci.sh

just --dry-run rust-omni-agent-profiles >/dev/null
just --dry-run rust-omni-agent-dependency-assertions >/dev/null
just --dry-run rust-omni-agent-mcp-facade-smoke >/dev/null
just --dry-run rust-omni-agent-backend-role-contracts >/dev/null
just --dry-run rust-omni-agent-embedding-role-perf-smoke >/dev/null
just --dry-run rust-xiuxian-llm-mcp >/dev/null
just --dry-run rust-retrieval-audits >/dev/null
just --dry-run gate-wendao-ppr >/dev/null
just --dry-run validate-wendao-ppr-reports >/dev/null
just --dry-run wendao-ppr-rollout-status >/dev/null
just --dry-run telegram-session-isolation-rust >/dev/null
just --dry-run valkey-live >/dev/null
just --dry-run memory-gate-nightly >/dev/null
just --dry-run agent-channel-discord-ingress-stress >/dev/null
just --dry-run verify-native-runtime >/dev/null
just --dry-run benchmark-mcp-tools-list-sweep >/dev/null
