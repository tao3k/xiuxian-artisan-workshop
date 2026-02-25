"""
research_entry.py - Entry point for Sharded Deep Research Workflow

Uses Qianji Engine to run Repo_Analyzer_Array.
"""

from __future__ import annotations

import asyncio
import json
import os
import subprocess
import urllib.parse
import uuid
from typing import Any

from omni.foundation.api.decorators import skill_command
from omni.foundation.config.logging import get_logger

logger = get_logger("researcher.entry")


async def _run_subprocess(
    args: list[str],
    *,
    cwd: str,
    text: bool = True,
) -> subprocess.CompletedProcess[str]:
    """Run subprocess in a worker thread to avoid blocking the event loop."""
    env = os.environ.copy()
    # Remove uv/python specific vars that might break nested cargo runs on macOS
    env.pop("DYLD_LIBRARY_PATH", None)

    return await asyncio.to_thread(
        subprocess.run,
        args,
        cwd=str(cwd),
        capture_output=True,
        text=text,
        env=env,
    )


async def run_qianji_engine(
    project_root: str, context: dict[str, Any], session_id: str
) -> tuple[bool, dict[str, Any], str]:
    """Execute the Qianji engine with repo_analyzer.toml."""
    from omni.foundation.config.skills import SKILLS_DIR

    manifest_path = str(SKILLS_DIR("researcher") / "workflows" / "repo_analyzer.toml")

    cmd = [
        "cargo",
        "run",
        "--release",
        "--quiet",
        "-p",
        "xiuxian-qianji",
        "--features",
        "llm",
        "--bin",
        "qianji",
        "--",
        project_root,
        manifest_path,
        json.dumps(context),
        session_id,
    ]

    proc = await _run_subprocess(cmd, cwd=".")

    if proc.returncode != 0:
        logger.error(f"Qianji Engine Failed: {proc.stderr}")
        return False, {}, proc.stderr

    # Parse output after "=== Final Qianji Execution Result ==="
    parts = proc.stdout.split("=== Final Qianji Execution Result ===")
    if len(parts) > 1:
        try:
            result_json = json.loads(parts[1].strip())
            return True, result_json, ""
        except json.JSONDecodeError as e:
            return False, {}, f"JSON decode error: {e}\nOutput: {parts[1]}"

    return False, {}, "Could not find result JSON marker in engine output."


@skill_command(
    name="git_repo_analyer",
    description="""
    Research a git repository: clone, analyze by shards, and produce an index.

    This autonomously analyzes large repositories using a Map-Plan-Loop-Synthesize pattern:

    1. **Setup**: Clone repository and generate file tree map
    2. **Architect (Plan)**: LLM breaks down the repo into 3-5 logical shards (subsystems)
    3. **Process Shard (Loop)**: For each shard - compress with repomix + analyze with LLM
    4. **Synthesize**: Generate index.md linking all shard analyses

    This approach handles large codebases that exceed LLM context limits by analyzing
    one subsystem at a time, then combining results.

    Args:
        - repo_url: str - Git repository URL to analyze (required)
        - request: str = "Analyze the architecture" - Specific analysis goal
        - action: str = "start" - "start" (propose shards) or "approve" (run deep analysis)
        - session_id: str = "" - Required for "approve" action
        - approved_shards: str = "" - JSON string of approved shards for "approve" action

    Returns:
        dict with success status, harvest directory path, and shard summaries
    """,
    category="research",
    read_only=False,
    destructive=False,
    idempotent=True,
    open_world=True,
)
async def run_research_graph(
    repo_url: str,
    request: str = "Analyze the architecture",
    action: str = "start",
    session_id: str = "",
    approved_shards: str = "",
) -> dict[str, Any]:
    """Execute the Sharded Deep Research workflow using Qianji Engine."""
    logger.info(
        "Sharded research workflow invoked via Qianji",
        repo_url=repo_url,
        request=request,
        action=action,
    )

    repo_name = urllib.parse.urlparse(repo_url).path.strip("/").split("/")[-1]
    if repo_name.endswith(".git"):
        repo_name = repo_name[:-4]

    if action == "start":
        session_id = str(uuid.uuid4())[:8]
        repo_dir = f"/tmp/omni_research_{repo_name}_{session_id}"

        context = {
            "repo_url": repo_url,
            "repo_dir": repo_dir,
            "request": request,
            "project_root": repo_dir,
        }

        success, result_json, err = await run_qianji_engine(".", context, session_id)

        if not success:
            return {"success": False, "error": f"Workflow Failed: {err}"}

        suspend_prompt = result_json.get("suspend_prompt", "")
        proposed_plan = result_json.get(
            "analysis_trace", []
        )  # Example output key from Architect node

        return {
            "success": True,
            "session_id": session_id,
            "message": suspend_prompt,
            "proposed_plan": proposed_plan,
            "next_action": "Call action='approve' with this session_id and approved_shards (JSON string)",
            "context": result_json,
        }

    elif action == "approve":
        if not session_id:
            return {"success": False, "error": "session_id is required for approve action"}
        if not approved_shards:
            return {
                "success": False,
                "error": "approved_shards JSON is required for approve action",
            }

        context = {
            "approved_shards": approved_shards,
        }

        success, result_json, err = await run_qianji_engine(".", context, session_id)

        if not success:
            return {"success": False, "error": f"Workflow Failed: {err}"}

        return {
            "success": True,
            "session_id": session_id,
            "analysis_result": result_json.get("analysis_result", ""),
            "full_context": result_json,
        }

    return {"success": False, "error": f"Unknown action: {action}"}


__all__ = ["run_research_graph"]
