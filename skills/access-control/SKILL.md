---
name: access-control
description: "Graph-level Solid-style access control over a sparq RDF dataset with the opt-in sparq-solid crate: store pods as named-graph-per-document, keep WAC (.acl) / ACP (.acr) documents as queryable triples, materialize their semantics to a queryable authorization view (urn:sparq:auth) via N3 rules, then filter every SPARQL query/update per (WebID, client) session to the authorized graph set — fail-closed. Use when gating which named graphs a session may read/append/write, mapping WAC/ACP to an allow-deny model, or querying the materialized auth view. RESEARCH/architecture track — this is the authorization LAYER, NOT a production Solid Pod HTTP server, and it does NOT authenticate (the caller asserts the WebID)."
license: MIT
metadata:
  version: "0.1.0"
  homepage: https://github.com/jeswr/sparq
---

# sparq-solid — graph-level WAC/ACP access control

`sparq-solid` is an **opt-in** crate that enforces Solid-style access control over a
sparq RDF dataset at the **named-graph** granularity. A pod is stored as
**one named graph per document**; the WAC (`.acl`) and ACP (`.acr`) access-control
documents stay as plain queryable triples; their semantics are encoded as **N3 rules**
(run by `sparq-reason`) that **materialize** a queryable authorization view in the
reserved `<urn:sparq:auth>` graph. Every query/update is then filtered per
`(WebID, client)` session to the named graphs that session may access — **fail-closed**,
with zero Solid-specific code in the engine itself (the crate is a dependency of nothing
in the workspace).

## What this is — and is NOT (read first)

This is an **architecture / research-track** layer, not a deployable Solid server.

- It is the **authorization** half only. It answers *"may this `(WebID, client)`
  session read/append/write named graph G?"* by materializing WAC/ACP into a queryable
  view and filtering the dataset. It does **NOT authenticate**: a `Session.agent` is a
  caller-asserted WebID string — there is **no WebID-OIDC / DPoP / token verification**.
  The relying application is responsible for authenticating the WebID before it hands one
  to `query_as`. Treat a `Session` as a trusted claim, not a verified one.
- It is **NOT a Solid Pod HTTP server.** There is no HTTP resource protocol, no LDP
  container CRUD, no `Link rel=acl` discovery over the wire, no Solid notifications. The
  document→named-graph naming is a storage convention (design §2.2), not an HTTP server.
- The honest support matrix, threat model, security boundaries, and measured baseline
  live in the design record
  ([`research/solid-access-control-design.md`](../../research/solid-access-control-design.md));
  the README's support matrix and `cargo doc -p sparq-solid` are the user docs. Do not
  present this crate as a production Solid implementation.

## Quickstart

`crates/sparq-solid/Cargo.toml` (consumes `sparq-core` / `sparq-engine` / `sparq-reason`;
no cargo features of its own):

```toml
[dependencies]
sparq-core  = { path = "../sparq-core" }
sparq-solid = { path = "../sparq-solid" }
```

The same query, different sessions, different results — fail-closed:

```rust
use sparq_core::Graph;
use sparq_solid::{Mode, PodStore, Session, wac_fixture};

# fn main() -> Result<(), String> {
// A pod is a dataset, one named graph per document (here the bundled fixture).
let graph = Graph::load_dataset(&wac_fixture(), "nquads")?;
let mut store = PodStore::new(graph);
store.materialize_wac()?;                 // run the N3 rules → install <urn:sparq:auth>

let q = "SELECT ?title WHERE { ?s <https://ex.dev/ns#title> ?title }";

// WebID is CALLER-ASSERTED — sparq-solid does NOT authenticate it.
let alice = Session { agent: Some("https://alice.ex/card#me"), client: None };
let authorized = store.query_as(&alice, Mode::Read, q)?;            // alice's authorized graphs
let public_only = store.query_as(&Session::default(), Mode::Read, q)?; // anonymous: public graphs only
let _ = (authorized.rows.len(), public_only.rows.len());
# Ok(()) }
```

## Public API

Materialize the authorization view from the access-control documents, then enforce:

- `PodStore::new(graph) -> PodStore` — wrap a loaded dataset. Before the first
  `materialize_*` call **every** session (including the owner) sees nothing.
- `store.materialize_wac()` / `store.materialize_acp() -> Result<MaterializeStats, _>` —
  run the N3 rules to (re)install `<urn:sparq:auth>`.
- `store.query_as(&Session, Mode, sparql)` → `QueryResult` (`.rows`);
  `store.query_json_as(...)` → JSON string; `store.ask_as(...)` → `bool`. These evaluate
  through the engine's **zero-copy `DatasetView`** filtered to the session's authorized
  graphs (the default, fast path).
- `store.query_as_rewrite(&Session, Mode, sparql)` — the v1 **`FROM NAMED` rewrite**
  portability path: enforces the same policy on any standard SPARQL 1.1 engine (one
  deliberate semantic difference noted in-source: a caller `FROM <g>` can only restrict
  the view, never widen it).
- `store.update_as(&Session, sparql)` / `store.update_as_acp(...)` — **write-path
  gating**: check every graph an update could mutate *before* applying, and
  auto-re-materialize on `.acl`/`.acr` writes.
- `store.accessible(&Session, Mode) -> Arc<Vec<NamedNode>>` /
  `store.accessible_set(...)` / `store.view_for(...) -> DatasetView` /
  `store.auth() -> &AuthIndex` — inspect the authorized graph set or the materialized
  index directly.
- `Session { agent: Option<&str>, client: Option<&str> }` (caller-asserted WebID +
  `acl:origin`/`acp:client`; `None` = anonymous / any client); `Mode::{Read, Write,
  Append, Control}`; `wac_fixture()` / `acp_fixture()` (bundled demo pods).

The materialized view is itself just triples — *"who can read G?"* is one SPARQL
pattern: `GRAPH <urn:sparq:auth> { ?who <https://sparq.dev/ns/auth#read> ?doc }`.

## Capability notes (WAC + ACP)

- **WAC** (`.acl`) — `acl:agent` (WebID), `acl:agentClass foaf:Agent` (public) /
  `acl:AuthenticatedAgent`, `acl:agentGroup`, `acl:default` inheritance, the
  `acl:Read/Write/Append/Control` modes.
- **ACP** (`.acr`) — policies / matchers with the `allOf` / `anyOf` / `noneOf`
  combinators and normative **deny-overrides**.
- Principal lattice: `Public ⊒ Authenticated ⊒ concrete-WebID` and
  `AnyClient ⊒ concrete-client`.

## Security posture — fail-closed

Absence of a grant means a graph is **invisible**, and a non-authorized graph is
indistinguishable from an absent one. The reasoner is fed only ACL/ACR + structural
facts — **never pod content** — so no writable document can grant itself access; the
reserved `urn:sparq:` namespace is rejected on input and any forged `<urn:sparq:auth>`
graph is stripped at load. (Again: this is the *authorization* boundary — authentication
of the WebID is the relying application's job.)

## Related skills

- [`http-server`](../http-server/SKILL.md) — the sparq SPARQL HTTP server has **no**
  per-user authz of its own; front it with `sparq-solid` (or a gateway) for that.
- [`usage-control-policy`](../usage-control-policy/SKILL.md) — `sparq-policy` (W3C ODRL
  2.2) is the **usage-control** layer *above* this access-control layer: where
  `sparq-solid` answers "may this agent read graph G?", `sparq-policy` answers "may this
  party *use* this asset for purpose P, with obligation O, until time T?".
- [`inference`](../inference/SKILL.md) — `sparq-reason` runs the N3 rules that
  materialize the auth view.

_(status: RESEARCH / architecture track. The crate is `shipped` per the design record
(`research/solid-access-control-design.md`) — the zero-copy `DatasetView` query path and
the write-gating path are implemented and tested (`tests/update.rs` + the crate suite) —
but the SCOPE is the graph-level authorization layer ONLY: it does NOT authenticate (the
WebID is caller-asserted) and it is NOT a Solid Pod HTTP server (no HTTP/LDP/OIDC). Verified
against `crates/sparq-solid/src/{lib,authindex,fixture}.rs` + README on branch
`feat-skill-drift-catchup` (2026-06-16); workspace v0.1.0, opt-in (depends on nothing in
the workspace), native-side. Conservative WAC/ACP sub-cases are noted in design §4.4; perf
is measured by `cargo run -p sparq-solid --example bench` and NOT baked into docs. Code
carries [OPUS-4.8] review markers pending re-review.)_
