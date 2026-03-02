"""
Interactive Search Graph - Native Workflow Runtime.

Orchestrates strategy-aware search execution with:
- Intent classification for strategy selection
- AST/Vector/Grep execution with per-node gating
- Result synthesis and XML formatting
- Optional checkpoint handle injection
"""

from datetime import datetime
from typing import Any

from omni.tracer.pipeline_checkpoint import compile_workflow
from omni.tracer.workflow_engine import END_NODE, NativeStateGraph

from .nodes import classifier, engines, formatter
from .state import SearchGraphState


def _gated_ast_search(state: SearchGraphState) -> dict[str, Any]:
    """Execute AST engine only when selected by classifier."""
    strategies = state.get("strategies", [])
    if "ast" not in strategies:
        return {}
    return engines.node_run_ast_search(state)


def _gated_vector_search(state: SearchGraphState) -> dict[str, Any]:
    """Execute vector engine only when selected by classifier."""
    strategies = state.get("strategies", [])
    if "vector" not in strategies:
        return {}
    return engines.node_run_vector_search(state)


def _gated_grep_search(state: SearchGraphState) -> dict[str, Any]:
    """Execute grep engine only when selected by classifier."""
    strategies = state.get("strategies", [])
    if "grep" not in strategies:
        return {}
    return engines.node_run_grep_search(state)


def create_search_graph() -> NativeStateGraph:
    """Create the interactive native search graph."""
    workflow = NativeStateGraph(SearchGraphState)

    # Add nodes
    workflow.add_node("classify", classifier.classify_intent)
    workflow.add_node("run_ast", _gated_ast_search)
    workflow.add_node("run_vector", _gated_vector_search)
    workflow.add_node("run_grep", _gated_grep_search)
    workflow.add_node("synthesize", formatter.synthesize_results)

    # Set entry point
    workflow.set_entry_point("classify")

    # Sequential execution with gating wrappers.
    workflow.add_edge("classify", "run_ast")
    workflow.add_edge("run_ast", "run_vector")
    workflow.add_edge("run_vector", "run_grep")
    workflow.add_edge("run_grep", "synthesize")
    workflow.add_edge("synthesize", END_NODE)

    return workflow


def create_initial_state(query: str, thread_id: str = "default") -> SearchGraphState:
    """Create initial state for the search graph."""
    return {
        "query": query,
        "strategies": [],  # Filled by classifier
        "raw_results": [],
        "iteration": 0,
        "needs_clarification": False,
        "clarification_prompt": "",
        "final_output": "",
        "thread_id": thread_id,
        "timestamp": datetime.now().isoformat(),
    }


# Global search graph state (lazily initialized).
_search_graph: NativeStateGraph | None = None
_compiled_search_graph: Any | None = None


def get_search_graph() -> NativeStateGraph:
    """Get or create the compiled search graph."""
    global _search_graph
    if _search_graph is None:
        workflow = create_search_graph()
        _search_graph = workflow
    return _search_graph


def get_compiled_search_graph() -> Any:
    """Get or create the compiled search graph with checkpoint support."""
    global _compiled_search_graph
    if _compiled_search_graph is None:
        graph = get_search_graph()
        _compiled_search_graph = compile_workflow(graph, use_memory_saver=True)
    return _compiled_search_graph


async def execute_search(query: str, thread_id: str = "default") -> dict:
    """Execute the search graph asynchronously.

    Args:
        query: Search query
        thread_id: Session ID for checkpointing

    Returns:
        Final state with formatted output
    """
    compiled = get_compiled_search_graph()

    # Create initial state
    state = create_initial_state(query, thread_id)

    # Execute the graph
    final_state = await compiled.ainvoke(state, config={"configurable": {"thread_id": thread_id}})

    return final_state
