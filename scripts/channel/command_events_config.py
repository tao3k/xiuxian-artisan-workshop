#!/usr/bin/env python3
"""Compatibility facade for command-events CLI config helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

from command_events_config_parser import build_parser

if TYPE_CHECKING:
    import argparse


def parse_args(*, suites: tuple[str, ...]) -> argparse.Namespace:
    """Parse command-events runner CLI arguments."""
    return build_parser(suites=suites).parse_args()


__all__ = ["build_parser", "parse_args"]
