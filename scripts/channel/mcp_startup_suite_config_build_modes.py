#!/usr/bin/env python3
"""Mode/restart-flag helpers for MCP startup suite config construction."""

from __future__ import annotations

import sys
from typing import Any


def resolve_restart_and_mode_flags(
    args: Any,
) -> tuple[str | None, bool, bool, bool]:
    """Resolve restart command and hot/cold enablement flags."""
    restart_mcp_cmd = args.restart_mcp_cmd.strip() or None
    allow_mcp_restart = bool(args.allow_mcp_restart or restart_mcp_cmd is not None)
    skip_hot = bool(args.skip_hot)
    skip_cold = bool(args.skip_cold)
    if skip_hot and not skip_cold and not allow_mcp_restart:
        raise ValueError(
            "cold-only startup suite requires MCP restart permission. "
            "Use --allow-mcp-restart or --restart-mcp-cmd."
        )
    if not skip_cold and not allow_mcp_restart:
        print(
            "[mcp-startup-suite] cold mode auto-skipped (restart not allowed). "
            "Use --allow-mcp-restart to enable cold restart checks.",
            file=sys.stderr,
        )
        skip_cold = True
    if skip_hot and skip_cold:
        raise ValueError("At least one mode must run (do not set both --skip-hot and --skip-cold).")
    return restart_mcp_cmd, allow_mcp_restart, skip_hot, skip_cold
