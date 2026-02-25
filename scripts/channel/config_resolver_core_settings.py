#!/usr/bin/env python3
"""Settings YAML parsing helpers for channel config resolution."""

from __future__ import annotations

import os
import re
from pathlib import Path

from config_resolver_core_scalars import parse_yaml_scalar_list, strip_inline_comment, unquote


def repo_root_from(start: Path) -> Path:
    """Resolve repository root by walking upward until .git is found."""
    for candidate in [start, *start.parents]:
        if (candidate / ".git").exists():
            return candidate
    return start


def read_telegram_key_from_yaml(path: Path, key: str) -> str | None:
    """Read telegram.<key> scalar value from settings YAML."""
    if not path.exists():
        return None

    lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
    in_telegram = False
    telegram_indent = 0
    key_re = re.compile(rf"^\s*{re.escape(key)}\s*:\s*(.*)$")

    for raw in lines:
        line = raw.rstrip("\n")
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue

        indent = len(line) - len(line.lstrip(" "))
        if not in_telegram:
            if re.match(r"^\s*telegram\s*:\s*$", line):
                in_telegram = True
                telegram_indent = indent
            continue

        if indent <= telegram_indent:
            break

        match = key_re.match(line)
        if not match:
            continue
        payload = unquote(strip_inline_comment(match.group(1)))
        if payload in {"", "null", "None", "~"}:
            return ""
        return payload

    return None


def settings_candidates(repo_root: Path) -> list[Path]:
    """Return settings candidates in precedence order: user then system."""
    prj_config_home = Path(os.environ.get("PRJ_CONFIG_HOME", str(repo_root / ".config")))
    user_settings = prj_config_home / "omni-dev-fusion" / "settings.yaml"
    system_settings = repo_root / "packages" / "conf" / "settings.yaml"
    return [user_settings, system_settings]


def read_telegram_acl_allow_users(path: Path) -> list[str] | None:
    """Read telegram.acl.allow.users as a normalized list."""
    if not path.exists():
        return None

    lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
    in_telegram = False
    telegram_indent = 0
    in_acl = False
    acl_indent = 0
    in_allow = False
    allow_indent = 0
    collecting_block = False
    users_key_indent = 0
    block_values: list[str] = []

    for raw in lines:
        line = raw.rstrip("\n")
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        indent = len(line) - len(line.lstrip(" "))

        if not in_telegram:
            if re.match(r"^\s*telegram\s*:\s*$", line):
                in_telegram = True
                telegram_indent = indent
            continue

        if indent <= telegram_indent:
            break

        if in_allow and collecting_block:
            if indent <= users_key_indent:
                return block_values
            block_match = re.match(r"^\s*-\s*(.*)$", line)
            if block_match:
                value = unquote(strip_inline_comment(block_match.group(1)))
                if value and value not in {"null", "None", "~"}:
                    block_values.append(value)
            continue

        if in_allow:
            if indent <= allow_indent:
                in_allow = False
                collecting_block = False
            else:
                users_match = re.match(r"^\s*users\s*:\s*(.*)$", line)
                if users_match:
                    users_key_indent = indent
                    payload = users_match.group(1).strip()
                    if payload == "":
                        collecting_block = True
                        block_values = []
                        continue
                    return parse_yaml_scalar_list(payload)
            continue

        if in_acl and indent <= acl_indent:
            in_acl = False

        if in_acl:
            if re.match(r"^\s*allow\s*:\s*$", line):
                in_allow = True
                allow_indent = indent
            continue

        if re.match(r"^\s*acl\s*:\s*$", line):
            in_acl = True
            acl_indent = indent

    if collecting_block:
        return block_values
    return None
