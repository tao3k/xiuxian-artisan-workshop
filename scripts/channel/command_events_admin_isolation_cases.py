#!/usr/bin/env python3
"""Case builders for command-events admin isolation probes."""

from __future__ import annotations

from typing import Any


def build_admin_list_isolation_case(
    *,
    make_probe_case: Any,
    chat_id: int,
    admin_user_id: int | None,
    thread_id: int | None,
    expected_override_count: int,
) -> Any:
    """Build recipient-isolation list probe case."""
    return make_probe_case(
        case_id=f"session_admin_list_json_isolation_{chat_id}_{expected_override_count}",
        prompt="/session admin list json",
        event_name="telegram.command.session_admin_json.replied",
        suites=("admin",),
        user_id=admin_user_id,
        chat_id=chat_id,
        thread_id=thread_id,
        extra_args=(
            "--expect-reply-json-field",
            "json_kind=session_admin",
            "--expect-reply-json-field",
            f"json_override_admin_count={expected_override_count}",
        ),
    )


def build_admin_list_topic_isolation_case(
    *,
    make_probe_case: Any,
    chat_id: int,
    admin_user_id: int | None,
    thread_id: int,
    expected_override_count: int,
) -> Any:
    """Build topic-isolation list probe case."""
    return make_probe_case(
        case_id=(
            "session_admin_list_json_topic_isolation_"
            f"{chat_id}_{thread_id}_{expected_override_count}"
        ),
        prompt="/session admin list json",
        event_name="telegram.command.session_admin_json.replied",
        suites=("admin",),
        user_id=admin_user_id,
        chat_id=chat_id,
        thread_id=thread_id,
        extra_args=(
            "--expect-reply-json-field",
            "json_kind=session_admin",
            "--expect-reply-json-field",
            f"json_override_admin_count={expected_override_count}",
        ),
    )
