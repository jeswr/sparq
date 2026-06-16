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
let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None };
let _authorized = store.query_as(&alice, Mode::Read, q)?.rows.len();
let _public_only = store.query_as(&Session::default(), Mode::Read, q)?.rows.len();
# Ok(()) }
```

## ✨ Features

- **WAC + ACP** — Web Access Control (`.acl`) and Access Control Policy (`.acr`), including
  inheritance, agent classes, groups, the `allOf`/`anyOf`/`noneOf` combinators, the ACP
  matcher's `acp:agent` / `acp:client` / `acp:issuer` attributes (the three-dimensional
  `(agent, client, issuer)` principal — a Matcher can gate on the OIDC issuer that vouched
  for the requester, not just the WebID), and normative deny-overrides. The full support
  matrix is in the design doc (linked below).
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
  honours it with **no new enforcement engine**. A matched **Prohibition** materializes the dual
  `auth:deny*` triple (`materialize_prohibition` / `materialize_policy`), and the existing
  enforcement applies **deny-overrides** ([OPUS-4.8] sq-w693). `materialize_odrl_permission_conditional`
  (sq-hiz4) persists a faithfully-mappable recipient/assignee constraint as a **re-checked**
  ACP `auth:ConditionalGrant` instead of a one-shot allow. See below.

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

### Prohibitions → explicit `auth:deny*` (deny-overrides) — [OPUS-4.8] sq-w693

A matched ODRL **Prohibition** is the dual of a Permit: `materialize_prohibition` /
`PodStore::materialize_odrl_prohibition` materializes it as the explicit
`principal auth:deny<Mode> graph` triple the enforcement already understands, via the **same**
action→mode mapping above (`denyRead` / `denyWrite` / `denyAppend` / `denyControl`).
`materialize_policy` / `PodStore::materialize_odrl_policy` does both sides at once.

| ODRL Prohibition action (`odrl:`)             | materialized deny predicate |
|-----------------------------------------------|-----------------------------|
| `read`, `display`, `present`, `print`, `play` | `auth:denyRead`             |
| `append`                                      | `auth:denyAppend`           |
| `modify`, `delete`, `write`                   | `auth:denyWrite`            |
| anything else (incl. the `odrl:use` umbrella) | **unmapped → no deny**      |

**Deny-overrides:** the deny is honoured by the **existing, unchanged** enforcement — the
session layer already computes `∪ allow ∖ ∪ deny` (`AuthIndex::accessible`) and `Mode::from_pred`
already parses `auth:deny*`, so a materialized deny **beats any allow grant** for the same
principal+target+mode. No new enforcement engine; the bridge only emits the triple. (Within a
single policy the ODRL evaluator *also* applies deny-overrides upstream — the would-be Permit
returns `allow == false`, so its allow triple is never even emitted when a prohibition carves
the request out.)

**Fail-closed (deny):** a deny is materialized **only** when a prohibition *matches* the request
(decided by `sparq_policy::matched_prohibition` — the evaluator's own conflict test, *not*
`Decision.allow == false`, which conflates a carve-out with a plain no-permission deny) *and* the
action is mappable *and* the party+target are concrete. An unmatched / unmapped / partyless /
targetless prohibition materializes **nothing**; an unmappable carve-out is *reported* in
`reasons`, never silently dropped (dropping a deny would widen access).

### Constraint → ACP **conditional** grant (`materialize_odrl_permission_conditional`) — [OPUS-4.8] sq-hiz4

The one-shot `materialize_odrl_permission` *freezes* every constraint into a single allow
scoped to the supplied request party. `materialize_odrl_permission_conditional` persists a
**faithfully-mappable** constraint as an ACP `auth:ConditionalGrant` (the existing `noneOf`
machinery) so the granted agent is **re-checked per session** through the unchanged
enforcement path — not by re-running the ODRL evaluator. Constraints with no faithful ACP
analogue keep the one-shot behaviour.

| ODRL constraint | Operator | Maps to | Behaviour |
|---|---|---|---|
| `odrl:recipient` / `odrl:assignee` | `eq` / `isA` | `auth:agent <webid>` (agent matcher) | **re-checked condition** |
| `odrl:recipient` / `odrl:assignee` | `isPartOf` | one `auth:agent` head per set member | **re-checked condition** |
| `odrl:recipient` / `odrl:assignee` | `neq` ("everyone EXCEPT X") | `auth:Public` grant + `auth:exceptMatcher` carving out `X` (ACP `noneOf`) | **re-checked condition** ([OPUS-4.8] sq-5037) |
| `odrl:recipient` / `odrl:assignee` | order (`lt`/`gt`/…) | — (not meaningful on a recipient) | one-shot (frozen) |
| `odrl:purpose` | any | — (ACP session has no purpose) | one-shot (frozen) |
| `odrl:dateTime` / time window | any | — (ACP has no "now") | one-shot (frozen) |
| `odrl:count` | any | — (ACP is stateless; no per-session usage counter) | one-shot (frozen) in the bridge¹ |
| *no constraint* | — | `auth:agent auth:Public` | re-checked (public) |

**Why only recipient/assignee:** the ACP session re-check carries exactly `(agent, client)`,
and the recipient-of-data *is* the session agent — so an agent matcher re-checks it with
identical semantics. Purpose/time/count have no stateless `(agent, client)` analogue, so
persisting them would require a looser approximation that could over-grant — rejected.

**`recipient neq X` → ACP `noneOf` ("everyone EXCEPT X")** — [OPUS-4.8] sq-5037. A
`recipient neq X` rule emits a `ConditionalGrant` whose head is the positive recipient set
(or `auth:Public` when there is none) plus one `auth:exceptMatcher <m>` per excluded `X`;
the matcher `<m>` carries the accept-set facts the session layer reads
(`solidx:acceptsAgentP <X>` + `solidx:acceptsClientP auth:AnyClient`). `AuthIndex` then
suppresses the grant for any session the matcher accepts — `X` under any client — so every
session keeps the grant **except** `X`. This is byte-for-byte the shape the ACP `noneOf`
rules (`rules/acp-c.n3`) emit, re-checked by the same `cond_applies` path. **Fail-closed:** a
`neq` recipient inside the reserved pair encoding cannot become an enforceable matcher (it
would impersonate a minted pair principal), so rather than emit an exception that silently
fails to bite — which would re-admit `X` — the whole rule falls back to the one-shot path
(never widen to a public everyone-except grant on an unenforceable exclusion).

**Combined `recipient eq A AND neq B` (one rule)** — [OPUS-4.8] sq-5037. The constraints are
AND-combined: the bridge emits a single `ConditionalGrant` headed by `A` (the positive `eq`) carrying
an `auth:exceptMatcher` carving out `B` (the `neq`) — the per-head exception. Only `A` keeps
the grant (everyone else fails the `eq` head; `B` is doubly excluded).

### Constraint-conditional **DENY** (`materialize_odrl_prohibition_conditional`) — [OPUS-4.8] sq-4r70

The dual of the conditional grant: a matched **prohibition** whose recipient/assignee constraints
map faithfully (same table above) is persisted as a re-checked `auth:ConditionalGrant` with
**`auth:effect auth:Deny`** rather than a frozen one-shot `auth:deny*`. The carve-out is
re-verified per session through the SAME `AuthIndex::accessible` path, and **composes with
deny-overrides**: a matching deny condition adds the target to the `denied` set, which is
subtracted from `allowed` — so a conditional deny **beats any allow** for the same
principal+target+mode. A prohibition `recipient eq carol` → a deny on carol's sessions;
`recipient neq bob` → a deny on everyone EXCEPT bob (an `exceptMatcher` carving bob back IN).
**Fail-closed:** a prohibition carrying an unmappable constraint (`purpose`/`dateTime`/`count`)
falls back to the one-shot `materialize_odrl_prohibition` (frozen) so the bound is still enforced;
a reserved-encoded recipient falls back to one-shot rather than emit a deny that silently fails
to bite (which would FAIL OPEN — a dropped deny widens access). Tracked as
`BridgeKind::ProhibitionConditional`; refresh re-checks the carve-out per session and retracts the
deny only when the prohibition is genuinely withdrawn (deny-retraction, sq-2pcf).

¹ **Stateful `odrl:count` enforcement** — [OPUS-4.8] sq-zi5w. `odrl:count` limits the *number
of times* a permission may be exercised; faithful enforcement is **stateful** (a usage counter
persisting across requests), which ACP — stateless, with static matcher accept-sets and no
per-session counter — cannot express, so the bridge keeps it **one-shot** (the limit is checked
once against any count value the request supplies, at materialization). The actual stateful
enforcement lives in `sparq-policy`'s opt-in `count-enforcement` feature
(`evaluate_and_exercise` + the injectable `UsageCounterStore`; atomic `try_consume`,
fail-closed on an unavailable counter — see the [`sparq-policy` README](../sparq-policy/README.md)).
Wiring that path *through* this stateless ACP bridge — so a bridged grant self-retracts once the
count is reached — is a distinct, more invasive change and is a **deferred bead**.

**`odrl:purpose` enforcement through the bridge (faithful, fail-closed)** — [OPUS-4.8] sq-q56r.
Purpose has no re-checked-condition analogue (above), so it stays **one-shot**: the bound is
checked **once**, against the request's stated purpose, by the same `sparq_policy::evaluate`
the one-shot `materialize_odrl_permission` / `materialize_odrl_prohibition` run — then a grant
(or `auth:deny*`) is materialized only if it held. So a purpose-gated rule is enforced
end-to-end through the *real* `accessible` / `query_as` path, never claimed-but-unchecked:
a **matching** stated purpose grants; a **mismatch** denies; a **missing** purpose is
*unprovable* → fail-closed (the permission does not grant; the prohibition is not carved out
— "no purpose stated" is never "any purpose allowed"). **Match is exact** (IRI/string
equality, or `isPartOf` over the named set, or `neq`) — **no** purpose hierarchy / DPV
subsumption. Because the check is one-shot, a purpose-gated grant is scoped to the request
party it was materialized for; a *changed* stated purpose is re-evaluated on the next
`refresh_odrl_grant` (sq-dpk4). A DPV purpose taxonomy / hierarchy match is a deferred bead.

**`odrl:dateTime` time-window enforcement through the bridge** — [OPUS-4.8] sq-idnv. Like
purpose, a time window has **no** re-checked-condition analogue (ACP matcher accept-sets are
static — there is no "now" dimension), so it stays **one-shot**: the window is checked once,
against the instant the request supplies (`Request::at(..)`), by the same `sparq_policy::evaluate`
the one-shot path runs — a grant (or `auth:deny*`) materializes only if the instant was inside the
window. A **missing** time is *unprovable* → fail-closed. Because the check is one-shot, a *lapsed*
window is caught on the next `refresh_odrl_grant` (re-evaluate with a `now` past the bound → the
grant emits nothing → retracted; sq-dpk4). Re-checking the clock live inside ACP is a deferred bead.

**Fail-safe on mixed constraints:** a condition is persisted **only** when *every* constraint
on the rule maps faithfully. A rule mixing a mappable recipient with an unmappable
`dateTime`/`purpose`/`count` falls back **entirely** to the one-shot path — persisting only
the recipient would silently drop the other bound and over-grant. Reserved-encoded recipient
IRIs are dropped from the grant head (anti-impersonation).

### Refresh / revocation of bridged grants — [OPUS-4.8] sq-dpk4

The `materialize_odrl_*` calls only **append**. When the ODRL policy changes — a permission is
**withdrawn**, a **time window lapses**, or a re-evaluation now **Denies** — a previously
materialized grant must lose access (the sq-h3uk/#280 correctness gap), and a wholesale static
WAC/ACP re-materialization must not silently clobber a still-valid bridged grant. A `PodStore`
tracks each bridged grant in a **ledger** and provides a refresh entry point:

- **Provenance.** Every bridged auth triple is mirrored verbatim into a separate reserved graph
  `<urn:sparq:auth-bridged>` (`AUTH_BRIDGED_GRAPH`): a triple is **bridged** iff it appears
  there, **static** otherwise. The enforcement reader (`AuthIndex`) is unchanged — it still
  reads `<urn:sparq:auth>`. The provenance graph lives in the reserved `urn:sparq:` space, so a
  loaded dataset cannot forge it.
- **Refresh / retract.** `PodStore::refresh_odrl_grant(&policy, &request, kind)` updates the
  tracked grant slot `(kind, target, party)` with the new policy / request context, then
  rebuilds the view as `static_baseline ∪ replay(still-valid bridged entries)`: it resets
  `<urn:sparq:auth>` to the static baseline captured at the last `materialize_wac`/`_acp`,
  clears the provenance graph, and re-evaluates every tracked `(policy, request)` through its
  original bridge entry point. An entry that no longer holds emits nothing → it is **retracted**
  (access gone). `refresh_odrl_grants()` (no args) replays everything as-tracked; a static
  re-materialization auto-reconciles (valid bridged grants are replayed back on top).
- **Fail-closed (access retraction).** A withdrawn / lapsed / now-Denied / now-prohibited /
  ambiguous re-evaluation of an **allow grant** loses access — the underlying evaluator is
  fail-closed, so on doubt the grant is retracted, never left stale. A **static** WAC/ACP grant
  is never in the ledger, never re-evaluated, and always in the captured baseline (captured as
  the materializer output verbatim, not by subtracting provenance — so a static grant
  byte-identical to a bridged one still survives a refresh) — refresh can neither widen nor drop it.
- **Deny retraction is asymmetric — [OPUS-4.8] sq-2pcf.** A bridged `auth:deny*` (a
  `BridgeKind::Prohibition` / `Policy` entry) carves access *out*, so retracting it *restores*
  access — that must happen only when the ODRL Prohibition is **definitely** withdrawn or
  lapsed, never on doubt. Reusing the grant rule would be **fail-OPEN** (an unprovable carve-out
  would restore access). Deny refresh therefore consults `sparq_policy::prohibition_status`
  (`Applies` / `Ambiguous` / `Withdrawn`): the deny is **retracted only on `Withdrawn`** — no
  prohibition structurally names the request, or every one carries a constraint that is
  *definitely* false given the supplied evidence (e.g. a `dateTime < bound` window with an
  actual time past the bound). On `Applies` **or** `Ambiguous` (a structurally-matching
  prohibition whose constraint is unprovable for lack of evidence) the deny is **kept**
  (re-emitted). A retracted deny may re-expose an allow grant for the same
  principal+target+mode — correct, because the prohibition is genuinely gone. Static `auth:deny*`
  rules are never in the ledger and so are never retracted.

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
