#!/usr/bin/env python3
"""Compatibility facade for Discord ingress stress parser argument groups."""

from __future__ import annotations

import discord_ingress_stress_config_group_identity as _identity
import discord_ingress_stress_config_group_output as _output
import discord_ingress_stress_config_group_runtime as _runtime

add_runtime_args = _runtime.add_runtime_args
add_identity_args = _identity.add_identity_args
add_output_and_quality_args = _output.add_output_and_quality_args
