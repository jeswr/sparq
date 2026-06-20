<!-- [OPUS-4.8] sq-inzv: README brought to template. -->
# sparq-solid

<p>
  <a href="https://crates.io/crates/sparq-solid"><img src="https://img.shields.io/crates/v/sparq-solid.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-solid"><img src="https://docs.rs/sparq-solid/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**Solid Pod access control** over the [sparq](../../README.md) engine. Pods are stored as **named
graph per document**; their WAC (`.acl`) / ACP (`.acr`) access-control documents stay as plain,
queryable triples, and their semantics are encoded as **N3 rules** (run by `sparq-reason`) that
materialize a queryable authorization view in `<urn:sparq:auth>`. Every SPARQL query is then filtered
per `(WebID, client)` session to the authorized graph set — **fail-closed**, with zero Solid-specific
code in the engine (this crate is depended on by nothing else in the workspace).

## 🚀 Quickstart

```rust
# // [OPUS-4.8] hidden main returns Result<(), String>: engine API errors are `String` (no Error impl, so `?` can't widen to Box<dyn Error>).
# fn main() -> Result<(), String> {
use sparq_core::Graph;
use sparq_solid::{Mode, PodStore, Session};

// A pod is a dataset, one named graph per document (here the bundled fixture).
let graph = Graph::load_dataset(&sparq_solid::wac_fixture(), "nquads")?;
let mut store = PodStore::new(graph);
store.materialize_wac()?; // run the N3 rules → install <urn:sparq:auth>

// The SAME query, different sessions, different results — fail-closed.
let q = "SELECT ?title WHERE { ?s <https://ex.dev/ns#title> ?title }";
let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None, now: None };
let _authorized = store.query_as(&alice, Mode::Read, q)?.rows.len();
let _public_only = store.query_as(&Session::default(), Mode::Read, q)?.rows.len();
# Ok(()) }
```

## ✨ Features

- **WAC + ACP** — Web Access Control (`.acl`) and Access Control Policy (`.acr`): inheritance, agent
  classes, groups, `allOf`/`anyOf`/`noneOf`, the ACP `(agent, client, issuer)` principal,
  `acp:CreatorAgent`/`acp:OwnerAgent`, normative deny-overrides (matrix in the design doc).
- **Trusted creator/owner provenance** — `acp:CreatorAgent`/`acp:OwnerAgent` resolve only against
  per-resource WebIDs the storage layer supplies through the trusted `AccessProvenance` channel,
  **never** graph content. The loader hard-rejects any control-document triple in the derivation-internal
  `solidx:` namespace, so a writer cannot self-grant; with no provenance, no such matcher grants.
- **Triples-native + zero-copy enforcement** — pods, ACL/ACR docs, and the auth view are all
  ordinary named graphs ("who can read G?" is one SPARQL pattern); the default path evaluates
  through the engine's zero-copy dataset view, with a v1 `FROM NAMED` rewrite kept as a portability
  path for standard SPARQL 1.1 engines.
- **Write-path gating** — `update_as` / `update_as_acp` check every graph an update could mutate
  before applying, and auto-re-materialize on `.acl`/`.acr` writes.
- **ODRL bridge (opt-in `odrl-bridge`, research-track — not a production cutover)** — runs the
  [`sparq-policy`](../sparq-policy) ODRL evaluator and materializes the equivalent WAC/ACP grant (or
  dual `auth:deny*`) into the auth view, no new enforcement engine (zero ODRL code by default). See below.
- **Trust-graph admission PoC (opt-in `trust-graph`, research — NOT a security guarantee)** — an admission
  stratum ([`sparq-trust`](../sparq-trust)) admits an issuer-signed, trusted-source-scoped credential fact
  ahead of the materialiser; OFF = byte-identical WAC/ACP. No privacy/ZK (`sq-qhy4` unaudited).

## ODRL → AUTH_GRAPH bridge (opt-in `odrl-bridge`)

A matched ODRL `Permission` becomes a concrete `principal auth:<mode> graph` triple **appended** to
`<urn:sparq:auth>`; the *request* action maps conservatively to the narrowest WAC mode (`odrl:use`
deliberately **unmapped → no grant**), and a Prohibition maps to the dual `auth:deny*`.
**Fail-closed:** a grant materializes **only** on a definite Permit + a mappable action + a concrete
party + target (a deny **only** on a genuine prohibition match) — a Deny, unsatisfied constraint,
undischarged duty, unmapped action, or partyless/targetless request materializes **nothing**.

**Re-checked conditions vs frozen one-shots.** `materialize_odrl_permission_conditional` persists a
*faithfully-mappable* constraint as an ACP `auth:ConditionalGrant` re-checked per session (the
`recipient`/`assignee` matchers, incl. `neq` "everyone EXCEPT X", and an `odrl:dateTime` inclusive
window re-checked against `Session::now`, **fail-closed with no clock**). `purpose`/`count`/strict
bounds have **no** stateless analogue, so they stay **one-shot** (checked once via
`sparq_policy::evaluate` — a missing value is *unprovable* → fail-closed, **no DPV purpose-hierarchy
subsumption**; a lapsed bound is caught on the next `refresh_odrl_grant`). A mixed rule with any
unmappable constraint falls back **entirely** to one-shot (never drop a bound and over-grant).
**`odrl:count` is stateful, ACP is stateless** — it stays one-shot; real stateful enforcement is the opt-in `count-enforcement` feature. Full mapping detail in the SKILL.

**Refresh / revocation.** Bridged grants are tracked in a ledger; `refresh_odrl_grant(s)` rebuilds
the view from the static baseline plus a replay of still-valid entries, retracting any that no longer
hold. **Deny retraction is asymmetric (fail-OPEN risk):** a bridged `auth:deny*` is retracted **only**
on a *definite* `Withdrawn` verdict (`sparq_policy::prohibition_status`) — kept on `Applies`/`Ambiguous`,
since reusing the grant rule would re-admit a carved-out party. Static grants are never in the ledger,
so refresh can't widen or drop them.

## Conformance, security & containment

- **WAC + ACP conformance harnesses** (`sparq_solid::wac_conformance` / `conformance`) assert the
  engine against the [WAC](https://solidproject.org/TR/wac) / [ACP](https://solidproject.org/TR/acp)
  specs at the *library* level (data-declared `(agent, client, mode, resource) → allow|deny`
  scenarios). **Scope (honest):** the realistic library-level oracle, **not** the Solid
  CTH-over-HTTP (no HTTP surface here) — see `research/sparq-solid-scope.md` §4.
- **In-repo differential oracle** (`tests/differential_oracle.rs`) runs the shared corpus through
  THREE deciders — the engine (N3 rules), an **independent procedural reference evaluator**
  (`tests/reference/`, a *different paradigm*, no shared code) and the hand `Expect` table — and
  asserts **zero divergence** (fail-closed). A correctness oracle, **not** a security audit.
- **Security posture — fail-closed.** Absence of a grant makes a graph **invisible**. The reasoner
  is fed only ACL/ACR + structural facts — **never pod content** — so no writable document can
  grant itself access; the reserved `urn:sparq:` namespace is rejected on input and forged
  `<urn:sparq:auth>` graphs are stripped at load.
- **`ldp:contains` is PSS-written, not sparq-derived** — stored as opaque content, **never** derived
  from IRI structure, mutated on a write, or read into the reasoner. Containment *ancestry* is derived
  structurally only to drive ACL inheritance (pinned by `tests/containment_view_ownership.rs`).

## 📚 Learn more

- **How-to** — [`skills/access-control/SKILL.md`](../../skills/access-control/SKILL.md) (public
  API, WAC/ACP notes, conformance harnesses + the differential oracle, ODRL-bridge mapping detail).
- **Design + threat model** —
  [`research/solid-access-control-design.md`](../../research/solid-access-control-design.md) (model,
  matrix, strata, boundaries) + [scope](../../research/sparq-solid-scope.md).
- **API reference** — [docs.rs/sparq-solid](https://docs.rs/sparq-solid); walk-through
  `cargo run -p sparq-solid --example quickstart --release`.
- **Performance** — not baked into docs; measured via `cargo run -p sparq-solid --example bench
  --release` + the [benchmarks dashboard](https://jeswr.github.io/sparq/dev/bench).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md), [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
