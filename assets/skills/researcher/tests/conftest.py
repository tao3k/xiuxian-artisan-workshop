"""Shared test setup for researcher skill tests (Qianji workflow)."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

RESEARCHER_SCRIPTS = Path(__file__).parent.parent / "scripts"
if str(RESEARCHER_SCRIPTS) not in sys.path:
    sys.path.insert(0, str(RESEARCHER_SCRIPTS))


@pytest.fixture(scope="session")
def researcher_scripts_path() -> Path:
    """Return researcher skill script directory."""
    return RESEARCHER_SCRIPTS


@pytest.fixture(autouse=True)
def skip_slow_tests_marker(request: pytest.FixtureRequest) -> None:
    """Skip tests marked slow during quick test runs."""
    if hasattr(request.config, "getoption") and request.config.getoption("--quick", default=False):
        if any(marker.name == "slow" for marker in request.iter_markers()):
            pytest.skip(f"Slow test: {request.node.name}")
