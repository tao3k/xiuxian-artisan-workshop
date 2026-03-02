"""workflow_engine.py - Native async workflow engine for tracer pipelines.

This module provides a minimal `StateGraph`-like API used by pipeline and graphflow
without depending on third-party graph runtimes.
"""

from __future__ import annotations

import inspect
from collections import defaultdict
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import Any

from omni.foundation.config.logging import get_logger

logger = get_logger("omni.tracer.workflow_engine")

END_NODE = "__END__"
MAX_WORKFLOW_STEPS = 10_000

NodeFn = Callable[[dict[str, Any]], dict[str, Any] | Awaitable[dict[str, Any]]]
ConditionFn = Callable[[dict[str, Any]], Any]


@dataclass(slots=True)
class _ConditionalEdge:
    condition: ConditionFn
    destinations: dict[str, str] | None = None


class NativeCompiledWorkflow:
    """Compiled native workflow application."""

    def __init__(
        self,
        *,
        nodes: dict[str, NodeFn],
        edges: dict[str, list[str]],
        conditional_edges: dict[str, list[_ConditionalEdge]],
        entry_node: str | None,
        checkpointer: Any | None = None,
    ) -> None:
        self._nodes = dict(nodes)
        self._edges = {key: list(values) for key, values in edges.items()}
        self._conditional_edges = {key: list(values) for key, values in conditional_edges.items()}
        self._entry_node = entry_node
        self._checkpointer = checkpointer

    async def ainvoke(
        self,
        initial_state: dict[str, Any] | Any | None = None,
        config: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        """Execute workflow until END and return final state."""
        state = self._coerce_state(initial_state)
        current = self._entry_node
        step_count = 0

        while current is not None:
            if current == END_NODE:
                break
            if current not in self._nodes:
                raise KeyError(f"Unknown workflow node: {current}")

            step_count += 1
            if step_count > MAX_WORKFLOW_STEPS:
                raise RuntimeError(
                    f"Workflow exceeded max step budget ({MAX_WORKFLOW_STEPS}); possible loop"
                )

            node_fn = self._nodes[current]
            node_result = node_fn(state)
            update = await node_result if inspect.isawaitable(node_result) else node_result

            if isinstance(update, dict):
                state = {**state, **update}
            elif update is None:
                pass
            else:
                logger.warning("workflow_node_non_dict_update_ignored", node=current)

            current = self._resolve_next(current, state)

        self._persist_checkpoint(state, config)
        return state

    @staticmethod
    def _coerce_state(initial_state: dict[str, Any] | Any | None) -> dict[str, Any]:
        if initial_state is None:
            return {}
        if isinstance(initial_state, dict):
            return dict(initial_state)
        try:
            return dict(initial_state)
        except Exception:
            return {"value": initial_state}

    def _resolve_next(self, current: str, state: dict[str, Any]) -> str | None:
        conditional_routes = self._conditional_edges.get(current, [])
        for route in conditional_routes:
            candidate = route.condition(state)
            if candidate is None:
                continue
            node = self._map_route_candidate(candidate, route.destinations)
            if node is not None:
                return node

        outgoing = self._edges.get(current, [])
        if not outgoing:
            return None
        next_node = outgoing[0]
        return None if next_node == END_NODE else next_node

    def _map_route_candidate(
        self,
        candidate: Any,
        destinations: dict[str, str] | None,
    ) -> str | None:
        route_key = str(candidate)

        if destinations:
            mapped = destinations.get(route_key)
            if mapped is None and route_key in destinations.values():
                mapped = route_key
            if mapped is None and route_key in self._nodes:
                mapped = route_key
            if mapped is None:
                return None
            return None if mapped == END_NODE else mapped

        if route_key in {END_NODE, "END", "__END__"}:
            return None
        return route_key if route_key in self._nodes else None

    def _persist_checkpoint(self, state: dict[str, Any], config: dict[str, Any] | None) -> None:
        del config
        if self._checkpointer is None:
            return
        # Native runtime keeps checkpointer injection contract for callers/tests.
        # Persistence backends can be integrated incrementally where needed.


class NativeStateGraph:
    """Minimal StateGraph-compatible builder for native runtime execution."""

    def __init__(self, state_schema: type[Any] | None = None) -> None:
        self._state_schema = state_schema
        self._nodes: dict[str, NodeFn] = {}
        self._edges: dict[str, list[str]] = defaultdict(list)
        self._conditional_edges: dict[str, list[_ConditionalEdge]] = defaultdict(list)
        self._entry_node: str | None = None

    def add_node(self, node_name: str, node_fn: NodeFn) -> None:
        self._nodes[node_name] = node_fn

    def set_entry_point(self, node_name: str) -> None:
        self._entry_node = node_name

    def add_edge(self, from_node: str, to_node: str) -> None:
        self._edges[from_node].append(to_node)

    def add_conditional_edges(
        self,
        from_node: str,
        condition: ConditionFn,
        destinations: dict[str, str] | None = None,
    ) -> None:
        self._conditional_edges[from_node].append(
            _ConditionalEdge(condition=condition, destinations=destinations)
        )

    def compile(self, *, checkpointer: Any | None = None) -> NativeCompiledWorkflow:
        del self._state_schema
        return NativeCompiledWorkflow(
            nodes=self._nodes,
            edges=self._edges,
            conditional_edges=self._conditional_edges,
            entry_node=self._entry_node,
            checkpointer=checkpointer,
        )


__all__ = [
    "END_NODE",
    "MAX_WORKFLOW_STEPS",
    "NativeCompiledWorkflow",
    "NativeStateGraph",
]
