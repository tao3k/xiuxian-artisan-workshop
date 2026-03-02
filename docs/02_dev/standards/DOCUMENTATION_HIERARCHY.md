---
type: knowledge
metadata:
  title: "Standard: Filesystem-Based Documentation Hierarchy (DFS-2026)"
---

# Standard: Filesystem-Based Documentation Hierarchy (DFS-2026)

> **Authority:** CyberXiuXian Artisan Studio  
> **Purpose:** To eliminate documentation entropy and provide a deterministic path-based indexing system for both Humans and LLMs.

## 1. Hierarchy Specification

All documentation must reside in one of the following six namespaces. Files outside these paths are considered "Unclassified" and must be audited or deleted.

### 1.1 `docs/00_vision/` (The "Why")

- **Responsibility:** High-level philosophy, Trinity/Quadrilateral Manifesto, and long-term strategic vision.
- **Stability:** High (rarely changes).
- **Primary Audience:** Humans (Stakeholders).

### 1.2 `docs/01_core/` (The "How")

- **Responsibility:** Technical specifications, mathematical models (PPR, CCS, MemRL), and architectural blueprints.
- **Structure:** Sub-divided by component (e.g., `wendao/`, `omega/`).
- **Primary Audience:** LLMs & Core Developers.

### 1.3 `docs/02_dev/` (The "Rules")

- **Responsibility:** Coding standards, workflows, traceability policies, and tool handbooks.
- **Primary Audience:** Developers.

### 1.4 `docs/03_features/` (The "Truth")

- **Responsibility:** Detailed documentation of **Done and Verified** features.
- **Requirement:** Every file here must link to verified integration tests in the `tests/` directory.
- **Primary Audience:** LLMs (Knowledge Retrieval) & Auditors.

### 1.5 `docs/04_chronicles/` (The "Past")

- **Responsibility:** Historical milestones, backlogs, archived plans, and retrospective reports.
- **Primary Audience:** Humans (Historical Context).

### 1.6 `docs/99_llm/` (The "Eyes")

- **Responsibility:** Machine-optimized indices, system context snapshots, and prompt injection contracts.
- **Primary Audience:** **LLMs Only**.

---

## 2. Naming Conventions

- **Case:** Use `SCREAMING_SNAKE_CASE.md` for root-level indices and manifestos.
- **Case:** Use `kebab-case.md` for detailed technical specs and features.
- **Prefixing:** Files within a component directory should prefix with the component name (e.g., `wendao-ppr-math.md`).
