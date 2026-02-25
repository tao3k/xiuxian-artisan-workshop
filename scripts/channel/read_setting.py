#!/usr/bin/env python3
"""Read arbitrary settings value via unified settings loader."""

from __future__ import annotations

import argparse

from omni.foundation.config.settings import get_setting


def read_setting(key: str) -> str:
    value = get_setting(key)
    if value is None:
        return ""
    return str(value).strip()


def main() -> int:
    parser = argparse.ArgumentParser(description="Read setting value")
    parser.add_argument(
        "--key", required=True, help="dot-path setting key (for example: gateway.bind)"
    )
    args = parser.parse_args()
    print(read_setting(str(args.key)), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
