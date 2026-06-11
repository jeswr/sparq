# sparq-solid

Solid Pod access control over the sparq engine — pods stored as **named graph per
document**, WAC/ACP access-control documents stored **as plain triples** (named graphs
too), their semantics encoded as **N3 rules** (`rules/*.n3`, run by `sparq-reason`)
that materialize a queryable authorization view in `<urn:sparq:auth>`, and queries
filtered per (WebID, client) session to the authorized graph set — fail-closed.

[Solid](https://solidproject.org/) is a decentralized-web specification stack in which
people keep their data in personal online datastores ("pods") of RDF documents and
grant agents and applications selective access through one of two declarative
access-control languages: [Web Access Control (WAC)](https://solidproject.org/TR/wac),
where each resource is governed by an `.acl` document of `acl:Authorization`s with
nearest-ancestor inheritance, and [Access Control Policy (ACP)](https://solidproject.org/TR/acp),
where each resource has an `.acr` of policies combining matchers via
`allOf`/`anyOf`/`noneOf` with cumulative inheritance and normative deny-overrides.
This crate stores pods inside a sparq dataset and enforces either language on SPARQL
queries, with **zero Solid-specific code in the engine** (this crate is a dependency
of nothing in the workspace).

Full design rationale, prior-art survey, and the measured v1 baseline:
[`research/solid-access-control-design.md`](../../research/solid-access-control-design.md)
(referenced below as "design doc").

## Quick start

```rust
use sparq_core::Graph;
use sparq_solid::{Mode, PodStore, Session};

// 1. load a pod: named graph per document (here the bundled ~1.1k-graph fixture)
let graph = Graph::load_dataset(&sparq_solid::wac_fixture(), "nquads")?;

// 2. wrap it (fail-closed: nobody sees anything yet) and materialize the auth view
let mut store = PodStore::new(graph);
store.materialize_wac()?; // or store.materialize_acp() for .acr pods

// 3. the SAME query, different sessions, different results
let q = "SELECT ?title WHERE { ?s <https://ex.dev/ns#title> ?title }";
let alice = Session { agent: Some("https://alice.ex/card#me"), client: None };
assert_eq!(store.query_as(&alice, Mode::Read, q)?.rows.len(), 599);
assert_eq!(store.query_as(&Session::default(), Mode::Read, q)?.rows.len(), 144); // public only
```

Runnable version (plus client-restricted sessions and querying the auth view
directly): `cargo run -p sparq-solid --example quickstart --release`.
Benchmark: `cargo run -p sparq-solid --example bench --release`.

## Storage model (design doc §2)

Everything is triples — a binding requirement (design-doc decisions D1–D4):

- **Pods**: every pod document is one named graph whose **name is the resource IRI**;
  container listings (`ldp:contains`) live in the container's own graph. Ancestry is
  *structural* (Solid slash semantics, derived from the IRI by rule). The store
  default graph is reserved for non-pod data; pod queries never touch it.
- **Access-control documents**: the ACL of resource `R` is the named graph
  `<R + ".acl">` (CSS-compatible; for ACP, `".acr"`). They are ordinary, queryable
  named graphs — what HTTP discovers via `Link rel=acl`, the store fixes by naming
  convention (a different convention only changes the loader, not the rules).
- **The materialized authorization view** is *also* triples (**D1**), in the reserved
  named graph `<urn:sparq:auth>`:

  ```turtle
  @prefix auth: <https://sparq.dev/ns/auth#> .

  # one grant = one triple: PRINCIPAL  mode-predicate  GRAPH-NAME
  <https://bob.ex/card#me>  auth:read   <https://pod.ex/notes/n1> .
  auth:Public               auth:read   <https://pod.ex/blog/post1> .
  auth:Authenticated        auth:append <https://pod.ex/inbox/> .

  # client-restricted grants use a deterministically minted PAIR principal
  # (components RFC 3986 percent-encoded — string:encodeForUri — so the minting is
  #  injective: no WebID can smuggle the &client= delimiter into someone else's pair)
  <urn:sparq:pair?agent=https%3A%2F%2Fbob.ex%2Fcard%23me&client=https%3A%2F%2Fapp.ex>
      auth:read <https://pod.ex/shared/doc> .

  # ACP deny half (deny-overrides is resolved per session, not here)
  <https://dave.ex/card#me> auth:denyRead <https://pod.ex/mixed/> .

  # ACP noneOf -> conditional grant nodes carrying their exception matchers
  <urn:sparq:grant?...> a auth:ConditionalGrant ;
      auth:effect auth:Allow ; auth:agent auth:Public ; auth:client auth:AnyClient ;
      auth:mode acl:Read ; auth:graph <https://pod.ex/blog/> ;
      auth:exceptMatcher <https://pod.ex/blog/.acr#blocked> .
  ```

- **D2**: any custom (non-triple) authorization storage must beat the measured
  baseline below, or it is rejected. **D3**: the per-session graph-set cache is a
  *transient index* over these triples (dropped wholesale on re-materialization), not
  a storage substitute. **D4 (fail-closed)**: the view is an allow-list; no grant =
  invisible graph.

**The principal model**: a grant is conceptually (agent, client, mode, graph).
Collapsing (agent, client) into one principal term — a WebID, `auth:Public`,
`auth:Authenticated`, or a minted `urn:sparq:pair?agent=A&client=C` — and mode into
the predicate makes the common case a single triple: maximally queryable ("who can
read G?" is one pattern) and maximally materializable by rules (design doc §2.3).

## WAC / ACP support matrix (design doc §3.6)

| feature | status |
|---|---|
| **WAC** `acl:accessTo` | ✓ (own-ACL authorizations only) |
| `acl:default` + nearest-ancestor effective-ACL discovery | ✓ (multi-level; does not apply to the container itself) |
| `acl:agent` / `acl:agentClass` (`foaf:Agent`, `acl:AuthenticatedAgent`) | ✓ |
| `acl:agentGroup` (vcard group documents, themselves pod graphs) | ✓ |
| `acl:origin` | ✓ (as a minted (agent, origin) pair principal) |
| modes `Read`/`Write`/`Append`/`Control` | ✓ (`Control` grants read+write **on the ACL resource**, per spec) |
| `acl:accessToClass` | ✗ (extension; out of scope) |
| **ACP** `acp:accessControl` / `acp:memberAccessControl` | ✓ (member control transitive ⇒ cumulative over all ancestors) |
| `acp:allOf` / `acp:anyOf` / `acp:noneOf` | ✓ (`noneOf` → conditional grants resolved per session) |
| `acp:agent` (incl. `PublicAgent`, `AuthenticatedAgent`) | ✓ |
| `acp:client` (incl. `PublicClient`) — the native user/app pair | ✓ |
| `acp:allow` / `acp:deny` with normative deny-overrides | ✓ (deny resolved per session: `∪allow ∖ ∪deny`) |
| `acp:issuer`, `acp:vc` | ✗ (issuer would extend the principal to a triple — design doc §7) |
| `acp:CreatorAgent` / `acp:OwnerAgent` | ✗ (need per-resource creator facts from the storage layer) |
| custom ACP mode IRIs | ✗ (the four standard `acl:` modes are mapped) |

## How it works

### L2 — the rules (`rules/*.n3`)

WAC/ACP semantics live in N3 rule files run by `sparq-reason`'s N3 engine — not in
Rust. `rules/common.n3` derives container ancestry from IRI structure (pure N3:
`string:scrape` + `log:uri`). `rules/wac.n3` is a **single stratum** (its
negation-as-failure only scopes over loader-input predicates). ACP needs
"∀ allOf-matcher accepts" = ¬∃ rejection where rejection is itself ¬accepts, so it
runs as **three strata** — `acp-a.n3` (accept-sets) → `acp-b.n3` (rejections) →
`acp-c.n3` (grants) — because the engine's NAF never retracts: each negated predicate
must be complete before its stratum runs (design doc §3.5). Each file's header
comment documents its stratum, inputs, and outputs.

### Loading conventions (the loader, design doc §4.2)

The reasoner is single-graph, so the materializer *assembles* its input: the
`.acl`/`.acr` graphs' triples (blank nodes skolemized per graph), group documents
referenced via `acl:agentGroup`, and synthesized structural facts
(`solidx:isResource`, `solidx:ownAcl <R.acl>`, `solidx:inDoc` provenance,
`solidx:isWebId`). Pod *content* graphs are **never** fed to the reasoner (see
Security below). The filtered closure is installed as `<urn:sparq:auth>`, replacing
any previous view.

### The session model (design doc §4.3)

A `Session { agent, client }` (both optional; `agent: None` = anonymous) expands to
at most 6 principals: `{Public} ∪ {Authenticated, A if agent} ∪ {pair(p, C) for each,
if client}`. `PodStore::accessible(session, mode)` computes
`∪ allow(principals) ∖ ∪ deny(principals)`, applying conditional grants whose
exception matchers don't accept the session, and caches the sorted graph set per
(agent, client, mode). **Invalidation**: re-materializing (the response to *any*
ACL/ACR/group-document change) bumps an epoch and drops the entire cache — a revoked
grant takes effect at the next query. Everything `accessible` answers is also
derivable from the auth-view triples with plain SPARQL (`MINUS` over the deny half).

### The query path — zero-copy dataset view (default; design doc §4.4 + §5)

`PodStore::query_as` (and `query_json_as` / `ask_as`) wraps the query
(`wrap_for_view`) and evaluates it through the engine's L1 zero-copy `DatasetView`
(`sparq_engine::query_view` / `query_json_view` / `ask_view`):

1. every default-graph triple/path pattern is wrapped in `GRAPH ?fresh { … }`
   (union-default emulation; cross-document joins keep working — a triple asserted
   in k accessible graphs yields k rows where an RDF merge would yield 1, `DISTINCT`
   restores set semantics);
2. the engine evaluates under a `DatasetView` built from the session cache:
   `named` = the authorized graph set (`Arc<FxHashSet<Term>>`, shared per call — the
   engine holds no session state), `default` = `DefaultGraphMode::Empty` (pod data
   never lives in the store default graph).

Visibility is an O(1) hash check per graph name; evaluation runs in place on the
existing sub-graphs — zero decode, zero rebuild, zero copy — and a non-authorized
graph is *indistinguishable* from an absent one: `GRAPH <g>` yields nothing,
`GRAPH ?g` never enumerates it, and a caller-supplied `FROM (NAMED)` clause is
intersected with the view (queries can restrict, never widen). A grant-less session
gets a view over the empty set — fail-closed. For entry points without a session
wrapper (`construct`, chunked JSON, …), take `PodStore::view_for` /
`PodStore::accessible_set` and run under `sparq_engine::with_view`:

```rust,ignore
let view = store.view_for(&session, Mode::Read);                  // cached, Arc-shared
let wrapped = sparq_solid::wrap_for_view(sparql)?;                // step 1 only
let g = sparq_engine::with_view(&view, || sparq_engine::construct(view.base, &wrapped))?;
```

### The v1 rewrite path (kept; portability + differential oracle)

`PodStore::query_as_rewrite` / `rewrite_for` implement the same policy with **no
view API**: step 1 above plus a dataset-clause injection — `FROM NAMED <g>` for
exactly the authorized graphs (intersected with any pre-existing `FROM NAMED`; an
empty set uses the absent sentinel `<urn:sparq:nothing>` so the clause survives
serialization, fail-closed). The rewritten query enforces the policy on **any**
SPARQL 1.1 engine with standard dataset-clause semantics — that is the portability
story, and `tests/e2e.rs` uses it as a differential oracle (both paths must return
byte-identical JSON for every fixture session).

The honest cost of the rewrite path: the engine's `FROM NAMED` handling
(`build_active`) decodes and rebuilds every listed graph **per query** — measured
12 ms/query at 3k quads, 59 ms at 46k (linear in authorized data). That copy is
exactly what the default view path deletes — measured below.

## Security model

Fail-closed throughout — absence of a grant means a graph is invisible (D4), and a
non-authorized graph behaves exactly like an absent one (indistinguishability):

- **No view, no access**: before the first `materialize_*` call every session —
  including the pod owner's — sees the empty graph set.
- **Reasoner input boundary** (design doc §2.4): only `.acl`/`.acr` graphs, group
  documents, and synthesized structural facts feed the reasoner. Pod content graphs
  are excluded — otherwise any agent able to write a document could embed `acl:`/
  `acp:` triples granting themselves access. Writing the `.acl`/`.acr` graphs is
  exactly what `acl:Control` / ACR write gates.
- **Reserved namespace** `urn:sparq:` (pair/candidate/grant principals, the auth
  view, the rewrite sentinel): the loader **rejects** agent/client/origin IRIs inside
  it or containing the pair delimiter `&client=` (pair minting percent-encodes its
  components, so such a collision is no longer constructible — the validation stays
  as defense in depth); sessions carrying
  such values get the **empty** graph set; and `PodStore::new`/the materializer
  **strip** all reserved-named graphs from loaded datasets — a dataset cannot smuggle
  in a forged `<urn:sparq:auth>`; only the materializer creates it.
- **No query escape**: explicit `GRAPH <private>` patterns and attacker-supplied
  `FROM NAMED` clauses cannot reach outside the authorized set on either path
  (view intersection / rewrite intersection semantics; regression-tested in
  `tests/hardening.rs` and `tests/e2e.rs`, which also asserts the two paths return
  byte-identical JSON).

## Measured (design doc §6 + §6.1)

M1 MacBook Air, `--release`, fixture = 1148 named graphs / 3060 quads ("fat" = same
tree with 50 filler triples per document = 46 260 quads). Reproduce with
`cargo run -p sparq-solid --example bench --release` (both query paths are measured
in the same run, so the v1-vs-v2 comparison is honest under machine-load variance).

| measurement | value |
|---|---|
| WAC auth-view materialization (full pipeline, 1 stratum) | ~0.5–1.0 s → 3 783 auth triples |
| ACP auth-view materialization (3 strata) | ~0.6–1.1 s → 6 168 auth triples |
| re-materialization after an ACL change (= full re-run) | same |
| session graph-set, cold / cached | 0.30 ms / 0.3 µs |

v1 rewrite path vs the default v2 dataset-view path (same run, 2026-06-11):

| per-query measurement | v1 rewrite+copy | v2 dataset view | speedup |
|---|---|---|---|
| titles query, 800 authorized graphs (3k quads) | 28.98 ms | **18.35 ms** | 1.6× |
| path overhead isolated (empty pattern), 3k quads | 11.52 ms | **1.72 ms** | **6.7×** |
| titles query, fat fixture (46k quads) | 67.46 ms | **20.75 ms** | 3.3× |
| path overhead isolated, fat fixture | 43.21 ms | **1.58 ms** | **27×** |
| FULL-dataset query, no security, fat fixture | 33.58 ms | — | — |

Reading honestly: materialization is cheap enough to re-run on every ACL change at
pod scale. The v1 per-query copy scales linearly with authorized data (~1.3 s
extrapolated at 1M quads) and on the fat fixture is 2× slower than querying the
whole dataset with no security; the v2 view's overhead is **flat** (~1–1.7 ms at
both sizes — the union-default `GRAPH` wrap, not a copy), and on the fat fixture
the restricted query is *faster* than the unrestricted scan (the view prunes
non-visible graphs before they are touched). These numbers are also the **D2 gate**
for any custom authorization storage.

## Limitations & follow-ups (design doc §7)

- **Incremental auth maintenance**: v1 re-runs the full pipeline (~1 s) on any
  ACL/ACR/group change; counting-based N3-incremental maintenance under stratified
  NAF is a `sparq-reason` follow-up.
- **N3 builtin gaps**: ~~`string:encodeForUri`~~ DONE (sparq-reason; pair/candidate
  IRI minting now percent-encodes its components — injective — with the
  reserved-namespace validation kept as defense in depth); still open: a
  multi-stratum entry point (`reason_n3_stratified` — the ACP pipeline re-serializes
  closures between strata) and count-over-property-values (would collapse ACP
  strata B+C).
- **Union-default semantics** are emulated per pattern (duplicate rows vs RDF merge);
  a true zero-copy union default needs shared-dictionary named graphs (design doc
  §5.4).
- Unsupported vocabulary: see the support matrix above.
- Update-path enforcement (write gating + auto-re-materialization on `.acl` writes)
  is designed but not wired (design doc §4.4).
