#!/usr/bin/env python3
"""Session-A reset flow step templates for session matrix runner."""

from __future__ import annotations

from typing import Any


def build_session_a_reset_validation_steps(
    cfg: Any,
    *,
    matrix_step_cls: Any,
    session_context_result_fields_fn: Any,
    session_memory_result_fields_fn: Any,
) -> tuple[Any, ...]:
    """Build steps validating session isolation after resetting session A."""
    return (
        matrix_step_cls(
            name="reset_session_a",
            prompt="/reset",
            chat_id=cfg.chat_id,
            event="telegram.command.session_reset.replied",
            user_id=cfg.user_a,
            thread_id=cfg.thread_a,
        ),
        matrix_step_cls(
            name="session_status_session_a_after_reset",
            prompt="/session json",
            chat_id=cfg.chat_id,
            event="telegram.command.session_status_json.replied",
            user_id=cfg.user_a,
            thread_id=cfg.thread_a,
            expect_reply_json_fields=session_context_result_fields_fn(
                cfg.chat_id,
                cfg.user_a,
                cfg.thread_a,
                cfg.session_partition,
            ),
        ),
        matrix_step_cls(
            name="session_memory_session_a_after_reset",
            prompt="/session memory json",
            chat_id=cfg.chat_id,
            event="telegram.command.session_memory_json.replied",
            user_id=cfg.user_a,
            thread_id=cfg.thread_a,
            expect_reply_json_fields=session_memory_result_fields_fn(
                cfg.chat_id,
                cfg.user_a,
                cfg.thread_a,
                cfg.session_partition,
            ),
        ),
        matrix_step_cls(
            name="resume_status_session_a",
            prompt="/resume status",
            chat_id=cfg.chat_id,
            event="telegram.command.session_resume_status.replied",
            user_id=cfg.user_a,
            thread_id=cfg.thread_a,
        ),
        matrix_step_cls(
            name="session_status_session_b_after_a_reset",
            prompt="/session json",
            chat_id=cfg.chat_b,
            event="telegram.command.session_status_json.replied",
            user_id=cfg.user_b,
            thread_id=cfg.thread_b,
            expect_reply_json_fields=session_context_result_fields_fn(
                cfg.chat_b,
                cfg.user_b,
                cfg.thread_b,
                cfg.session_partition,
            ),
        ),
        matrix_step_cls(
            name="session_memory_session_b_after_a_reset",
            prompt="/session memory json",
            chat_id=cfg.chat_b,
            event="telegram.command.session_memory_json.replied",
            user_id=cfg.user_b,
            thread_id=cfg.thread_b,
            expect_reply_json_fields=session_memory_result_fields_fn(
                cfg.chat_b,
                cfg.user_b,
                cfg.thread_b,
                cfg.session_partition,
            ),
        ),
        matrix_step_cls(
            name="session_status_session_c_after_a_reset",
            prompt="/session json",
            chat_id=cfg.chat_c,
            event="telegram.command.session_status_json.replied",
            user_id=cfg.user_c,
            thread_id=cfg.thread_c,
            expect_reply_json_fields=session_context_result_fields_fn(
                cfg.chat_c,
                cfg.user_c,
                cfg.thread_c,
                cfg.session_partition,
            ),
        ),
        matrix_step_cls(
            name="session_memory_session_c_after_a_reset",
            prompt="/session memory json",
            chat_id=cfg.chat_c,
            event="telegram.command.session_memory_json.replied",
            user_id=cfg.user_c,
            thread_id=cfg.thread_c,
            expect_reply_json_fields=session_memory_result_fields_fn(
                cfg.chat_c,
                cfg.user_c,
                cfg.thread_c,
                cfg.session_partition,
            ),
        ),
    )
