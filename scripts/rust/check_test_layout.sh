#!/usr/bin/env bash
set -euo pipefail

search_root="${1:-packages/rust/crates}"

echo "Checking Rust test layout under ${search_root}..."

failed=0

legacy_test_files="$(find "${search_root}" -type f -path '*/src/*' \( -name 'tests.rs' -o -name '*_tests.rs' -o -name 'test_*.rs' \) -print)"
if [[ -n ${legacy_test_files} ]]; then
  echo "Found test implementation files under src/ (forbidden):"
  echo "${legacy_test_files}"
  failed=1
fi

nested_test_dirs="$(find "${search_root}" -type f -path '*/src/**/tests/*' -name '*.rs' -print)"
if [[ -n ${nested_test_dirs} ]]; then
  echo "Found nested src/**/tests/*.rs files (forbidden):"
  echo "${nested_test_dirs}"
  failed=1
fi

inline_test_modules="$(rg -n --multiline '#\[cfg\(test\)\]\s*\n\s*mod [A-Za-z0-9_]+\s*\{' "${search_root}" --glob '**/src/**/*.rs' || true)"
if [[ -n ${inline_test_modules} ]]; then
  echo "Found inline #[cfg(test)] mod ... { ... } blocks in src/ (forbidden):"
  echo "${inline_test_modules}"
  failed=1
fi

missing_path_mounts="$(
  python - <<'PY'
import glob
import os
import re

issues = []

for path in glob.glob("packages/rust/crates/**/src/**/*.rs", recursive=True):
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().splitlines()

    for idx, line in enumerate(lines):
        module_match = re.search(r"\bmod\s+([A-Za-z0-9_]+(?:_tests)?)\s*;", line)
        if not module_match:
            continue
        name = module_match.group(1)
        if name != "tests" and not name.endswith("_tests"):
            continue

        attrs = "\n".join(lines[max(0, idx - 8):idx])
        if "#[cfg(test)]" not in attrs:
            continue

        path_attr = None
        for prev in range(idx - 1, max(-1, idx - 10), -1):
            path_match = re.search(r'#\[path\s*=\s*"([^"]+)"\]', lines[prev])
            if path_match:
                path_attr = path_match.group(1)
                break

        if not path_attr:
            issues.append(f"{path}:{idx + 1}: missing #[path = \"...\"] for mod {name};")
            continue

        target = os.path.normpath(os.path.join(os.path.dirname(path), path_attr))
        if not os.path.exists(target):
            issues.append(f"{path}:{idx + 1}: path target does not exist: {path_attr}")

if issues:
    print("\n".join(issues))
PY
)"

if [[ -n ${missing_path_mounts} ]]; then
  echo "Found invalid/missing test module path mounts:"
  echo "${missing_path_mounts}"
  failed=1
fi

if [[ ${failed} -ne 0 ]]; then
  echo "Rust test layout check failed."
  exit 1
fi

echo "Rust test layout check passed."
