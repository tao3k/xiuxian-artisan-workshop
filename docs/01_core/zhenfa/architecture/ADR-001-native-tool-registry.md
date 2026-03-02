---
type: knowledge
title: "ADR-001: Transition to Native In-Process Tool Registry"
status: "Accepted"
date: "2026-02-26"
category: "architecture"
tags:
  - zhenfa
  - adr
  - native-call
  - registry
metadata:
  title: "ADR-001: Transition to Native In-Process Tool Registry"
---

# ADR-001: Transition to Native In-Process Tool Registry

## 1. Context and Problem Statement

Initially, `xiuxian-zhenfa` was envisioned as a purely HTTP-based JSON-RPC gateway. The goal was to decouple the "Brain" (`omni-agent` / LLM) from the "Limbs" (`wendao` search, `qianji` workflows) using standard web protocols.

However, as we scaled up for complex scenarios like the "Adversarial Agenda Subgraph", profiling and architectural audits revealed severe bottlenecks:

- **Serialization Tax**: Converting massive datasets (e.g., thousands of LinkGraph hits from `wendao`) into JSON, transmitting them over HTTP, and deserializing them causes unacceptably high latency (often 50ms-100ms+ overhead per turn).
- **Latency Cascading**: In multi-step agent reasoning, these delays compound, shattering the "real-time" streaming experience for the user.
- **Context Pollution**: Returning deeply nested JSON arrays directly into the LLM's context window consumes massive token budgets and induces "Formatting Hallucinations."
- **State Fragmentation**: HTTP boundaries break Rust's native asynchronous context, making it difficult to share fast memory constructs (like `Arc<RwLock<T>>`) or cancellation tokens between the agent and its tools.

## 2. Decision

We are abandoning the "HTTP-only" microservices paradigm for internal tool execution.

Instead, we are adopting a **Microkernel & Native Plugins Architecture**, heavily inspired by the internal design of OpenAI's Codex (`ToolRegistry` and `ToolOrchestrator`).

1.  **In-Process Execution**: Core domain crates (`xiuxian-wendao`, `xiuxian-qianji`) will be compiled and linked directly into the `omni-agent` / `zhenfa` host process.
2.  **`ZhenfaRegistry` & `ZhenfaTool`**: We will introduce a centralized, memory-based registry (`HashMap<String, Arc<dyn ZhenfaTool>>`). The LLM orchestrator will invoke tools directly via Rust trait dynamic dispatch (`call_native`).
3.  **The Orchestrator & Valkey**: Execution will be wrapped by a `ZhenfaOrchestrator`, which intercepts all calls to enforce sandbox policies, handle Valkey-based distributed locking (for mutations), and cache "stripped" results.
4.  **The Stripping Layer**: Tools must not return raw JSON data intended for the LLM. They must return "Stripped Context" (lean XML-Lite strings) to maximize Attention Economy.
5.  **Dual-Mode Gateway**: The HTTP API (Axum) will be retained _only_ for external integrations (e.g., Python scripts, n8n), acting merely as a thin wrapper over the Native Registry.

## 3. Technical Design & Rust Contracts

### 3.1 The Tool Trait

All tools must implement the following trait to be mounted into the matrix:

```rust
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait ZhenfaTool: Send + Sync {
    /// Unique identifier for the LLM (e.g., "wendao.search")
    fn id(&self) -> &str;

    /// Returns the JSON Schema used to teach the LLM how to call this tool
    fn definition(&self) -> Value;

    /// The actual execution logic.
    /// Inputs: Validated JSON arguments from the LLM.
    /// Outputs: A pre-rendered, "Stripped" String (e.g., XML-Lite) ready for the LLM's context.
    async fn call_native(
        &self,
        ctx: &ZhenfaContext,
        args: Value
    ) -> Result<String, ZhenfaError>;
}
```

### 3.2 The Orchestrator

The orchestrator sits between the Agent and the Tool. It handles cross-cutting concerns:

```rust
pub struct ZhenfaOrchestrator {
    registry: ToolRegistry,
    valkey_client: Arc<ValkeyClient>, // For distributed locks, memoization, and telemetry
}

impl ZhenfaOrchestrator {
    pub async fn dispatch(&self, tool_id: &str, args: Value) -> Result<String, ZhenfaError> {
        // 1. Check Valkey for cached result
        // 2. Obtain Valkey mutation lock if tool modifies state
        // 3. Resolve tool from memory
        let tool = self.registry.get(tool_id).ok_or(ZhenfaError::NotFound)?;

        // 4. Native Execution (Zero-copy dispatch)
        let result = tool.call_native(&ctx, args).await?;

        // 5. Fire Webhook / Audit trail to Valkey
        // 6. Return Stripped XML to LLM
        Ok(result)
    }
}
```

## 4. Consequences

### Positive

- **Extreme Performance**: Tool invocation latency drops from milliseconds (HTTP/JSON) to microseconds (Pointer indirection).
- **Reduced Hallucinations**: Enforcing the "Stripping Layer" ensures the LLM only sees clean XML-Lite, saving tokens and improving reasoning accuracy.
- **Robust Telemetry**: Centralizing execution in the `ZhenfaOrchestrator` allows us to easily dump execution metrics to Valkey for the entire system without modifying individual tools.

### Negative / Trade-offs

- **Monolithic Binary**: The host executable (`omni-agent`) will become larger as it statically links all domain crates. (Acceptable trade-off for desktop/agent-OS scenarios).
- **Strict Trait Boundaries**: Domain teams must adapt their responses to return strings instead of complex domain structs when interfacing with Zhenfa.

## 5. Implementation Plan

1.  Create the native core (`ZhenfaTool`, `ZhenfaRegistry`, `ZhenfaOrchestrator`) in `packages/rust/crates/xiuxian-zhenfa/src/native/`. (Completed)
2.  Refactor `xiuxian-wendao`'s search entrypoint to implement `ZhenfaTool`, modifying its output to return lean `<hit>` XML tags.
3.  Refactor `xiuxian-qianhuan`'s template rendering to implement `ZhenfaTool`.
4.  Wire `omni-agent` to bypass HTTP and directly invoke `ZhenfaOrchestrator::dispatch`.
