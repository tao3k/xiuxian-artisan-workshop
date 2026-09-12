# WAS Modernization Proof

This package formalizes the hot-path architecture invariants used by the WAS
Rust modernization work.

The model records five implementation decisions:

1. metadata filtering borrows JSON values instead of cloning them;
2. independent storage reads execute in one concurrent round;
3. key enumeration uses incremental cursor scans instead of `KEYS`;
4. cursor-scan results are deduplicated before snapshot assembly.
5. ASP is the sole Org authority; Wendao does not own an Org task read model.

The model is not a mechanically verified translation of Rust. Its latency
comparison assumes independent equal-latency reads without contention overhead.
`SCAN COUNT` is a hint, not a reply-size or latency bound. Counterexamples cover
oversized replies and empty nonterminal pages. The admission transition models
the Rust collector's retained-key and byte limits; correspondence remains a
manual audit. Additional obligations model one mutation submission and preservation
of a newer connection generation after stale failure. Wall-clock deadlines,
transport allocation, and server latency are not formally proved.

Run the proof from this directory with:

```sh
lake build
```

The architecture and cost-model sources exist only as executable Mermaid and
Typst Org Babel blocks in
`docs/30_research/2026-09-12-was-modernization-proof.org`. They intentionally
do not create parallel `.mmd` or `.typ` files.
