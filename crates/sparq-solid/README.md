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
- **ODRL bridge (opt-in, research-track)** — behind the off-by-default `odrl-bridge`
  cargo feature, `materialize_permission` / `PodStore::materialize_odrl_permission` runs the
  [`sparq-policy`](../sparq-policy) ODRL evaluator and, on a **definite Permit**, materializes
  the equivalent WAC/ACP grant into the auth view — so the existing graph-level enforcement
  honours it with **no new enforcement engine**. See below.

## ODRL → AUTH_GRAPH bridge (opt-in `odrl-bridge` feature) — [OPUS-4.8] sq-h3uk

The single-node bridge of epic sq-3183 (**research-track, not a production cutover**). Enable
it with `--features odrl-bridge` (it pulls in the optional `sparq-policy` dependency only then;
the default build carries zero ODRL code). A matched ODRL `Permission` becomes a concrete
`principal auth:<mode> graph` triple in `<urn:sparq:auth>`, **appended** to whatever WAC/ACP
view already exists.

**Action → mode mapping** (the ODRL *request* action is mapped; conservative — a Permit only
ever grants the narrowest mode the action denotes):

| ODRL action (`odrl:`)                         | WAC/ACP mode            |
|-----------------------------------------------|-------------------------|
| `read`, `display`, `present`, `print`, `play` | `acl:Read`              |
| `append`                                      | `acl:Append`            |
| `modify`, `delete`, `write`                   | `acl:Write`             |
| anything else (incl. the `odrl:use` umbrella) | **unmapped → no grant** |

`odrl:use` is deliberately left unmapped: it subsumes every action, so picking one WAC mode
for it would have to pick the widest — request `odrl:read` explicitly instead (a `use`
permission in the policy still *grants* a concrete `read` request, and the bridge maps that
concrete request).

**Fail-closed:** a grant is materialized **only** on a definite Permit *and* a mappable action
*and* a concrete party (WebID) + target graph. A Deny, an unsatisfied constraint, an
undischarged duty, an unmapped action, or a partyless/targetless request materializes
**nothing** — access is never widened on ambiguity.

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
