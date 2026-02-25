"""Auto-generated Python types from shared schema.
Generated from: tool.schema
"""

from __future__ import annotations

from pydantic import BaseModel, Field


class ToolSchema(BaseModel):
    """"""

    info: dict = Field(..., description="")
    tools: list[dict] = Field(..., description="")
