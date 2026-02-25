"""
Memory package - Long-term memory storage and retrieval.

Components:
- archiver: Flushes messages from RAM to Vector DB (LanceDB)
- retrospective: Post-execution memory distillation
- hippocampus: Experience-driven reasoning for long-term memory
- schemas: Pydantic models for memory data structures
"""

from .archiver import MemoryArchiver
from .hippocampus import (
    HIPPOCAMPUS_COLLECTION,
    Hippocampus,
    create_hippocampus_trace,
    get_hippocampus,
)
from .retrospective import (
    create_session_retrospective,
    extract_knowledge_to_save,
    format_retrospective,
)
from .schemas import (
    ExecutionStep,
    ExperienceMetadata,
    ExperienceRecallResult,
    HippocampusTrace,
)

__all__ = [
    "HIPPOCAMPUS_COLLECTION",
    "ExecutionStep",
    "ExperienceMetadata",
    "ExperienceRecallResult",
    "Hippocampus",
    "HippocampusTrace",
    "MemoryArchiver",
    "create_hippocampus_trace",
    "create_session_retrospective",
    "extract_knowledge_to_save",
    "format_retrospective",
    "get_hippocampus",
]
