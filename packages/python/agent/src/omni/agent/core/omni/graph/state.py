import operator
from typing import Annotated, Any, TypedDict


class AgentState(TypedDict):
    # Input
    user_query: str

    # Context (Long-term & Session)
    system_prompt: str
    messages: Annotated[list[dict[str, Any]], operator.add]

    # Tool Management
    available_tools: list[dict[str, Any]]

    # Execution State
    step_count: int
    tool_calls_count: int
    consecutive_errors: int
    tool_hash_history: list[str]

    # Current Turn Data
    last_response: str
    tool_calls: list[dict[str, Any]]
    tool_results: list[dict[str, Any]]

    # Flow Control
    status: str  # "thinking", "acting", "reflecting", "done", "failed"
    exit_reason: str
