"""
config.py - OmniLoop Configuration

Pydantic model for configuring OmniLoop behavior.
"""

from pydantic import BaseModel


class OmniLoopConfig(BaseModel):
    """Configuration for the OmniLoop.

    Attributes:
        max_tokens: Maximum tokens for context (default: 128K)
        retained_turns: Number of conversation turns to retain (default: 10)
        max_tool_output: Max characters for tool output in pruner (default: 1000)
        auto_summarize: Enable automatic context summarization
        max_tool_calls: Max tool calls per turn for safety (default: 20)
        verbose: Enable verbose logging with DEBUG output
        suppress_atomic_tools: Enable adaptive skill projection (default: True)
        max_tool_schemas: Maximum number of tool schemas to expose (default: 20)
        max_consecutive_errors: Max consecutive errors before stopping (default: 3)
    """

    max_tokens: int = 128000
    retained_turns: int = 10
    max_tool_output: int = 1000
    auto_summarize: bool = False
    max_tool_calls: int = 20
    verbose: bool = False
    suppress_atomic_tools: bool = True
    max_tool_schemas: int = 20
    max_consecutive_errors: int = 3
