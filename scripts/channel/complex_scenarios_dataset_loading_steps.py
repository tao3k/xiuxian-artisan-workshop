#!/usr/bin/env python3
"""Step parsing and dependency validation for complex scenario datasets."""

from __future__ import annotations

from typing import Any

from complex_scenarios_dataset_requirements import required_str_field

_ALLOWED_SESSIONS = {"a", "b", "c"}


def parse_string_tuple_field(
    step_obj: dict[str, object],
    field: str,
    *,
    step_ctx: str,
) -> tuple[str, ...]:
    """Parse optional string-list field into normalized tuple."""
    raw = step_obj.get(field, [])
    if not isinstance(raw, list):
        raise ValueError(f"{step_ctx}: {field} must be an array")
    return tuple(str(item).strip() for item in raw if str(item).strip())


def parse_step(
    *,
    scenario_id: str,
    step_index: int,
    step_obj: dict[str, object],
    seen_step_ids: set[str],
    step_spec_cls: Any,
) -> Any:
    """Parse one scenario step object into typed step spec."""
    step_ctx = f"scenario '{scenario_id}' step[{step_index}]"
    step_id = required_str_field(step_obj, "id", ctx=step_ctx)
    if step_id in seen_step_ids:
        raise ValueError(f"scenario '{scenario_id}' duplicate step id: {step_id}")
    seen_step_ids.add(step_id)

    session_alias = required_str_field(step_obj, "session", ctx=step_ctx).lower()
    if session_alias not in _ALLOWED_SESSIONS:
        raise ValueError(f"{step_ctx}: session must be one of a|b|c")

    prompt = required_str_field(step_obj, "prompt", ctx=step_ctx)
    expect_event_raw = step_obj.get("expect_event")
    expect_event = None if expect_event_raw is None else (str(expect_event_raw).strip() or None)

    expect_reply_json_fields = parse_string_tuple_field(
        step_obj,
        "expect_reply_json_fields",
        step_ctx=step_ctx,
    )
    expect_log_regexes = parse_string_tuple_field(
        step_obj,
        "expect_log_regexes",
        step_ctx=step_ctx,
    )
    expect_bot_regexes = parse_string_tuple_field(
        step_obj,
        "expect_bot_regexes",
        step_ctx=step_ctx,
    )
    forbid_log_regexes = parse_string_tuple_field(
        step_obj,
        "forbid_log_regexes",
        step_ctx=step_ctx,
    )
    depends_on = parse_string_tuple_field(step_obj, "depends_on", step_ctx=step_ctx)
    if step_id in depends_on:
        raise ValueError(f"{step_ctx}: step cannot depend on itself")

    tags_raw = step_obj.get("tags", [])
    if not isinstance(tags_raw, list):
        raise ValueError(f"{step_ctx}: tags must be an array")
    tags = tuple(str(item).strip().lower() for item in tags_raw if str(item).strip())

    return step_spec_cls(
        step_id=step_id,
        session_alias=session_alias,
        prompt=prompt,
        expect_event=expect_event,
        expect_reply_json_fields=expect_reply_json_fields,
        expect_log_regexes=expect_log_regexes,
        expect_bot_regexes=expect_bot_regexes,
        forbid_log_regexes=forbid_log_regexes,
        allow_no_bot=bool(step_obj.get("allow_no_bot", False)),
        tags=tags,
        depends_on=depends_on,
        order=step_index,
    )


def validate_dependencies(*, scenario_id: str, steps: list[Any]) -> None:
    """Ensure all step dependencies reference known step ids."""
    step_ids = {step.step_id for step in steps}
    for step in steps:
        missing = [dep for dep in step.depends_on if dep not in step_ids]
        if missing:
            raise ValueError(
                f"scenario '{scenario_id}' step '{step.step_id}' has unknown dependencies: {missing}"
            )
