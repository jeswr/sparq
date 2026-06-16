---
name: access-control
description: "Graph-level Solid-style access control over a sparq RDF dataset with the opt-in sparq-solid crate: store pods as named-graph-per-document, keep WAC (.acl) / ACP (.acr) documents as queryable triples, materialize their semantics to a queryable authorization view (urn:sparq:auth) via N3 rules, then filter every SPARQL query/update per (WebID, client, issuer) session to the authorized graph set — fail-closed. Use when gating which named graphs a session may read/append/write, mapping WAC/ACP to an allow-deny model, or querying the materialized auth view. RESEARCH/architecture track — this is the authorization LAYER, NOT a production Solid Pod HTTP server, and it does NOT authenticate (the caller asserts the WebID, client, and issuer)."
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
`(WebID, client, issuer)` session to the named graphs that session may access —
**fail-closed**, with zero Solid-specific code in the engine itself (the crate is a
dependency of nothing in the workspace).

## What this is — and is NOT (read first)

This is an **architecture / research-track** layer, not a deployable Solid server.

- It is the **authorization** half only. It answers *"may this `(WebID, client, issuer)`
  session read/append/write named graph G?"* by materializing WAC/ACP into a queryable
  view and filtering the dataset. It does **NOT authenticate**: a `Session.agent` is a
  caller-asserted WebID string — there is **no WebID-OIDC / DPoP / token verification**.
  The relying application is responsible for authenticating the WebID before it hands one
  to `query_as`. Treat a `Session` as a trusted claim, not a verified one.
- The `Session.issuer` field (ACP `acp:issuer`) does **NOT** add WebID-OIDC
  authentication. It is one more **caller-asserted** matcher dimension: the caller states
  *which OIDC issuer it claims vouched for the WebID*, and an ACP `acp:issuer` matcher
  decides whether that asserted issuer is acceptable. sparq-solid never contacts an IdP,
  validates an ID token, or verifies the issuer binding — it only string-matches the
  asserted issuer against the grant. As with `agent`, the relying application must verify
  the issuer↔WebID binding before asserting it. WAC has no issuer notion, so a WAC pod
  ignores the field entirely.
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

// WebID / client / issuer are all CALLER-ASSERTED — sparq-solid does NOT authenticate them.
let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None };
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
- `store.materialize_odrl_permission(&Policy, &Request) -> BridgeOutcome` — **opt-in**
  (`odrl-bridge` feature, OFF by default; [OPUS-4.8] sq-h3uk): run the `sparq-policy` ODRL
  evaluator and, on a *definite Permit*, materialize the equivalent `principal auth:<mode>
  graph` grant into the auth view, then reindex — so this same enforcement path applies it.
  Fail-closed (Deny / unmapped action / partyless / targetless → no grant). See the
  [`usage-control-policy`](../usage-control-policy/SKILL.md) skill for the action→mode mapping.
- `store.materialize_odrl_prohibition(&Policy, &Request)` / `materialize_odrl_policy(...)` —
  same opt-in feature ([OPUS-4.8] sq-w693): a matched ODRL **Prohibition** materializes the dual
  `principal auth:deny<Mode> graph` triple, honoured by this enforcement under **deny-overrides**
  (`∪ allow ∖ ∪ deny` — a deny beats any allow for the same principal+target+mode). `…_policy`
  does both sides at once. Same fail-closed rules; no new enforcement engine.
- `store.materialize_odrl_permission_conditional(&Policy, &Request) -> BridgeOutcome` —
  **opt-in** (`odrl-bridge`; [OPUS-4.8] sq-hiz4): persists a *faithfully-mappable* ODRL
  constraint as a re-checked ACP `auth:ConditionalGrant` (agent matcher) instead of a
  one-shot allow — so the granted agent is verified **per session**, not frozen to the
  materializing party. Only `odrl:recipient`/`odrl:assignee` (`eq`/`isA`/`isPartOf`) maps
  faithfully (recipient-of-data = session agent); `odrl:purpose`/`dateTime`/`count` have no
  stateless `(agent, client)` analogue and STAY one-shot; a rule mixing mappable +
  unmappable constraints falls back **entirely** to one-shot (fail-safe — never drops a
  bound). Mapping table in the [`usage-control-policy`](../usage-control-policy/SKILL.md) skill.
- `store.refresh_odrl_grant(&Policy, &Request, BridgeKind)` / `refresh_odrl_grants()` —
  **opt-in** (`odrl-bridge`; [OPUS-4.8] sq-dpk4): re-evaluate **bridged** ODRL grants when
  the policy changes and **retract** the ones that no longer hold (a withdrawn permission, a
  lapsed time window, a re-evaluation that now Denies) while preserving static WAC/ACP grants
  and still-valid bridged grants. Bridged triples are tracked in a ledger and mirrored into a
  provenance graph `<urn:sparq:auth-bridged>` (`AUTH_BRIDGED_GRAPH`) so they are structurally
  distinguishable from static grants; refresh rebuilds the view as `static_baseline ∪
  replay(valid bridged entries)`. **Fail-closed**: any ambiguous re-eval of an *allow grant*
  retracts (access never left stale); a static grant is never re-evaluated or dropped. A
  wholesale static re-materialization (`materialize_wac`/`materialize_acp`) auto-reconciles —
  valid bridged grants are replayed back on top. A bridged **deny** ([OPUS-4.8] sq-2pcf) is
  the dual and uses the OPPOSITE fail-closed rule: a materialized `auth:deny*` is retracted
  only when the ODRL Prohibition is **definitely** withdrawn/lapsed (`prohibition_status ==
  Withdrawn`); an *ambiguous* re-eval **keeps** the deny (never restore access on missing
  evidence). A retracted deny may re-expose an allow grant — correct, since the prohibition
  is genuinely gone.
- `Session { agent: Option<&str>, client: Option<&str>, issuer: Option<&str> }`
  (all three caller-asserted: WebID + `acl:origin`/`acp:client` + the OIDC `acp:issuer`;
  `None` = anonymous / any client / any issuer respectively). [OPUS-4.8] sq-3jtd.6: the
  `issuer` field is the third matcher dimension (ACP only — WAC ignores it) and is a
  STRING MATCH on a caller-asserted issuer, **not** an authentication step (see
  "What this is — and is NOT"). `Mode::{Read, Write, Append, Control}`; `wac_fixture()` /
  `acp_fixture()` (bundled demo pods).
- `triple_principal(agent, client, issuer) -> String` / `pair_principal(agent, client) ->
  String` — the deterministic minted principal IRIs (`urn:sparq:triple?…` /
  `urn:sparq:pair?…`) that the ACP/WAC N3 rules emit for an issuer- / client-constrained
  grant, kept in sync with the rules. Both use RFC 3986 percent-encoding and are
  INJECTIVE, so no agent/client/issuer value can smuggle a `&client=`/`&issuer=`
  delimiter into another principal's term. Exposed for inspection/round-trip tests.
- `ANY_ISSUER` / `ANY_CLIENT` / `PUBLIC` / `AUTHENTICATED` — the principal-lattice top
  IRIs (`https://sparq.dev/ns/auth#AnyIssuer` etc.). [OPUS-4.8] sq-3jtd.6: an ACP grant
  with no `acp:issuer` matcher is issuer-unconstrained ⇒ `ANY_ISSUER` (the issuer-dimension
  top); a session with `issuer: None` matches it.

The materialized view is itself just triples — *"who can read G?"* is one SPARQL
pattern: `GRAPH <urn:sparq:auth> { ?who <https://sparq.dev/ns/auth#read> ?doc }`.

## Capability notes (WAC + ACP)

- **WAC** (`.acl`) — `acl:agent` (WebID), `acl:agentClass foaf:Agent` (public) /
  `acl:AuthenticatedAgent`, `acl:agentGroup`, `acl:default` inheritance, the
  `acl:Read/Write/Append/Control` modes.
- **ACP** (`.acr`) — policies / matchers with the `allOf` / `anyOf` / `noneOf`
  combinators and normative **deny-overrides**. [OPUS-4.8] sq-3jtd.6: matchers may now
  also constrain on `acp:issuer` (the caller-asserted OIDC issuer), the third principal
  dimension.
- Principal lattice — three independent dimensions (agent, client, issuer):
  `Public ⊒ Authenticated ⊒ concrete-WebID`, `AnyClient ⊒ concrete-client`, and
  ([OPUS-4.8] sq-3jtd.6) `AnyIssuer ⊒ concrete-issuer`. A session expands to the agent
  principals, plus — when a client is given — the minted `(agent, client)` pair, plus —
  when an issuer is given — the minted `(agent, client, issuer)` triple (with
  `AnyClient` as the client component when no client was supplied), for ≤12 grant lookups.
  Only an issuer-CONSTRAINED grant mints a triple principal; an issuer-unconstrained grant
  (`AnyIssuer`) reuses the agent / pair term, so issuer-blind pods are unaffected.

## Security posture — fail-closed

Absence of a grant means a graph is **invisible**, and a non-authorized graph is
indistinguishable from an absent one. The reasoner is fed only ACL/ACR + structural
facts — **never pod content** — so no writable document can grant itself access; the
reserved `urn:sparq:` namespace is rejected on input and any forged `<urn:sparq:auth>`
graph is stripped at load. A session whose `agent`/`client`/`issuer` falls inside the
reserved `urn:sparq:` encoding (or carries the `&client=`/`&issuer=` delimiter) fails
CLOSED — it cannot impersonate a minted pair/triple principal. (Again: this is the
*authorization* boundary — authenticating the WebID **and verifying the issuer↔WebID
binding** are the relying application's job.)

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
WebID, client, and `acp:issuer` are all caller-asserted — the issuer dimension matches a
presented credential issuer, it does NOT add WebID-OIDC authentication) and it is NOT a
Solid Pod HTTP server (no HTTP/LDP/OIDC). Verified against
`crates/sparq-solid/src/{lib,authindex,fixture}.rs` + README on branch
`feat/sq-3jtd-acp-issuer` (2026-06-16; the (agent, client, issuer) three-dimension
principal model, sq-3jtd.6); workspace v0.1.0, opt-in (depends on nothing in
the workspace), native-side. Conservative WAC/ACP sub-cases are noted in design §4.4; perf
is measured by `cargo run -p sparq-solid --example bench` and NOT baked into docs. Code
carries [OPUS-4.8] review markers pending re-review.)_
