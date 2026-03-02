---
type: knowledge
metadata:
  title: "xiuxian-qianji (千机)"
---

# xiuxian-qianji (千机)

> **"The Dao of logic is like a thousand interlocking gears; only through extreme precision can one ascend from computational chaos."**

`xiuxian-qianji` (千机 - Thousand Mechanisms) is the high-performance, probabilistic execution heart of the **Quadrilateral Cognitive Architecture**. It serves as the "Divine Artifact" that orchestrates the flow of reasoning, transforming fragmented agent actions into a seamless, clockwork artifact of pure logic.

---

## 1. Philosophy & Culture: The Qianji Box (千机匣)

In the lore of **CyberXiuXian**, a "Qianji Box" is a legendary mechanical device of immense complexity and infinite adaptability. It represents the pinnacle of craftsmanship, where a thousand hidden mechanisms work in perfect unison to achieve a singular, transcendent purpose.

### 1.1 From Entropy to Ascension

Standard AI workflows built on legacy Python graph runtimes often suffer from **"Computational Entropy"**—loose scripts that become unmanageable as complexity scales. `xiuxian-qianji` rejects this chaos. We treat every agentic workflow as a **Refined Artifact**.

- **The Iron Frame:** Like the tempered steel of a cultivation blade, our graph kernel is unyielding and formally verified.
- **The Divine Logic:** Like the flow of Qi through meridians, our scheduling is dynamic, probabilistic, and self-aware.

### 1.2 The Artisan's Way

We believe that an Agent should not just "execute code"—it should **"Cultivate Reasoning."** By moving the entire orchestration logic into this Rust-native engine, we achieve a state of **Intelligence-Knowledge Decoupling**, allowing the system to outlive the foundational models it employs.

---

## 2. Core Architecture: The Triple Mechanisms

### 2.1 The Iron Frame (Kernel)

Based on `petgraph::StableGraph`, the Iron Frame provides the physical structure. It supports millions of nodes with near-zero traversal overhead and utilizes **LTL (Linear Temporal Logic)** guards to ensure that no Agent falls into an "Infinite Loop" (the Zen of Termination).

### 2.2 The Divine Logic (Scheduling)

- **Probabilistic MDP Routing:** Decisions are not binary. Edges carry weights influenced by **Omega's Confidence**, allowing the system to explore multiple paths based on probability.
- **Adversarial Loops:** Natively supports the **Synapse-Audit** pattern, where nodes actively challenge and verify each other’s evidence links.

### 2.3 The Mirror Face (Qianhuan Integration)

Qianji is a **High-Performance Annotator**. In the milliseconds before a node executes, it calls upon `xiuxian-qianhuan` to transmute raw data into persona-aligned context, ensuring the Agent always wears the correct "Face" for the task.

---

## 3. Declarative Orchestration (The YAML Script)

True to the **"Rust-Hard, Python-Thin"** mandate, the "Thousand Mechanisms" are defined via a declarative YAML manifest.

```yaml
name: "Artifact_Refining_Pipeline"
nodes:
  - id: "Seeker"
    task_type: "knowledge"
  - id: "Auditor"
    task_type: "calibration"
edges:
  - from: "Seeker"
    to: "Auditor"
    label: "Verify"
```

---

## 4. Performance Baselines

| Metric           | Result           | Philosophy                       |
| :--------------- | :--------------- | :------------------------------- |
| **Compilation**  | **< 1ms**        | Swift as a Thought.              |
| **Node Jump**    | **< 100ns**      | Precision at the Speed of Light. |
| **Safety Audit** | **Pre-verified** | No Demon (Loop) shall pass.      |

---

## 5. Quick Start

```rust
let engine = compiler.compile(manifest_yaml)?;
let result = scheduler.run(initial_context).await?;
```

---

## License

Apache-2.0 - Developed with artisan precision by **CyberXiuXian Artisan Studio**.
