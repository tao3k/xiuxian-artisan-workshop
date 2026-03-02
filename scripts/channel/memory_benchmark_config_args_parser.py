#!/usr/bin/env python3
"""Argument parser builder for memory benchmark runner."""

from __future__ import annotations

import argparse
from typing import TYPE_CHECKING

from memory_benchmark_config_args_parser_groups import (
    add_dataset_and_paths_args,
    add_identity_args,
    add_output_and_policy_args,
    add_runtime_args,
)

if TYPE_CHECKING:
    from pathlib import Path


def parse_args(
    *,
    script_dir: Path,
    default_log_file: str,
    default_max_wait: int,
    default_max_idle_secs: int,
) -> argparse.Namespace:
    """Parse command-line arguments for memory benchmark script."""
    parser = argparse.ArgumentParser(
        description=(
            "Run live A/B memory benchmark against local omni-agent webhook runtime "
            "(baseline vs adaptive feedback mode)."
        )
    )
    add_dataset_and_paths_args(
        parser,
        script_dir=script_dir,
        default_log_file=default_log_file,
    )
    add_identity_args(parser)
    add_runtime_args(
        parser,
        default_max_wait=default_max_wait,
        default_max_idle_secs=default_max_idle_secs,
    )
    add_output_and_policy_args(parser)
    return parser.parse_args()
