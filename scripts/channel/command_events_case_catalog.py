#!/usr/bin/env python3
"""Case catalog helpers for command-events probes."""

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


def select_cases(
    cases: tuple[Any, ...],
    suites: tuple[str, ...],
    case_ids: tuple[str, ...],
) -> list[Any]:
    """Filter cases by explicit ids or suite selection."""
    if case_ids:
        requested = set(case_ids)
        selected = [case for case in cases if case.case_id in requested]
        missing = sorted(requested - {case.case_id for case in selected})
        if missing:
            raise ValueError(
                "Unknown case id(s): "
                + ", ".join(missing)
                + ". Use --list-cases to inspect available ids."
            )
        return selected

    suite_set = set(suites)
    if "all" in suite_set:
        return list(cases)
    return [case for case in cases if any(suite in suite_set for suite in case.suites)]
