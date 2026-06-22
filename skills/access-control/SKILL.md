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
// `now` is the request clock (xsd:dateTime), used only by time-windowed conditional grants.
let alice =
    Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None, now: None };
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
  run the N3 rules to (re)install `<urn:sparq:auth>`. On `wasm32-unknown-unknown` the
  informational `MaterializeStats::millis` is reported as `0.0` (no `std::time::Instant`);
  the auth view is identical ([OPUS-4.8] sq-7agop).
- `store.materialize_acp_with(&AccessProvenance) -> Result<MaterializeStats, _>` —
  ([OPUS-4.8] sq-3jtd.5) ACP materialization that ALSO resolves `acp:CreatorAgent` /
  `acp:OwnerAgent` matchers against per-resource creator/owner WebIDs supplied by the
  TRUSTED caller. `AccessProvenance::set_creator(resource, webid)` /
  `set_owner(resource, webid)` build the map; the loader synthesizes
  `<r> solidx:creator|owner <w>` facts from THAT map ONLY. The loader hard-rejects any
  control-document triple whose predicate is in the derivation-internal `solidx:` namespace
  (§2.4), so a writer who embeds `<r> solidx:creator <self>` in a content document OR in the
  `.acr` they control cannot self-grant (this also blocks forged `solidx:appliesToResource`
  policy redirection). Each grant is RESOURCE-SCOPED (the creator of `R1` is never
  granted `R2`) and composes with the matcher's own `acp:client`/`acp:issuer` constraints.
  `materialize_acp()` is `materialize_acp_with(&AccessProvenance::new())` — no provenance ⇒
  no `CreatorAgent`/`OwnerAgent` grant (fail-closed).
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
  materializing party. `odrl:recipient`/`odrl:assignee` (`eq`/`isA`/`isPartOf`/`neq`) maps
  faithfully (recipient-of-data = session agent); an `odrl:dateTime` **inclusive** bound
  (`lteq` → `auth:notAfter`, `gteq` → `auth:notBefore`) maps to a **live-clock window**
  re-checked against `Session::now` per request ([OPUS-4.8] sq-0q7n — a lapsed window denies
  immediately, no `refresh_odrl_grant` needed); `odrl:purpose`/`count`/a *strict* `dateTime`
  bound have no faithful analogue and STAY one-shot; a rule mixing mappable + unmappable
  constraints falls back **entirely** to one-shot (fail-safe — never drops a bound). A
  dateTime window is mapped only on an **allow** (a lapsed *deny* would fail open). Mapping
  table in the [`usage-control-policy`](../usage-control-policy/SKILL.md) skill.
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
- `Session { agent: Option<&str>, client: Option<&str>, issuer: Option<&str>, now: Option<&str> }`
  (agent/client/issuer caller-asserted: WebID + `acl:origin`/`acp:client` + the OIDC
  `acp:issuer`; `None` = anonymous / any client / any issuer respectively). [OPUS-4.8]
  sq-3jtd.6: the `issuer` field is the third matcher dimension (ACP only — WAC ignores it)
  and is a STRING MATCH on a caller-asserted issuer, **not** an authentication step (see
  "What this is — and is NOT"). [OPUS-4.8] sq-0q7n: `now` is the request clock (an
  `xsd:dateTime` lexical string), consulted **only** by time-windowed conditional grants —
  a windowed grant with `now == None` fails closed; set it with `Session::at(now)`.
  `Mode::{Read, Write, Append, Control}`; `wac_fixture()` / `acp_fixture()` (bundled demo
  pods).
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

## Request-pipeline integration for a Solid server (incl. WAC-Allow headers) — issue #1126

`examples/quickstart.rs` shows the load → `materialize_wac` → `query_as`/`accessible`
loop. A Solid **request handler** wraps that with three steps: (1) build a `Session`
from the authenticated request, (2) gate the request with `accessible(...)` /
`query_as(...)`, (3) emit a [`WAC-Allow`](https://solidproject.org/TR/wac#wac-allow)
response header advertising the modes the agent (and the public) hold on the target
resource. There is **no public `WAC-Allow` helper in the crate yet** (tracked as a
follow-up); it is a few lines over the existing `accessible` API, shown here so a
server can drop it in. The header lists the modes available to `user` (the
authenticated agent) and to `public` (an anonymous `Session::default()`):

```rust
use sparq_solid::{Mode, PodStore, Session};
use oxrdf::NamedNode;

// One `Session` per request; `client`/`issuer` come from the auth layer (None = any).
fn session_from_request<'a>(webid: Option<&'a str>, origin: Option<&'a str>) -> Session<'a> {
    Session { agent: webid, client: origin, issuer: None, now: None }
}

// Does this session have `mode` on the graph backing `resource`? (fail-closed)
fn may(store: &mut PodStore, s: &Session, mode: Mode, resource: &NamedNode) -> bool {
    store.accessible(s, mode).iter().any(|g| g == resource)
}

// RFC-style WAC-Allow: `user="read write",public="read"` — only the modes actually held.
fn wac_allow(store: &mut PodStore, s: &Session, resource: &NamedNode) -> String {
    let modes = [(Mode::Read, "read"), (Mode::Write, "write"),
                 (Mode::Append, "append"), (Mode::Control, "control")];
    let held = |sess: &Session| -> String {
        modes.iter()
            .filter(|(m, _)| may(store, sess, *m, resource))
            .map(|(_, name)| *name).collect::<Vec<_>>().join(" ")
    };
    let anon = Session::default();
    format!(r#"user="{}",public="{}""#, held(s), held(&anon))
}
```

Per request: `session_from_request(...)` → `may(&mut store, &s, Mode::Read, &resource)`
to allow/deny (a deny is `403`/`404` per your fail-closed policy) → set the response's
`WAC-Allow` header to `wac_allow(&mut store, &s, &resource)`. `accessible` is an O(1)
hash check over the materialized index (it caches per session), so the four-mode sweep
is cheap. **Scope caveat:** sparq-solid is a *library-level* authoriser — there is no
HTTP surface here (no `Link`/`acl:` resource discovery, no `.well-known`), so mapping a
**request path to its named graph** (`resource`) and authenticating the WebID are the
server's job (see `research/sparq-solid-scope.md` §4). The header builder above is
illustrative, not a conformance-tested helper.

## Capability notes (WAC + ACP)

- **WAC** (`.acl`) — `acl:agent` (WebID), `acl:agentClass foaf:Agent` (public) /
  `acl:AuthenticatedAgent`, `acl:agentGroup`, `acl:default` inheritance, the
  `acl:Read/Write/Append/Control` modes.
- **ACP** (`.acr`) — policies / matchers with the `allOf` / `anyOf` / `noneOf`
  combinators and normative **deny-overrides**. [OPUS-4.8] sq-3jtd.6: matchers may now
  also constrain on `acp:issuer` (the caller-asserted OIDC issuer), the third principal
  dimension. [OPUS-4.8] sq-3jtd.5: matchers may also use `acp:agent acp:CreatorAgent` /
  `acp:OwnerAgent` — the context agent must be the resource's creator / owner, resolved
  against the TRUSTED per-resource provenance supplied via `materialize_acp_with` (above),
  resource-scoped and fail-closed.
- Principal lattice — three independent dimensions (agent, client, issuer):
  `Public ⊒ Authenticated ⊒ concrete-WebID`, `AnyClient ⊒ concrete-client`, and
  ([OPUS-4.8] sq-3jtd.6) `AnyIssuer ⊒ concrete-issuer`. A session expands to the agent
  principals, plus — when a client is given — the minted `(agent, client)` pair, plus —
  when an issuer is given — the minted `(agent, client, issuer)` triple (with
  `AnyClient` as the client component when no client was supplied), for ≤12 grant lookups.
  Only an issuer-CONSTRAINED grant mints a triple principal; an issuer-unconstrained grant
  (`AnyIssuer`) reuses the agent / pair term, so issuer-blind pods are unaffected.

## ACP conformance harness — [OPUS-4.8] sq-3jtd.9

`sparq_solid::conformance` is a **library-level ACP conformance harness**: a table-driven
scenario runner over this crate's own ACP engine (`materialize_acp` +
`AuthIndex::accessible`). A scenario is declared as **data** — an ACR-document corpus plus a
table of expected `(agent, client, mode, resource) → Allow | Deny` decisions — and the
harness asserts the engine reproduces every expected decision, reporting **all** mismatches
at once. The module is **always compiled** (no feature gate; it depends only on the
always-present ACP path). The scenario corpus is a single reusable source in
`crates/sparq-solid/tests/common/` (`common::acp_corpus()`, 12 scenarios / 40 decisions),
consumed by `crates/sparq-solid/tests/conformance_acp.rs` (the parity test, plus a negative
control asserting a wrong expectation is *reported*, not panicked) so a second test target —
the differential oracle (`sq-t58w.7`) — can run the IDENTICAL scenarios without copy-paste.

What it is — and is NOT (honest scope, mirrors the module/README/`research/sparq-solid-scope.md` §4):

- It is the **realistic, achievable** conformance signal for an authorization *oracle* —
  library-level **decision parity** against the [Solid ACP spec](https://solidproject.org/TR/acp)
  semantics. It is **NOT a normative-CTH pass** and makes no such claim.
- **CTH-over-HTTP** (the Solid Conformance Test Harness / `solid/specification-tests`, which
  drives a *server* and asserts on HTTP outputs) is **out-of-scope** — this crate has no HTTP
  surface; that belongs to a Solid server, conformance-tested *through* it.
- A **JS-reference differential oracle** (Community Solid Server / Inrupt ESS over the same
  corpus) is the credible *second* oracle but is research-open (JS-toolchain cost) and is
  **not** built here — captured as a follow-up.
- It is a **test/oracle harness over the existing engine** — it does not change authorization
  behaviour. It is complementary to `tests/acp.rs` (a hand-derived access matrix over one
  realistic pod): this harness gives spec-construct coverage with small, independently-failing
  cases.
- The **WAC** conformance harness (`sq-3jtd.8`, `sparq_solid::wac_conformance`) mirrors this
  one — the same harness shape with a WAC `.acl` corpus builder; see its section below. The
  two share the `conformance::{Decision, Expect, ScenarioReport}` vocabulary.

Public surface (`pub mod conformance`, re-exported from the crate root for the entry types):

- `AcrBuilder` — build the ACR-document corpus as N-Quads. `acr.access_control(resource,
  |p| …)` / `acr.member_access_control(resource, |p| …)` attach a policy (`acp:accessControl`
  / `acp:memberAccessControl` for cumulative ancestor inheritance); `acr.document(resource)`
  declares the protected resource graph; `acr.into_nquads()` emits the corpus.
- `PolicyBuilder<'a>` (the `|p| …` closure argument) — `allow(Mode)` / `deny(Mode)`,
  `any_of_agent` / `all_of_agent` / `none_of_agent`, `any_of_client` / `all_of_client`,
  `all_of_pair(agent, client)` — the `acp:allOf` / `acp:anyOf` / `acp:noneOf` combinators over
  `acp:agent` / `acp:client` matchers, with `PUBLIC_AGENT` / `AUTHENTICATED_AGENT` consts for
  the `acp:PublicAgent` / `acp:AuthenticatedAgent` matcher IRIs.
- `AcpScenario` — `new(name)`, `.acr(AcrBuilder)` / `.nquads(&str)` (the corpus),
  `.expect(Expect)` / `.expect_all(iter)` (the decision table), `.run() ->
  Result<ScenarioReport, String>` (materializes + checks). `run_corpus(&[AcpScenario])` runs
  a whole corpus. Read-only accessors `.name()`, `.nquads_str() -> &str` (the raw N-Quad
  corpus the engine loads) and `.expects() -> &[Expect]` (the decision table) let a SECOND
  consumer — the differential oracle — feed the IDENTICAL corpus to an independent decider.
- `Expect` — one expected decision, built fluently: `Expect::agent(webid)` /
  `Expect::anonymous()` / `Expect::pair(agent, client)`, then `.read/.write/.append/.control(
  resource)`, then `.is(Decision::Allow | Decision::Deny)`.
- `ScenarioReport` — `.passed()`, `.checked()`, `.mismatches()` (the failing
  `DecisionResult`s), `.name()`, and a readable mismatch `Display`. `Decision::{Allow, Deny}`
  is the binary the harness compares against the engine verdict (graph in the session's
  accessible set ⇒ `Allow`).

## WAC conformance harness — [OPUS-4.8] sq-3jtd.8

`sparq_solid::wac_conformance` is the **WAC sibling** of the ACP harness above: the same
table-driven scenario runner, but over this crate's **WAC** engine (`materialize_wac` +
`AuthIndex::accessible`) against the [Solid WAC spec](https://solidproject.org/TR/wac)
semantics. A scenario is an `.acl`-document corpus (`AclBuilder`) plus a table of expected
`(agent, client, mode, resource) → Allow | Deny` decisions; the harness asserts the engine
reproduces every one, reporting all mismatches at once. Always compiled (no feature gate).
The corpus is a single reusable source in `crates/sparq-solid/tests/common/`
(`common::wac_corpus()`, 12 scenarios / 40+ decisions), consumed by
`crates/sparq-solid/tests/conformance_wac.rs` (the parity test, a `run_via_podstore` parity
test over the full `PodStore` method-form path, and a negative control) and reusable by a
second test target — the differential oracle (`sq-t58w.7`) — without copy-paste. The
honest-scope caveats are **identical** to the ACP harness's (library
decision parity, NOT a normative-CTH pass; CTH-over-HTTP out-of-scope; CSS differential oracle
research-open).

Public surface (`pub mod wac_conformance`, re-exported from the crate root for the entry types;
the decision/expectation/report types are the **shared** `conformance::{Decision, Expect,
ScenarioReport}`):

- `AclBuilder` — build the `.acl` corpus as N-Quads. `acl.access_to(resource, |a| …)` /
  `acl.default_for(resource, |a| …)` / `acl.access_to_and_default(resource, |a| …)` attach an
  `acl:Authorization` (`acl:accessTo` on the resource's own ACL / `acl:default` for
  **nearest-ancestor** inheritance over members / both); `acl.document(resource)` declares the
  protected resource graph + inheritance anchor; `acl.into_nquads()` emits the corpus.
- `AuthBuilder<'a>` (the `|a| …` closure argument) — `mode(Mode)`, `agent(webid)`,
  `public()` / `authenticated()` / `agent_class(class)` (`acl:agentClass foaf:Agent` /
  `acl:AuthenticatedAgent`), `agent_group(group, &[members])` (emits the `vcard:hasMember`
  group document), `origin(client)` (the `acl:origin` (user, app) pair). `PUBLIC_AGENT`
  (`foaf:Agent`) / `AUTHENTICATED_AGENT` consts name the two recognised classes.
- `WacScenario` — `new(name)`, `.acl(AclBuilder)` / `.nquads(&str)`, `.expect(Expect)` /
  `.expect_all(iter)`, `.run() -> Result<ScenarioReport, String>` (materializes via the free
  `AuthIndex` path + checks), and `.run_via_podstore()` (the same check through the full
  `PodStore` method form — proves the path a PSS integration uses). `run_corpus(&[WacScenario])`
  runs a whole corpus. Read-only accessors `.name()`, `.nquads_str() -> &str` and `.expects()
  -> &[Expect]` expose the raw corpus + decision table to the differential oracle (below).
- The `Expect` builder, `Decision`, and `ScenarioReport` are the same shared types as the ACP
  harness — `Expect::agent(webid)` / `Expect::anonymous()` / `Expect::pair(agent, client)`,
  `.read/.write/.append/.control(resource)`, `.is(Decision::Allow | Deny)`.

## Differential oracle — [OPUS-4.8] sq-t58w.7

`crates/sparq-solid/tests/differential_oracle.rs` is a **three-way agreement check** that
runs the shared parity corpus (`common::wac_corpus()` / `common::acp_corpus()`) through
THREE deciders for every `(agent, client, mode, resource)` request and asserts they all
agree:

1. **the engine** — `materialize_{wac,acp}` + `AuthIndex::accessible` (the **N3-rules**
   paradigm — `rules/*.n3` run by `sparq-reason`);
2. **an independent reference evaluator** — `crates/sparq-solid/tests/reference/{wac,acp}.rs`,
   a from-scratch **PROCEDURAL** reading of the WAC/ACP spec over a hand-parsed model. Its
   whole value is being a **different paradigm**: it shares **no** code with `materialize.rs`
   / `loader.rs` / `rules/*.n3`, and parses the corpus with its own tiny N-Quad reader (not
   `Graph::load_dataset`), so a shared bug cannot hide in both deciders;
3. **the hand `Expect` table** — the human-authored expected decision in each scenario.

A **divergence** is recorded whenever any two disagree; an unclassifiable/erroring request
(failed load/materialize, or a reference parse error) counts as a divergence — **fail-closed**.
The test asserts `divergences == 0` and prints, in the SHACL/geo runner shape (so a CI ratchet
can grep it):

```text
WAC differential pairs <N> / divergences 0 (floor 0)
ACP differential pairs <N> / divergences 0 (floor 0)
```

It is a **correctness oracle over the parity corpus, not a security audit**, and the reference
evaluator implements exactly the constructs the corpus exercises (anything it does not
recognise fails closed → a divergence). Pure in-crate Rust — no JS toolchain, no network, no
clock, no Docker. A JS-reference differential twin (vs. `@solidlab/policy-engine` /
`@solid/acl-check`) is a separate research-open follow-up (`sq-t58w.9`), NOT built here.

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

## Trust-graph authorisation PoC — [OPUS-4.8] sq-pfae (issue #940, opt-in `trust-graph`)

The **`sparq-trust`** crate (`crates/sparq-trust`) is a **research proof-of-concept** that adds
an **admission stratum** ahead of the WAC/ACP derivation stratum: *"is this externally-attested
fact from a source I trust for this statement-type?"* On success it injects the issuer-tagged
fact so the shipped N3 reasoner merges it with the `.acr` rules to derive access — the age>18
worked example (a trusted-government VC `<Jesse> age 25` + a trust policy + the `.acr` rule
`{ ?x age ?y . ?y math:greaterThan 18 } => { ?x auth:read R }` ⇒ `<Jesse>` gains read).

It is wired into `sparq-solid` behind the **default-OFF `trust-graph` cargo feature**. With the
feature off, `sparq-solid` is byte-identical to WAC/ACP today (**strict additivity**); with it
on, `PodStore` gains two install methods. The admission gate is the §6.0 algorithm:
RDFC-1.0 canonicalise (`sparq-canon`) → a **checked** issuer signature over the commitment
(`sparq-zk`, never a self-asserted triple) → statement-type scoping via a real SHACL shape
(`sparq-shacl`) → freshness (a per-request check) → the clear-WebID holder binding.

- **`admit_trust_credential_static(credential, rules, target, abac_rule_n3)`** — the
  **materialise-time** path (the `sq-xc4y` static/dynamic split). It runs only the
  session-INDEPENDENT class (signature, type-scope, scope) and installs each derived grant as an
  `auth:ConditionalGrant` whose holder (`auth:agent`) and freshness (`auth:notAfter`) are
  re-checked **per request** by the shipped sq-0q7n `cond_applies` path — so holder/freshness are
  never frozen into the materialise-once view (a stale or wrong-holder request is denied at query
  time). **Use this for a long-lived view.**
- **`admit_trust_credential_with_rule(credential, rules, session, target, abac_rule_n3)`** — the
  single-request **snapshot** path: it runs the COMBINED gate against one live `Session` and
  installs an UNCONDITIONAL grant valid for that request only (do not use it to populate a
  long-lived view).

Both install on top of the unchanged auth view (the ODRL-bridge precedent). The
public surface is `sparq_trust::{vocab, policy, admit, wire}` — see
[`crates/sparq-trust/README.md`](../../crates/sparq-trust/README.md) and the design record
`research/solid-trust-graph-authz-design.md` §6.0 (tracked in
[issue #940](https://github.com/jeswr/sparq/issues/940); landing via design PR
<https://github.com/jeswr/sparq/pull/951>).

**Honest scope (read first).** This is a RESEARCH prototype, **NOT a security guarantee**. It
does **not** provide privacy, unlinkability, or anonymity: the credential is admitted in the
clear and the holder binding authenticates the WebID in the clear (the non-anonymous degraded
path, `sq-wvne`). The ZK estate it composes with is externally **unaudited**
(`sq-qhy4`, pending external accredited-cryptographer sign-off). Issuer keys are
operator-asserted (no DID resolver yet — `sq-pfae.3`, the live forgery vector D′). Open problems
are respected as documented limitations, never solved: `sq-tu4e` (no in-reasoner NAF over
derived facts; `revoked` is input-only; no deny-on-disagreement) and `sq-wvne` (ZK privacy) are
out of PoC scope. `sq-xc4y` (per-request holder/freshness admission vs the materialise-once view)
is **RESOLVED** by the static/dynamic split: see `admit_trust_credential_static` above and design
§3.3 A′. `sq-l5og` (the delegation invocation-binding gate, `sparq_trust::delegation`) IS
implemented: it binds each hop's `delegate_key` into the delegator-signed preimage, defeating the
key-substitution stolen-chain replay — but it does **not** claim full non-replayability, because
the delegator's own key is still operator-asserted (`sq-pfae.3`); see the crate README *Honest scope*.

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
