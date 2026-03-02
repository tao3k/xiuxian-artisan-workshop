#!/usr/bin/env python3
"""Expectation token parsing helpers for blackbox runtime logs."""

from __future__ import annotations


def parse_expected_field(value: str) -> tuple[str, str]:
    """Parse `key=value` expectation token."""
    if "=" not in value:
        raise ValueError(
            f"Invalid --expect-reply-json-field value '{value}'. Expected format: key=value"
        )
    key, expected = value.split("=", 1)
    key = key.strip()
    expected = expected.strip()
    if not key or expected == "":
        raise ValueError(
            f"Invalid --expect-reply-json-field value '{value}'. Expected format: key=value"
        )
    return key, expected


def parse_allow_chat_ids(values: list[str]) -> tuple[int, ...]:
    """Parse and de-duplicate allowlisted chat ids."""
    ordered: list[int] = []
    for raw in values:
        token = raw.strip()
        if not token:
            continue
        try:
            chat_id = int(token)
        except ValueError as error:
            raise ValueError(
                f"Invalid chat id '{raw}' in allowlist. Expected integer Telegram chat id."
            ) from error
        if chat_id not in ordered:
            ordered.append(chat_id)
    return tuple(ordered)
