"""
cortex.py - Prefrontal Cortex Module

Parallel task orchestration and dynamic sub-graph dispatch.

Components:
- nodes: TaskNode, TaskGroup, TaskGraph
- planner: TaskDecomposer for goal decomposition
- orchestrator: CortexOrchestrator for parallel execution
- transaction: TransactionShield for Git isolation
- homeostasis: Homeostasis integration layer

Integration:
    TaskDecomposer → CortexOrchestrator → TransactionShield → OmniCell
                                              ↓
                                    Homeostasis (Conflict Detection)
"""

from .homeostasis import (
    Homeostasis,
    HomeostasisConfig,
    HomeostasisResult,
)
from .nodes import (
    TaskGraph,
    TaskGroup,
    TaskNode,
    TaskPriority,
    TaskStatus,
)
from .orchestrator import (
    CortexOrchestrator,
    ExecutionConfig,
    ExecutionResult,
)
from .planner import (
    DecompositionResult,
    FileAnalysis,
    TaskDecomposer,
)
from .transaction import (
    ConflictDetector,
    ConflictReport,
    ConflictSeverity,
    Transaction,
    TransactionShield,
    TransactionStatus,
)

__all__ = [
    # Nodes
    "TaskNode",
    "TaskGroup",
    "TaskGraph",
    "TaskStatus",
    "TaskPriority",
    # Planner
    "TaskDecomposer",
    "DecompositionResult",
    "FileAnalysis",
    # Orchestrator
    "CortexOrchestrator",
    "ExecutionConfig",
    "ExecutionResult",
    # Transaction
    "TransactionShield",
    "Transaction",
    "TransactionStatus",
    "ConflictDetector",
    "ConflictReport",
    "ConflictSeverity",
    # Homeostasis
    "Homeostasis",
    "HomeostasisConfig",
    "HomeostasisResult",
]
