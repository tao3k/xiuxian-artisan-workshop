#!/usr/bin/env python3
"""Compatibility facade for reconstructed trace summary helpers."""

from __future__ import annotations

import trace_reconstruction_summary_build as _build
import trace_reconstruction_summary_flags as _flags
import trace_reconstruction_summary_health as _health

DEFAULT_REQUIRED_STAGES = _flags.DEFAULT_REQUIRED_STAGES
STAGE_TO_FLAG = _flags.STAGE_TO_FLAG
STAGE_ERROR_MESSAGE = _flags.STAGE_ERROR_MESSAGE
first_index = _flags.first_index
collect_injection_modes = _flags.collect_injection_modes
build_trace_summary = _build.build_trace_summary
evaluate_trace_health = _health.evaluate_trace_health
