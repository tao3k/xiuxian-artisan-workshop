#!/usr/bin/env python3
"""Compatibility facade for memory benchmark datamodels."""

from __future__ import annotations

import memory_benchmark_models_config as _config
import memory_benchmark_models_specs as _specs
import memory_benchmark_models_summary as _summary
import memory_benchmark_models_turn as _turn

QuerySpec = _specs.QuerySpec
ScenarioSpec = _specs.ScenarioSpec
TurnResult = _turn.TurnResult
ModeSummary = _summary.ModeSummary
BenchmarkConfig = _config.BenchmarkConfig
