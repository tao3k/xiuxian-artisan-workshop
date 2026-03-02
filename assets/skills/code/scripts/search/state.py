"""
Search State Definition - TypedDict for native workflow runtime.

Defines the state structure for the Interactive Search Graph,
enabling strategy-aware execution and state persistence.
"""

import operator
from typing import Annotated, Literal

from typing_extensions import TypedDict


class SearchResult(TypedDict):
    """Single search result from any engine."""

    engine: Literal["ast", "vector", "grep"]  # Which engine found this
    file: str  # Relative file path
    line: int  # Line number
    content: str  # Matched content
    score: float  # Relevance score (0-1)


class SearchGraphState(TypedDict):
    """State passed through the Search Graph.

    Uses Annotated with operator.add to enable reducer semantics:
    results from parallel nodes are accumulated, not replaced.
    """

    # Input
    query: str

    # Routing decisions
    strategies: Annotated[list[str], operator.add]  # ["ast", "vector", "grep"]

    # Accumulated results from parallel execution
    raw_results: Annotated[list[SearchResult], operator.add]

    # Control flow
    iteration: int
    needs_clarification: bool
    clarification_prompt: str

    # Final output
    final_output: str

    # Metadata for checkpointing
    thread_id: str
    timestamp: str


# Constants for clarity
MAX_RESULTS_DEFAULT = 10
MAX_RESULTS_BROAD = 50
