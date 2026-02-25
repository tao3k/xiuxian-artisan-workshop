#!/usr/bin/env python3
"""
Memory-focused validation suite for omni-agent Telegram channel/runtime.

This script combines:
  1) command-level black-box checks (webhook/runtime path)
  2) targeted Rust regression checks for memory behaviors
  3) optional Valkey cross-instance memory continuity check
"""

from __future__ import annotations

import importlib
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

load_sibling_module = importlib.import_module("module_loader").load_sibling_module

_resolver_module = load_sibling_module(
    module_name="config_resolver",
    file_name="config_resolver.py",
    caller_file=__file__,
    error_context="resolver module",
)
normalize_telegram_session_partition_mode = (
    _resolver_module.normalize_telegram_session_partition_mode
)
session_partition_mode_from_runtime_log = _resolver_module.session_partition_mode_from_runtime_log
telegram_session_partition_mode = _resolver_module.telegram_session_partition_mode

_regressions_module = load_sibling_module(
    module_name="memory_suite_regressions",
    file_name="memory_suite_regressions.py",
    caller_file=__file__,
    error_context="memory suite regressions helpers",
)
_blackbox_module = load_sibling_module(
    module_name="memory_suite_blackbox",
    file_name="memory_suite_blackbox.py",
    caller_file=__file__,
    error_context="memory suite black-box helpers",
)
_cli_module = load_sibling_module(
    module_name="memory_suite_cli",
    file_name="memory_suite_cli.py",
    caller_file=__file__,
    error_context="memory suite CLI helpers",
)

DEFAULT_MAX_WAIT = int(os.environ.get("OMNI_BLACKBOX_MAX_WAIT_SECS", "25"))
DEFAULT_MAX_IDLE_SECS = int(os.environ.get("OMNI_BLACKBOX_MAX_IDLE_SECS", "25"))
DEFAULT_VALKEY_URL = os.environ.get("VALKEY_URL", "redis://127.0.0.1:6379/0")
FORBIDDEN_LOG_PATTERN = "tools/call: Mcp error"
DEFAULT_EVOLUTION_SCENARIO_ID = "memory_self_correction_high_complexity_dag"
TARGET_SESSION_SCOPE_PLACEHOLDER = _blackbox_module.TARGET_SESSION_SCOPE_PLACEHOLDER
BlackboxCase = _blackbox_module.BlackboxCase


def default_valkey_prefix(tag: str) -> str:
    safe_tag = tag.strip().lower() or "memory-suite"
    return f"omni-agent:session:{safe_tag}:{os.getpid()}:{int(time.time() * 1000)}"


def default_report_path(filename: str) -> Path:
    runtime_root = Path(os.environ.get("PRJ_RUNTIME_DIR", ".run"))
    if not runtime_root.is_absolute():
        project_root = Path(os.environ.get("PRJ_ROOT", Path.cwd()))
        runtime_root = project_root / runtime_root
    return runtime_root / "reports" / filename


def runtime_log_file() -> Path:
    return Path(os.environ.get("OMNI_CHANNEL_LOG_FILE", ".run/logs/omni-agent-webhook.log"))


def resolve_runtime_partition_mode() -> str | None:
    override = os.environ.get("OMNI_BLACKBOX_SESSION_PARTITION_MODE", "").strip()
    normalized_override = normalize_telegram_session_partition_mode(override)
    if normalized_override:
        return normalized_override

    mode_from_log = session_partition_mode_from_runtime_log(runtime_log_file())
    if mode_from_log:
        return mode_from_log

    return telegram_session_partition_mode()


def parse_args() -> argparse.Namespace:
    return _cli_module.parse_args(
        default_max_wait=DEFAULT_MAX_WAIT,
        default_max_idle_secs=DEFAULT_MAX_IDLE_SECS,
        default_valkey_url=DEFAULT_VALKEY_URL,
        default_evolution_dataset=str(
            Path(__file__).resolve().parent / "fixtures" / "memory_evolution_complex_scenarios.json"
        ),
        default_evolution_scenario_id=DEFAULT_EVOLUTION_SCENARIO_ID,
        default_evolution_output_json=str(default_report_path("omni-agent-memory-evolution.json")),
        default_evolution_output_markdown=str(
            default_report_path("omni-agent-memory-evolution.md")
        ),
    )


def run_command(
    cmd: list[str],
    *,
    title: str,
    env: dict[str, str] | None = None,
) -> None:
    print()
    print(f">>> {title}", flush=True)
    print("+ " + " ".join(cmd), flush=True)
    subprocess.run(cmd, check=True, env=env)


def blackbox_cases(require_live_turn: bool) -> tuple[BlackboxCase, ...]:
    return _blackbox_module.blackbox_cases(
        require_live_turn,
        case_cls=BlackboxCase,
        target_session_scope_placeholder=TARGET_SESSION_SCOPE_PLACEHOLDER,
    )


def run_blackbox_suite(
    script_dir: Path,
    *,
    max_wait: int,
    max_idle_secs: int,
    username: str,
    require_live_turn: bool,
) -> None:
    _blackbox_module.run_blackbox_suite(
        script_dir,
        max_wait=max_wait,
        max_idle_secs=max_idle_secs,
        username=username,
        require_live_turn=require_live_turn,
        forbidden_log_pattern=FORBIDDEN_LOG_PATTERN,
        run_command_fn=run_command,
        resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode,
        blackbox_cases_fn=blackbox_cases,
    )


def run_memory_evolution_scenario(
    script_dir: Path,
    *,
    max_wait: int,
    max_idle_secs: int,
    username: str,
    dataset_path: Path,
    scenario_id: str,
    max_parallel: int,
    output_json: Path,
    output_markdown: Path,
) -> None:
    _blackbox_module.run_memory_evolution_scenario(
        script_dir,
        max_wait=max_wait,
        max_idle_secs=max_idle_secs,
        username=username,
        dataset_path=dataset_path,
        scenario_id=scenario_id,
        max_parallel=max_parallel,
        output_json=output_json,
        output_markdown=output_markdown,
        run_command_fn=run_command,
    )


def run_rust_memory_regressions() -> None:
    _regressions_module.run_rust_memory_regressions(run_command_fn=run_command)


def ensure_valkey_cli() -> None:
    _regressions_module.ensure_valkey_cli()


def check_valkey_connectivity(valkey_url: str) -> None:
    _regressions_module.check_valkey_connectivity(valkey_url)


def run_valkey_cross_instance_regression(valkey_url: str, valkey_prefix: str) -> None:
    _regressions_module.run_valkey_cross_instance_regression(
        valkey_url,
        valkey_prefix,
        run_command_fn=run_command,
    )


def main() -> int:
    args = parse_args()
    if args.max_wait <= 0:
        print("Error: --max-wait must be a positive integer.", file=sys.stderr)
        return 2
    if args.max_idle_secs <= 0:
        print("Error: --max-idle-secs must be a positive integer.", file=sys.stderr)
        return 2
    if args.evolution_max_parallel <= 0:
        print("Error: --evolution-max-parallel must be a positive integer.", file=sys.stderr)
        return 2

    script_dir = Path(__file__).resolve().parent
    try:
        if not args.skip_blackbox:
            print("Running memory black-box probes...", flush=True)
            run_blackbox_suite(
                script_dir=script_dir,
                max_wait=args.max_wait,
                max_idle_secs=args.max_idle_secs,
                username=args.username,
                require_live_turn=args.require_live_turn,
            )
            if args.suite == "full" and not args.skip_evolution:
                print()
                print("Running memory self-evolution DAG scenario...", flush=True)
                run_memory_evolution_scenario(
                    script_dir=script_dir,
                    max_wait=args.max_wait,
                    max_idle_secs=args.max_idle_secs,
                    username=args.username,
                    dataset_path=Path(args.evolution_dataset).expanduser().resolve(),
                    scenario_id=args.evolution_scenario.strip(),
                    max_parallel=args.evolution_max_parallel,
                    output_json=Path(args.evolution_output_json).expanduser().resolve(),
                    output_markdown=Path(args.evolution_output_markdown).expanduser().resolve(),
                )
        if args.suite == "full" and not args.skip_rust:
            print()
            print("Running memory Rust regressions...", flush=True)
            run_rust_memory_regressions()
        if args.with_valkey:
            print()
            print("Running optional Valkey memory continuity regression...", flush=True)
            valkey_prefix = args.valkey_prefix.strip() or default_valkey_prefix("memory-suite")
            run_valkey_cross_instance_regression(args.valkey_url, valkey_prefix)
        print()
        print("Memory suite passed.", flush=True)
        return 0
    except (subprocess.CalledProcessError, FileNotFoundError, RuntimeError) as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
