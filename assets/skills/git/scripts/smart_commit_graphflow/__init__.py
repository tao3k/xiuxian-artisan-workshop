"""
smart_commit_graphflow - Smart Commit Workflow Module

A modular CLI-based workflow for git commits delegated to Rust Qianji Engine:
- Automatic staging and security scanning via AST
- Lefthook pre-commit integration
- Submodule change detection and commit support (Python wrapped)
- Human-in-the-loop approval via Valkey Checkpoint Suspend
- Persistent state via Rust Valkey/Redis Checkpoint Store
"""

from ._enums import (
    SmartCommitAction,
    SmartCommitStatus,
    WorkflowRouting,
)
from .commands import smart_commit

__all__ = [
    "SmartCommitAction",
    "SmartCommitStatus",
    "WorkflowRouting",
    "smart_commit",
]
