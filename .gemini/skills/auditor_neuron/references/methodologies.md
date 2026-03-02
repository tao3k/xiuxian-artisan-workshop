# The Knowledge Fortress: Global Quality Standards (V3.0)

## 1. Blueprint-Driven Evolution (The First Law)

- **Standard**: No implementation shall occur without an approved **Draft Blueprint** in `.data/blueprints/`.
- **Audit Rule**: Cross-reference the physical `.rs` code against the logical intent in the linked blueprint. Any deviation without technical justification is a blocker.

## 2. Hyper-Modularity & Namespace Sovereignty

- **Standard**: Logic must be surgically split into domain-specific modules.
- **Audit Rule**:
  - Reject any file exceeding 300 lines.
  - Verify that every symbol belongs to its declared namespace.
  - Prohibit "logic bleeding" between the core and the executors.

## 3. Test Isolation & Integrity

- **Standard**: Tests must be physically isolated from production logic.
- **Audit Rule**:
  - Unit tests MUST reside in `mod tests` or a dedicated `tests/` directory.
  - Integration tests must be contained in the package root's `tests/` folder.
  - Standard: "One Logic, One Test File."

## 4. Performance & Memory (Zero-Copy)

- **Standard**: Zero-copy via `Arc<str>` or `SharedString` in all VFS/LLM paths.
- **Audit Rule**: Flag any `String::clone()` or `.to_string()` in hot code paths as a performance blocker.

## 5. Physical Structural Integrity (The Red Line)

- **Standard**: `SKILL.md` is the only physical blocker for discovery.
- **Audit Rule**:
  - Enforce the **Authorized Scope** (required + default + optional).
  - Flag any undefined physical entry as an "Out-of-Scope Warning."

## 6. Dashboard Implementation Protocol (NEW)

### 6.1 Alchemical Implementation Plan (AIP)

Before any implementation begins, the Auditor MUST output an AIP Dashboard.

- **Goal**: Align the Sovereign's implementation steps with the logical Blueprint.

### 6.2 Artisan Audit Verdict (AAV)

After implementation, the Auditor MUST output an AAV Dashboard.

- **Goal**: Provide a definitive quality score and refinement path based on the rules above.
