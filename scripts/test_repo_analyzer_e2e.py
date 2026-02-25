import asyncio
import sys
from pathlib import Path

# Setup PYTHONPATH to include assets/skills
root_dir = Path(__file__).parent.parent
sys.path.insert(0, str(root_dir / "assets/skills"))

from researcher.scripts.research_entry import run_research_graph


async def main():
    print("Testing Repo Analyzer Start...")

    # We will test on a very small public repo or just the local dir
    repo_url = "https://github.com/tao3k/omni-dev-fusion.git"

    start_result = await run_research_graph(
        repo_url=repo_url, request="Analyze the top-level architecture.", action="start"
    )

    start_payload = (
        start_result["content"][0]["text"]
        if isinstance(start_result, dict) and "content" in start_result
        else start_result
    )
    print("\\nSTART RESULT:")
    print(start_payload)

    import json

    try:
        start_data = json.loads(start_payload)
    except:
        start_data = start_result if isinstance(start_result, dict) else {}

    if not start_data.get("success"):
        print("Failed to start workflow.")
        return

    session_id = start_data.get("session_id")
    print(f"\\nExtracted session_id: {session_id}")

    # Normally we would review proposed_plan here

    # Now simulate approval
    print("\\nSimulating approval with dummy shards...")

    approve_result = await run_research_graph(
        repo_url=repo_url,
        request="Analyze the top-level architecture.",
        action="approve",
        session_id=session_id,
        approved_shards='[{"shard_id": "test", "paths": ["src/"]}]',
    )

    print("\\nAPPROVE RESULT:")
    print(approve_result)


if __name__ == "__main__":
    asyncio.run(main())
