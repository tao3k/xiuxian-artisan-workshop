"""
MCP Server Tools

Modular tool definitions for the Agent MCP Server.

Modules:
- embedding: Embedding generation via preloaded model
"""

from .embedding import register_embedding_tools

__all__ = [
    "register_embedding_tools",
]
