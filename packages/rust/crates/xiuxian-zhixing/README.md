---
type: knowledge
metadata:
  title: "xiuxian-zhixing"
---

# xiuxian-zhixing

> Naming origin: "Xiuxian" (修仙) and "Zhixing-Heyi" (知行合一), used in this project to represent a knowledge-to-action execution loop.

`xiuxian-zhixing` is the execution and manifestation engine in the Xiuxian ecosystem. It bridges structured knowledge and real-world action.

## Philosophy

Inspired by Wang Yangming's "unity of knowledge and action", this package focuses on turning intent into verifiable outcomes:

- knowledge is represented in graph and notebook artifacts,
- action is represented in executable tasks, reminders, and review loops,
- reflection feeds back into the next execution cycle.

## Core Objectives

1. Manifestation over planning: if a task is accepted into agenda state, it should become actionable and observable.
2. Markdown-first integration: agenda/journal persistence uses markdown artifacts while remaining graph-indexable.
3. Feedback loop: reflections can influence future scheduling and execution decisions.

## Architecture Role

- `xiuxian-wendao`: retrieval, indexing, and graph representation layer.
- `xiuxian-zhixing`: agenda/journal/reminder/blocker domain layer.
- `xiuxian-qianhuan`: rendering/injection layer for human-readable output.

## Related Documentation

- `docs/03_features/xiuxian_zhixing_heyi.md`
- `docs/99_llm/xiuxian_zhixing_theory.md`
