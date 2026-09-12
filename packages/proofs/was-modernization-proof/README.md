# WAS Modernization Proof

This package formalizes the hot-path architecture invariants used by the WAS
Rust modernization work.

The proof covers four implementation decisions:

1. metadata filtering borrows JSON values instead of cloning them;
2. independent storage reads execute in one concurrent round;
3. key enumeration uses bounded cursor scans instead of blocking `KEYS`;
4. cursor-scan results are deduplicated before snapshot assembly.
5. ASP is the sole Org authority; Wendao does not own an Org task read model.

Run the proof from this directory with:

```sh
lake build
```

The architecture and cost-model sources exist only as executable Mermaid and
Typst Org Babel blocks in
`docs/30_research/2026-09-12-was-modernization-proof.org`. They intentionally
do not create parallel `.mmd` or `.typ` files.
