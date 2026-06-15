# sparq-canon

RDFC-1.0 — the W3C **[RDF Dataset Canonicalization]** (the URDNA2015 successor)
— as a small, **opt-in** public API over sparq's term model.

Canonicalization gives an RDF dataset a deterministic, blank-node-relabelled
form: two datasets are RDF-isomorphic **iff** their canonical N-Quads
serializations are byte-for-byte identical. That underpins dataset hashing,
signing, diffing, deduplication, and content-addressing.

[RDF Dataset Canonicalization]: https://www.w3.org/TR/rdf-canon/

> Model: Opus 4.8 (Fable unavailable — flag for re-review when Fable returns).
> Surfaced from `sparq-zk::canon` per bead sq-0qip.

## 🚀 Quickstart

```rust
use oxrdf::{Quad, NamedNode, NamedOrBlankNode, Term, Literal, GraphName, BlankNode};

let q = Quad::new(
    NamedOrBlankNode::BlankNode(BlankNode::new("x").unwrap()),
    NamedNode::new("http://ex/p").unwrap(),
    Term::Literal(Literal::new_simple_literal("v")),
    GraphName::DefaultGraph,
);
// Dataset -> canonical N-Quads (blank nodes relabelled to c14nN).
let nquads: String = sparq_canon::canonicalize(&[q.clone()]).unwrap();
// Just the blank-node issuer map (input label -> canonical label).
let map = sparq_canon::issued_identifiers(&[q]).unwrap();
```

## ✨ Features

- **Dataset API** — [`canonicalize`]/`canonicalize_quads` return the canonical
  N-Quads `String`; `issued_identifiers`/`issue_quads` return the blank-node
  issuer map. `*_with::<D: Digest>` selects a non-default hash profile (the spec
  default is SHA-256; `sha2::Sha384` gives the SHA-384 profile).
- **Single-graph API** — `canonicalize_triples` / `canonicalize_graph_content`
  return a `CanonicalGraph` (sorted canonical N-Quads lines + re-parsed
  canonical triples). This is what the ZK per-graph commitment pipeline
  consumes (`leaf_index = line index`).
- **Fail-closed on poison graphs** — RDFC-1.0's pathological blow-ups hit the
  HNDQ call-limit guard and surface as `CanonError::Canonicalization`.
- **W3C-conformant** — validated against the official [rdf-canon test suite]
  (eval + issued-map + negative cases, SHA-256 and SHA-384) through this crate's
  own public API (`tests/rdf_canon_suite.rs`).

[rdf-canon test suite]: https://github.com/w3c/rdf-canon

### Opt-in, single-sourced

Nothing in sparq's default build or the wasm artifact depends on this crate —
`sparq-core` stays lean. The RDFC-1.0 **algorithm** is the maintained zkp-ld
[`rdf-canon`](https://crates.io/crates/rdf-canon) crate (oxrdf 0.2); this crate
owns the single canonical-N-Quads-text bridge from sparq's oxrdf 0.3, so the
bridge lives in exactly one place. `sparq-zk` depends on it (its `canon` module
is now a re-export).

## 📚 Learn more

- [W3C RDF Dataset Canonicalization (RDFC-1.0)](https://www.w3.org/TR/rdf-canon/)
- [`skills/rdf-canon/SKILL.md`](../../skills/rdf-canon/SKILL.md) — how to use this surface.
- `tests/rdf-canon-testdata/PROVENANCE.md` — the vendored W3C suite snapshot.

This crate is `publish = false`: like `sparq-zk`, nothing in the workspace's
default dependency graph depends on it, so the default build and the wasm
artifact are byte-identical with or without it.

## License

MIT.
