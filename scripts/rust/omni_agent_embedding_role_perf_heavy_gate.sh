#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"${script_dir}/omni_agent_embedding_role_perf_smoke.sh" \
  "50" \
  "20" \
  "128" \
  "16" \
  "350" \
  "1200" \
  "35" \
  ".run/reports/omni-agent-embedding-role-perf-smoke.heavy.json"
