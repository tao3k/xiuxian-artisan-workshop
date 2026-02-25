#!/usr/bin/env python3
"""Config construction and validation for Discord ingress stress probe."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from path_resolver import resolve_path

if TYPE_CHECKING:
    import argparse

    from discord_ingress_stress_models import StressConfig


def dedup_non_empty(values: list[str]) -> tuple[str, ...]:
    """Deduplicate non-empty tokens while preserving order."""
    ordered: list[str] = []
    for value in values:
        token = value.strip()
        if not token:
            continue
        if token not in ordered:
            ordered.append(token)
    return tuple(ordered)


def build_config(
    args: argparse.Namespace,
    *,
    config_cls: type[StressConfig],
) -> StressConfig:
    """Validate CLI args and build typed stress config."""
    if args.rounds <= 0:
        raise ValueError("--rounds must be positive.")
    if args.warmup_rounds < 0:
        raise ValueError("--warmup-rounds must be >= 0.")
    if args.parallel <= 0:
        raise ValueError("--parallel must be positive.")
    if args.requests_per_worker <= 0:
        raise ValueError("--requests-per-worker must be positive.")
    if args.timeout_secs <= 0:
        raise ValueError("--timeout-secs must be positive.")
    if args.cooldown_secs < 0:
        raise ValueError("--cooldown-secs must be >= 0.")
    if args.quality_max_failure_rate < 0 or args.quality_max_failure_rate > 1:
        raise ValueError("--quality-max-failure-rate must be in range [0,1].")

    ingress_url = args.ingress_url.strip()
    if not ingress_url:
        raise ValueError("--ingress-url must be non-empty.")
    channel_id = args.channel_id.strip()
    user_id = args.user_id.strip()
    if not channel_id:
        raise ValueError("--channel-id is required.")
    if not user_id:
        raise ValueError("--user-id is required.")

    project_root = resolve_path(args.project_root, Path.cwd())
    output_json = resolve_path(args.output_json, project_root)
    output_markdown = resolve_path(args.output_markdown, project_root)
    log_file = resolve_path(args.log_file, project_root)

    quality_max_p95_ms = float(args.quality_max_p95_ms)
    quality_min_rps = float(args.quality_min_rps)

    return config_cls(
        rounds=int(args.rounds),
        warmup_rounds=int(args.warmup_rounds),
        parallel=int(args.parallel),
        requests_per_worker=int(args.requests_per_worker),
        timeout_secs=float(args.timeout_secs),
        cooldown_secs=float(args.cooldown_secs),
        ingress_url=ingress_url,
        log_file=log_file,
        secret_token=args.secret_token.strip() or None,
        channel_id=channel_id,
        user_id=user_id,
        guild_id=args.guild_id.strip() or None,
        username=args.username.strip() or None,
        role_ids=dedup_non_empty(args.role_id),
        prompt=args.prompt,
        output_json=output_json,
        output_markdown=output_markdown,
        quality_max_failure_rate=float(args.quality_max_failure_rate),
        quality_max_p95_ms=quality_max_p95_ms if quality_max_p95_ms > 0 else None,
        quality_min_rps=quality_min_rps if quality_min_rps > 0 else None,
    )
