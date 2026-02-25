# xiuxian-zhixing (修仙-知行合一)

> "To know and not to do is not to know." (知而不行，只是未知) —— Wang Yangming (王阳明)

`xiuxian-zhixing` is the execution and manifestation engine of the Xiuxian ecosystem. It bridges the gap between structured knowledge (Zhi/知) and real-world action (Xing/行).

## Philosophy / 哲学理念

Inspired by Wang Yangming's **"Unity of Knowledge and Action" (知行合一)**, this package focuses on the realization of intentions.

- **Zhi (Knowledge/知)**: Insights, plans, and cultivation techniques stored in the Knowledge Graph (`xiuxian-wendao`).
- **Xing (Action/行)**: The actual completion of tasks and the application of learned wisdom.

In this system, a schedule is not just a list of strings; it is a **Vow (愿)**. To learn a technique is to practice it; to set an agenda is to manifest it.

本项目深受王阳明**“知行合一”**心学启发，是修仙生态中的执行与显化引擎。它连接了结构化的“学识”（知）与现实世界的“落地”（行）。

- **知**：存储在知识图谱（`xiuxian-wendao`）中的见解、计划和修炼法门。
- **行**：任务的实际完成以及所学智慧的实践应用。

在这个系统中，一份议程不仅仅是文字记录，它是一份**“宏愿”**。学到了，就要做到；制定了计划，就要去圆满它。

## Core Objectives

1.  **Manifestation over Planning**: Focus on the completion of the "Backlog". If it is in the Agenda, it must be reflected in the user's reality.
2.  **Markdown-First Integration**: Prioritize Markdown for journal and agenda storage, utilizing `xiuxian-wendao` as the powerful underlying engine for parsing and searching.
3.  **Feedback Loop**: Reflections in the Journal are analyzed by LLMs to update the Knowledge Graph, which in turn refines the future Agenda.

## Architecture Role

- **xiuxian-wendao**: The "Brain". Handles parsing (`.md`, `.org`), indexing, keywords, and semantic search. It is the Source of Truth for "Knowledge".
- **xiuxian-zhixing**: The "Hands". Handles the logic of "Doing". It tracks task states, manages schedules, and ensures that what is known is being acted upon.
