#!/usr/bin/env python3
"""Capture Telegram test-group IDs from webhook logs and persist profile files."""

from __future__ import annotations

from pathlib import Path

from capture_telegram_group_profile_config import parse_args
from capture_telegram_group_profile_discovery import discover_groups
from capture_telegram_group_profile_output import write_profile


def main() -> int:
    """Run capture flow and emit user-readable summary."""
    args = parse_args()
    titles = [item.strip() for item in args.titles.split(",") if item.strip()]
    if len(titles) < 1:
        raise ValueError("--titles must include at least one title.")

    log_file = Path(args.log_file)
    if not log_file.exists():
        raise FileNotFoundError(f"log file not found: {log_file}")

    discovered = discover_groups(log_file=log_file, targets=titles)
    missing = [title for title in titles if title not in discovered]
    if missing and not args.allow_missing:
        missing_joined = ", ".join(missing)
        raise RuntimeError(
            f"missing group titles in log: {missing_joined}. "
            "Send '/help' (or any message) in each target group and retry."
        )

    output_json = Path(args.output_json)
    output_env = Path(args.output_env)
    write_profile(
        output_json=output_json,
        output_env=output_env,
        ordered_titles=titles,
        discovered=discovered,
        user_id_override=args.user_id,
    )

    present = [title for title in titles if title in discovered]
    print("Captured Telegram test-group profile.")
    print(f"  present_titles={present}")
    if missing:
        print(f"  missing_titles={missing}")
    for title in present:
        obs = discovered[title]
        print(f"  title={title} chat_id={obs.chat_id} chat_type={obs.chat_type}")
    print(f"  output_json={output_json}")
    print(f"  output_env={output_env}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
