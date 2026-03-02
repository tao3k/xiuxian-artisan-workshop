#!/usr/bin/env python3
"""Case builder for command-events probe catalog."""

from __future__ import annotations

from typing import Any


def build_cases(
    admin_user_id: int | None,
    group_chat_id: int | None,
    group_thread_id: int | None,
    *,
    make_probe_case: Any,
    target_session_scope_placeholder: str,
) -> tuple[Any, ...]:
    """Build default command-event probe cases."""
    delegated_admin_target = 1001
    return (
        make_probe_case(
            case_id="session_status_json",
            prompt="/session json",
            event_name="telegram.command.session_status_json.replied",
            suites=("core",),
            extra_args=("--expect-reply-json-field", "json_kind=session_context"),
        ),
        make_probe_case(
            case_id="session_budget_json",
            prompt="/session budget json",
            event_name="telegram.command.session_budget_json.replied",
            suites=("core",),
            extra_args=("--expect-reply-json-field", "json_kind=session_budget"),
        ),
        make_probe_case(
            case_id="session_memory_json",
            prompt="/session memory json",
            event_name="telegram.command.session_memory_json.replied",
            suites=("core",),
            extra_args=(
                "--expect-reply-json-field",
                "json_kind=session_memory",
                "--expect-reply-json-field",
                f"json_session_scope={target_session_scope_placeholder}",
            ),
        ),
        make_probe_case(
            case_id="session_feedback_up_json",
            prompt="/session feedback up json",
            event_name="telegram.command.session_feedback_json.replied",
            suites=("core",),
        ),
        make_probe_case(
            case_id="agenda_view_markdown",
            prompt="/agenda",
            event_name="telegram.zhixing.sync.completed",
            suites=("core",),
            max_wait_secs=120,
            max_idle_secs=60,
            extra_args=("--expect-bot-regex", r"(?i)agenda"),
        ),
        make_probe_case(
            case_id="session_reset",
            prompt="/reset",
            event_name="telegram.command.session_reset.replied",
            suites=("control",),
            user_id=admin_user_id,
        ),
        make_probe_case(
            case_id="session_resume_status",
            prompt="/resume status",
            event_name="telegram.command.session_resume_status.replied",
            suites=("control",),
        ),
        make_probe_case(
            case_id="session_resume_drop",
            prompt="/resume drop",
            event_name="telegram.command.session_resume_drop.replied",
            suites=("control",),
            user_id=admin_user_id,
        ),
        make_probe_case(
            case_id="session_reset_snapshot_state",
            prompt="/reset",
            event_name="telegram.command.session_reset.replied",
            suites=("control",),
            user_id=admin_user_id,
            extra_args=(
                "--expect-event",
                "telegram.command.session_reset.snapshot_state",
                "--expect-log-regex",
                'snapshot_state="?none"?',
            ),
        ),
        make_probe_case(
            case_id="session_admin_add",
            prompt=f"/session admin add {delegated_admin_target}",
            event_name="telegram.command.session_admin.replied",
            suites=("admin",),
            user_id=admin_user_id,
            chat_id=group_chat_id,
            thread_id=group_thread_id,
        ),
        make_probe_case(
            case_id="session_admin_list_json",
            prompt="/session admin list json",
            event_name="telegram.command.session_admin_json.replied",
            suites=("admin",),
            user_id=admin_user_id,
            chat_id=group_chat_id,
            thread_id=group_thread_id,
            extra_args=("--expect-reply-json-field", "json_kind=session_admin"),
        ),
        make_probe_case(
            case_id="session_admin_clear",
            prompt="/session admin clear",
            event_name="telegram.command.session_admin.replied",
            suites=("admin",),
            user_id=admin_user_id,
            chat_id=group_chat_id,
            thread_id=group_thread_id,
        ),
    )
