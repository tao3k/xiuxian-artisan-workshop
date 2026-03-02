---
type: knowledge
title: "ADR-011: The Transmutation Layer (Zhenfa Data Port)"
status: "Accepted"
date: "2026-02-28"
category: "architecture"
tags:
  - zhenfa
  - protocol
  - parsing
  - standardization
metadata:
  title: "ADR-011: The Transmutation Layer (Zhenfa Data Port)"
---

# ADR-011: The Transmutation Layer (Zhenfa Data Port)

## 1. Context and Problem Statement

Data processing logic—such as XML parsing, Markdown versioning, and LLM output cleaning—is currently fragmented. To maintain architectural integrity, we must centralize these responsibilities while ensuring that high-level engines like `Qianji` remain focused on orchestration rather than data formats.

## 2. Decision: Ecological Alignment & The Central Artery

We will implement an **Ecological Alignment** strategy where each crate focuses on its primary domain while linking through standardized interfaces.

### 2.1 Separation of Concerns

- **`xiuxian-qianji` (Orchestration)**: A pure workflow engine. It identifies semantic placeholders (`$`) but **delegates** their resolution to Zhenfa.
- **`xiuxian-zhenfa` (Protocol Hub)**: The "Immune System" and "Data Port." It provides the unified `resolve_and_wash()` interface.
- **`xiuxian-wendao` (Data Source)**: The "Memory Bus." It provides raw bytes to Zhenfa upon request.

### 2.2 The `resolve_and_wash` Artery

This chain represents the standard path for any resource to enter an LLM context:

1.  **Request**: `Qianji` encounters a placeholder and calls `zhenfa::resolve_and_wash(uri)`.
2.  **Resolution**: `Zhenfa` requests raw data from `Wendao` via the `wendao://` protocol.
3.  **Transmutation**: `Zhenfa` performs XML-Lite cleaning, structural validation, and semantic sealing.
4.  **Delivery**: `Zhenfa` returns a "Sanitized String" back to `Qianji` for immediate node execution.

## 3. Technical Design: The Unified Interface

### 3.1 ZhenfaTransmuter (The Ecological Gatekeeper)

`xiuxian-zhenfa` is the sole authority for:

- **XML-Lite Extraction**: Identifying and stripping `<tag>` values.
- **Structural Validation**: Ensuring XML/MD integrity before LLM presentation.
- **Format Normalization**: Standardizing diverse inputs into clean LLM context.

### 3.2 Late Binding Implementation

The `Qianji` runtime intercepts node parameters. If a value starts with `$`, it triggers the `resolve_and_wash()` artery. This ensures that even dynamic content (like search hits) is "washed" by the same protocol center as static assets.

## 4. Consequences

### Positive

- **Architectural Sanctity**: Each crate stays within its domain boundary.
- **Unified Quality**: All data fed to LLMs passes through a single, rigorous validation center.
- **Future-Proof**: Changes to the data format only affect `xiuxian-zhenfa`.

## 5. Implementation Plan

1.  **Develop `resolve_and_wash`** in `xiuxian-zhenfa`.
2.  **Integrate with `Qianji`**: Add the placeholder resolver to the workflow engine.
3.  **Refactor Workflows**: Update all TOMLs to use the `$` mapping syntax.
