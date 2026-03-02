---
type: knowledge
metadata:
  title: "Rust Embedding Stack Audit: Role-Oriented `litellm-rs` + `mistralrs` Integration"
---

# Rust Embedding Stack Audit: Role-Oriented `litellm-rs` + `mistralrs` Integration

Date: 2026-02-24  
Scope: current workspace state only (local source and local benchmarks)

Positioning note:

- This audit does **not** treat `litellm-rs` and `mistralrs` as competing alternatives.
- They are evaluated as different layers in one stack: provider routing (`litellm-rs`) and local runtime (`mistralrs`).

## 1. Method

This audit is based on:

- Workspace code under `packages/rust/crates/*`
- Local Cargo source cache for `litellm-rs`:
  - `/Users/guangtao/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/litellm-rs-0.3.1/`
- Local benchmark artifacts:
  - `/tmp/omni_embed_bench_nocache_1771964244.json`
  - `/tmp/omni_embed_bench_1771964165.json` (cache-affected, not used for final conclusion)

No external decision claims are required for this report.

## 2. Current Integration Reality

### 2.1 `litellm-rs` is in dependency graph; `mistralrs` crate is not

- `omni-agent` depends on `litellm-rs = 0.3.1` behind feature `agent-provider-litellm`:  
  `packages/rust/crates/omni-agent/Cargo.toml`
- `cargo metadata` for the workspace contains `litellm-rs` package, but no `mistralrs*` crate package.
- `xiuxian-llm` currently has no `mistralrs` crate dependency; it only contains runtime/process wrappers:  
  `packages/rust/crates/xiuxian-llm/Cargo.toml:10`

### 2.2 `mistralrs` in this repo is currently an external process boundary

- The current implementation is process lifecycle and health probing for `mistralrs-server`, not in-process model inference:
  - `packages/rust/crates/xiuxian-llm/src/mistral/config.rs:3`
  - `packages/rust/crates/xiuxian-llm/src/mistral/process.rs:24`
- `omni-agent` gateway can auto-start this external server when backend hint is mistral:
  - `packages/rust/crates/omni-agent/src/gateway/http/runtime.rs:91`

### 2.3 Protocol compatibility clarification: why the module is named `openai_compat`

- `openai_compat` is used to express API protocol compatibility (`/v1/embeddings` request/response shape), not provider/vendor lock-in.
- In this workspace, the same protocol path is used for local serving scenarios (for example local Ollama OpenAI-compatible endpoint and future local runtimes), not only OpenAI-hosted inference.
- Current implementation evidence:
  - URL normalization to `/v1/embeddings`: `packages/rust/crates/xiuxian-llm/src/embedding/openai_compat.rs:22`
  - OpenAI-compatible embedding execution entrypoint: `packages/rust/crates/xiuxian-llm/src/embedding/openai_compat.rs:35`

### 2.4 Responsibility boundary matrix (authoritative)

| Component                                           | Primary responsibility                                                | Requires API key                           | Current role                            |
| --------------------------------------------------- | --------------------------------------------------------------------- | ------------------------------------------ | --------------------------------------- |
| `xiuxian-llm::mistral`                              | Manage local `mistralrs-server` process lifecycle + readiness probe   | No                                         | Local runtime control plane             |
| OpenAI-compatible HTTP path (`openai_http`)         | Call generic `/v1/embeddings` protocol endpoint (local or remote)     | Not inherently; depends on upstream policy | Generic protocol path                   |
| Local mistral runtime path (`mistral_sdk`)          | Explicit local `mistralrs-server` runtime route over `/v1/embeddings` | No                                         | Preferred local runtime path            |
| `litellm-rs` embedding provider path (`litellm_rs`) | In-process provider routing for API providers                         | Yes for provider-backed routes             | Provider fallback / cloud-provider path |

Code anchors:

- `mistralrs-server` lifecycle wrapper: `packages/rust/crates/xiuxian-llm/src/mistral/mod.rs:1`
- Mistral auto-start gate in gateway runtime: `packages/rust/crates/omni-agent/src/gateway/http/runtime.rs:195`
- Embedding backend kinds (`http`/`openai_http`/`mistral_sdk`/`litellm_rs`): `packages/rust/crates/xiuxian-llm/src/embedding/backend.rs:1`
- `litellm_rs` dispatch behavior and no-key skip logic: `packages/rust/crates/omni-agent/src/embedding/client/backend_dispatch.rs:86`

### 2.5 Selection rules: backend label vs model prefix

- Backend label decides transport family:
  - `http` -> `/embed/batch`
  - `openai_http` -> generic `/v1/embeddings`
  - `mistral_sdk` (aliases `mistral`, `mistral_rs`, `mistral_server`) -> local `mistralrs-server` route
  - `litellm_rs` -> provider route (with Rust HTTP fallback chain)
- LLM backend labels now follow the same explicit split:
  - `mistral_sdk` stays first-class (not collapsed into generic `http`)
  - dispatch remains OpenAI-compatible HTTP transport for now, but with explicit mode identity for routing/logging/config resolution
- Model prefix only affects behavior inside selected backend:
  - Under `litellm_rs`, `ollama/...` has dedicated direct-path optimization and Rust fallback chain.
  - Under `litellm_rs`, `mistral/...` is treated as provider model id; when no API key is configured, provider path is skipped and runtime stays on Rust HTTP paths.

Code anchors:

- Alias normalization (including `mistral*` -> `mistral_sdk`): `packages/rust/crates/xiuxian-llm/src/embedding/backend.rs:38`
- `litellm_rs` model-specific branching: `packages/rust/crates/omni-agent/src/embedding/client/backend_dispatch.rs:104`

## 3. Key Findings from `litellm-rs` Source Audit

### 3.1 Embedding router is functionally narrower than provider surface

- `core::embedding::EmbeddingRouter` only auto-registers OpenAI with `OPENAI_API_KEY`:  
  `.../litellm-rs-0.3.1/src/core/embedding/router.rs:27`
- Dynamic embedding path requires `options.api_key` and builds an OpenAI provider path:  
  `.../core/embedding/router.rs:170`
- If no matched provider exists, it emits:
  - `"No embedding provider found for '...'. Make sure the API key is set."`  
    `.../core/embedding/router.rs:111`

This exactly explains the previous runtime symptom for `ollama/...` model with missing/incorrect key path.

### 3.2 `OllamaProvider` embedding support exists but is not used by the default embedding router

- `OllamaProvider` implements `LLMProvider::embeddings` and calls Ollama `/api/embed` behavior:
  - `.../core/providers/ollama/provider.rs:608`
- It strips `ollama/` prefix and parses Ollama embedding payload:
  - `.../core/providers/ollama/provider.rs:615`

This means the capability exists in crate code, but default `core::embedding` path does not route to it.

### 3.3 Provider factory maturity gap

- `create_provider(...)` returns `not_implemented` placeholder:
  - `.../core/providers/mod.rs:705`
  - `.../core/providers/mod.rs:729`

This is a maintainability risk if we expect consistent provider creation across all call paths.

### 3.4 HTTP execution path does not enforce status before parsing in OpenAI embedding method

- Shared request executor returns `reqwest::Response` directly without status-based error mapping:
  - `.../core/providers/base/connection_pool.rs:177`
- OpenAI embedding method immediately parses response body JSON:
  - `.../core/providers/openai/api_methods.rs:55`
  - `.../core/providers/openai/api_methods.rs:69`

This can produce parse errors when upstream sends non-JSON error payloads.

### 3.5 OpenAI config validation is strict on key prefix

- OpenAI config requires key prefix `sk-` or `sk-proj-`:
  - `.../core/providers/openai/config.rs:137`

This is incompatible with generic placeholder-key strategy unless caller bypasses this path.

### 3.6 `litellm-rs` Mistral provider is API-provider integration, not `mistralrs` runtime embedding

- `litellm-rs` has a dedicated Mistral provider module:
  - `/Users/guangtao/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/litellm-rs-0.3.1/src/core/providers/mistral/mod.rs`
  - `/Users/guangtao/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/litellm-rs-0.3.1/src/core/providers/mistral/embedding.rs`
- The provider config defaults to `https://api.mistral.ai/v1` and requires a Mistral API key:
  - `.../providers/mistral/mod.rs:47`
  - `.../providers/mistral/mod.rs:59`
  - `.../providers/mistral/mod.rs:68`
- This is a different integration class than our workspace `xiuxian-llm::mistral` process-wrapper path for local `mistralrs-server`.

## 4. Omni-Side Mitigations Already Present

`omni-agent` embedding path already includes pragmatic safeguards:

- For `ollama/...`, it tries OpenAI-compatible direct path first, then `/embed/batch`, then `litellm-rs` provider (when key is present):
  - `packages/rust/crates/omni-agent/src/embedding/client/backend_dispatch.rs:118`
- When no provider key is configured, provider path is skipped and runtime remains Rust-only (no Python MCP embedding fallback):
  - `packages/rust/crates/omni-agent/src/embedding/client/backend_dispatch.rs:174`
- The embedding startup log now emits resolved runtime parameters (backend/source/base_url/mcp_url/default_model/api_key_source):
  - `packages/rust/crates/omni-agent/src/embedding/client/mod.rs:194`
  - `packages/rust/crates/omni-agent/src/embedding/client/support.rs:47`
- `transport_litellm` normalizes `ollama/...` into `openai/...` + `/v1` base normalization:
  - `packages/rust/crates/omni-agent/src/embedding/transport_litellm.rs:27`
- Gateway base URL resolution now distinguishes `mistral_sdk` and prefers `mistral.base_url` for that mode:
  - `packages/rust/crates/omni-agent/src/gateway/http/runtime.rs:61`

## 5. Performance Result (No-Cache Benchmark)

Source: `/tmp/omni_embed_bench_nocache_1771964244.json`

- Rust gateway `/embed/batch` and Rust gateway `/v1/embeddings` are nearly identical (same upstream, negligible transport overhead difference).
- Rust gateway is materially faster than Python MCP embedding service:
  - Sequential single-text avg latency: ~1.96x faster
  - Sequential batch-8 avg latency: ~2.77x faster
  - Concurrent single-text throughput: ~3.50x higher RPS
  - Concurrent batch-8 throughput: ~2.75x higher RPS

Therefore, prioritizing Rust-side embedding serving is justified by measured data.

## 6. Progress Since Initial Audit

The following medium-term items from the initial audit are now implemented in this branch:

- Startup diagnostics for embedding runtime selection and key-source visibility (without exposing secret values):
  - `packages/rust/crates/omni-agent/src/embedding/client/mod.rs:194`
  - `packages/rust/crates/omni-agent/src/embedding/client/support.rs:47`
- Integration tests that lock Rust-only fallback behavior for `litellm_rs` (`ollama/...` and `mistral/...`) without Python MCP embedding fallback:
  - `packages/rust/crates/omni-agent/tests/embedding_client.rs:418`
- OpenAI-compatible response parse hardening and failure observability in `xiuxian-llm`:
  - `packages/rust/crates/xiuxian-llm/src/embedding/openai_compat.rs:84`
  - `packages/rust/crates/xiuxian-llm/tests/embedding_openai_compat.rs:58`

## 7. Decision Recommendation

### Short-term (recommended now)

1. Keep Rust embedding runtime as default path for production workloads.
2. Treat backend policy as:
   - Default: `litellm_rs` (hybrid provider-first policy)
   - Local explicit runtime: `mistral_sdk` (managed by `xiuxian-llm::mistral`)
   - Rust protocol paths: `openai_http` and `/embed/batch` as Rust-local fallbacks
3. Keep embedding runtime Rust-only (no Python MCP fallback in the embedding dispatch path).
4. Keep role boundaries explicit: do not evaluate `litellm-rs` and `mistralrs` as mutually exclusive choices.

### Medium-term (to reduce ambiguity/risk)

1. Add role-based benchmark gates:
   - `mistral_sdk`: cold start/warm start latency + sustained local throughput.
   - `litellm_rs`: provider-route latency/success-rate by provider mode.
2. Add explicit CI assertions for both `/embed/batch` and `/v1/embeddings` compatibility contracts.
3. Keep provider-routing contract tests for prefix-sensitive models (`ollama/...`, `openai/...`, `mistral/...`) to avoid accidental fallback order regressions.

### Long-term (if we choose deep `mistralrs` integration)

1. Introduce a dedicated crate boundary (for example `xiuxian-llm-runtime`) for model runtime adapters.
2. Keep `omni-agent` as orchestration/control plane only.
3. Gate heavy local-runtime features behind optional Cargo features and explicit config.

## 8. Why This Direction

- It aligns with measured performance in this workspace.
- It minimizes migration risk while keeping the embedding runtime Rust-only.
- It preserves modularity: runtime concerns stay in LLM-focused crates, agent remains orchestration-first.
- It avoids premature hard-coupling to a not-yet-integrated `mistralrs` crate in this workspace.

## 9. Role Perf Smoke (Local)

Script:

- `scripts/rust/omni_agent_embedding_role_perf_smoke.py`

Latest local report artifact:

- `.run/reports/omni-agent-embedding-role-perf-smoke.json`

Latest run summary (`single_runs=20`, `batch_runs=10`, `concurrent_total=64`, `concurrent_width=8`):

- `litellm_rs`:
  - single p95: `73.33 ms`
  - batch(8) p95: `204.09 ms`
  - concurrent single throughput: `109.22 req/s`
- `mistral_sdk`:
  - single p95: `80.84 ms`
  - batch(8) p95: `222.67 ms`
  - concurrent single throughput: `91.39 req/s`

Gate mode is supported by thresholds:

- `--max-single-p95-ms`
- `--max-batch8-p95-ms`
- `--min-concurrent-rps`
