# sparq-solid

<p>
  <a href="https://crates.io/crates/sparq-solid"><img src="https://img.shields.io/crates/v/sparq-solid.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-solid"><img src="https://docs.rs/sparq-solid/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**Solid Pod access control** over the [sparq](../../README.md) engine.

Pods are stored as **named graph per document**; their WAC (`.acl`) / ACP (`.acr`)
access-control documents stay as plain, queryable triples, and their semantics are encoded
as **N3 rules** (run by `sparq-reason`) that materialize a queryable authorization view in
`<urn:sparq:auth>`. Every SPARQL query is then filtered per `(WebID, client)` session to the
authorized graph set — **fail-closed**, with zero Solid-specific code in the engine (this
crate is a dependency of nothing in the workspace).

## 🚀 Quickstart

```rust
# // [OPUS-4.8] hidden main returns Result<(), String>: the engine's API errors are
# // `String`, which does not impl std::error::Error, so `?` cannot widen to Box<dyn Error>.
# fn main() -> Result<(), String> {
use sparq_core::Graph;
use sparq_solid::{Mode, PodStore, Session};

// A pod is a dataset, one named graph per document (here the bundled fixture).
let graph = Graph::load_dataset(&sparq_solid::wac_fixture(), "nquads")?;
let mut store = PodStore::new(graph);
store.materialize_wac()?; // run the N3 rules → install <urn:sparq:auth>

// The SAME query, different sessions, different results — fail-closed.
let q = "SELECT ?title WHERE { ?s <https://ex.dev/ns#title> ?title }";
let alice = Session { agent: Some("https://alice.ex/card#me"), client: None };
let _authorized = store.query_as(&alice, Mode::Read, q)?.rows.len();
let _public_only = store.query_as(&Session::default(), Mode::Read, q)?.rows.len();
# Ok(()) }
```

## ✨ Features

- **WAC + ACP** — Web Access Control (`.acl`) and Access Control Policy (`.acr`), including
  inheritance, agent classes, groups, the `allOf`/`anyOf`/`noneOf` combinators, and
  normative deny-overrides. The full support matrix is in the design doc (linked below).
- **Triples-native** — pods, ACL/ACR documents, and the materialized authorization view are
  all ordinary named graphs; "who can read G?" is one SPARQL pattern.
- **Zero-copy enforcement** — the default query path evaluates through the engine's zero-copy
  dataset view (no per-query graph copy); a v1 `FROM NAMED` rewrite is kept as a portability
  path that enforces the same policy on any standard SPARQL 1.1 engine.
- **Write-path gating** — `update_as` / `update_as_acp` check every graph an update could
  mutate before applying it, and auto-re-materialize on `.acl`/`.acr` writes.

## Security posture — fail-closed

Absence of a grant means a graph is **invisible**, and a non-authorized graph is
indistinguishable from an absent one. Before the first `materialize_*` call every session
(including the pod owner's) sees nothing. The reasoner is fed only ACL/ACR + structural facts
— never pod *content* — so no writable document can grant itself access; the reserved
`urn:sparq:` namespace is rejected on input and forged `<urn:sparq:auth>` graphs are stripped
at load. See [`SECURITY.md`](../../SECURITY.md) for the project security policy and the
design doc for the full threat model.

## 📚 Learn more

- **Design + threat model + measured baseline** —
  [`research/solid-access-control-design.md`](../../research/solid-access-control-design.md)
  (storage model, WAC/ACP support matrix, the strata, security boundaries).
- **API reference** — [docs.rs/sparq-solid](https://docs.rs/sparq-solid); runnable walk-through
  `cargo run -p sparq-solid --example quickstart --release`.
- **Performance** — not baked into docs; the two query paths are measured side by side in
  `cargo run -p sparq-solid --example bench --release` and on the
  [benchmarks dashboard](https://jeswr.github.io/sparq/dev/bench).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
