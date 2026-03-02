#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo_bin="${CARGO_BIN:-${script_dir}/cargo_exec.sh}"
single_runs="${1:-20}"
batch_runs="${2:-10}"
concurrent_total="${3:-64}"
concurrent_width="${4:-8}"
max_single_p95_ms="${5:-}"
max_batch8_p95_ms="${6:-}"
min_concurrent_rps="${7:-}"
report_json="${8:-}"

echo "Running omni-agent embedding role perf smoke (litellm_rs + mistral_sdk)..."

local_host="${XIUXIAN_WENDAO_LOCAL_HOST:-localhost}"
default_ollama_base_url="${XIUXIAN_WENDAO_OLLAMA_BASE_URL:-http://${local_host}:11434}"

env_args=(
  "OLLAMA_MODELS=${OLLAMA_MODELS:-${PRJ_DATA_HOME:-.data}/models}"
  "OMNI_EMBED_UPSTREAM_BASE_URL=${OMNI_EMBED_UPSTREAM_BASE_URL:-${default_ollama_base_url}}"
  "OMNI_EMBED_SINGLE_RUNS=${single_runs}"
  "OMNI_EMBED_BATCH_RUNS=${batch_runs}"
  "OMNI_EMBED_CONCURRENT_TOTAL=${concurrent_total}"
  "OMNI_EMBED_CONCURRENT_WIDTH=${concurrent_width}"
)

if [[ -n ${max_single_p95_ms} ]]; then
  env_args+=("OMNI_EMBED_MAX_SINGLE_P95_MS=${max_single_p95_ms}")
fi
if [[ -n ${max_batch8_p95_ms} ]]; then
  env_args+=("OMNI_EMBED_MAX_BATCH8_P95_MS=${max_batch8_p95_ms}")
fi
if [[ -n ${min_concurrent_rps} ]]; then
  env_args+=("OMNI_EMBED_MIN_CONCURRENT_RPS=${min_concurrent_rps}")
fi
if [[ -n ${report_json} ]]; then
  env_args+=("OMNI_AGENT_EMBED_ROLE_PERF_REPORT=${report_json}")
fi

env "${env_args[@]}" "${cargo_bin}" test -p omni-agent --test embedding_role_perf_smoke -- --ignored --nocapture
