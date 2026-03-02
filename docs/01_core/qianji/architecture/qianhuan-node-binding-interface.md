---
type: knowledge
title: "Qianji-Qianhuan Interface: Node-Level Binding Specification"
category: "architecture"
tags:
  - qianji
  - qianhuan
  - interface
  - toml
saliency_base: 8.0
decay_rate: 0.02
metadata:
  title: "Qianji-Qianhuan Interface: Node-Level Binding Specification"
---

# Qianji-Qianhuan Interface: Node-Level Binding Specification

This specification defines the explicit data contract between the `xiuxian-qianji` workflow engine and the `xiuxian-qianhuan` manifestation engine.

To achieve **Multi-Persona Adversarial Loops** without polluting the core Python logic or the global Rust agent state, Qianji must be able to instruct Qianhuan on a _per-node_ basis.

## 1. The TOML Schema (`[nodes.qianhuan]`)

Every node defined in a Qianji manifest (`.toml`) can optionally include a `[nodes.qianhuan]` sub-table. This table acts as the configuration payload passed directly to the `ManifestationManager` when the node enters the `Transmuting` state.

### 1.1 Schema Definition

```toml
[[nodes]]
id = "Strict_Auditor_Node"
task_type = "llm_evaluation"
weight = 1.0

  # The explicit Qianhuan binding contract
  [nodes.qianhuan]
  # (Required) The ID of the persona to inject (must exist in PersonaRegistry)
  persona_id = "strict_architecture_auditor"

  # (Required) The template to render the final prompt (must be resolvable by ManifestationManager)
  template_target = "critique_report.j2"

  # (Optional) "isolated" (default) or "appended". Dictates context window boundaries.
  # See context-window-management.md for details.
  execution_mode = "isolated"

  # (Optional) Which outputs from previous nodes should be injected into the template context.
  input_keys = ["proposer_node.output_xml"]

  # (Optional) Under what key the omni-window history should be injected.
  history_key = "qianhuan_history"
```

## 2. The Rust Contract (Execution Flow)

When the Qianji engine executes a node, the following exact sequence MUST occur if a `[nodes.qianhuan]` block is present:

### Phase 1: Context Gathering (State Marshalling)

Qianji gathers data from its internal graph state based on the `input_keys`. It creates a JSON `Value` representing the raw data.
_Crucially, if `execution_mode = "isolated"`, it DOES NOT gather the raw conversational `messages` array._

### Phase 2: The Render Request

Qianji constructs a `ManifestationRenderRequest` and sends it across the boundary to Qianhuan:

```rust
// Conceptual pseudo-code for the interface boundary
let request = ManifestationRenderRequest {
    target: ManifestationTemplateTarget::Custom(node.qianhuan.template_target.clone()),
    data: gathered_inputs_json,
    runtime: ManifestationRuntimeContext {
        persona_id: Some(node.qianhuan.persona_id.clone()),
        state_context: Some("adversarial_loop".to_string()),
        domain: Some("workflow".to_string()),
        extra: extracted_history_if_applicable,
    },
};

// The handoff
let final_xml_prompt = manifestation_manager.render_request(&request)?;
```

### Phase 3: Isolated Execution

Qianji passes the `final_xml_prompt` to `xiuxian-llm`. Because the prompt is fully formed as XML, it is sent as a _single_ `system` or `user` message, guaranteeing that the LLM's attention mechanism is strictly bound to the injected persona and template.

## 3. Design Constraints

1. **No Implicit Fallbacks**: If `persona_id` is defined but missing from the `PersonaRegistry`, Qianji MUST fail the node execution. It must not silently fall back to a default persona during an adversarial loop.
2. **Immutability**: The `[nodes.qianhuan]` definition is static per node. Dynamic persona shifting must occur by routing to a _different_ node in the DAG, not by mutating the current node's configuration at runtime.
