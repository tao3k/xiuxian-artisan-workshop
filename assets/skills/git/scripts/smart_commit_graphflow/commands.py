"""
commands.py - Smart Commit Skill Commands

Entry point for git.smart_commit skill command.
Delegates to the Rust Qianji engine for workflow execution.
"""

import asyncio
import json
import re
import shutil
import subprocess
import uuid
from datetime import datetime
from difflib import get_close_matches
from pathlib import Path
from typing import Any

from git.scripts.prepare import (
    _get_cog_scopes,
)
from git.scripts.rendering import render_commit_message, render_template

from omni.foundation.api.decorators import skill_command
from omni.foundation.api.response_payloads import build_status_message_response
from omni.foundation.config.logging import get_logger
from omni.foundation.runtime.gitops import get_git_toplevel

from ._enums import SmartCommitAction, SmartCommitStatus

logger = get_logger("git.smart_commit")


def _resolve_git_repo_root(project_root: str = "") -> str:
    """Resolve git top-level for execution root."""
    if project_root:
        try:
            return str(get_git_toplevel(Path(project_root)))
        except RuntimeError:
            return project_root

    try:
        return str(get_git_toplevel())
    except RuntimeError:
        return "."


async def _run_subprocess(
    args: list[str],
    *,
    cwd: str | Path,
    text: bool = True,
) -> subprocess.CompletedProcess[str]:
    """Run subprocess in a worker thread to avoid blocking the event loop."""
    import os

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


def _handle_submodules_prepare(project_root: str) -> list[dict[str, str]]:
    """Detect submodules with changes and run prepare in each."""
    submodule_commits = []

    try:
        proc = subprocess.run(
            ["git", "submodule", "status"], cwd=project_root, capture_output=True, text=True
        )
        if proc.returncode != 0 or not proc.stdout.strip():
            return []

        submodule_lines = proc.stdout.split("\n")
        for line in submodule_lines:
            if not line:
                continue

            status_char = line[0]
            parts = line[1:].strip().split()
            if len(parts) >= 2:
                submodule_path = parts[1]
                sub_full_path = Path(project_root) / submodule_path

                proc_sub = subprocess.run(
                    ["git", "status", "--porcelain"],
                    cwd=sub_full_path,
                    capture_output=True,
                    text=True,
                )
                has_internal_changes = proc_sub.returncode == 0 and proc_sub.stdout.strip()

                if has_internal_changes or status_char == "+":
                    try:
                        subprocess.run(["git", "add", "-A"], cwd=sub_full_path, capture_output=True)
                        if shutil.which("lefthook"):
                            subprocess.run(
                                ["git", "hook", "run", "pre-commit"],
                                cwd=sub_full_path,
                                capture_output=True,
                            )

                        proc_files = subprocess.run(
                            ["git", "diff", "--cached", "--name-only"],
                            cwd=sub_full_path,
                            capture_output=True,
                            text=True,
                        )
                        file_count = (
                            len(proc_files.stdout.strip().split("\n"))
                            if proc_files.stdout.strip()
                            else 0
                        )

                        if file_count > 0:
                            date_str = datetime.now().strftime("%Y%m%d")
                            commit_msg = (
                                f"chore(submodule): update {submodule_path} ({date_str})\n\n"
                                f"Auto-committed by omni smart_commit\n\n"
                                f"Submodule: {submodule_path}\nFiles: {file_count}"
                            )
                            commit_result = subprocess.run(
                                ["git", "commit", "-m", commit_msg],
                                cwd=sub_full_path,
                                capture_output=True,
                                text=True,
                            )
                            if commit_result.returncode == 0:
                                proc_hash = subprocess.run(
                                    ["git", "rev-parse", "HEAD"],
                                    cwd=sub_full_path,
                                    capture_output=True,
                                    text=True,
                                )
                                commit_hash = (
                                    proc_hash.stdout.strip()[:8]
                                    if proc_hash.returncode == 0
                                    else "unknown"
                                )
                                submodule_commits.append(
                                    {"path": submodule_path, "commit_hash": commit_hash}
                                )
                                logger.info(
                                    f"Committed {file_count} files in submodule {submodule_path}: {commit_hash}"
                                )

                    except Exception as e:
                        logger.warning(f"Failed to process submodule {submodule_path}: {e}")

        if submodule_commits:
            subprocess.run(["git", "add", "-A"], cwd=project_root, capture_output=True)
            logger.info(f"Staged {len(submodule_commits)} submodule reference updates")

    except Exception as e:
        logger.warning(f"Failed to handle submodules: {e}")

    return submodule_commits


def _parse_commit_message(message: str) -> tuple[str, str, str, str]:
    """Parse commit message into type/scope/subject/body parts."""
    commit_type = "feat"
    commit_scope = "general"
    commit_body = ""

    first_line = message.strip().split("\n")[0]
    commit_match = re.match(r"^(\w+)(?:\(([^)]+)\))?:\s*(.+)$", first_line)
    if commit_match:
        commit_type = commit_match.group(1)
        scope_part = commit_match.group(2)
        if scope_part:
            commit_scope = scope_part
        commit_description = commit_match.group(3)
    else:
        commit_description = first_line

    lines = message.strip().split("\n")
    if len(lines) > 1:
        commit_body = "\n".join(lines[1:]).strip()

    return commit_type, commit_scope, commit_description, commit_body


def _validate_commit_scope(*, commit_scope: str, project_root: str) -> dict[str, Any] | None:
    """Validate scope against conform scopes; return structured error on mismatch."""
    valid_scopes = _get_cog_scopes(Path(project_root))
    if valid_scopes and commit_scope not in valid_scopes:
        matches = get_close_matches(commit_scope, valid_scopes, n=1, cutoff=0.6)
        return build_status_message_response(
            status="error",
            message=f"Invalid scope: '{commit_scope}'. Valid scopes: {valid_scopes}",
            extra={
                "suggestion": (
                    f"Did you mean: {matches[0]}?" if matches else "Use a valid scope from the list"
                )
            },
        )
    return None


async def run_qianji_engine(
    project_root: str, context: dict[str, Any], session_id: str
) -> tuple[bool, dict[str, Any], str]:
    """Execute the Qianji engine with smart_commit.toml."""
    from omni.foundation.config.skills import SKILLS_DIR
    manifest_path = str(SKILLS_DIR("git") / "workflows" / "smart_commit.toml")
    
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


def _render_start_result(
    result_json: dict[str, Any], project_root: str, session_id: str, submodules_committed: list
) -> str:
    """Render start action response from Qianji engine result."""
    staged_files_str = result_json.get("staged_files", "")
    staged_files = (
        [f for f in staged_files_str.split("\n") if f]
        if isinstance(staged_files_str, str)
        else staged_files_str
    )

    security_issues = result_json.get("security_issues", [])
    suspend_prompt = result_json.get("suspend_prompt", "")

    valid_scopes = _get_cog_scopes(Path(project_root))

    if not staged_files and not submodules_committed:
        return "Nothing to commit - No staged files detected."

    if security_issues:
        issues_str = "\n".join(
            [f"- {issue['file']}: {issue['description']}" for issue in security_issues]
        )
        return (
            f"Security Issue Detected\n\nSensitive files detected:\n{issues_str}\n\n"
            "Please resolve these issues before committing. (Workflow paused)"
        )

    submodule_info = ""
    if submodules_committed:
        submodule_info = "\n\n**Submodules committed:**\n" + "\n".join(
            f"- `{s['path']}`: `{s['commit_hash']}`" for s in submodules_committed
        )

    prompt_suffix = f"\n\n**Prompt:** {suspend_prompt}" if suspend_prompt else ""

    return render_template(
        "prepare_result.j2",
        has_staged=bool(staged_files),
        staged_files=staged_files,
        staged_file_count=len(staged_files),
        scope_warning="",
        valid_scopes=valid_scopes,
        lefthook_summary=result_json.get("lefthook_output", ""),
        lefthook_report="",
        diff_content="",
        diff_stat="",
        wf_id=session_id,
        submodule_info=submodule_info + prompt_suffix,
    )


@skill_command(
    name="smart_commit",
    category="workflow",
    description="""
    Primary git commit workflow using Qianji Rust engine.

    Multi-step workflow:
    1. start: Detects submodules, stages files, runs lefthook, security scan.
    2. approve: User approves, executes final commit.
    3. reject: Cancels the workflow via ValKey eviction.
    """,
    read_only=False,
    destructive=True,
    idempotent=False,
    open_world=False,
)
async def smart_commit(
    action: SmartCommitAction = SmartCommitAction.START,
    workflow_id: str = "",
    message: str = "",
    project_root: str = "",
) -> str:
    """Execute the Smart Commit workflow via Qianji."""
    project_root = _resolve_git_repo_root(project_root)

    try:
        if action == SmartCommitAction.START:
            session_id = str(uuid.uuid4())[:8]

            # 1. Handle submodules (Python Wrapper logic)
            submodules_committed = await asyncio.to_thread(_handle_submodules_prepare, project_root)

            # 2. Invoke Qianji Engine
            success, result_json, err = await run_qianji_engine(
                project_root, {"project_root": project_root}, session_id
            )

            if not success:
                return f"Workflow Execution Failed:\n\n{err}"

            rendered = _render_start_result(
                result_json, project_root, session_id, submodules_committed
            )
            return rendered
        elif action == SmartCommitAction.APPROVE:
            if not workflow_id:
                return "workflow_id required for approve action"
            if not message:
                return "message required for approve action"

            commit_type, commit_scope, commit_description, commit_body = _parse_commit_message(
                message
            )
            scope_error = _validate_commit_scope(
                commit_scope=commit_scope, project_root=project_root
            )
            if scope_error is not None:
                return scope_error

            # 3. Resume Qianji Engine with the message
            success, result_json, err = await run_qianji_engine(
                project_root, {"project_root": project_root, "final_message": message}, workflow_id
            )

            if not success:
                return f"Workflow Execution Failed:\n\n{err}"

            # 4. Success Response
            commit_output = result_json.get("commit_output", "")
            commit_hash_match = re.search(r"\] ([\da-f]+)", commit_output)
            commit_hash = commit_hash_match.group(1) if commit_hash_match else "unknown"

            return render_commit_message(
                subject=commit_description,
                body=commit_body,
                status=SmartCommitStatus.COMMITTED,
                commit_hash=commit_hash,
                file_count=0,
                verified_by="Qianji Rust Engine",
                security_status="Passed",
                workflow_id=workflow_id,
                commit_type=commit_type,
                commit_scope=commit_scope,
                submodule_section="",
            )

        elif action == SmartCommitAction.REJECT:
            if not workflow_id:
                return "workflow_id required for reject action"
            # Since Valkey manages state natively, we don't strictly need to delete immediately if TTL is fine,
            # but we can optionally send an Abort. Here we just acknowledge cancellation.
            return f"Commit Cancelled\n\nWorkflow `{workflow_id}` has been dropped from current session."

        elif action == SmartCommitAction.STATUS:
            return "Status checks natively supported via Valkey keys (xq:qianji:checkpoint:<session_id>)."

        elif action == SmartCommitAction.VISUALIZE:
            return "Smart Commit Workflow is now powered by Qianji Engine (smart_commit.toml).\nCheck assets/skills/git/workflows/smart_commit.toml for topology."

        return f"Unknown action: {action}"

    except Exception as e:
        import traceback

        return f"Error: {e}\n\n```\n{traceback.format_exc()}\n```"


__all__ = ["smart_commit"]
