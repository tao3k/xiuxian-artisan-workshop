#!/usr/bin/env python3
"""Case catalog/filter helpers for Discord ACL probes."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from discord_acl_events_models import ProbeCase


def build_cases(user_id: str, *, case_cls: type[ProbeCase]) -> list[ProbeCase]:
    """Build default case set for a target user."""
    return [
        case_cls(
            case_id="discord_control_admin_denied",
            prompt=f"/session admin add {user_id}",
            event_name="discord.command.control_admin_required.replied",
            suites=("core",),
        ),
        case_cls(
            case_id="discord_slash_permission_denied",
            prompt="/session memory",
            event_name="discord.command.slash_permission_required.replied",
            suites=("core",),
        ),
    ]


def filter_cases(
    cases: list[ProbeCase],
    suites: tuple[str, ...],
    requested_case_ids: tuple[str, ...],
) -> list[ProbeCase]:
    """Filter cases by suite and explicit case id selections."""
    result: list[ProbeCase] = []
    for case in cases:
        if requested_case_ids and case.case_id not in requested_case_ids:
            continue
        if "all" not in suites and not any(suite in suites for suite in case.suites):
            continue
        result.append(case)
    return result


def list_cases(cases: list[ProbeCase]) -> int:
    """Print case listing and return success code."""
    print("Available Discord ACL cases:")
    for case in cases:
        print(f"- {case.case_id} ({','.join(case.suites)}) -> {case.prompt}")
    return 0
