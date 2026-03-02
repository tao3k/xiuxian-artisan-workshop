#!/usr/bin/env python3
"""Identity/payload argument groups for Discord ingress stress parser."""

from __future__ import annotations

from typing import Any

import discord_ingress_stress_config_args_env as _env


def add_identity_args(parser: Any) -> None:
    """Add synthetic identity and payload arguments."""
    parser.add_argument(
        "--secret-token",
        default=_env.default_secret_token(),
        help="Optional ingress secret token for header x-omni-discord-ingress-token.",
    )
    parser.add_argument(
        "--channel-id",
        default=_env.default_channel_id(),
        help="Synthetic Discord channel_id.",
    )
    parser.add_argument(
        "--user-id",
        default=_env.default_user_id(),
        help="Synthetic Discord user_id.",
    )
    parser.add_argument(
        "--guild-id",
        default=_env.default_guild_id(),
        help="Optional synthetic guild_id.",
    )
    parser.add_argument(
        "--username",
        default=_env.default_username(),
        help="Optional synthetic Discord username.",
    )
    parser.add_argument(
        "--role-id",
        action="append",
        default=[],
        help="Optional synthetic member role id (repeatable).",
    )
    parser.add_argument(
        "--prompt",
        default="stress ingress probe",
        help="Synthetic message content sent in ingress event payload.",
    )
