#!/usr/bin/env python3
"""Compatibility facade for dedup probe config helpers."""

from __future__ import annotations

from typing import Any

from dedup_probe_config_args import parse_args as _parse_args_impl
from dedup_probe_config_build import build_config as _build_config_impl


def parse_args(**kwargs: Any) -> Any:
    """Parse CLI args for deterministic dedup probe."""
    return _parse_args_impl(**kwargs)


def build_config(args: Any, **kwargs: Any) -> Any:
    """Build validated probe config from parsed args."""
    return _build_config_impl(args, **kwargs)
