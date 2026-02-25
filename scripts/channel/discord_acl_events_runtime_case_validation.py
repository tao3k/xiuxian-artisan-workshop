#!/usr/bin/env python3
"""Validation helpers for Discord ACL runtime case assertions."""

from __future__ import annotations

import sys


def validate_target_command_reply(
    case_id: str,
    event_name: str,
    expected_recipient: str,
    expected_sessions: tuple[str, ...],
    command_reply_observations: list[dict[str, object]],
) -> tuple[int, str]:
    """Validate target command-reply event belongs to one of expected sessions."""
    target_obs = None
    for observation in command_reply_observations:
        if observation.get("event") != event_name:
            continue
        if str(observation.get("recipient") or "") != expected_recipient:
            continue
        target_obs = observation
        break
    if target_obs is None:
        print(f"[{case_id}] missing target-scoped command reply observation.", file=sys.stderr)
        return 10, ""

    observed_session = str(target_obs.get("session_key") or "")
    if observed_session and observed_session not in expected_sessions:
        print(f"[{case_id}] command reply session_key mismatch.", file=sys.stderr)
        print(f"  expected_session_keys={list(expected_sessions)}", file=sys.stderr)
        print(f"  observed_session_key={observed_session}", file=sys.stderr)
        return 10, observed_session
    return 0, observed_session


def validate_target_json_summary(
    case_id: str,
    event_name: str,
    expected_recipient: str,
    expected_sessions: tuple[str, ...],
    expected_session_scopes_values: tuple[str, ...],
    json_reply_summary_observations: list[dict[str, str]],
) -> tuple[int, str]:
    """Validate optional JSON summary session key/scope for target command reply."""
    target_summary = None
    for summary in json_reply_summary_observations:
        if summary.get("event") != event_name:
            continue
        if str(summary.get("recipient") or "") != expected_recipient:
            continue
        target_summary = summary
        break

    if target_summary is None:
        return 0, ""

    observed_summary_session = str(target_summary.get("session_key") or "")
    if observed_summary_session and observed_summary_session not in expected_sessions:
        print(f"[{case_id}] command reply json summary session_key mismatch.", file=sys.stderr)
        print(f"  expected_session_keys={list(expected_sessions)}", file=sys.stderr)
        print(f"  observed_session_key={observed_summary_session}", file=sys.stderr)
        return 10, ""

    observed_session_scope = str(target_summary.get("json_session_scope") or "")
    if observed_session_scope and observed_session_scope not in expected_session_scopes_values:
        print(f"[{case_id}] command reply json summary session_scope mismatch.", file=sys.stderr)
        print(
            f"  expected_json_session_scopes={list(expected_session_scopes_values)}",
            file=sys.stderr,
        )
        print(f"  observed_json_session_scope={observed_session_scope}", file=sys.stderr)
        return 10, observed_session_scope
    return 0, observed_session_scope
