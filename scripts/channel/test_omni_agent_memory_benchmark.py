#!/usr/bin/env python3
"""A/B memory benchmark runner for local Telegram webhook runtime."""

from __future__ import annotations

import importlib
import os
import subprocess
import sys
from functools import partial
from pathlib import Path
from typing import Any

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

load_module_bindings = importlib.import_module(
    "memory_benchmark_module_bindings"
).load_module_bindings

_MODULES = load_module_bindings(__file__)
_ENTRY_BINDINGS = _MODULES.entry_bindings_module
_CONFIG = _MODULES.config_module
_MODELS = _MODULES.models_module
_RUNTIME_BINDINGS = _MODULES.runtime_bindings_module
_EXECUTION = _MODULES.execution_module
_SIGNALS = _MODULES.signals_module
_ANALYSIS = _MODULES.analysis_module
_REPORT = _MODULES.report_module
_OUTPUT = _MODULES.output_module
_SharedLogCursor = _MODULES.shared_log_cursor_cls
_shared_init_log_cursor = _MODULES.shared_init_log_cursor
_shared_read_new_log_lines_with_cursor = _MODULES.shared_read_new_log_lines_with_cursor

DEFAULT_MAX_WAIT = int(os.environ.get("OMNI_BLACKBOX_MAX_WAIT_SECS", "40"))
DEFAULT_MAX_IDLE_SECS = int(os.environ.get("OMNI_BLACKBOX_MAX_IDLE_SECS", "30"))
DEFAULT_LOG_FILE = os.environ.get("OMNI_CHANNEL_LOG_FILE", ".run/logs/omni-agent-webhook.log")
FORBIDDEN_LOG_PATTERN = "tools/call: Mcp error"
RESET_EVENT = "telegram.command.session_reset.replied"
FEEDBACK_EVENT = "telegram.command.session_feedback_json.replied"
CONTROL_ADMIN_REQUIRED_EVENT = "telegram.command.control_admin_required.replied"
RECALL_PLAN_EVENT = "agent.memory.recall.planned"
RECALL_INJECTED_EVENT = "agent.memory.recall.injected"
RECALL_SKIPPED_EVENT = "agent.memory.recall.skipped"
RECALL_FEEDBACK_EVENT = "agent.memory.recall.feedback_updated"
EMBEDDING_TIMEOUT_FALLBACK_EVENT = "agent.memory.embedding.timeout_fallback_hash"
EMBEDDING_COOLDOWN_FALLBACK_EVENT = "agent.memory.embedding.cooldown_fallback_hash"
EMBEDDING_UNAVAILABLE_FALLBACK_EVENT = "agent.memory.embedding.unavailable_fallback_hash"
BOT_MARKER = "→ Bot:"

QuerySpec = _MODELS.QuerySpec
ScenarioSpec = _MODELS.ScenarioSpec
TurnResult = _MODELS.TurnResult
ModeSummary = _MODELS.ModeSummary
BenchmarkConfig = _MODELS.BenchmarkConfig

parse_args = partial(
    _CONFIG.parse_args,
    script_dir=Path(__file__).resolve().parent,
    default_log_file=DEFAULT_LOG_FILE,
    default_max_wait=DEFAULT_MAX_WAIT,
    default_max_idle_secs=DEFAULT_MAX_IDLE_SECS,
)
default_report_path = _CONFIG.default_report_path
infer_session_ids_from_runtime_log = _MODULES.session_ids_from_runtime_log
normalize_telegram_session_partition_mode = _MODULES.normalize_telegram_session_partition_mode
session_partition_mode_from_runtime_log = _MODULES.session_partition_mode_from_runtime_log
telegram_session_partition_mode = _MODULES.telegram_session_partition_mode
keyword_hit_ratio = _ANALYSIS.keyword_hit_ratio
select_feedback_direction = _ANALYSIS.select_feedback_direction
has_event = _SIGNALS.has_event
strip_ansi = _SIGNALS.strip_ansi


def resolve_runtime_partition_mode(log_file: Path) -> str | None:
    return _ENTRY_BINDINGS.resolve_runtime_partition_mode(
        log_file,
        config_module=_CONFIG,
        normalize_telegram_session_partition_mode_fn=normalize_telegram_session_partition_mode,
        session_partition_mode_from_runtime_log_fn=session_partition_mode_from_runtime_log,
        telegram_session_partition_mode_fn=telegram_session_partition_mode,
    )


def build_config(args: Any) -> BenchmarkConfig:
    return _CONFIG.build_config(
        args,
        config_cls=BenchmarkConfig,
        infer_session_ids_fn=infer_session_ids_from_runtime_log,
        resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode,
    )


load_scenarios = partial(
    _CONFIG.load_scenarios,
    query_spec_cls=QuerySpec,
    scenario_spec_cls=ScenarioSpec,
)


def count_lines(path: Path) -> int:
    return _ENTRY_BINDINGS.count_lines(path, init_log_cursor_fn=_shared_init_log_cursor)


def read_new_lines(path: Path, cursor: int) -> tuple[int, list[str]]:
    return _ENTRY_BINDINGS.read_new_lines(
        path,
        cursor,
        read_new_log_lines_with_cursor_fn=_shared_read_new_log_lines_with_cursor,
        log_cursor_cls=_SharedLogCursor,
    )


def run_probe(
    config: BenchmarkConfig,
    *,
    prompt: str,
    expect_event: str,
    allow_no_bot: bool = False,
) -> list[str]:
    # Preserve legacy monkeypatch seam: tests patch this module's subprocess.
    _EXECUTION.subprocess = subprocess
    return _ENTRY_BINDINGS.run_probe(
        config,
        prompt=prompt,
        expect_event=expect_event,
        allow_no_bot=allow_no_bot,
        runtime_bindings_module=_RUNTIME_BINDINGS,
        execution_module=_EXECUTION,
        count_lines_fn=count_lines,
        read_new_lines_fn=read_new_lines,
        strip_ansi_fn=strip_ansi,
        has_event_fn=has_event,
        control_admin_required_event=CONTROL_ADMIN_REQUIRED_EVENT,
        forbidden_log_pattern=FORBIDDEN_LOG_PATTERN,
    )


parse_turn_signals = partial(
    _ENTRY_BINDINGS.parse_turn_signals,
    runtime_bindings_module=_RUNTIME_BINDINGS,
    execution_module=_EXECUTION,
    parse_turn_signals_fn=_SIGNALS.parse_turn_signals,
    forbidden_log_pattern=FORBIDDEN_LOG_PATTERN,
    bot_marker=BOT_MARKER,
    recall_plan_event=RECALL_PLAN_EVENT,
    recall_injected_event=RECALL_INJECTED_EVENT,
    recall_skipped_event=RECALL_SKIPPED_EVENT,
    recall_feedback_event=RECALL_FEEDBACK_EVENT,
    embedding_timeout_fallback_event=EMBEDDING_TIMEOUT_FALLBACK_EVENT,
    embedding_cooldown_fallback_event=EMBEDDING_COOLDOWN_FALLBACK_EVENT,
    embedding_unavailable_fallback_event=EMBEDDING_UNAVAILABLE_FALLBACK_EVENT,
)


def _parse_turn_signals_dynamic(lines: list[str]) -> dict[str, Any]:
    return parse_turn_signals(lines)


build_turn_result = partial(
    _ENTRY_BINDINGS.build_turn_result,
    runtime_bindings_module=_RUNTIME_BINDINGS,
    execution_module=_EXECUTION,
    parse_turn_signals_fn=_parse_turn_signals_dynamic,
    keyword_hit_ratio_fn=_ANALYSIS.keyword_hit_ratio,
    token_as_int_fn=_SIGNALS.token_as_int,
    token_as_float_fn=_SIGNALS.token_as_float,
    trim_text_fn=_SIGNALS.trim_text,
    turn_result_cls=TurnResult,
)
summarize_mode = partial(
    _ENTRY_BINDINGS.summarize_mode,
    runtime_bindings_module=_RUNTIME_BINDINGS,
    execution_module=_EXECUTION,
    summarize_mode_data_fn=_ANALYSIS.summarize_mode_data,
    mode_summary_cls=ModeSummary,
)


def _run_probe_dynamic(
    config: BenchmarkConfig,
    *,
    prompt: str,
    expect_event: str,
    allow_no_bot: bool = False,
) -> list[str]:
    return run_probe(config, prompt=prompt, expect_event=expect_event, allow_no_bot=allow_no_bot)


run_reset = partial(
    _ENTRY_BINDINGS.run_reset,
    runtime_bindings_module=_RUNTIME_BINDINGS,
    execution_module=_EXECUTION,
    run_probe_fn=_run_probe_dynamic,
    reset_event=RESET_EVENT,
)
run_feedback = partial(
    _ENTRY_BINDINGS.run_feedback,
    runtime_bindings_module=_RUNTIME_BINDINGS,
    execution_module=_EXECUTION,
    run_probe_fn=_run_probe_dynamic,
    feedback_event=FEEDBACK_EVENT,
)
run_non_command_turn = partial(
    _ENTRY_BINDINGS.run_non_command_turn,
    runtime_bindings_module=_RUNTIME_BINDINGS,
    execution_module=_EXECUTION,
    run_probe_fn=_run_probe_dynamic,
    recall_plan_event=RECALL_PLAN_EVENT,
)
run_mode = partial(
    _ENTRY_BINDINGS.run_mode,
    runtime_bindings_module=_RUNTIME_BINDINGS,
    execution_module=_EXECUTION,
    run_reset_fn=run_reset,
    run_non_command_turn_fn=run_non_command_turn,
    build_turn_result_fn=build_turn_result,
    select_feedback_direction_fn=select_feedback_direction,
    run_feedback_fn=run_feedback,
)

serialize_turn = _OUTPUT.serialize_turn
to_iso_utc = _ENTRY_BINDINGS.to_iso_utc


def main() -> int:
    return _ENTRY_BINDINGS.run_main(
        runner_module=_MODULES.runner_module,
        parse_args_value=parse_args(),
        build_config_fn=build_config,
        load_scenarios_fn=load_scenarios,
        run_mode_fn=run_mode,
        summarize_mode_fn=summarize_mode,
        compare_mode_summaries_fn=_ANALYSIS.compare_mode_summaries,
        build_markdown_report_fn=_REPORT.build_markdown_report,
        build_json_payload_fn=_OUTPUT.build_json_payload,
        write_outputs_fn=_OUTPUT.write_outputs,
        print_summary_fn=_OUTPUT.print_summary,
        to_iso_utc_fn=to_iso_utc,
    )


if __name__ == "__main__":
    raise SystemExit(main())
