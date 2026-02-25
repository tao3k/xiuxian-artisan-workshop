#!/usr/bin/env python3
"""Compatibility facade for acceptance runner config helpers."""

from __future__ import annotations

from acceptance_runner_config_args import parse_args
from acceptance_runner_config_build import (
    build_config,
    parse_optional_env_int,
    resolve_thread_ids,
    validate_args,
)

# Backward-compatible private alias.
_parse_optional_env_int = parse_optional_env_int

__all__ = [
    "_parse_optional_env_int",
    "build_config",
    "parse_args",
    "parse_optional_env_int",
    "resolve_thread_ids",
    "validate_args",
]
