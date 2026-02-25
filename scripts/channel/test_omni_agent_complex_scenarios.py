#!/usr/bin/env python3
"""Run complex multi-step Telegram black-box scenarios and emit JSON/Markdown reports."""

from __future__ import annotations

import importlib
import os
import re
import sys
from datetime import UTC, datetime
from functools import partial
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

load_module_bindings = importlib.import_module(
    "complex_scenarios_module_bindings"
).load_module_bindings
_MODULES = load_module_bindings(__file__)
_ENTRY_BINDINGS = _MODULES.entry_bindings_module
_RUNTIME_BINDINGS = _MODULES.runtime_bindings_module
_EXECUTION = _MODULES.execution_module
_EVALUATION = _MODULES.evaluation_module
_CONFIG = _MODULES.config_module
_RUNTIME_CONFIG = _MODULES.runtime_config_module
_MODELS = _MODULES.models_module

build_report = _MODULES.report_module.build_report
write_outputs = _MODULES.report_module.write_outputs

DEFAULT_MAX_WAIT = int(os.environ.get("OMNI_BLACKBOX_MAX_WAIT_SECS", "40"))
DEFAULT_MAX_IDLE_SECS = int(os.environ.get("OMNI_BLACKBOX_MAX_IDLE_SECS", "30"))
DEFAULT_LOG_FILE = os.environ.get("OMNI_CHANNEL_LOG_FILE", ".run/logs/omni-agent-webhook.log")

DEFAULT_FORBID_LOG_REGEXES = (
    "tools/call: Mcp error",
    "Telegram sendMessage failed",
)

SessionIdentity = _MODELS.SessionIdentity
ScenarioStepSpec = _MODELS.ScenarioStepSpec
ComplexityRequirement = _MODELS.ComplexityRequirement
QualityRequirement = _MODELS.QualityRequirement
ScenarioSpec = _MODELS.ScenarioSpec
ComplexityProfile = _MODELS.ComplexityProfile
QualityProfile = _MODELS.QualityProfile
StepRunResult = _MODELS.StepRunResult
ScenarioRunResult = _MODELS.ScenarioRunResult
RunnerConfig = _MODELS.RunnerConfig

parse_args = partial(
    _ENTRY_BINDINGS.parse_args,
    config_module=_CONFIG,
    script_dir=Path(__file__).resolve().parent,
    webhook_url_default=os.environ.get("OMNI_WEBHOOK_URL")
    or _MODULES.default_telegram_webhook_url(),
    default_log_file=DEFAULT_LOG_FILE,
    default_max_wait=DEFAULT_MAX_WAIT,
    default_max_idle_secs=DEFAULT_MAX_IDLE_SECS,
)

normalize_telegram_session_partition_mode = _MODULES.normalize_telegram_session_partition_mode
session_partition_mode_from_runtime_log = _MODULES.session_partition_mode_from_runtime_log
telegram_session_partition_mode = _MODULES.telegram_session_partition_mode
session_ids_from_runtime_log = _MODULES.session_ids_from_runtime_log
allowed_users_from_settings = _MODULES.allowed_users_from_settings
username_from_settings = _MODULES.username_from_settings
username_from_runtime_log = _MODULES.username_from_runtime_log
telegram_webhook_secret_token = _MODULES.telegram_webhook_secret_token


def expected_session_keys(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None = None,
) -> tuple[str, ...]:
    return _RUNTIME_BINDINGS.expected_session_keys(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        session_keys_module=_MODULES.session_keys_module,
        normalize_partition_fn=normalize_telegram_session_partition_mode,
    )


def expected_session_key(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None = None,
) -> str:
    return _RUNTIME_BINDINGS.expected_session_key(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        session_keys_module=_MODULES.session_keys_module,
        normalize_partition_fn=normalize_telegram_session_partition_mode,
    )


def expected_session_log_regex(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    partition_mode: str | None = None,
) -> str:
    return _RUNTIME_BINDINGS.expected_session_log_regex(
        chat_id,
        user_id,
        thread_id,
        partition_mode,
        session_keys_module=_MODULES.session_keys_module,
        normalize_partition_fn=normalize_telegram_session_partition_mode,
    )


def resolve_runtime_partition_mode(log_file: Path) -> str | None:
    return _RUNTIME_BINDINGS.resolve_runtime_partition_mode(
        log_file,
        runtime_config_module=_RUNTIME_CONFIG,
        env_get_fn=os.environ.get,
        normalize_partition_fn=normalize_telegram_session_partition_mode,
        partition_mode_from_runtime_log_fn=session_partition_mode_from_runtime_log,
        partition_mode_from_settings_fn=telegram_session_partition_mode,
    )


tail_text = _RUNTIME_BINDINGS.tail_text
merge_requirements = partial(
    _EXECUTION.merge_requirements,
    requirement_cls=ComplexityRequirement,
)
merge_quality_requirements = partial(
    _EXECUTION.merge_quality_requirements,
    quality_requirement_cls=QualityRequirement,
)
build_execution_waves = _EVALUATION.build_execution_waves
load_scenarios = partial(
    _EXECUTION.load_scenarios,
    scenario_spec_cls=ScenarioSpec,
    step_spec_cls=ScenarioStepSpec,
    requirement_cls=ComplexityRequirement,
    quality_requirement_cls=QualityRequirement,
    build_execution_waves_fn=build_execution_waves,
)
select_scenarios = _EXECUTION.select_scenarios
compute_complexity_profile = partial(
    _EVALUATION.compute_complexity_profile,
    complexity_profile_cls=ComplexityProfile,
)
evaluate_complexity = _EVALUATION.evaluate_complexity
compute_quality_profile = partial(
    _EVALUATION.compute_quality_profile,
    quality_profile_cls=QualityProfile,
)
evaluate_quality = _EVALUATION.evaluate_quality
run_cmd = _EXECUTION.run_cmd
extract_bot_excerpt = _EXECUTION.extract_bot_excerpt
detect_memory_event_flags = _EXECUTION.detect_memory_event_flags
extract_memory_metrics, extract_mcp_metrics = (
    _MODULES.signal_bindings_module.build_signal_extractors(
        execution_module=_EXECUTION,
        regex_module=re,
    )
)


def run_step(
    cfg: RunnerConfig,
    scenario_id: str,
    step: ScenarioStepSpec,
    session: SessionIdentity,
    wave_index: int,
) -> StepRunResult:
    return _ENTRY_BINDINGS.run_step(
        cfg,
        scenario_id,
        step,
        session,
        wave_index,
        runtime_bindings_module=_RUNTIME_BINDINGS,
        execution_module=_EXECUTION,
        expected_session_key_fn=expected_session_key,
        expected_session_log_regex_fn=expected_session_log_regex,
        run_cmd_fn=run_cmd,
        detect_memory_event_flags_fn=detect_memory_event_flags,
        extract_memory_metrics_fn=extract_memory_metrics,
        extract_mcp_metrics_fn=extract_mcp_metrics,
        extract_bot_excerpt_fn=extract_bot_excerpt,
        tail_text_fn=tail_text,
        step_run_result_cls=StepRunResult,
    )


def skipped_step_result(
    scenario_id: str,
    step: ScenarioStepSpec,
    session: SessionIdentity,
    wave_index: int,
    reason: str,
    runtime_partition_mode: str | None = None,
) -> StepRunResult:
    return _ENTRY_BINDINGS.skipped_step_result(
        scenario_id,
        step,
        session,
        wave_index,
        reason,
        runtime_bindings_module=_RUNTIME_BINDINGS,
        execution_module=_EXECUTION,
        runtime_partition_mode=runtime_partition_mode,
        expected_session_key_fn=expected_session_key,
        step_run_result_cls=StepRunResult,
    )


def run_scenario(cfg: RunnerConfig, scenario: ScenarioSpec) -> ScenarioRunResult:
    skipped_step_result_fn = partial(
        skipped_step_result,
        runtime_partition_mode=cfg.runtime_partition_mode,
    )
    return _ENTRY_BINDINGS.run_scenario(
        cfg,
        scenario,
        runtime_bindings_module=_RUNTIME_BINDINGS,
        execution_module=_EXECUTION,
        merge_requirements_fn=merge_requirements,
        merge_quality_requirements_fn=merge_quality_requirements,
        compute_complexity_profile_fn=compute_complexity_profile,
        evaluate_complexity_fn=evaluate_complexity,
        quality_profile_cls=QualityProfile,
        build_execution_waves_fn=build_execution_waves,
        run_step_fn=run_step,
        skipped_step_result_fn=skipped_step_result_fn,
        compute_quality_profile_fn=compute_quality_profile,
        evaluate_quality_fn=evaluate_quality,
        scenario_run_result_cls=ScenarioRunResult,
    )


def build_config(args) -> RunnerConfig:
    return _ENTRY_BINDINGS.build_config(
        args,
        runtime_bindings_module=_RUNTIME_BINDINGS,
        runtime_config_module=_RUNTIME_CONFIG,
        expected_session_keys_fn=expected_session_keys,
        expected_session_key_fn=expected_session_key,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log,
        allowed_users_from_settings_fn=allowed_users_from_settings,
        username_from_settings_fn=username_from_settings,
        username_from_runtime_log_fn=username_from_runtime_log,
        telegram_webhook_secret_token_fn=telegram_webhook_secret_token,
        resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode,
        session_identity_cls=SessionIdentity,
        runner_config_cls=RunnerConfig,
        complexity_requirement_cls=ComplexityRequirement,
        quality_requirement_cls=QualityRequirement,
        default_forbid_log_regexes=DEFAULT_FORBID_LOG_REGEXES,
    )


def main() -> int:
    return _ENTRY_BINDINGS.run_main(
        runner_module=_MODULES.runner_module,
        parse_args_fn=parse_args,
        build_config_fn=build_config,
        load_scenarios_fn=load_scenarios,
        select_scenarios_fn=select_scenarios,
        run_scenario_fn=run_scenario,
        build_report_fn=build_report,
        write_outputs_fn=write_outputs,
        datetime_cls=datetime,
        utc_tz=UTC,
    )


if __name__ == "__main__":
    raise SystemExit(main())
