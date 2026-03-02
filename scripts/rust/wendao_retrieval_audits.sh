#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo_bin="${CARGO_BIN:-${script_dir}/cargo_exec.sh}"
target_dir="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"

CARGO_TARGET_DIR="${target_dir}" \
  XIUXIAN_WENDAO_ENFORCE_PPR_P95=1 XIUXIAN_WENDAO_PPR_P95_MS_BUDGET=50 \
  "${cargo_bin}" test -p xiuxian-wendao --test test_link_graph_ppr_benchmark -- --ignored --nocapture

CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-wendao --test test_link_graph_seed_and_priors -- --nocapture
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-wendao --test test_wendao_cli provisional_links_are_isolated_before_promotion -- --nocapture
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-wendao --test test_wendao_cli promoted_overlay_resolves_mixed_alias_forms -- --nocapture
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-wendao --test test_wendao_cli promoted_overlay_is_isolated_by_key_prefix -- --nocapture
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p xiuxian-wendao --test test_wendao_cli agentic_run_emits_discovery_quality_signals -- --nocapture
