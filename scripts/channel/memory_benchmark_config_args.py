#!/usr/bin/env python3
"""Compatibility facade for memory benchmark CLI argument parsing helpers."""

from __future__ import annotations

import memory_benchmark_config_args_parser as _parser
import memory_benchmark_config_args_paths as _paths

parse_args = _parser.parse_args
default_report_path = _paths.default_report_path
