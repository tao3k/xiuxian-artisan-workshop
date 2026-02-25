#!/usr/bin/env python3
"""CLI config helpers for MCP startup suite runner."""

from __future__ import annotations

from mcp_startup_suite_config_args import parse_args
from mcp_startup_suite_config_build import build_config

__all__ = ["build_config", "parse_args"]
