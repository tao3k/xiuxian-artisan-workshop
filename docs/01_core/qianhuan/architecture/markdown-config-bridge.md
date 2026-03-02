---
type: knowledge
title: "Qianhuan-Wendao Markdown Configuration Bridge"
category: "architecture"
tags:
  - qianhuan
  - wendao
  - configuration
  - markdown
saliency_base: 8.0
decay_rate: 0.02
metadata:
  title: "Qianhuan-Wendao Markdown Configuration Bridge"
---

# Qianhuan-Wendao Markdown Configuration Bridge

This document specifies the architectural transition from pure TOML/J2 files to a **Wendao-driven Markdown-to-Configuration** paradigm for managing the `xiuxian-qianhuan` engine's personas, templates, and skill manuals.

## 1. The Core Problem: Configuration Sprawl

Currently, the `Qianhuan` engine requires physical `.toml` files for Personas and `.j2` files for Templates, scattered across `resources/` and `assets/` directories.

As the number of Agent scenarios, adversarial sub-graphs, and tool manuals grows, managing hundreds of isolated text files becomes unmaintainable. Furthermore, flat files cannot be semantically searched or logically linked (e.g., "Show me all Personas that use the Agenda validation template").

## 2. The Solution: Wendao as the Configuration Control Plane

The **Qianhuan-Wendao Bridge** treats the `xiuxian-wendao` Knowledge Graph as the ultimate source of truth for runtime configurations.

Instead of writing fragmented TOML files, Developers and Operators will author cohesive **Markdown Documents** that structurally define Personas, Templates, and Skill Manuals. Wendao's indexing engine parses these documents, stores them as formal `Entity` and `Relation` nodes in the graph, and projects them into the runtime configurations required by Qianhuan.

### 2.1 The Markdown Link-based Reference Schema

A single Markdown file (e.g., `skill.md` or `agenda_scenario.md`) acts as the **Cognitive Control Plane**. Instead of inlining large Jinja2 templates, it uses standard Markdown links or Wiki-links to point to the physical template files.

```markdown
---
type: "qianhuan_scenario"
domain: "zhixing"
---

# Scenario: Adversarial Agenda Validation

This document manages the coordination between personas and their respective templates.

## Persona: Agenda Steward

<!-- id: "agenda_steward", type: "persona" -->

**Background**: You are a helpful assistant.
**Guidelines**:

- Follow user intent.
- Use [[Standard Agenda Template|packages/rust/crates/xiuxian-zhixing/resources/zhixing/templates/draft_agenda.j2]] for output.

## Template: Critique Report

<!-- id: "critique_report", type: "template" -->

**Reference**: [critique_agenda.j2](../../../xiuxian-zhixing/resources/zhixing/templates/critique_agenda.j2)
```

### 2.2 Link Resolution & Dynamic Handoff

Because `xiuxian-wendao` is an expert at parsing Markdown AST and resolving path-based relations, the pipeline is refined as follows:

1. **Relation Extraction (Wendao)**:
   - The parser scans the Heading and its HTML property tag.
   - It looks for **Markdown Links** (`[name](path)`) or **Wiki-links** (`[[path]]`) within that heading's section.
   - It resolves these relative paths into absolute canonical paths within the workspace.
2. **Graph Materialization**:
   - Wendao stores the `Entity` (e.g., `TEMPLATE(critique_report)`) with a `path` property instead of a `content` property.
3. **Runtime Loading (Qianhuan)**:
   - When Qianhuan requests `template_id = "critique_report"`, Wendao returns the **Resolved File Path**.
   - Qianhuan's `ManifestationManager` then performs a standard disk load (utilizing its existing Hot-Reload and Fallback logic) from that specific path.

### 2.3 Advantages of Link-based Management

1. **Centralized Visibility**: A single `skill.md` provides a high-level map of an entire complex agent scenario.
2. **Template Reusability**: Multiple Scenarios can link to the same `.j2` file without duplication.
3. **Rust Internal Efficiency**: `xiuxian-wendao` provides the "Address Book" (Paths), and `xiuxian-qianhuan` provides the "Execution" (Loading). No large string blobs are passed between crates if not needed.

### 2.4 Internal Embedded Mode (Bypassing Valkey)

For **Internal / Built-in** resources stored within a crate's `resources/` directory (e.g., `xiuxian-zhixing/resources/zhixing/skill.md`), the system bypasses the global indexing pipeline.

1. **Library Usage**: `xiuxian-zhixing` (or any internal crate) uses `xiuxian-wendao` as a pure parsing library.
2. **Local Resolution**:
   - At startup, the crate reads its internal Markdown files into memory as strings.
   - It calls `wendao::extract_markdown_config_blocks(&content)` directly.
   - This returns a collection of `MarkdownConfigBlock` (Personas, Templates, etc.) containing the extracted text and properties.
3. **Pure Memory Bootstrapping**:
   - The results are injected directly into the `PersonaRegistry` or `ManifestationManager` in memory.
   - **Zero-Valkey Footprint**: These built-in configurations never hit the Valkey DB or the Zhenfa gateway, ensuring that the system is fully operational and "Self-Contained" even before the network or database layers are fully initialized.

By adopting this mode, we completely replace legacy TOML persona files with structured Markdown documents for internal crate management.

## 4. Execution Tracking & Engineering Rollout

To transition to this zero-export, AST-driven architecture, the following engineering tasks must be executed.

| QH-MD-01 | **AST Property Parser** | `xiuxian-wendao` | ✅ Done |
| | Implement a Rust parser using `comrak` (the existing AST engine in Wendao) to scan Markdown documents and extract Org-Mode style HTML comment properties (`<!-- id: "...", type: "..." -->`) immediately following heading nodes. This is strictly a Wendao responsibility. | | |
| QH-MD-02 | **Code Block Extraction** | `xiuxian-wendao` | ✅ Done |
| | Extend the Wendao AST parser to extract the raw string contents of fenced code blocks (e.g., ````jinja2`) that are structurally child nodes of a tagged heading. | | |
| QH-MD-03 | **$O(1)$ Memory Indexing** | `xiuxian-wendao` | ✅ Done |
| | Ensure that extracted entities are stored in the graph/memory with their `id` property as the primary key. This guarantees that `Qianji` can retrieve them via an $O(1)$ index lookup rather than a slow, fuzzy semantic search. | | |
| QH-MD-04 | **Zero-Export Load Interface** | `xiuxian-qianhuan` | ✅ Done |
| | Refactor `ManifestationManager` to expose a clean interface that accepts template strings and Persona definitions directly from Wendao's in-memory index output, bridging the two systems without physical files. | | |
| QH-MD-05 | **Live E2E Validation** | `omni-agent` | ✅ Done |
| | Create a test Markdown file (e.g., `test_personas.md`), tag it, and verify that a Qianji workflow can successfully inject the persona and template directly from the AST-parsed memory state. | | |

### 4.1 Landed Implementation Paths (2026-02-26)

- `xiuxian-wendao`:
  - `packages/rust/crates/xiuxian-wendao/src/enhancer/markdown_config.rs`
  - `packages/rust/crates/xiuxian-wendao/tests/test_enhancer.rs`
- `xiuxian-qianhuan`:
  - `packages/rust/crates/xiuxian-qianhuan/src/manifestation/manager.rs`
  - `packages/rust/crates/xiuxian-qianhuan/src/persona/registry.rs`
  - `packages/rust/crates/xiuxian-qianhuan/tests/test_manifestation_manager.rs`
  - `packages/rust/crates/xiuxian-qianhuan/tests/unit_persona.rs`
  - `packages/rust/crates/xiuxian-qianhuan/tests/test_markdown_config_bridge.rs`
- `omni-agent`:
  - `packages/rust/crates/omni-agent/tests/agent/native_tools_zhixing.rs` (`task_add_render_supports_markdown_ast_memory_bridge`)

### 4.2 Implementation Reference (`comrak` AST Traversal)

For the engineer executing **QH-MD-01** and **QH-MD-02**, the following is the mandated architectural approach using the `comrak` crate (already present in the `xiuxian-wendao` workspace).

**Do NOT use regex to parse code blocks.** Use strict AST node matching:

```rust
use comrak::{parse_document, Arena, ComrakOptions, nodes::NodeValue};

// 1. Initialize Arena and parsing tree
let arena = Arena::new();
let root = parse_document(&arena, markdown_text, &ComrakOptions::default());

let mut current_id_scope: Option<String> = None;

// 2. Traverse AST nodes
for node in root.descendants() {
    let data = node.data.borrow();
    match &data.value {
        NodeValue::Heading(_) => {
            // A new heading resets the scope.
            current_id_scope = None;
        }
        NodeValue::HtmlBlock(html) => {
            // Detect Org-Mode style properties: <!-- id: "agenda_steward" -->
            if html.literal.contains("<!-- id:") {
                current_id_scope = extract_id_from_html(&html.literal);
            }
        }
        NodeValue::CodeBlock(c) => {
            // If we are currently under a tagged heading and find a jinja2 block
            if let Some(id) = &current_id_scope {
                let info_string = String::from_utf8_lossy(&c.info);
                if info_string.starts_with("jinja2") {
                    let template_content = String::from_utf8_lossy(&c.literal);
                    // SUCCESS: We have perfectly extracted the Zero-Export template!
                    // Insert into Wendao Graph using `id` as the primary key.
                    insert_to_wendao_graph(id, &template_content);
                }
            }
        }
        _ => {}
    }
}
```

This ensures absolute deterministic parsing regardless of trailing whitespaces or markdown formatting anomalies.

### 4.3 Internal Rust API Call Flow (Precise Retrieval)

For internal workspace calls (e.g., during `omni-agent` bootstrap), the following Rust sequence is used to perform precise retrieval from Markdown AST without hitting the network gateway.

#### 1. Ingestion (Wendao Side)

```rust
use xiuxian_wendao::enhancer::markdown_config::{extract_markdown_config_blocks, MarkdownConfigMemoryIndex};

// Parse the source Markdown text into an O(1) memory index
let blocks = extract_markdown_config_blocks(&markdown_content);
let index = MarkdownConfigMemoryIndex::from_blocks(blocks);

// Precise lookup by ID
if let Some(block) = index.get("agenda_steward") {
    println!("Found {} block: {}", block.config_type, block.content);
}
```

#### 2. Injection (Qianhuan Side)

Once retrieved from the index, the payloads are injected into Qianhuan's runtime registries:

```rust
// Injecting a Persona
persona_registry.register_from_memory_toml(
    &block.id,
    &block.content
)?;

// Injecting a Template
manifestation_manager.upsert_template_from_memory(
    MemoryTemplateRecord::new(&block.id, block.target.clone(), &block.content)
)?;
```

This flow guarantees that configuration retrieval is deterministic, type-safe, and extremely high-performance within the Rust runtime.

### 4.4 The Embedded Resource Registry Pattern (`include_dir`)

To ensure **Nix-compatible sandboxing** and zero-dependency runtime booting, internal resources are embedded into the binary and indexed via Wendao's AST engine.

#### 1. Compile-Time Embedding

Each crate (e.g., `xiuxian-zhixing`) uses the `include_dir` crate to bake its `resources/` directory into the static binary.

```rust
use include_dir::{include_dir, Dir};
static RES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/resources");
```

#### 2. Startup-Time Indexing (Wendao Integration)

At startup, the crate passes this embedded directory to Wendao's registry builder. Wendao performs a recursive scan of all `.md` files.

```rust
// Logic provided by xiuxian-wendao
let wendao_registry = WendaoResourceRegistry::build_from_embedded(&RES_DIR)?;

// The registry now contains a structured map of all Markdown files and their internal AST links.
// Example: accessing a resolved template path from skill.md
let template_path = wendao_registry
    .file("zhixing/skill.md")
    .link_to("Agenda Proposer Template") // Resolved from [[Name|./path]]
    .as_path();

// Now Qianhuan can load the template content directly from the embedded virtual filesystem
let template_content = RES_DIR.get_file(template_path).unwrap().contents_utf8();
```

#### 3. Why This Pattern Wins

- **Strongly Typed Access**: Configurations are no longer "magic strings" but resolvable paths within the crate's internal virtual tree.
- **Atomic Reliability**: Since everything is in `include_dir`, it is impossible to have a "Missing File" error at runtime. If the Markdown links to a template that doesn't exist in the same `resources/` folder, Wendao's `build_from_embedded` will throw a **compile-time or startup-time Error**, preventing broken deployments.
- **Markdown-First Management**: The `skill.md` remains the "Source of Truth" for descriptions, rules, and relationships, while the actual code just "follows the map" generated by Wendao.
