import importlib
import shutil
import sys
import types
from pathlib import Path

import pytest
from omni.test_kit.decorators import omni_skill

from omni.foundation.api.mcp_schema import parse_result_payload


def _load_search_module():
    scripts_dir = Path(__file__).parent.parent / "scripts"
    if str(scripts_dir) not in sys.path:
        sys.path.insert(0, str(scripts_dir))
    # Avoid cross-skill module collision (other skills also expose `search`).
    sys.modules.pop("search", None)
    return importlib.import_module("search")


class TestAdvancedToolsPatternMode:
    """Unit tests for literal-vs-regex mode selection."""

    def test_fixed_strings_enabled_for_literal_pattern(self):
        module = _load_search_module()
        _should_use_fixed_strings = module._should_use_fixed_strings

        assert _should_use_fixed_strings("Hard Constraints") is True
        assert _should_use_fixed_strings("README.md") is False

    def test_fixed_strings_disabled_for_regex_pattern(self):
        module = _load_search_module()
        _should_use_fixed_strings = module._should_use_fixed_strings

        assert _should_use_fixed_strings(r"Hard\\s+Constraints") is False
        assert _should_use_fixed_strings("foo(bar)") is False

    def test_resolve_search_root_supports_relative_path(self, tmp_path: Path):
        module = _load_search_module()
        _resolve_search_root = module._resolve_search_root

        fixture_dir = tmp_path / "fixtures"
        fixture_dir.mkdir(parents=True, exist_ok=True)

        resolved = _resolve_search_root(str(tmp_path), "fixtures")
        assert resolved == str(fixture_dir.resolve())

    def test_resolve_search_root_rejects_missing_path(self, tmp_path: Path):
        module = _load_search_module()
        _resolve_search_root = module._resolve_search_root

        with pytest.raises(ValueError):
            _resolve_search_root(str(tmp_path), "missing")

    def test_resolve_exec_uses_cached_which_results(self, monkeypatch: pytest.MonkeyPatch):
        module = _load_search_module()
        _resolve_exec = module._resolve_exec
        _which_cached = module._which_cached

        _which_cached.cache_clear()
        calls: list[str] = []

        def _fake_which(name: str) -> str | None:
            calls.append(name)
            if name == "fd":
                return "/usr/bin/fd"
            return None

        monkeypatch.setattr(module.shutil, "which", _fake_which)

        first = _resolve_exec("fd", "fdfind")
        second = _resolve_exec("fd", "fdfind")

        assert first == "/usr/bin/fd"
        assert second == "/usr/bin/fd"
        assert calls == ["fd"]

    def test_parse_vimgrep_line_returns_normalized_match(self):
        module = _load_search_module()
        _parse_vimgrep_line = module._parse_vimgrep_line

        line = "docs/guide.md:42:7:Hard Constraints section"
        parsed = _parse_vimgrep_line(line)

        assert parsed is not None
        assert parsed["file"] == "docs/guide.md"
        assert parsed["line"] == 42
        assert parsed["content"] == "Hard Constraints section"

    def test_parse_vimgrep_line_rejects_invalid_payload(self):
        module = _load_search_module()
        _parse_vimgrep_line = module._parse_vimgrep_line

        assert _parse_vimgrep_line("not-a-vimgrep-line") is None

    def test_can_use_python_filename_fast_path_requires_scoped_literal_query(self):
        module = _load_search_module()
        _can_use_python_filename_fast_path = module._can_use_python_filename_fast_path

        assert _can_use_python_filename_fast_path(
            pattern="benchmark_note",
            exclude=None,
            resolved_search_root="/tmp",
        )
        assert not _can_use_python_filename_fast_path(
            pattern="test_*.py",
            exclude=None,
            resolved_search_root="/tmp",
        )
        assert not _can_use_python_filename_fast_path(
            pattern="benchmark_note",
            exclude="target",
            resolved_search_root="/tmp",
        )
        assert not _can_use_python_filename_fast_path(
            pattern="benchmark_note",
            exclude=None,
            resolved_search_root=None,
        )

    def test_python_fast_find_files_filters_hidden_and_extension(self, tmp_path: Path):
        module = _load_search_module()
        _python_fast_find_files = module._python_fast_find_files

        docs_dir = tmp_path / "docs"
        hidden_dir = docs_dir / ".private"
        docs_dir.mkdir(parents=True, exist_ok=True)
        hidden_dir.mkdir(parents=True, exist_ok=True)

        (docs_dir / "benchmark_note.md").write_text("visible")
        (hidden_dir / "benchmark_note.md").write_text("hidden")
        (docs_dir / "benchmark_note.txt").write_text("text")

        files = _python_fast_find_files(
            project_root=str(tmp_path),
            search_root=str(docs_dir),
            pattern="benchmark_note",
            extension="md",
            max_results=100,
        )

        assert files == ["docs/benchmark_note.md"]

    def test_smart_search_reuses_cached_result_for_identical_query(
        self,
        monkeypatch: pytest.MonkeyPatch,
        tmp_path: Path,
    ) -> None:
        module = _load_search_module()
        clear_smart_search_cache = module.clear_smart_search_cache
        smart_search = module.smart_search

        calls = {"count": 0}

        def _fake_run_rg_with_retry(cmd: list[str], root: str, max_retries: int = 2):
            del cmd, root, max_retries
            calls["count"] += 1
            return "docs/guide.md:42:7:Hard Constraints section\n", "", 0

        monkeypatch.setattr(module, "_run_rg_with_retry", _fake_run_rg_with_retry)

        clear_smart_search_cache()
        try:
            paths = types.SimpleNamespace(project_root=str(tmp_path))
            first = smart_search(
                pattern="Hard Constraints",
                file_globs="*.md",
                case_sensitive=True,
                context_lines=0,
                paths=paths,
            )
            second = smart_search(
                pattern="Hard Constraints",
                file_globs="*.md",
                case_sensitive=True,
                context_lines=0,
                paths=paths,
            )
        finally:
            clear_smart_search_cache()

        assert first == second
        assert calls["count"] == 1


@pytest.mark.asyncio
@omni_skill(name="advanced_tools")
class TestAdvancedToolsModular:
    """Modular tests for advanced_tools skill."""

    async def test_smart_search(self, skill_tester):
        """Test smart_search execution."""
        result = await skill_tester.run("advanced_tools", "smart_search", pattern="import pytest")
        assert result.success
        payload = parse_result_payload(result.output)
        assert payload["tool"] == "ripgrep"
        assert isinstance(payload["matches"], list)

    async def test_smart_find(self, skill_tester, project_root):
        """Test smart_find execution."""
        # Check if fd is available
        if not shutil.which("fd"):
            pytest.skip("fd command not installed")

        # Use specific pattern to limit results
        result = await skill_tester.run(
            "advanced_tools", "smart_find", pattern="test_*.py", extension="py"
        )
        assert result.success, f"Expected success but got error: {result.error}"
        payload = parse_result_payload(result.output)
        assert payload["tool"] == "fd"
        assert isinstance(payload["files"], list)

    async def test_regex_replace(self, skill_tester, project_root):
        """Test regex_replace execution."""
        if not shutil.which("sed"):
            pytest.skip("sed command not installed")

        # Use project_root fixture instead of hardcoded path
        test_file = project_root / "test_regex_replace_temp.txt"
        test_file.write_text("Hello World")

        try:
            result = await skill_tester.run(
                "advanced_tools",
                "regex_replace",
                file_path=str(test_file),
                pattern="World",
                replacement="Modular",
            )

            assert result.success, f"Expected success but got error: {result.error}"
            assert test_file.read_text().strip() == "Hello Modular"
        finally:
            # Cleanup
            test_file.unlink(missing_ok=True)

    async def test_batch_replace_dry_run(self, skill_tester, project_root):
        """Test batch_replace execution (dry run)."""
        # Use project_root fixture - create test files in a subdir
        test_dir = project_root / "test_batch_temp"
        test_dir.mkdir(exist_ok=True)

        try:
            (test_dir / "file1.py").write_text("old_val = 1")
            (test_dir / "file2.py").write_text("old_val = 2")

            result = await skill_tester.run(
                "advanced_tools",
                "batch_replace",
                pattern="old_val",
                replacement="new_val",
                file_glob="test_batch_temp/*.py",
                dry_run=True,
            )

            assert result.success, f"Expected success but got error: {result.error}"
            payload = parse_result_payload(result.output)
            assert payload["mode"] == "Dry-Run"
            assert payload["count"] == 2
            # Files should NOT be changed
            assert "old_val" in (test_dir / "file1.py").read_text()
        finally:
            # Cleanup
            shutil.rmtree(test_dir, ignore_errors=True)
