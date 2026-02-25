#!/usr/bin/env python3
"""Validation/build helpers for acceptance runner config."""

from __future__ import annotations

import os
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse

    from acceptance_runner_models import AcceptanceConfig


def parse_optional_env_int(name: str) -> int | None:
    """Parse optional integer from env var."""
    raw = os.environ.get(name, "").strip()
    if not raw:
        return None
    try:
        return int(raw)
    except ValueError as error:
        raise ValueError(f"{name} must be an integer, got '{raw}'.") from error


def validate_args(args: argparse.Namespace) -> None:
    """Validate numeric bounds for acceptance runner args."""
    if args.max_wait <= 0:
        raise ValueError("--max-wait must be positive")
    if args.max_idle_secs <= 0:
        raise ValueError("--max-idle-secs must be positive")
    if args.evolution_max_wait <= 0:
        raise ValueError("--evolution-max-wait must be positive")
    if args.evolution_max_idle_secs <= 0:
        raise ValueError("--evolution-max-idle-secs must be positive")
    if args.evolution_max_parallel <= 0:
        raise ValueError("--evolution-max-parallel must be positive")
    if args.retries <= 0:
        raise ValueError("--retries must be positive")


def resolve_thread_ids(
    args: argparse.Namespace,
    *,
    parse_optional_env_int_fn: object = parse_optional_env_int,
) -> tuple[int | None, int | None]:
    """Resolve acceptance thread pair from args/env defaults."""
    parse_env = parse_optional_env_int_fn  # local alias for typing simplicity
    assert callable(parse_env)

    group_thread_id = args.group_thread_id
    if group_thread_id is None:
        group_thread_id = parse_env("OMNI_TEST_GROUP_THREAD_ID")

    group_thread_id_b = args.group_thread_id_b
    if group_thread_id_b is None:
        group_thread_id_b = parse_env("OMNI_TEST_GROUP_THREAD_B")
    if group_thread_id_b is None and group_thread_id is not None:
        group_thread_id_b = group_thread_id + 1
    if (
        group_thread_id is not None
        and group_thread_id_b is not None
        and int(group_thread_id) == int(group_thread_id_b)
    ):
        raise ValueError(
            "group thread acceptance checks require distinct thread ids; "
            f"got both={group_thread_id}."
        )
    return group_thread_id, group_thread_id_b


def build_config(
    args: argparse.Namespace,
    *,
    config_cls: type[AcceptanceConfig],
    validate_args_fn: object = validate_args,
    resolve_thread_ids_fn: object = resolve_thread_ids,
) -> AcceptanceConfig:
    """Build validated acceptance runner config."""
    validate_callable = validate_args_fn
    thread_callable = resolve_thread_ids_fn
    assert callable(validate_callable)
    assert callable(thread_callable)

    validate_callable(args)
    group_thread_id, group_thread_id_b = thread_callable(args)
    return config_cls(
        titles=args.titles.strip(),
        log_file=Path(args.log_file),
        output_json=Path(args.output_json),
        output_markdown=Path(args.output_markdown),
        group_profile_json=Path(args.group_profile_json),
        group_profile_env=Path(args.group_profile_env),
        max_wait=int(args.max_wait),
        max_idle_secs=int(args.max_idle_secs),
        group_thread_id=group_thread_id,
        group_thread_id_b=group_thread_id_b,
        evolution_max_wait=int(args.evolution_max_wait),
        evolution_max_idle_secs=int(args.evolution_max_idle_secs),
        evolution_max_parallel=int(args.evolution_max_parallel),
        retries=int(args.retries),
    )
