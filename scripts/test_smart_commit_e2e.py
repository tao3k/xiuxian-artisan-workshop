import asyncio
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

# Setup PYTHONPATH to include assets/skills
root_dir = Path(__file__).parent.parent
sys.path.insert(0, str(root_dir / "assets/skills"))

from git.scripts.smart_commit_graphflow.commands import smart_commit
from git.scripts.smart_commit_graphflow._enums import SmartCommitAction


async def main():
    base_dir = Path("/tmp/test_smart_commit_repo")
    if base_dir.exists():
        shutil.rmtree(base_dir)

    base_dir.mkdir(parents=True)

    print(f"Initializing test repository at {base_dir} ...")

    # Init main repo
    subprocess.run(["git", "init"], cwd=base_dir, check=True, capture_output=True)
    (base_dir / ".gitignore").write_text("*.tmp")
    subprocess.run(["git", "add", ".gitignore"], cwd=base_dir, check=True)
    subprocess.run(["git", "commit", "-m", "init main"], cwd=base_dir, check=True)

    # Create sub1
    sub1_dir = base_dir / "sub1"
    sub1_dir.mkdir()
    subprocess.run(["git", "init"], cwd=sub1_dir, check=True, capture_output=True)
    (sub1_dir / "file1.txt").write_text("hello sub1")
    subprocess.run(["git", "add", "."], cwd=sub1_dir, check=True)
    subprocess.run(["git", "commit", "-m", "init sub1"], cwd=sub1_dir, check=True)

    # Create sub2
    sub2_dir = base_dir / "sub2"
    sub2_dir.mkdir()
    subprocess.run(["git", "init"], cwd=sub2_dir, check=True, capture_output=True)
    (sub2_dir / "file2.txt").write_text("hello sub2")
    subprocess.run(["git", "add", "."], cwd=sub2_dir, check=True)
    subprocess.run(["git", "commit", "-m", "init sub2"], cwd=sub2_dir, check=True)

    # Add submodules to main repo
    subprocess.run(
        ["git", "submodule", "add", "./sub1", "sub1"], cwd=base_dir, check=True, capture_output=True
    )
    subprocess.run(
        ["git", "submodule", "add", "./sub2", "sub2"], cwd=base_dir, check=True, capture_output=True
    )
    subprocess.run(
        ["git", "commit", "-m", "add submodules"], cwd=base_dir, check=True, capture_output=True
    )

    print("Modifying submodules and main repo...")
    # Modify submodules
    (sub1_dir / "file1.txt").write_text("hello sub1 modified\\nwith more text\\n")
    (sub2_dir / "file2.txt").write_text("hello sub2 modified\\n")

    # Modify main repo
    (base_dir / "main.txt").write_text("hello main\\nthis is a new file\\n")

    print("\\n=============================================")
    print("--- Running smart_commit START Action ---")
    print("=============================================\\n")

    start_result_dict = await smart_commit(
        action=SmartCommitAction.START, project_root=str(base_dir)
    )
    print("START RAW DICT:", repr(start_result_dict))

    start_result = (
        start_result_dict["content"][0]["text"]
        if isinstance(start_result_dict, dict)
        else start_result_dict
    )
    print("START RESULT:")
    print(repr(start_result))

    # Parse workflow ID
    match = re.search(r"workflow_id='([^']+)'", start_result)
    if not match:
        print("\\n[ERROR] Failed to find workflow_id in start result!")
        return

    wf_id = match.group(1)
    print(f"\\n✅ Extracted workflow_id: {wf_id}")

    print("\\n=============================================")
    print("--- Running smart_commit APPROVE Action ---")
    print("=============================================\\n")

    approve_result_dict = await smart_commit(
        action=SmartCommitAction.APPROVE,
        workflow_id=wf_id,
        message="feat(core): test smart commit end to end\\n\\n- Updated submodules\\n- Added main.txt",
        project_root=str(base_dir),
    )
    approve_result = (
        approve_result_dict["content"][0]["text"]
        if isinstance(approve_result_dict, dict)
        else approve_result_dict
    )
    print(approve_result)

    print("\\n=============================================")
    print("--- Verification ---")
    print("=============================================\\n")

    proc = subprocess.run(
        ["git", "log", "-1", "--stat"], cwd=base_dir, capture_output=True, text=True
    )
    print("Final commit in main repo:")
    print(proc.stdout)

    proc = subprocess.run(
        ["git", "log", "-1", "--oneline"], cwd=sub1_dir, capture_output=True, text=True
    )
    print("Final commit in sub1:")
    print(proc.stdout)


if __name__ == "__main__":
    asyncio.run(main())
