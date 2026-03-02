from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

import pytest

_MODULE_PATH = Path(__file__).resolve().with_name("bootcamp_distributed_swarm.py")
_SPEC = importlib.util.spec_from_file_location("bootcamp_distributed_swarm", _MODULE_PATH)
assert _SPEC and _SPEC.loader
_MODULE = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = _MODULE
_SPEC.loader.exec_module(_MODULE)


def test_default_agents_include_student_steward_teacher() -> None:
    agents = _MODULE.default_agents()
    assert [agent.role_class for agent in agents] == ["student", "steward", "teacher"]


def test_parse_agent_spec_supports_optional_weight() -> None:
    spec = _MODULE.parse_agent_spec("professor_1:teacher:1.5")
    assert spec.agent_id == "professor_1"
    assert spec.role_class == "teacher"
    assert spec.weight == pytest.approx(1.5)

    default_weight = _MODULE.parse_agent_spec("steward_1:steward")
    assert default_weight.weight == pytest.approx(1.0)


def test_build_qianji_command_contains_required_arguments(tmp_path: Path) -> None:
    command = _MODULE.build_qianji_command(
        project_root=tmp_path,
        manifest_path=tmp_path / "flow.toml",
        context_json=json.dumps({"hello": "world"}),
        session_id="swarm_sess",
        cargo_bin="cargo",
        features="llm",
    )
    assert command[:8] == [
        "cargo",
        "run",
        "-p",
        "xiuxian-qianji",
        "--features",
        "llm",
        "--bin",
        "qianji",
    ]
    assert command[-1] == "swarm_sess"
