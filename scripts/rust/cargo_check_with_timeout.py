#!/usr/bin/env python3
"""Run `cargo check --workspace --all-targets` with a hard timeout."""

from __future__ import annotations

import subprocess
import sys

from omni.foundation.runtime.cargo_subprocess_env import prepare_cargo_subprocess_env


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: cargo_check_with_timeout.py <timeout-seconds>", file=sys.stderr)
        return 2

    timeout_secs = int(sys.argv[1])
    command = ["cargo", "check", "--workspace", "--all-targets"]
    env = prepare_cargo_subprocess_env()

    try:
        subprocess.run(command, check=True, timeout=timeout_secs, env=env)
    except subprocess.TimeoutExpired:
        print(
            f"ERROR: cargo check exceeded timeout ({timeout_secs}s). "
            "Set a larger timeout if needed: `just rust-check <seconds>`.",
            file=sys.stderr,
        )
        return 124
    except subprocess.CalledProcessError as error:
        return error.returncode

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
