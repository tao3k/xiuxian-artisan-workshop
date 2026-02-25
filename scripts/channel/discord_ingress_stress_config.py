#!/usr/bin/env python3
"""CLI config helpers for Discord ingress stress probe."""

from __future__ import annotations

from discord_ingress_stress_config_args import parse_args
from discord_ingress_stress_config_build import build_config, dedup_non_empty
from discord_ingress_stress_config_urls import (
    default_ingress_url,
    normalize_ingress_bind_for_local_url,
)

__all__ = [
    "build_config",
    "dedup_non_empty",
    "default_ingress_url",
    "normalize_ingress_bind_for_local_url",
    "parse_args",
]
