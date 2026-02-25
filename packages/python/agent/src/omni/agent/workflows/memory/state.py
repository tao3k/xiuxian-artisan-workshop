import operator
from typing import Annotated, Any, TypedDict


class MemoryState(TypedDict):
    # Input
    query: str
    content: str | None
    mode: str  # "recall" | "store"

    # Internal
    retrieved_docs: list[dict[str, Any]]
    trace: Annotated[list[dict[str, Any]], operator.add]

    # Output
    final_context: str
    storage_result: str
