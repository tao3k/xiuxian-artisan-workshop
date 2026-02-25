#!/usr/bin/env python3
"""Parser builder for command-events blackbox suite CLI."""

from __future__ import annotations

import argparse

from command_events_config_parser_admin import add_admin_args
from command_events_config_parser_core import add_core_args
from command_events_config_parser_selection import add_output_args, add_selection_args


def build_parser(*, suites: tuple[str, ...]) -> argparse.ArgumentParser:
    """Build argparse parser for command-events runner."""
    parser = argparse.ArgumentParser(
        description=(
            "Run a strict Telegram command black-box matrix against local webhook runtime. "
            "Each probe requires a command-specific event and forbids MCP tool-call errors."
        )
    )
    add_core_args(parser)
    add_admin_args(parser)
    add_selection_args(parser, suites=suites)
    add_output_args(parser)
    return parser
