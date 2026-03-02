#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"${script_dir}/omni_agent_embedding_role_perf_smoke.sh" \
  "30" \
  "12" \
  "64" \
  "8" \
  "250" \
  "900" \
  "20" \
  ".run/reports/omni-agent-embedding-role-perf-smoke.medium.json"
