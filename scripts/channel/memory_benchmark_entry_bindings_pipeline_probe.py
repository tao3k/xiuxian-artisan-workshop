#!/usr/bin/env python3
"""Compatibility facade for memory benchmark probe/signal entry bindings."""

from __future__ import annotations

import memory_benchmark_entry_bindings_pipeline_probe_result as _result
import memory_benchmark_entry_bindings_pipeline_probe_run as _run
import memory_benchmark_entry_bindings_pipeline_probe_signals as _signals
import memory_benchmark_entry_bindings_pipeline_probe_summary as _summary

run_probe = _run.run_probe
parse_turn_signals = _signals.parse_turn_signals
build_turn_result = _result.build_turn_result
summarize_mode = _summary.summarize_mode
