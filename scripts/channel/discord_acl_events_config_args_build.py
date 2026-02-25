#!/usr/bin/env python3
"""Config construction helpers for Discord ACL probes."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse

    from discord_acl_events_models import ProbeConfig


def normalize_partition_mode(value: str) -> str:
    """Normalize user input into supported Discord session partition modes."""
    token = value.strip().lower().replace("-", "_")
    if token in {"guild_channel_user", "channel_user", "guildchanneluser"}:
        return "guild_channel_user"
    if token in {"channel", "channel_only", "channelonly"}:
        return "channel"
    if token in {"user", "user_only", "useronly"}:
        return "user"
    if token in {"guild_user", "guilduser"}:
        return "guild_user"
    raise ValueError(
        "invalid --session-partition; expected guild_channel_user|channel|user|guild_user"
    )


def dedup(values: list[str]) -> tuple[str, ...]:
    """Deduplicate tokens while preserving original order."""
    ordered: list[str] = []
    for value in values:
        token = value.strip()
        if not token:
            continue
        if token not in ordered:
            ordered.append(token)
    return tuple(ordered)


def build_config(args: argparse.Namespace, *, config_cls: type[ProbeConfig]) -> ProbeConfig:
    """Build validated probe config from CLI args."""
    channel_id = args.channel_id.strip()
    user_id = args.user_id.strip()
    if not channel_id or not user_id:
        raise ValueError(
            "--channel-id and --user-id are required (or set OMNI_TEST_DISCORD_CHANNEL_ID "
            "and OMNI_TEST_DISCORD_USER_ID)."
        )
    return config_cls(
        ingress_url=args.ingress_url,
        log_file=Path(args.log_file),
        max_wait_secs=args.max_wait,
        max_idle_secs=args.max_idle_secs,
        channel_id=channel_id,
        user_id=user_id,
        guild_id=(
            args.guild_id.strip() if isinstance(args.guild_id, str) and args.guild_id else None
        ),
        username=args.username,
        role_ids=dedup(args.role_id),
        secret_token=args.secret_token,
        session_partition=normalize_partition_mode(args.session_partition),
        no_follow=bool(args.no_follow),
    )


def selected_suites(args: argparse.Namespace) -> tuple[str, ...]:
    """Resolve enabled suite set from CLI args."""
    if not args.suite:
        return ("all",)
    ordered = dedup(args.suite)
    if "all" in ordered:
        return ("all",)
    return ordered
