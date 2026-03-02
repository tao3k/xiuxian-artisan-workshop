#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo_bin="${CARGO_BIN:-${script_dir}/cargo_exec.sh}"
target_dir="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"

# LLM runtime role boundary:
# - minimax provider setting should resolve minimax OpenAI-compatible URL.
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --bin omni-agent \
  resolve_runtime_inference_url_uses_minimax_provider_default_when_configured

# Embedding runtime role boundary:
# - non-mistral backend must not consume mistral.base_url.
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --lib \
  resolve_runtime_embed_base_url_ignores_mistral_base_url_for_non_mistral_backend

# Backend parsing contracts:
# - embedding parser keeps mistral_sdk explicit.
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --bin omni-agent \
  parse_embedding_backend_mode_supports_mistral_sdk_aliases
