---
type: knowledge
metadata:
  for_tools: researcher.git_repo_analyer
  title: Run Repository Analyzer Workflow
  description: How to use the Qianji-based repository analyzer workflow.
  routing_keywords:
    - "research"
    - "graph"
    - "workflow"
    - "url"
    - "github url"
    - "repository url"
    - "research url"
    - "repo url"
  intents:
    - "Deep research"
    - "Repository analysis"
    - "Run the sharded deep research workflow on a repo URL"
    - "Help me analyze or research a GitHub repository with a structured workflow"
    - "I want to research this repository and get an index of analyses"
---

# Run Repository Analyzer Workflow

This document describes how to use the `git_repo_analyer` command.

## Overview

Execute the sharded repository analysis workflow via Qianji with explicit start/approve actions.

## Args

- **repo_url** (required): Git repository URL to analyze.
- **request** (optional): Specific analysis goal (default: "Analyze the architecture").
- **action** (optional): `start` or `approve` (default `start`).
- **session_id** (required for `approve`): Session id returned by `start`.
- **approved_shards** (required for `approve`): Approved shards JSON string.

## Example

```python
@omni("researcher.git_repo_analyer", {"repo_url": "https://github.com/owner/repo", "request": "Analyze the architecture", "action": "start"})
```

## See also

- [SKILL.md](../SKILL.md) for the researcher skill overview.
