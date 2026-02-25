# Theory & Implementation: Xiuxian-Zhixing-Heyi (The Manifestation Paradigm)

## Core Philosophy / 核心理念 (2025 Revised)

### 1. The Unity of System and Intelligence (系统与模型的合一)

The ultimate user experience is not solved by the system alone, nor by the LLM alone, but by their seamless integration:

- **System (The Anchor)**: Provides deterministic search (`wendao`), time-series tracking (`zhixing`), and structural templates (`qianhuan`).
- **LLM (The Soul)**: Provides semantic understanding, causal reasoning, and empathetic interaction.

### 2. Search-Driven Manifestation (搜索驱动的显化)

If it can be searched by `wendao`, it can be manifested by the Agent.

- `wendao` acts as a **Spatio-Temporal Search Engine**, capable of retrieving data across days, weeks, months, or specific entities.
- The LLM uses these search results as "Context Seeds" to generate personalized advice and reflections.

---

## Engineering Design / 工程设计

### 1. The TTP (Tool-Template-Persona) Pipeline

To provide the best UX without reinventing the wheel:

1.  **Search Phase**: LLM calls `wendao` search tools based on user intent.
2.  **Context Injection**: `zhixing` logic (like TTL/Status) is attached to the search results.
3.  **Template Rendering**: `qianhuan` applies Markdown templates tailored for Discord/Telegram, ensuring beautiful formatting.
4.  **Final Polish**: The LLM wraps the rendered content in a persona-driven narrative (The Steward's voice).

### 2. Collaborative Logic (协同逻辑)

- **Zhixing** handles the "Hard Logic" (When does a task expire? How does the state transition?).
- **Wendao** handles the "Hard Data" (Where is this file? What are the keywords?).
- **Qianhuan** + **LLM** handles the "Soft Experience" (How should this look to the user? What advice should be given?).

---

## Manifestation Strategies / 显化策略

- **Multi-platform Adaptability**: Automated rendering for Discord (Embeds/Markdown) and Telegram (Markdown v2/HTML).
- **Proactive Check-ins**: When `zhixing` detects a TTL expiry, it triggers the LLM to initiate a conversation, not just a notification.
- **Dynamic Dashboarding**: Generating summaries on-the-fly based on any search query (e.g., "Show my progress on Rust projects this month").
