#!/usr/bin/env python3
"""Runtime argument groups for session matrix parser."""

from __future__ import annotations

from typing import Any

import session_matrix_config_args_env as _env


def add_runtime_args(parser: Any, *, webhook_url_default: str) -> None:
    """Add runtime/wait/log arguments."""
    parser.add_argument(
        "--max-wait",
        type=int,
        default=_env.default_wait_secs(),
        help="Max wait per probe in seconds (default: 35).",
    )
    parser.add_argument(
        "--max-idle-secs",
        type=int,
        default=_env.default_idle_secs(),
        help="Max idle seconds per probe (default: 25).",
    )
    parser.add_argument(
        "--webhook-url",
        default=webhook_url_default,
        help="Webhook URL.",
    )
    parser.add_argument(
        "--log-file",
        default=_env.default_log_file(),
        help="Runtime log file path.",
    )
