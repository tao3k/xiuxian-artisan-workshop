"""Unit tests for shared cargo subprocess environment normalization."""

from __future__ import annotations

import sys
from pathlib import Path

from omni.foundation.runtime.cargo_subprocess_env import prepare_cargo_subprocess_env


def test_prepare_cargo_subprocess_env_rebinds_stale_pyo3_python() -> None:
    """Stale interpreter hints should be replaced with a valid Python executable."""
    env = {
        "PYO3_PYTHON": "/nix/store/does-not-exist-python/bin/python",
        "PYO3_ENVIRONMENT_SIGNATURE": "stale",
        "PYO3_CONFIG_FILE": "/tmp/stale-config",
        "PYO3_NO_PYTHON": "1",
        "PYTHON": sys.executable,
        "DYLD_LIBRARY_PATH": "/tmp/stale-dyld",
    }

    prepared = prepare_cargo_subprocess_env(env)

    assert prepared["PYO3_PYTHON"] == sys.executable
    assert prepared["PYO3_ENVIRONMENT_SIGNATURE"] == sys.executable
    assert prepared["DYLD_LIBRARY_PATH"] == "/tmp/stale-dyld"
    assert "PYO3_CONFIG_FILE" not in prepared
    assert "PYO3_NO_PYTHON" not in prepared


def test_prepare_cargo_subprocess_env_keeps_valid_pyo3_python() -> None:
    """Valid interpreter hints should remain unchanged."""
    env = {"PYO3_PYTHON": sys.executable}
    prepared = prepare_cargo_subprocess_env(env)

    assert prepared["PYO3_PYTHON"] == sys.executable
    assert prepared["PYO3_ENVIRONMENT_SIGNATURE"] == sys.executable


def test_prepare_cargo_subprocess_env_drops_unresolvable_hints(monkeypatch) -> None:
    """When no interpreter can be resolved, PYO3 hints should be cleared."""
    monkeypatch.setattr(
        "omni.foundation.runtime.cargo_subprocess_env.sys.executable",
        "/tmp/does-not-exist-python",
    )
    monkeypatch.setattr(
        "omni.foundation.runtime.cargo_subprocess_env.shutil.which",
        lambda _name: None,
    )

    env = {
        "PYO3_PYTHON": "/tmp/does-not-exist-python",
        "PYO3_ENVIRONMENT_SIGNATURE": "stale",
        "PYTHON": "/tmp/also-missing-python",
    }
    prepared = prepare_cargo_subprocess_env(env)

    assert "PYO3_PYTHON" not in prepared
    assert "PYO3_ENVIRONMENT_SIGNATURE" not in prepared


def test_prepare_cargo_subprocess_env_prepends_devenv_profile_bin(tmp_path: Path) -> None:
    """When available, `.devenv/profile/bin` should be prioritized for cargo subprocesses."""
    profile_bin = tmp_path / ".devenv" / "profile" / "bin"
    profile_bin.mkdir(parents=True)
    cargo_shim = profile_bin / "cargo"
    cargo_shim.write_text("#!/usr/bin/env bash\n", encoding="utf-8")

    env = {
        "DEVENV_ROOT": str(tmp_path),
        "PATH": "/usr/bin:/bin",
    }

    prepared = prepare_cargo_subprocess_env(env)

    assert prepared["PATH"].startswith(f"{profile_bin}:")
    assert prepared["CARGO"] == str(cargo_shim)


def test_prepare_cargo_subprocess_env_does_not_duplicate_profile_bin(tmp_path: Path) -> None:
    """Existing `.devenv/profile/bin` PATH entries should not be duplicated."""
    profile_bin = tmp_path / ".devenv" / "profile" / "bin"
    profile_bin.mkdir(parents=True)
    cargo_shim = profile_bin / "cargo"
    cargo_shim.write_text("#!/usr/bin/env bash\n", encoding="utf-8")

    env = {
        "DEVENV_ROOT": str(tmp_path),
        "PATH": f"{profile_bin}:/usr/bin:/bin",
    }

    prepared = prepare_cargo_subprocess_env(env)

    assert prepared["PATH"] == env["PATH"]
    assert prepared["CARGO"] == str(cargo_shim)
