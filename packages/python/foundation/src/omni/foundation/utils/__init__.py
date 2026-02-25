# utils
"""
Utilities Module

Provides common utility functions:
- templating.py: Template rendering
- skills.py: Skill-related utilities
- common.py: Common helper functions

Usage:
    from omni.foundation.utils.templating import render_template
    from omni.foundation.config.skills import SKILLS_DIR
    from omni.foundation.utils.common import is_binary
"""

from .asyncio import run_async_blocking
from .common import agent_src, common_src, project_root, setup_import_paths
from .fs import find_files_by_extension, find_markdown_files
from .json_codec import JSONDecodeError
from .json_codec import dump as json_dump
from .json_codec import dumps as json_dumps
from .json_codec import load as json_load
from .json_codec import loads as json_loads
from .skills import (
    current_skill_dir,
    skill_asset,
    skill_command,
    skill_data,
    skill_path,
    skill_reference,
)
from .templating import render_string

__all__ = [
    "JSONDecodeError",
    "agent_src",
    "common_src",
    "current_skill_dir",
    "find_files_by_extension",
    "find_markdown_files",
    "json_dump",
    "json_dumps",
    "json_load",
    "json_loads",
    "project_root",
    "render_string",
    "run_async_blocking",
    "setup_import_paths",
    "skill_asset",
    "skill_command",
    "skill_data",
    "skill_path",
    "skill_reference",
]
