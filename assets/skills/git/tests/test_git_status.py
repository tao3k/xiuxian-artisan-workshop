"""Import smoke tests for git skill scripts."""

from __future__ import annotations

import sys
from pathlib import Path

SKILLS_ROOT = Path(__file__).parent.parent.parent
if str(SKILLS_ROOT) not in sys.path:
    sys.path.insert(0, str(SKILLS_ROOT))


class TestGitScripts:
    """Verify script modules and key entrypoints exist."""

    def test_commit_script_imports(self) -> None:
        from git.scripts import commit

        assert hasattr(commit, "commit")

    def test_prepare_script_imports(self) -> None:
        from git.scripts import prepare

        assert hasattr(prepare, "stage_and_scan")

    def test_render_script_imports(self) -> None:
        from git.scripts import rendering

        assert hasattr(rendering, "render_commit_message")

    def test_commit_state_script_imports(self) -> None:
        from git.scripts import commit_state

        assert hasattr(commit_state, "create_initial_state")

    def test_smart_commit_commands_imports(self) -> None:
        from git.scripts.smart_commit_graphflow import commands

        assert hasattr(commands, "smart_commit")
        assert hasattr(commands, "run_qianji_engine")
