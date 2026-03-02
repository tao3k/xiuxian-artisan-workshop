"""Shared subprocess environment helpers for nested cargo executions."""

from __future__ import annotations

import os
import shutil
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping


def _resolve_python_interpreter(env: Mapping[str, str]) -> str | None:
    """Resolve a usable Python interpreter for nested PyO3 cargo builds."""
    candidates = [
        env.get("PYO3_PYTHON"),
        env.get("PYTHON"),
        sys.executable,
        shutil.which("python3"),
        shutil.which("python"),
    ]
    for candidate in candidates:
        if not candidate:
            continue
        if Path(candidate).exists():
            return candidate
    return None


def _resolve_devenv_profile_bin(env: Mapping[str, str]) -> Path | None:
    """Resolve a usable `.devenv/profile/bin` directory containing cargo shim."""
    candidates: list[Path] = []

    devenv_root = env.get("DEVENV_ROOT")
    if devenv_root:
        candidates.append(Path(devenv_root) / ".devenv" / "profile" / "bin")

    prj_root = env.get("PRJ_ROOT")
    if prj_root:
        candidates.append(Path(prj_root) / ".devenv" / "profile" / "bin")

    candidates.append(Path.cwd() / ".devenv" / "profile" / "bin")

    for candidate in candidates:
        cargo_shim = candidate / "cargo"
        if candidate.is_dir() and cargo_shim.exists():
            return candidate
    return None


def _prepend_path_entry(path_value: str, entry: str) -> str:
    """Prepend an entry to PATH only when it is not already present."""
    entries = path_value.split(os.pathsep) if path_value else []
    if entry in entries:
        return path_value
    return f"{entry}{os.pathsep}{path_value}" if path_value else entry


def prepare_cargo_subprocess_env(base_env: Mapping[str, str] | None = None) -> dict[str, str]:
    """Prepare env for cargo subprocesses in long-lived macOS/nix shells."""
    env = dict(base_env if base_env is not None else os.environ)

    # Preserve shell runtime dependencies; only clear stale PyO3-specific hints.
    pyo3_config_file = env.get("PYO3_CONFIG_FILE")
    if pyo3_config_file and not Path(pyo3_config_file).expanduser().exists():
        env.pop("PYO3_CONFIG_FILE", None)
    env.pop("PYO3_NO_PYTHON", None)

    # Clear stale interpreter hints and re-bind to an existing interpreter.
    chosen_python = _resolve_python_interpreter(env)
    if chosen_python:
        env["PYO3_PYTHON"] = chosen_python
        env["PYO3_ENVIRONMENT_SIGNATURE"] = chosen_python
    else:
        env.pop("PYO3_PYTHON", None)
        env.pop("PYO3_ENVIRONMENT_SIGNATURE", None)

    # Ensure nested cargo invocations prefer the project's cargo shim when available.
    profile_bin = _resolve_devenv_profile_bin(env)
    if profile_bin is not None:
        profile_bin_str = str(profile_bin)
        env["PATH"] = _prepend_path_entry(env.get("PATH", ""), profile_bin_str)
        env["CARGO"] = str(profile_bin / "cargo")

    return env


__all__ = ["prepare_cargo_subprocess_env"]
