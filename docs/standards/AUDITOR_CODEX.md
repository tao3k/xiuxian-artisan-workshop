---
type: knowledge
title: "The Auditor's Codex: Engineering Standards & Quality Gates"
category: "standards"
tags:
  - auditing
  - performance
  - quality
  - rust
metadata:
  title: "The Auditor's Codex (V10.0 - High-Standard Era)"
---

# The Auditor's Codex (V10.0 - High-Standard Era)

This document defines the **Mandatory Engineering Standards** for the CyberXiuXian Workshop. All implementations must pass these gates.

## 1. High-Standard Code Quality (The Artisan's Guard)

- **Hyper-Modularity**: Logic must be split into fine-grained modules. No file shall exceed 300 lines without a modularity review.
- **Namespace Sovereignty**: Every symbol and constant must reside in its specific domain. Zero "misc" or "util" buckets.
- **Test Isolation**:
  - Unit tests MUST reside in `mod tests` or a dedicated `tests/` directory.
  - Integration tests MUST NOT pollute the `src/` directory.
  - Standard: "One Logic, One Test File."

## 2. Safety & Physical Integrity

- [SKILL-ANCHOR]: `SKILL.md` is the only physical blocker for discovery.
- [SCOPE-VIGILANCE]: Any file outside the `skills.toml` authorized set triggers a warning.
- [ZERO-LEAKAGE]: System-level errors must be scrubbed by `ZhenfaTransmuter`.

## 3. Performance & Memory

- [ZERO-COPY]: Mandatory `Arc<str>` for resource sharing.
- [PARALLEL]: Mandatory `rayon` for all traversals.

## 4. Operational Governance

- [BLUEPRINT-FIRST]: Every non-trivial task requires a **Draft Blueprint** in `.data/blueprints/` before implementation.
- [AUDIT-ONLY]: The Auditor has no write-access to source code (`.rs`, `.toml`). Action is taken only through the Sovereign's implementation.
