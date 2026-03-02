# The Forge Laws: Universal Statutes for Soul Synthesis (V2.0)

This document defines the high-standard alchemical laws enforced by the Soul-Forge. Any synthesized persona must align with these statutes to pass the Forge-Guard.

## Law 1: Structural Integrity (V8.0 Alignment)

- **Top-Level Discovery**: Every persona MUST contain `name` and `description` anchors at the absolute top of the YAML frontmatter.
- **Anthropic Nesting**: All artisan descriptors (seniority, ethos, role_class) MUST be nested under the `metadata:` block.
- **Type Identity**: `type: persona` must be explicitly declared.

## Law 2: Identity Fidelity (Precision Positioning)

- **Seniority**: Every soul must possess 10+ years of domain seniority to ensure authoritative reasoning.
- **Professional Ethos**: A non-negotiable code of conduct must be defined.
- **Classification**: `role_class` must be chosen from [Auditor, Executor, Manager, Visionary].

## Law 3: Methodology Reliance (The Fortress)

- **Bible Linking**: New souls must not embed knowledge; they must link to existing or synthesized methodology files via `require_refs`.
- **Adversarial Mandate**: The soul must be instructed to challenge existing plan logic using its Knowledge Fortress.

## Law 4: The Possession Protocol

- **Synaptic Mantra**: Every soul definition MUST conclude with a direct `!SWITCH_TRIGGER` block using the **COMMAND / MISSION / OUTPUT** structure.
