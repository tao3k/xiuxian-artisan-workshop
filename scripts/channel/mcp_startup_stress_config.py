#!/usr/bin/env python3
"""Compatibility facade for MCP startup stress config helpers."""

from __future__ import annotations

from mcp_startup_stress_config_args import parse_args
from mcp_startup_stress_config_build import build_config, resolve_runtime_paths, validate_args

__all__ = ["build_config", "parse_args", "resolve_runtime_paths", "validate_args"]
