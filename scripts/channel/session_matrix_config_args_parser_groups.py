#!/usr/bin/env python3
"""Compatibility facade for session matrix parser argument groups."""

from __future__ import annotations

import session_matrix_config_args_group_identity as _identity
import session_matrix_config_args_group_output as _output
import session_matrix_config_args_group_runtime as _runtime

add_runtime_args = _runtime.add_runtime_args
add_identity_args = _identity.add_identity_args
add_probe_and_output_args = _output.add_probe_and_output_args
