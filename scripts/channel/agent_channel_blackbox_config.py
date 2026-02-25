#!/usr/bin/env python3
"""Argument parsing and config assembly for agent channel blackbox probe."""

from __future__ import annotations

from agent_channel_blackbox_config_args import parse_args
from agent_channel_blackbox_config_build import build_config
from agent_channel_blackbox_config_payload import build_probe_message, build_update_payload

__all__ = [
    "build_config",
    "build_probe_message",
    "build_update_payload",
    "parse_args",
]
