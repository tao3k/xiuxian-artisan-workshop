#!/usr/bin/env python3
"""Argument parser orchestration for complex scenarios config."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from complex_scenarios_config_env import env_int
from complex_scenarios_config_parser_core import add_core_args, build_parser
from complex_scenarios_config_parser_gates import add_output_args, add_quality_gate_args
from complex_scenarios_config_parser_sessions import add_session_identity_args

if TYPE_CHECKING:
    import argparse
    from pathlib import Path


def parse_args(
    *,
    script_dir: Path,
    webhook_url_default: str,
    default_log_file: str,
    default_max_wait: int,
    default_max_idle_secs: int,
    env_int_fn: Any | None = None,
) -> argparse.Namespace:
    """Parse command-line args for complex scenario suite."""
    parse_env_int = env_int if env_int_fn is None else env_int_fn
    parser = build_parser()
    add_core_args(
        parser,
        script_dir=script_dir,
        webhook_url_default=webhook_url_default,
        default_log_file=default_log_file,
        default_max_wait=default_max_wait,
        default_max_idle_secs=default_max_idle_secs,
    )
    add_session_identity_args(parser, env_int_fn=parse_env_int)
    add_quality_gate_args(parser)
    add_output_args(parser)
    return parser.parse_args()
