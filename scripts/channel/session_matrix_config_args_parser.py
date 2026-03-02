#!/usr/bin/env python3
"""Parser builder for session matrix CLI arguments."""

from __future__ import annotations

import argparse

from session_matrix_config_args_parser_groups import (
    add_identity_args,
    add_probe_and_output_args,
    add_runtime_args,
)


def parse_args(*, webhook_url_default: str) -> argparse.Namespace:
    """Parse session matrix CLI arguments."""
    parser = argparse.ArgumentParser(
        description=(
            "Run session isolation matrix against local Telegram webhook runtime "
            "and emit structured JSON/Markdown reports."
        )
    )
    add_runtime_args(parser, webhook_url_default=webhook_url_default)
    add_identity_args(parser)
    add_probe_and_output_args(parser)
    return parser.parse_args()
