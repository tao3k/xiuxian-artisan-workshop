from __future__ import annotations

import copy
from dataclasses import dataclass
from typing import Any

from .nodes import (
    advance_step,
    check_execution_progress,
    clarify_node,
    discovery_node,
    execute_node,
    plan_node,
    summary_node,
    validate_node,
)
from .reflection.node import reflection_node
from .review.node import review_node


@dataclass(slots=True)
class WorkflowSnapshot:
    """Snapshot of workflow state and pending next nodes."""

    values: dict[str, Any]
    next: tuple[str, ...]


class RobustTaskWorkflowApp:
    """Lightweight async state machine for robust task workflows."""

    def __init__(self, *, interrupt_before: set[str] | None = None) -> None:
        self._interrupt_before = interrupt_before or set()
        self._thread_states: dict[str, dict[str, Any]] = {}
        self._thread_next: dict[str, list[str]] = {}

    async def astream(self, current_input: dict[str, Any] | None, thread: dict[str, Any] | None):
        """Stream node updates while executing the workflow."""
        thread_id = self._resolve_thread_id(thread)

        if current_input is not None:
            state = self._with_defaults(current_input)
            self._thread_states[thread_id] = state
            self._thread_next[thread_id] = ["discovery"]

        if thread_id not in self._thread_states:
            return

        state = self._thread_states[thread_id]
        queue = self._thread_next.get(thread_id, [])

        while queue:
            node = queue[0]
            if self._should_interrupt(node, state):
                break

            queue.pop(0)
            update = await self._run_node(node, state)
            if update:
                self._merge_state(state, update)
            else:
                update = {}

            yield {node: update}

            queue[:0] = self._route_next(node, state)

        self._thread_next[thread_id] = queue

    async def ainvoke(
        self,
        current_input: dict[str, Any],
        thread: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        """Run the workflow to completion.

        If an interrupt checkpoint is reached at `review`, this method auto-approves so
        non-interactive call sites can still finish deterministically.
        """
        next_input: dict[str, Any] | None = current_input

        while True:
            async for _ in self.astream(next_input, thread):
                pass

            snapshot = await self.aget_state(thread)
            if not snapshot.next:
                return snapshot.values

            if snapshot.next == ("review",):
                approval = snapshot.values.get("approval_status", "pending")
                if approval in {"pending", "waiting_for_user"}:
                    self.update_state(thread, {"approval_status": "approved"})
                    next_input = None
                    continue

            return snapshot.values

    async def aget_state(self, thread: dict[str, Any] | None = None) -> WorkflowSnapshot:
        """Return latest workflow snapshot for a thread."""
        thread_id = self._resolve_thread_id(thread)
        state = self._thread_states.get(thread_id, self._with_defaults({}))
        queue = tuple(self._thread_next.get(thread_id, []))
        return WorkflowSnapshot(values=copy.deepcopy(state), next=queue)

    def update_state(self, thread: dict[str, Any] | None, values: dict[str, Any]) -> None:
        """Merge external updates (e.g., HITL approval)."""
        thread_id = self._resolve_thread_id(thread)
        state = self._thread_states.setdefault(thread_id, self._with_defaults({}))
        self._merge_state(state, values)

    @staticmethod
    def _resolve_thread_id(thread: dict[str, Any] | None) -> str:
        if not thread:
            return "default"
        configurable = thread.get("configurable", {})
        thread_id = configurable.get("thread_id")
        return str(thread_id) if thread_id else "default"

    @staticmethod
    def _with_defaults(state: dict[str, Any]) -> dict[str, Any]:
        defaults: dict[str, Any] = {
            "user_request": "",
            "clarified_goal": "",
            "context_files": [],
            "discovered_tools": [],
            "memory_context": "",
            "last_thought": "",
            "trace": [],
            "user_feedback": "",
            "approval_status": "pending",
            "plan": {"steps": [], "current_step_index": 0},
            "execution_history": [],
            "status": "clarifying",
            "retry_count": 0,
            "validation_result": {},
            "final_summary": "",
            "error": "",
        }
        merged = copy.deepcopy(defaults)
        for key, value in state.items():
            merged[key] = value
        return merged

    @staticmethod
    def _merge_state(state: dict[str, Any], update: dict[str, Any]) -> None:
        for key, value in update.items():
            if key in {"trace", "execution_history"} and isinstance(value, list):
                state.setdefault(key, [])
                state[key].extend(value)
                continue
            state[key] = value

    @staticmethod
    async def _run_node(node: str, state: dict[str, Any]) -> dict[str, Any]:
        if node == "discovery":
            return await discovery_node(state)
        if node == "clarify":
            return await clarify_node(state)
        if node == "plan":
            return await plan_node(state)
        if node == "review":
            return await review_node(state)
        if node == "execute":
            return await execute_node(state)
        if node == "advance":
            return advance_step(state)
        if node == "validate":
            return await validate_node(state)
        if node == "reflect":
            return await reflection_node(state)
        if node == "summary":
            return await summary_node(state)
        return {}

    def _should_interrupt(self, node: str, state: dict[str, Any]) -> bool:
        if node not in self._interrupt_before:
            return False
        if node != "review":
            return True
        approval = state.get("approval_status", "pending")
        return approval in {"pending", "waiting_for_user"}

    @staticmethod
    def _route_next(node: str, state: dict[str, Any]) -> list[str]:
        if node == "discovery":
            return ["clarify"]

        if node == "clarify":
            status = state.get("status")
            if status == "planning":
                return ["plan"]
            if status == "clarifying":
                return ["clarify"]
            return ["summary"]

        if node == "plan":
            return ["execute"]

        if node == "execute":
            transition = check_execution_progress(state)
            if transition == "continue":
                return ["advance"]
            return ["review"]

        if node == "advance":
            return ["execute"]

        if node == "review":
            approval = state.get("approval_status")
            if approval == "approved":
                return ["validate"]
            if approval == "modified":
                return ["plan"]
            if approval == "rejected":
                return ["summary"]
            return ["review"]

        if node == "validate":
            status = state.get("status")
            if status == "completed":
                return ["summary"]
            if state.get("retry_count", 0) < 3:
                return ["reflect"]
            return ["summary"]

        if node == "reflect":
            return ["plan"]

        return []


def build_graph(checkpointer=None) -> RobustTaskWorkflowApp:
    """Build robust task workflow app.

    The `checkpointer` argument is kept for API compatibility with older call sites.
    """
    _ = checkpointer
    return RobustTaskWorkflowApp(interrupt_before={"review"})
