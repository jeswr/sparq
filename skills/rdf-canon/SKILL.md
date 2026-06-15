---
name: rdf-canon
description: Canonicalize an RDF dataset with RDFC-1.0 (the W3C RDF Dataset Canonicalization, the URDNA2015 successor) using the opt-in sparq-canon crate — turn a set of oxrdf quads (or one graph's triples) into a deterministic, blank-node-relabelled canonical N-Quads string, or get just the canonical blank-node issuer map. Use when you need to hash, sign, diff, deduplicate, or content-address an RDF dataset, or to test two graphs for RDF-isomorphism (identical canonical form). Native + wasm; does not touch sparq-core's default build.
---

# sparq-canon — RDFC-1.0 dataset canonicalization

[RDFC-1.0](https://www.w3.org/TR/rdf-canon/) (the W3C **RDF Dataset
Canonicalization**, the URDNA2015 successor) computes a deterministic form of an
RDF dataset: blank nodes are relabelled to `c14n0`, `c14n1`, … and the quads are
emitted in a stable code-point order. Two datasets are **RDF-isomorphic iff their
canonical N-Quads serializations are byte-for-byte identical** — the basis for
dataset hashing, signing, diffing, deduplication, and content-addressing.

`sparq-canon` is the **opt-in public surface** for this. Add it explicitly; it is
**not** in sparq's default build (`sparq-core` stays lean, the wasm artifact is
unchanged unless you pull this in). The RDFC-1.0 algorithm itself is the
maintained zkp-ld [`rdf-canon`](https://crates.io/crates/rdf-canon) crate;
`sparq-canon` owns the single oxrdf-0.3 ↔ oxrdf-0.2 canonical-N-Quads bridge and
exposes a clean API over sparq's term model. `sparq-zk` depends on it.

## Add the dependency

```toml
[dependencies]
sparq-canon = { path = "crates/sparq-canon" }   # or your workspace path
oxrdf = { version = "0.3", features = ["rdf-12"] }
```

## Dataset API — quads in, canonical N-Quads out

The general case. Pass any slice of `oxrdf::Quad` (named graphs and the default
graph both work); get back the canonical N-Quads document.

```rust
use oxrdf::{Quad, NamedNode, NamedOrBlankNode, Term, Literal, GraphName, BlankNode};

let q = Quad::new(
    NamedOrBlankNode::BlankNode(BlankNode::new("x").unwrap()),
    NamedNode::new("http://example.org/p").unwrap(),
    Term::Literal(Literal::new_simple_literal("v")),
    GraphName::DefaultGraph,
);

// Full canonical N-Quads (input blank-node labels are erased -> c14nN):
let nquads: String = sparq_canon::canonicalize(&[q.clone()]).unwrap();
assert!(nquads.contains("_:c14n0"));

// Just the issuer map: input blank-node label -> canonical label.
let map = sparq_canon::issued_identifiers(&[q]).unwrap();
assert_eq!(map.get("x").map(String::as_str), Some("c14n0"));
```

`canonicalize_quads` / `issue_quads` are aliases (the `rdf_canon`-style names).

### Non-default hash profile

The spec default is SHA-256. To use the SHA-384 profile (or any
`digest::Digest`), use the `*_with` functions; `Digest` is re-exported so you do
not need a direct `digest` dependency:

```rust
let nq = sparq_canon::canonicalize_quads_with::<sha2::Sha384>(&dataset).unwrap();
```

## Single-graph API — one graph's triples

When you have one graph's content (a default-graph-only dataset), use the
triple-level API. It returns a `CanonicalGraph` with the sorted canonical lines
and the re-parsed canonical triples (line index = a stable total order — this is
exactly what the ZK per-graph commitment pipeline uses as the leaf order).

```rust
use sparq_canon::{canonicalize_triples, canonicalize_graph_content};

let canon = canonicalize_triples(&triples).unwrap();
let doc: String = canon.to_nquads();         // joined lines, each + '\n'
let n = canon.lines.len();                    // = canon.triples.len()

// Or straight from a stored graph:
let canon = canonicalize_graph_content(&graph).unwrap();   // graph: &sparq_core::Graph
```

`graph_triples(&graph)` materializes a stored graph's triples as `oxrdf::Triple`s
if you want them without canonicalizing.

## Errors — fail closed

`CanonError` has three variants:

- `TripleTerm` — the dataset contains an RDF-1.2 quoted triple as an object;
  these are outside RDFC-1.0's data model and cannot be canonicalized.
- `Canonicalization(String)` — `rdf-canon` rejected the dataset. This includes
  the **HNDQ call-limit guard**: RDFC-1.0 has pathological-input blow-ups, so a
  poison graph trips the limit and fails closed rather than running unbounded.
- `Bridge(String)` — an internal serialize/parse error (should not occur for
  well-formed RDFC-1.0-model input; surfaced rather than swallowed).

## Conformance

Validated against the official [W3C rdf-canon test
suite](https://github.com/w3c/rdf-canon) — all eval (canonical-output),
issued-map, and negative (poison-graph) cases, under both SHA-256 and SHA-384 —
through this crate's own public API. See `crates/sparq-canon/tests/`.

## When NOT to use it

- You only need a *syntactic* serialization (N-Triples/N-Quads as-is): use the
  oxrdf/oxttl serializers directly. Canonicalization is more expensive.
- The dataset has no blank nodes and a fixed order you already control:
  canonicalization is still correct but buys you nothing.

## Status

Verified against `sparq-canon` 0.1.0 source on branch `feat-rdfc-public-api`
(2026-06-15). The RDFC-1.0 algorithm is `rdf-canon` 0.15.3 (W3C-suite validated);
`sparq-canon` is the single-sourced bridge + public API. `publish = false`,
non-default workspace member — nothing in sparq's default graph depends on it.
