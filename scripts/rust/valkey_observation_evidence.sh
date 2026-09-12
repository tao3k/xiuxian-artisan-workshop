#!/usr/bin/env bash
# Release latency and allocation evidence are deliberately separate runs.
set -euo pipefail

: "${VALKEY_URL:?A dedicated Valkey instance is required}"
output="${PRJ_RUNTIME_DIR:-.run}/reports/valkey-observation"
mkdir -p "$output"
output="$(cd "$output" && pwd)"

{
  git rev-parse HEAD
  printf 'pr_head=%s\n' "${PR_HEAD_SHA:-local}"
  rustc --version
  uname -srm
  valkey-server --version
  heaptrack --version
} > "$output/environment.txt"

cargo test -p xiuxian-db-store --features valkey --release --test unit_test \
  --no-run --message-format=json > "$output/build.jsonl"
executable="$(jq -ser '[.[] | select(.reason == "compiler-artifact" and .target.name == "unit_test" and .profile.test and .executable != null)] | if length == 1 then .[0].executable else error("expected one test executable") end' "$output/build.jsonl")"
test_name="valkey_performance::valkey_observation_release_probe"

WAS_PROBE_MODE=both "$executable" --exact "$test_name" --ignored --nocapture \
  --test-threads=1 | tee "$output/latency.log"
grep -q 'test result: ok. 1 passed' "$output/latency.log"

for mode in serial bounded32; do
  WAS_PROBE_MODE="$mode" heaptrack --record-only --output "$output/$mode" \
    "$executable" --exact "$test_name" --ignored --nocapture --test-threads=1 \
    2>&1 | tee "$output/$mode-profile.log"
  grep -q 'test result: ok. 1 passed' "$output/$mode-profile.log"
  shopt -s nullglob
  traces=("$output/$mode.gz" "$output/$mode.zst")
  if [[ ${#traces[@]} -ne 1 ]]; then
    printf 'Expected one heap trace for %s, found %s\n' "$mode" "${#traces[@]}" >&2
    exit 1
  fi
  heaptrack_print "${traces[0]}" > "$output/$mode-allocations.txt"
done

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    printf '### Valkey Observation Evidence\n\n'
    printf 'Candidate remains experimental; profiling latency is not benchmark latency.\n\n```text\n'
    grep -E '^(profile=|keys=)' "$output/latency.log"
    printf '```\n'
  } >> "$GITHUB_STEP_SUMMARY"
fi
