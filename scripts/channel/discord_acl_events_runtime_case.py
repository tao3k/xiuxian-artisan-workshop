#!/usr/bin/env python3
"""Case execution loop for Discord ACL runtime probes."""

from __future__ import annotations

import sys
import time
from typing import Any

from discord_acl_events_runtime_case_monitor import monitor_case_until_completion
from discord_acl_events_runtime_case_validation import (
    validate_target_command_reply,
    validate_target_json_summary,
)


def _print_ingress_failure(case_id: str, status: int, body: str) -> int:
    print(f"[{case_id}] ingress POST failed: HTTP {status}", file=sys.stderr)
    for line in body.splitlines():
        print(f"  {line}", file=sys.stderr)
    return 1


def run_case(
    config: Any,
    case: Any,
    *,
    blackbox: Any,
    parse_expected_field_fn: Any,
    expected_session_keys_fn: Any,
    expected_session_scopes_fn: Any,
    now_event_id_fn: Any,
    build_ingress_payload_fn: Any,
    post_ingress_event_fn: Any,
    compile_patterns_fn: Any,
    reply_json_field_matches_fn: Any,
    forbidden_log_pattern: str,
    error_patterns: tuple[str, ...],
    target_session_scope_placeholder: str,
    session_scope_prefix: str,
    monotonic_fn: Any = time.monotonic,
    sleep_fn: Any = time.sleep,
) -> int:
    """Execute one Discord ACL probe case against runtime logs."""
    expect_fields = tuple(parse_expected_field_fn(item) for item in case.expect_reply_json_fields)
    expected_sessions = expected_session_keys_fn(
        config.session_partition,
        config.guild_id,
        config.channel_id,
        config.user_id,
    )
    expected_session_scopes_values = expected_session_scopes_fn(
        config.session_partition,
        config.guild_id,
        config.channel_id,
        config.user_id,
        session_scope_prefix=session_scope_prefix,
        expected_session_keys_fn=expected_session_keys_fn,
    )
    expected_recipient = config.channel_id

    event_id = now_event_id_fn()
    payload = build_ingress_payload_fn(config, event_id=event_id, prompt=case.prompt)
    status, body = post_ingress_event_fn(config.ingress_url, payload, config.secret_token)
    if status != 200:
        return _print_ingress_failure(case.case_id, status, body)

    monitor_status, monitor_payload = monitor_case_until_completion(
        config=config,
        case=case,
        blackbox=blackbox,
        expect_fields=expect_fields,
        expected_recipient=expected_recipient,
        expected_session_scopes_values=expected_session_scopes_values,
        compile_patterns_fn=compile_patterns_fn,
        reply_json_field_matches_fn=reply_json_field_matches_fn,
        forbidden_log_pattern=forbidden_log_pattern,
        error_patterns=error_patterns,
        target_session_scope_placeholder=target_session_scope_placeholder,
        monotonic_fn=monotonic_fn,
        sleep_fn=sleep_fn,
    )
    if monitor_status != 0:
        return monitor_status

    session_status, observed_session = validate_target_command_reply(
        case.case_id,
        case.event_name,
        expected_recipient,
        expected_sessions,
        monitor_payload["command_reply_observations"],
    )
    if session_status != 0:
        return session_status

    summary_status, observed_session_scope = validate_target_json_summary(
        case.case_id,
        case.event_name,
        expected_recipient,
        expected_sessions,
        expected_session_scopes_values,
        monitor_payload["json_reply_summary_observations"],
    )
    if summary_status != 0:
        return summary_status

    print(f"[{case.case_id}] pass")
    if monitor_payload["seen_bot"]:
        print(f"  bot_logs={len(monitor_payload['bot_lines'])}")
    print(f"  event={case.event_name}")
    print(f"  session_key={observed_session}")
    if observed_session_scope:
        print(f"  json_session_scope={observed_session_scope}")
    return 0
