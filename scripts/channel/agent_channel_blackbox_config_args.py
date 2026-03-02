#!/usr/bin/env python3
"""CLI parser for agent channel blackbox config."""

from __future__ import annotations

import argparse
import os
from typing import Any

from agent_channel_blackbox_config_args_core import add_core_args
from agent_channel_blackbox_config_args_expectations import add_expectation_args


def parse_args(
    *,
    default_telegram_webhook_url_fn: Any,
    target_session_scope_placeholder: str,
) -> argparse.Namespace:
    """Parse blackbox probe CLI arguments."""
    webhook_url_default = os.environ.get("OMNI_WEBHOOK_URL") or default_telegram_webhook_url_fn()
    parser = argparse.ArgumentParser(
        description="Inject one synthetic Telegram webhook update and wait for bot reply logs."
    )
    add_core_args(parser, webhook_url_default=webhook_url_default)
    add_expectation_args(
        parser,
        target_session_scope_placeholder=target_session_scope_placeholder,
    )
    return parser.parse_args()
