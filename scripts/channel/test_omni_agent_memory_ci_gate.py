#!/usr/bin/env python3
"""CI gate runner for omni-agent memory/channel verification."""

from __future__ import annotations

import importlib
import subprocess
import sys
from functools import partial
from pathlib import Path

from log_io import (
    init_log_cursor,
    iter_log_lines,
    read_log_tail_text,
    read_new_log_lines_with_cursor,
)

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_triage_module = importlib.import_module("memory_ci_gate_triage")
_quality_module = importlib.import_module("memory_ci_gate_quality")
_runner_module = importlib.import_module("memory_ci_gate_runner")
_runtime_module = importlib.import_module("memory_ci_gate_runtime")
_config_module = importlib.import_module("memory_ci_gate_config")
_models_module = importlib.import_module("memory_ci_gate_models")
_entry_bindings_module = importlib.import_module("memory_ci_gate_entry_bindings")
_log_bindings_module = importlib.import_module("memory_ci_gate_log_bindings")
_triage_bindings_module = importlib.import_module("memory_ci_gate_triage_bindings")

LOG_TAIL_SCAN_BYTES = 512 * 1024

GateConfig = _models_module.GateConfig
GateStepError = _models_module.GateStepError

default_valkey_prefix = _config_module.default_valkey_prefix
can_bind_tcp = _config_module.can_bind_tcp
default_run_suffix = _config_module.default_run_suffix
default_artifact_relpath = _runtime_module.default_artifact_relpath

allocate_free_tcp_port = partial(
    _config_module.allocate_free_tcp_port,
    can_bind_tcp_fn=can_bind_tcp,
)
resolve_runtime_ports = partial(
    _config_module.resolve_runtime_ports,
    can_bind_tcp_fn=can_bind_tcp,
    allocate_free_tcp_port_fn=allocate_free_tcp_port,
)

parse_args = partial(
    _entry_bindings_module.parse_args,
    config_module=_config_module,
    gate_config_cls=GateConfig,
    default_artifact_relpath_fn=default_artifact_relpath,
    resolve_runtime_ports_fn=resolve_runtime_ports,
    default_run_suffix_fn=default_run_suffix,
    default_valkey_prefix_fn=default_valkey_prefix,
)
run_command = partial(
    _entry_bindings_module.run_command,
    runtime_module=_runtime_module,
    gate_step_error_cls=GateStepError,
)

shell_quote_command = _triage_bindings_module.shell_quote_command
classify_gate_failure = _triage_module.classify_gate_failure

read_tail = partial(
    _log_bindings_module.read_tail,
    runtime_module=_runtime_module,
    read_log_tail_text_fn=read_log_tail_text,
    tail_bytes=LOG_TAIL_SCAN_BYTES,
)
count_log_event = partial(
    _log_bindings_module.count_log_event,
    runtime_module=_runtime_module,
    iter_log_lines_fn=iter_log_lines,
)
wait_for_log_regex = partial(
    _log_bindings_module.wait_for_log_regex,
    runtime_module=_runtime_module,
    read_tail_fn=read_tail,
    init_log_cursor_fn=init_log_cursor,
    read_new_log_lines_with_cursor_fn=read_new_log_lines_with_cursor,
)


def valkey_reachable(url: str) -> bool:
    result = subprocess.run(
        ["valkey-cli", "-u", url, "ping"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
        text=True,
    )
    return result.returncode == 0


def _triage_read_tail(path: Path, max_lines: int = 80) -> str:
    return read_tail(path, max_lines=max_lines)


_default_gate_failure_report_base_path = _triage_module.default_gate_failure_report_base_path
_artifact_rows = _triage_module.artifact_rows
_build_gate_failure_triage_payload = partial(
    _triage_module._build_gate_failure_triage_payload,
    read_tail_fn=_triage_read_tail,
    shell_quote_command_fn=shell_quote_command,
)

build_gate_failure_repro_commands = partial(
    _triage_bindings_module.build_gate_failure_repro_commands,
    triage_module=_triage_module,
)
write_gate_failure_triage_report = partial(
    _triage_bindings_module.write_gate_failure_triage_report,
    triage_module=_triage_module,
    read_tail_fn=_triage_read_tail,
)
write_gate_failure_triage_json_report = partial(
    _triage_bindings_module.write_gate_failure_triage_json_report,
    triage_module=_triage_module,
    read_tail_fn=_triage_read_tail,
)
print_gate_failure_triage = partial(
    _triage_bindings_module.print_gate_failure_triage,
    triage_module=_triage_module,
    classify_failure_fn=classify_gate_failure,
    read_tail_fn=_triage_read_tail,
)

wait_for_mock_health = _runtime_module.wait_for_mock_health
terminate_process = _runtime_module.terminate_process
ensure_parent_dirs = _log_bindings_module.ensure_parent_dirs
_yaml_inline_list = _runtime_module._yaml_inline_list
write_ci_channel_acl_settings = _runtime_module.write_ci_channel_acl_settings
start_background_process = partial(
    _log_bindings_module.start_background_process,
    runtime_module=_runtime_module,
    ensure_parent_dirs_fn=ensure_parent_dirs,
)

assert_evolution_quality = _quality_module.assert_evolution_quality
assert_benchmark_quality = _quality_module.assert_benchmark_quality
load_json = _quality_module.load_json
_safe_int = _quality_module.safe_int
assert_evolution_slow_response_quality = _quality_module.assert_evolution_slow_response_quality
assert_session_matrix_quality = _quality_module.assert_session_matrix_quality
assert_cross_group_complex_quality = _quality_module.assert_cross_group_complex_quality
assert_trace_reconstruction_quality = _quality_module.assert_trace_reconstruction_quality
assert_mcp_waiting_warning_budget = partial(
    _quality_module.assert_mcp_waiting_warning_budget,
    count_log_event_fn=count_log_event,
)
assert_memory_stream_warning_budget = partial(
    _quality_module.assert_memory_stream_warning_budget,
    count_log_event_fn=count_log_event,
)

run_reflection_quality_gate = partial(
    _runtime_module.run_reflection_quality_gate,
    run_command_fn=run_command,
)
run_discover_cache_gate = partial(
    _runtime_module.run_discover_cache_gate,
    run_command_fn=run_command,
)


def run_trace_reconstruction_gate(cfg: GateConfig, *, cwd: Path, env: dict[str, str]) -> None:
    _entry_bindings_module.run_trace_reconstruction_gate(
        cfg,
        cwd=cwd,
        env=env,
        runtime_module=_runtime_module,
        run_command_fn=run_command,
        assert_trace_reconstruction_quality_fn=assert_trace_reconstruction_quality,
    )


def run_cross_group_complex_gate(cfg: GateConfig, *, cwd: Path, env: dict[str, str]) -> None:
    _entry_bindings_module.run_cross_group_complex_gate(
        cfg,
        cwd=cwd,
        env=env,
        runtime_module=_runtime_module,
        run_command_fn=run_command,
        assert_cross_group_complex_quality_fn=assert_cross_group_complex_quality,
    )


run_gate = partial(
    _entry_bindings_module.run_gate,
    runner_module=_runner_module,
    ensure_parent_dirs_fn=ensure_parent_dirs,
    default_run_suffix_fn=default_run_suffix,
    write_ci_channel_acl_settings_fn=write_ci_channel_acl_settings,
    valkey_reachable_fn=valkey_reachable,
    run_command_fn=run_command,
    start_background_process_fn=start_background_process,
    wait_for_mock_health_fn=wait_for_mock_health,
    wait_for_log_regex_fn=wait_for_log_regex,
    run_reflection_quality_gate_fn=run_reflection_quality_gate,
    run_discover_cache_gate_fn=run_discover_cache_gate,
    run_trace_reconstruction_gate_fn=run_trace_reconstruction_gate,
    run_cross_group_complex_gate_fn=run_cross_group_complex_gate,
    assert_mcp_waiting_warning_budget_fn=assert_mcp_waiting_warning_budget,
    assert_memory_stream_warning_budget_fn=assert_memory_stream_warning_budget,
    assert_evolution_quality_fn=assert_evolution_quality,
    assert_evolution_slow_response_quality_fn=assert_evolution_slow_response_quality,
    assert_session_matrix_quality_fn=assert_session_matrix_quality,
    assert_benchmark_quality_fn=assert_benchmark_quality,
    terminate_process_fn=terminate_process,
)


def main() -> int:
    return _entry_bindings_module.run_main(
        project_root=Path(__file__).resolve().parents[2],
        parse_args_fn=parse_args,
        run_gate_fn=run_gate,
        print_gate_failure_triage_fn=print_gate_failure_triage,
    )


if __name__ == "__main__":
    raise SystemExit(main())
