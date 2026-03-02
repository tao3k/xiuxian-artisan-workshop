#!/usr/bin/env python3
"""Compatibility facade for memory benchmark parser argument groups."""

from __future__ import annotations

import memory_benchmark_config_args_group_dataset as _dataset
import memory_benchmark_config_args_group_runtime as _runtime

add_dataset_and_paths_args = _dataset.add_dataset_and_paths_args
add_identity_args = _dataset.add_identity_args
add_runtime_args = _runtime.add_runtime_args
add_output_and_policy_args = _runtime.add_output_and_policy_args
