#!/usr/bin/env python3
"""Compatibility facade for session-matrix identity resolution helpers."""

from __future__ import annotations

import session_matrix_config_runtime_build_identity_peers as _peers
import session_matrix_config_runtime_build_identity_primary as _primary
import session_matrix_config_runtime_build_identity_username as _username

resolve_primary_identity = _primary.resolve_primary_identity
resolve_peer_chats = _peers.resolve_peer_chats
resolve_threads = _peers.resolve_threads
resolve_peer_users = _peers.resolve_peer_users
resolve_username = _username.resolve_username
