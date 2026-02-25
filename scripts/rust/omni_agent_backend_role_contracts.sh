#!/usr/bin/env bash
set -euo pipefail

target_dir="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"

# LLM runtime role boundary:
# - mistral_local must resolve inference URL from mistral.base_url.
# - litellm_rs must ignore mistral.base_url and stay on provider routing path.
CARGO_TARGET_DIR="${target_dir}" cargo test -p omni-agent --bin omni-agent \
  resolve_runtime_inference_url_uses_mistral_base_url_for_mistral_backend
CARGO_TARGET_DIR="${target_dir}" cargo test -p omni-agent --bin omni-agent \
  resolve_runtime_inference_url_ignores_mistral_base_url_for_litellm_backend

# Embedding runtime role boundary:
# - non-mistral backend must not consume mistral.base_url.
# - mistral auto-start must require explicit mistral backend hint.
CARGO_TARGET_DIR="${target_dir}" cargo test -p omni-agent --lib \
  resolve_runtime_embed_base_url_ignores_mistral_base_url_for_non_mistral_backend
CARGO_TARGET_DIR="${target_dir}" cargo test -p omni-agent --lib \
  should_auto_start_mistral_requires_enabled_auto_start_and_mistral_backend

# Backend parsing contracts:
# - LLM and embedding parser aliases keep mistral_local explicit.
CARGO_TARGET_DIR="${target_dir}" cargo test -p omni-agent --lib \
  parse_backend_mode_accepts_mistral_aliases
CARGO_TARGET_DIR="${target_dir}" cargo test -p omni-agent --bin omni-agent \
  parse_embedding_backend_mode_supports_mistral_aliases
