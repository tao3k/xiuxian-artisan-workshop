from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

_MODULE_PATH = Path(__file__).resolve().with_name("bootcamp_distributed_swarm_gate.py")
_SPEC = importlib.util.spec_from_file_location("bootcamp_distributed_swarm_gate", _MODULE_PATH)
assert _SPEC and _SPEC.loader
_MODULE = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = _MODULE
_SPEC.loader.exec_module(_MODULE)


def test_gate_main_dry_run_skip_rust() -> None:
    rc = _MODULE.main(["--dry-run", "--skip-rust-test"])
    assert rc == 0


def test_gate_main_fails_when_manifest_missing(tmp_path: Path) -> None:
    context_file = tmp_path / "context.json"
    context_file.write_text("{}", encoding="utf-8")
    rc = _MODULE.main(
        [
            "--dry-run",
            "--manifest",
            str(tmp_path / "missing.toml"),
            "--context-file",
            str(context_file),
        ]
    )
    assert rc == 1
