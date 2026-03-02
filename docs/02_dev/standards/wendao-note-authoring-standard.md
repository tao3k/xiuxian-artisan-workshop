---
type: knowledge
title: "Wendao Note Authoring Standard"
category: "standards"
tags:
  - standards
  - wendao
  - link-graph
  - templates
saliency_base: 7.2
decay_rate: 0.03
metadata:
  title: "Wendao Note Authoring Standard"
---

# Wendao Note Authoring Standard

> Purpose: maximize `xiuxian-wendao` retrieval precision (graph + lexical) on Markdown corpora.

## 1. Scope

This standard applies to all Markdown notes intended for LinkGraph indexing:

- `docs/**/*.md`
- `assets/skills/**/SKILL.md`
- future notebook folders that are included by `link_graph.include_dirs`.

## 2. Hard Requirements

1. Every note must have one stable H1 title.
2. Every note must use deterministic slug-like filename (no random suffixes).
3. Every note must contain at least one explicit outbound link (except terminal archive notes).
4. Headings must be semantically meaningful; avoid placeholder headings like `Misc` or `Temp`.
5. Use English for technical content in repository-managed docs.

## 3. Frontmatter Contract

Use this minimal contract at the top of each note:

```yaml
---
title: "Human-readable title"
category: "architecture|reference|plans|standards|testing|how-to|explanation"
tags:
  - "domain-tag-1"
  - "domain-tag-2"
saliency_base: 5.0
decay_rate: 0.05
---
```

Guidelines:

- `title` should match the primary retrieval phrase users will search.
- `tags` should include both domain and action vocabulary (for example `router`, `schema`, `benchmark`).
- `saliency_base` / `decay_rate` should only be adjusted for curated long-lived notes.

## 4. Heading and Section Rules

1. `#` is note identity (single use).
2. `##` defines retrievable sections; each `##` should represent one query intent.
3. `###` is allowed for local decomposition, but avoid deep nesting beyond `###` unless required.
4. Do not place fake headings inside fenced code blocks for structure.

## 5. Link Authoring Rules

Use explicit links to improve graph traversal quality:

- Wiki links: `[[target-note-stem]]` for concept relationships.
- Markdown links: `[label](target-note.md)` for path-stable references.
- Anchor links: `[label](target-note.md#section-anchor)` when section-level precision is needed.

Recommended relation block:

```markdown
## Linked Notes

- Related: [[router]]
- Depends on: [[vector-router-schema-contract]]
- Compared with: [[link-graph-vs-librarian]]
```

## 6. Retrieval Anchor Rules

To improve lexical matching and route recall:

1. Include exact key terms users are likely to query.
2. Keep one concise definition sentence near the top of each `##` section.
3. Avoid synonym-only wording; include canonical project term at least once.

## 7. Template Set

Use the template files under:

- `docs/standards/templates/wendao/concept-note.template.md`
- `docs/standards/templates/wendao/decision-note.template.md`
- `docs/standards/templates/wendao/moc-note.template.md`
- `docs/standards/templates/wendao/experiment-note.template.md`

## 8. Review Checklist

Before merging a new/updated note:

1. Frontmatter is valid and complete.
2. H1/H2 structure is stable and meaningful.
3. At least one outbound link exists.
4. Query anchors (primary terms) are present.
5. File path and stem are deterministic.
