#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo_bin="${CARGO_BIN:-${script_dir}/cargo_exec.sh}"

# Temporary transitive exceptions for unresolved upstream advisories.
# Remove entries as dependency chains are upgraded.
ignore_args=(
  --ignore RUSTSEC-2023-0071
  --ignore RUSTSEC-2025-0141
  --ignore RUSTSEC-2024-0436
  --ignore RUSTSEC-2025-0134
  --ignore RUSTSEC-2026-0002
)

"${cargo_bin}" audit --deny warnings "${ignore_args[@]}"
