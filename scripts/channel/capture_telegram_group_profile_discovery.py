#!/usr/bin/env python3
"""Discovery helpers for Telegram group profile capture."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from capture_telegram_group_profile_config import normalize_title, parse_user_id
from capture_telegram_group_profile_models import (
    ANSI_ESCAPE_RE,
    PARSED_MESSAGE_RE,
    GroupObservation,
)
from log_io import iter_log_lines

if TYPE_CHECKING:
    from pathlib import Path


def discover_groups(
    log_file: Path,
    targets: list[str],
    *,
    normalize_title_fn: Any = normalize_title,
    parse_user_id_fn: Any = parse_user_id,
    iter_log_lines_fn: Any = iter_log_lines,
) -> dict[str, GroupObservation]:
    """Discover target groups from runtime logs keyed by canonical requested title."""
    normalized_targets = {normalize_title_fn(title): title for title in targets}
    found: dict[str, GroupObservation] = {}

    for idx, raw_line in enumerate(iter_log_lines_fn(log_file)):
        line = ANSI_ESCAPE_RE.sub("", raw_line)
        match = PARSED_MESSAGE_RE.search(line)
        if not match:
            continue

        chat_type = match.group("chat_type")
        if chat_type not in {"group", "supergroup"}:
            continue

        raw_title = (match.group("chat_title") or "").strip()
        if not raw_title:
            continue

        normalized = normalize_title_fn(raw_title)
        if normalized not in normalized_targets:
            continue

        canonical_title = normalized_targets[normalized]
        found[canonical_title] = GroupObservation(
            title=raw_title,
            chat_id=int(match.group("chat_id")),
            chat_type=chat_type,
            user_id=parse_user_id_fn(match.group("session_key")),
            line_index=idx,
        )

    return found
