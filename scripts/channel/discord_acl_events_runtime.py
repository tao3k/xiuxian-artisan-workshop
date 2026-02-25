#!/usr/bin/env python3
"""Runtime helpers for Discord ACL black-box probes."""

from __future__ import annotations

import importlib
import time
from typing import Any

_http_module = importlib.import_module("discord_acl_events_runtime_http")
_session_module = importlib.import_module("discord_acl_events_runtime_session")
_case_module = importlib.import_module("discord_acl_events_runtime_case")

now_event_id = _http_module.now_event_id
build_ingress_payload = _http_module.build_ingress_payload
post_ingress_event = _http_module.post_ingress_event
compile_patterns = _http_module.compile_patterns

expected_session_keys = _session_module.expected_session_keys
expected_session_scopes = _session_module.expected_session_scopes
reply_json_field_matches = _session_module.reply_json_field_matches


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
    forbidden_log_pattern: str,
    error_patterns: tuple[str, ...],
    target_session_scope_placeholder: str,
    session_scope_prefix: str,
    monotonic_fn: Any = time.monotonic,
    sleep_fn: Any = time.sleep,
) -> int:
    """Execute one Discord ACL probe case against runtime logs."""
    return _case_module.run_case(
        config,
        case,
        blackbox=blackbox,
        parse_expected_field_fn=parse_expected_field_fn,
        expected_session_keys_fn=expected_session_keys_fn,
        expected_session_scopes_fn=expected_session_scopes_fn,
        now_event_id_fn=now_event_id_fn,
        build_ingress_payload_fn=build_ingress_payload_fn,
        post_ingress_event_fn=post_ingress_event_fn,
        compile_patterns_fn=compile_patterns_fn,
        reply_json_field_matches_fn=reply_json_field_matches,
        forbidden_log_pattern=forbidden_log_pattern,
        error_patterns=error_patterns,
        target_session_scope_placeholder=target_session_scope_placeholder,
        session_scope_prefix=session_scope_prefix,
        monotonic_fn=monotonic_fn,
        sleep_fn=sleep_fn,
    )
