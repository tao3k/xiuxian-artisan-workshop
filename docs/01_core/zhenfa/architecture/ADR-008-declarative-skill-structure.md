---
type: knowledge
title: "ADR-008: Declarative Skill Structure and Validation"
status: "Accepted"
date: "2026-02-28"
category: "architecture"
tags:
  - zhenfa
  - scanner
  - validation
  - config
metadata:
  title: "ADR-008: Declarative Skill Structure and Validation"
---

# ADR-008: Declarative Skill Structure and Validation

## 1. Context and Problem Statement

The physical directory structure of an **Agent Skill** (e.g., `SKILL.md`, `scripts/`, `references/`) is currently defined by hardcoded `Default` implementations within the `xiuxian-skills` crate or scattered in legacy `settings.yaml` files.

This creates several issues:

- **Inflexibility**: Users cannot easily define or enforce their own organizational standards for skills without changing code.
- **Verification Gap**: There is no centralized, single-source-of-truth validation engine to ensure that new skills comply with our quality standards (e.g., ensuring all docs are in `references/`).
- **Legacy Debt**: The project is transitioning to a TOML-first configuration paradigm, rendering the YAML-based `settings.yaml` obsolete.

## 2. Decision

We will transition to a **Declarative Skill Structure** governed by a centralized TOML configuration. The `xiuxian-skills` utility will be refactored to become configuration-driven, acting as the primary validation gatekeeper for the system.

### 2.1 Centralized Configuration

A new configuration file, `packages/rust/crates/xiuxian-skills/resources/config/skills.toml`, will serve as the "Constitution" for skill organization. It will define:

- `required`: Files/Directories that MUST exist for a skill to be considered valid.
- `default`: The canonical layout used when scaffolding a new skill.
- `validation`: Semantic rules, such as prohibiting logic in `SKILL.md` or enforcing the `references/` hierarchy.

### 2.2 Configuration-Driven `xiuxian-skills`

The `SkillStructure` model in `xiuxian-skills` will be updated to support deserialization from this TOML. Hardcoded defaults will be removed.

### 2.3 Extensibility

Users can override or extend these rules in their local `xiuxian.toml` under the `[skills.architecture]` key, allowing for domain-specific constraints (e.g., "all research skills must contain a `papers/` folder").

## 3. Technical Design

### 3.1 The `skills.toml` Schema (Conceptual)

```toml
[architecture]
required = [
    { path = "SKILL.md", description = "Skill manifest" }
]
default = [
    { path = "scripts/", item_type = "dir" },
    { path = "references/", item_type = "dir" }
]

[validation]
strict_mode = true
enforce_references_folder = true
```

### 3.2 Integrated Validation Flow

When the `Wendao` indexer or the `Agent` boots up:

1. Load `skills.toml`.
2. `SkillScanner` iterates through `assets/skills/*`.
3. For each folder, `scanner.validate(path, config)` is called.
4. If validation fails, the skill is ignored or flagged with a warning, preventing the injection of malformed tools into the LLM context.

### 3.3 Internalized Resource Layout (Crate Self-Containment)

To eliminate path ambiguity and ensure portability, all system-level "Constitutions" (like `skills.toml`) will be stored within the **internal `resources/` directory** of the managing crate.

- **Standard Paths**:
  - `packages/rust/crates/xiuxian-skills/resources/config/skills.toml`
  - `packages/rust/crates/omni-agent/resources/config/xiuxian.toml`
- **Mechanism**: The crate uses a direct, non-escaping `include_str!("resources/config/skills.toml")`.
- **Portability**: This architecture makes the `xiuxian-skills` crate completely self-sufficient. It carries its own validation laws within its binary, requiring no external files to perform a baseline structural check.

## 4. Consequences

### Positive

- **Architectural Guardrails**: Forces all AI and human developers to follow the same organizational pattern.
- **Scalability**: New file types (e.g., `assets/`, `config/`) can be added to the standard without touching core logic.
- **Improved DX**: Clear, error-driven feedback when a developer misplaces a file.

### Negative

- **Startup Overhead**: Minor latency increase during boot to perform the structure scan (mitigated by Rust's speed and optional caching).

## 5. Implementation Plan

1.  **Create `packages/rust/crates/xiuxian-skills/resources/config/skills.toml`** using the finalized structure.
2.  **Refactor `xiuxian-skills`**: Update `structure.rs` to derive `serde::Deserialize` and implement the loading logic.
3.  **Update CLI**: Add a `just validate-skills` command to trigger the scanner's new validation mode.
