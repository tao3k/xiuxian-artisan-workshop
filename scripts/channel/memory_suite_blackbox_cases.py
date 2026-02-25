#!/usr/bin/env python3
"""Case definitions for memory-suite black-box probes."""

from __future__ import annotations

from dataclasses import dataclass

TARGET_SESSION_SCOPE_PLACEHOLDER = "__target_session_scope__"


@dataclass(frozen=True)
class BlackboxCase:
    """One black-box probe case definition."""

    prompt: str
    expected_event: str
    extra_args: tuple[str, ...] = ()


def blackbox_cases(
    require_live_turn: bool,
    *,
    case_cls: type[BlackboxCase] = BlackboxCase,
    target_session_scope_placeholder: str = TARGET_SESSION_SCOPE_PLACEHOLDER,
) -> tuple[BlackboxCase, ...]:
    """Build black-box probes for memory/session command and optional live turn checks."""
    base = (
        case_cls(
            prompt="/session memory json",
            expected_event="telegram.command.session_memory_json.replied",
            extra_args=(
                "--expect-reply-json-field",
                "json_kind=session_memory",
                "--expect-reply-json-field",
                f"json_session_scope={target_session_scope_placeholder}",
            ),
        ),
        case_cls(
            prompt="/session feedback up json",
            expected_event="telegram.command.session_feedback_json.replied",
            extra_args=("--expect-reply-json-field", "json_kind=session_feedback"),
        ),
        case_cls(
            prompt="/session feedback down json",
            expected_event="telegram.command.session_feedback_json.replied",
            extra_args=("--expect-reply-json-field", "json_kind=session_feedback"),
        ),
    )
    if not require_live_turn:
        return base
    return (
        *base,
        case_cls(
            prompt="Please reply in one short sentence for memory probe.",
            expected_event="agent.memory.recall.planned",
            extra_args=(
                "--expect-log-regex",
                r"agent\.memory\.recall\.(planned|injected|skipped)",
            ),
        ),
    )
