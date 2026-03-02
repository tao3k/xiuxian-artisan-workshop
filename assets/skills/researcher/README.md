---
type: knowledge
metadata:
  title: "Researcher Skill"
---

# Researcher Skill

Repository deep-research capability driven by the Rust **Qianji** runtime.

## Overview

The researcher flow uses a suspend/resume protocol:

1. `start`: clone + map + propose shard plan.
2. human approval: review proposed shard JSON.
3. `approve`: resume with approved shards and run deep analysis.

This keeps long-running repository analysis explicit and auditable.

## Command Surface

Primary command: `git_repo_analyer`

Parameters:

- `repo_url` (required)
- `request` (optional)
- `action`: `start` or `approve`
- `session_id` (required for `approve`)
- `approved_shards` (required for `approve`, JSON string)

## Runtime

- Python entrypoint: `scripts/research_entry.py`
- Rust engine: `xiuxian-qianji`
- Workflow manifest: `workflows/repo_analyzer.toml`

## Utility Tools

`scripts/research.py` provides reusable helpers:

- `clone_repo`
- `repomix_map`
- `repomix_compress_shard`
- `init_harvest_structure`
- `save_shard_result`
- `save_index`

## Output

`start` returns:

- `success`
- `session_id`
- `proposed_plan`
- `message`

`approve` returns:

- `success`
- `session_id`
- `analysis_result`
- `full_context`

## Testing

Run researcher tests only:

```bash
uv run pytest assets/skills/researcher/tests -q
```

Run all skill tests:

```bash
omni skill test --all
```
