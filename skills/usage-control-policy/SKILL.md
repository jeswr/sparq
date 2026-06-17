---
name: usage-control-policy
description: Evaluate W3C ODRL 2.2 usage-control policies over RDF with the opt-in sparq-policy crate — parse an ODRL Set/Offer/Agreement into a typed Policy of Permission/Prohibition rules (action, target, assignee, Constraint, Duty), then evaluate an access Request to a fail-closed ALLOW/DENY Decision. Use when gating a query/asset by purpose, recipient, time window, count, or a duty obligation; when mapping ODRL to the sparq-solid WAC/ACP allow-deny model; or when wiring usage control above access control. Single-node base case; federated ODRL-to-MPC disclosure is deferred.
---

# usage-control-policy

`sparq-policy` is the declarative **usage-control** layer above access control. Where `sparq-solid` answers "may this agent **read** graph G?", `sparq-policy` answers "may this party **use** this asset *for purpose P, with obligation O, until time T, disclosing only to recipient R*?" — by evaluating a [W3C ODRL 2.2](https://www.w3.org/TR/odrl-model/) policy.

It parses an ODRL policy from RDF into a typed model (`Policy → {permissions, prohibitions}`, each `Rule` carrying an `Action`, `target`, `assignee`/`assigner`, `Constraint`s and `Duty`s) and evaluates an access `Request` to a **fail-closed** `Decision { allow, matched_rules, unmet_constraints }`. This is the **single-node base case**: ODRL over one node's data, reducing to the same allow/deny shape `sparq-solid` enforces.

> **Scope.** Single-node only. The headline federated-disclosure / ODRL→MPC composition (per-node ODRL drives the `sparq-mpc` disclosed-vs-hidden split; ODRL `Duty` → ZK proof obligation) is **deferred** — it inherits the MPC honest-majority/LAN envelope and the open ZK-soundness remediation. See `research/feature-research-odrl-policy.md`.

## Prerequisites

- **Cargo dep** — `sparq-policy` is `publish = false`, a non-default workspace member (nothing in core depends on it; `cargo tree -p sparq-core` never shows it). Add it as a path dep:
  ```toml
  sparq-policy = { path = "crates/sparq-policy" }
  ```
- No external toolchain. It depends only on `sparq-core` + `sparq-engine` + `oxrdf`; zero `unsafe`.

## Quickstart

Parse an ODRL `Set` and evaluate a time-windowed permission. Compiles and runs against the current API.

```rust
use sparq_policy::{evaluate, parse_policy_str, Request, Value};

// alice MAY read asset-X, on or before 2026-12-31, for research purpose.
let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/1> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <urn:asset/x> ;
    odrl:assignee <https://alice.ex/me> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
                      odrl:rightOperand "2026-12-31T00:00:00Z"^^xsd:dateTime ] ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:eq ;
                      odrl:rightOperand <urn:purpose/research> ] ] .
"#;
let policy = parse_policy_str(ttl, "turtle").expect("parse");

const ODRL: &str = "http://www.w3.org/ns/odrl/2/";
let req = Request::new(format!("{ODRL}read"))
    .on("urn:asset/x")
    .by("https://alice.ex/me")
    .with(format!("{ODRL}dateTime"), Value::DateTime("2026-06-16T09:00:00Z".into()))
    .with(format!("{ODRL}purpose"),  Value::Iri("urn:purpose/research".into()));

let d = evaluate(&policy, &req);
assert!(d.allow);                      // in-window, right party, right purpose
assert_eq!(d.matched_rules.len(), 1);  // the granting permission, for audit
```

## The model

- **`Policy`** — `permissions: Vec<Rule>`, `prohibitions: Vec<Rule>`, optional `iri`. `Set`/`Offer`/`Agreement` parse identically (subtype affects contracting, not single-node eval).
- **`Rule`** — `action: Action`, `target`/`assignee`/`assigner: Option<String>`, `constraints: Vec<Constraint>`, `duties: Vec<Duty>` (duties on permissions only).
- **`Action`** — full IRI. `odrl:use` is the **umbrella** action: a permission for `use` permits *any* requested action. All others match by exact IRI.
- **`Constraint`** — `(left: leftOperand-IRI, operator, right: Value)`. The request supplies the *actual* value for `left` in its `context`; the constraint's `right` is the bound.
- **`Duty`** — an obligation (`action`) that must be in the request's `discharged_duties` set, or the permission is denied.

## Evaluation semantics (fail-closed)

1. A `Rule` **matches** when its action permits the request action, its `target`/`assignee` (if set) agree, and **every** `Constraint` is satisfied (logical AND).
2. A `Permission` grants iff it matches **and** all its `Duty`s are discharged.
3. A matching `Prohibition` **overrides** any permission (carve-out — ODRL Formal Semantics conflict default).
4. **DENY by default:** no matching+discharged permission, or any matching prohibition ⇒ DENY. An empty/malformed policy denies everything; a constraint with no request value, an unknown operator, or a structurally incomplete constraint all fail closed.

## Constraint operators

`eq`, `neq`, `lt`, `lteq`, `gt`, `gteq`, `isPartOf` (set membership: `right` is a `|`/space/comma-separated set — **or**, for the taxonomic dimensions `purpose`/`spatial`, a transitive subsumption match over a request-supplied closure, see below), `isA` (identity, ≈ `eq`). Numeric (`xsd:integer`/`decimal`/`double`/…) and `xsd:dateTime`/`date` operands compare by magnitude/instant; everything else by IRI/string value. Order comparison on non-orderable values is `false` (fail-closed). `dateTime` ordering compares the lexical form — mixed-offset normalization is a deferred bead.

## `odrl:spatial` region enforcement + region `isPartOf` trees — [OPUS-4.8] sq-wukl

A `spatial` constraint gates a rule to a **geographic region** (`spatial isPartOf <country/EU>`, "anywhere in the EU"). The new left-operand IRI is `ODRL_SPATIAL` (`odrl:spatial`); supply the request's region with `.with(ODRL_SPATIAL, Value::Iri(region))`, gated through the **same** `evaluate` constraint path as every other dimension.

The spatial dimension is **taxonomic**, so it reuses the **same** caller-supplied subsumption closure the DPV purpose taxonomy uses — there is *no* separate spatial evidence channel. Declare the region `isPartOf` tree with `.with_purpose_subsumption(narrower, broader)` (one edge) or `.with_purpose_taxonomy([(n, b), …])` (bulk), and a `spatial isPartOf <EU>` constraint is then satisfied by a request whose stated region is the EU **or transitively part-of** it — `Berlin ⊑ DEU ⊑ EU`.

- **Honesty / fail-closed:** the tree is *the requester's asserted subsumption*, never invented. With **no** edge supplied, `spatial` matching is exact membership (fully backward compatible) and a sub-region does **not** grant a broad-region permission — a missing edge fails closed, like a missing context value. Cycle-safe (the closure tolerates a malformed `A ⊑ B ⊑ A`).
- **Audit:** `spatial_status(&rule, &request) -> SpatialMatch` (`Satisfied`/`DefinitelyUnsatisfied`/`Unprovable`/`NotConstrained`) is the spatial twin of `purpose_status` — it runs the same subsumption-aware path `evaluate` does.

```rust,ignore
use sparq_policy::{evaluate, Request, Value, ODRL_SPATIAL};
// policy: distribute spatial isPartOf <country/EU>
let req = Request::new("http://www.w3.org/ns/odrl/2/distribute").on("urn:asset/x")
    .with(ODRL_SPATIAL, Value::Iri("urn:country/DEU".into()))
    .with_purpose_subsumption("urn:country/DEU", "urn:country/EU"); // DEU ⊑ EU
assert!(evaluate(&policy, &req).allow); // a sub-region of the named region grants
```

## Building the request

`Request::new(action_iri)` then chain `.on(target)`, `.by(party)`, `.with(left_operand_iri, Value)` for each context dimension (`dateTime`, `purpose`, `recipient`, `count`, `spatial`, …), `.with_purpose_subsumption(narrower, broader)` / `.with_purpose_taxonomy([(n, b), …])` per asserted subsumption edge (the DPV-purpose taxonomy / spatial region trees, above — one closure for both taxonomic dimensions), and `.discharge(duty_action_iri)` per discharged duty. `Value` is `Iri` | `Str` | `Num(f64)` | `DateTime(String)`. For purpose specifically, prefer the first-class `.for_purpose(Value)` (sugar over `.with(ODRL_PURPOSE, ..)`; read it back via `req.purpose()`); for the evaluation time prefer `.at(instant)` (sugar over `.with(ODRL_DATETIME, Value::DateTime(..))`; read it back via `req.request_time()`).

## `odrl:purpose` enforcement (faithful, fail-closed) — [OPUS-4.8] sq-q56r

A purpose constraint restricts a rule to a stated *purpose of use*. The request carries its purpose as **evidence** (`.for_purpose(Value)`), and it is gated through the **same** `evaluate` constraint path as every other dimension — so it is actually checked end-to-end (no claimed-but-unchecked enforcement), including through the opt-in bridge's real `accessible` / `query_as` path.

- **Match → grant; mismatch → deny; missing purpose → fail-closed.** A request stating **no** purpose is *unprovable*: a purpose-gated permission does **not** grant, and a purpose-gated prohibition is **not** withdrawn. "No purpose stated" is **never** treated as "any purpose allowed".
- **Match semantics (the boundary — do not over-claim):** **exact** IRI/string equality (`eq`/`isA`), or membership in the explicit `isPartOf` purpose *set* the constraint names, or `neq` (≠ the named one; still requires a stated purpose).
- **DPV / purpose-taxonomy subsumption — [OPUS-4.8] sq-z3ve.** Supply the request a purpose taxonomy with `.with_purpose_subsumption(narrower, broader)` (one `skos:broader`/`rdfs:subClassOf`/`dpv:isSubTypeOf` edge) or the bulk `.with_purpose_taxonomy([(n, b), …])`, and a purpose constraint naming `B` is also satisfied by a stated purpose `P` that is **transitively narrower** than `B` (`P ⊑ B`) — a `research` permission covers `clinical-research`, and a `neq research` carve-out *also* excludes that sub-purpose. **Sound, never over-claimed:** the `⊑` relation is the **caller-supplied transitive closure only** (never inferred from IRI string structure); with no taxonomy supplied it is byte-for-byte the exact-IRI base case, the broader-under-narrower direction never matches, and access is never widened on an unproven relation. The **same** closure also drives the `odrl:spatial` region tree (above) — one subsumption evidence channel for both taxonomic dimensions.
- **Audit:** `purpose_status(&rule, &request) -> PurposeMatch` reports exactly what the evaluator checks — `Satisfied` / `DefinitelyUnsatisfied` / `Unprovable` / `NotConstrained` (it runs the same subsumption-aware path `evaluate` does).
- Through the bridge, purpose stays **one-shot** (checked once at materialization; see the mapping table below) — it has no re-checked-condition analogue, so a changed purpose is re-evaluated on the next `refresh_odrl_grant`.

## `odrl:recipient` enforcement + `neq` / "everyone-except" — [OPUS-4.8] sq-5037

A `recipient` constraint restricts **who the data may be disclosed to**. The recipient-of-data is the requesting party, so a request that names a party (`.by(webid)`) but supplies **no** explicit `odrl:recipient` context is read as `recipient = party` — i.e. a `recipient` rule gates on *who is asking*, end-to-end through the same `evaluate` constraint path as every other dimension. An explicit `.with(ODRL_RECIPIENT, Value)` still takes precedence (the disclosure target need not be the authenticated principal in every deployment).

- **`recipient neq X` ("everyone EXCEPT X"):** grants/forbids for any recipient that is **not** `X`. A request whose recipient IS `X` is the carve-out (deny on a permission; the prohibition no longer carves *this* party out). **Missing identity (no `odrl:recipient` AND no party) is *unprovable* → fail-closed:** a `neq` permission does **not** grant to an unknown recipient, and a `neq` prohibition is **not** withdrawn.
- **`eq`/`isA`** = recipient IS the named party; **`isPartOf`** = recipient ∈ static set. Match is **exact** IRI/string equality (no recipient hierarchy).
- **Audit:** `recipient_status(&rule, &request) -> RecipientMatch` reports exactly what the evaluator checks — `Satisfied` / `DefinitelyUnsatisfied` / `Unprovable` / `NotConstrained` (the recipient dual of `purpose_status`).
- **Combined `recipient eq A AND neq B` (one rule):** the constraints are AND-combined — the recipient must BE `A` and must NOT BE `B`. Through the bridge this emits a single `ConditionalGrant` headed by `A` (positive) carrying an `exceptMatcher` carving out `B` (the per-head exception).
- Through the bridge, `recipient neq X` maps to an ACP **`noneOf`** exception (see the mapping table + the bridge note below) — re-checked per session.

## `odrl:dateTime` time-window enforcement (faithful, fail-closed) — [OPUS-4.8] sq-idnv

A `dateTime` constraint gates a permission/prohibition to a **time window** — e.g. `dateTime lteq T` (valid until `T`), `dateTime gteq T` (valid from `T`), or a two-sided window (`gteq lower` + `lteq upper`, AND-combined). The actual instant is the request's **evaluation time**, supplied as `xsd:dateTime` evidence via the first-class `Request::at(instant)` sugar (over `.with(ODRL_DATETIME, Value::DateTime(..))`); read it back via `req.request_time()`.

- **Semantics:** inside the window → grant (permission) / carve-out applies (prohibition); outside → deny / carve-out gone. Instants compare by magnitude (the `lt`/`lteq`/`gt`/`gteq`/`eq`/`neq` ordering). **Missing time → *unprovable* → fail-closed:** a time-gated permission does **not** grant on an unknown clock; a time-gated prohibition is **not** withdrawn (never silently read as "any time").
- **Audit:** `datetime_status(&rule, &request) -> DateTimeMatch` reports exactly what the evaluator checks — `Satisfied` / `DefinitelyUnsatisfied` / `Unprovable` / `NotConstrained` (the temporal dual of `purpose_status`/`recipient_status`).
- Through the bridge, `dateTime` stays **one-shot** (frozen at materialization — ACP matcher accept-sets are static, there is no "now" dimension to re-check; see the mapping table). A changed window is re-evaluated on the next `refresh_odrl_grant` — a lapsed window then retracts the grant. (`dateTime` ordering compares the lexical form — mixed-offset normalization is a deferred bead.)

## `odrl:count` enforcement (stateful, opt-in `count-enforcement`) — [OPUS-4.8] sq-zi5w

`odrl:count` limits the **number of times** a permission may be exercised ("read at most 5 times"). Unlike `purpose`/`dateTime`/`recipient` (stateless — a single test against evidence the request carries), a count limit is **stateful**: it lives in a usage counter that persists *across* requests. Behind the off-by-default `count-enforcement` feature on `sparq-policy` (the default build is unchanged — it treats `odrl:count` as the stateless numeric comparison), `evaluate_and_exercise` runs the **real** `evaluate` decision and, on a grant, **atomically consumes** one unit of budget.

```rust,ignore
// cargo: sparq-policy with --features count-enforcement
use sparq_policy::{evaluate_and_exercise, InMemoryCounterStore, Request};
let store = InMemoryCounterStore::new();           // injectable UsageCounterStore
let req = Request::new("http://www.w3.org/ns/odrl/2/read")
    .on("urn:asset/x").by("https://alice.ex/me");  // policy: count lteq 3
for _ in 0..3 { assert!(evaluate_and_exercise(&policy, &req, &store).allow); }
assert!(!evaluate_and_exercise(&policy, &req, &store).allow); // 4th: limit reached → DENY
```

- **Semantics through the real path.** A grant requires the base `evaluate` to grant (the `odrl:count` constraint is stripped from *permissions* for that base check, since the stateless evaluator would otherwise deny for a missing count value; everything else — action/target/assignee, prohibitions, purpose, dateTime, recipient, duties — is checked unchanged) AND the count budget to have room. **First *N* exercises grant; the *(N+1)*th denies.** A *denied* request consumes **nothing**. `lteq N`/`eq N` = at most *N*; `lt N` = at most *N-1*; `gt`/`gteq`/`neq`/`isPartOf` express no ceiling (left to the stateless path).
- **Counter-store seam.** `UsageCounterStore` is an injectable trait; `InMemoryCounterStore` is the in-memory reference impl. The budget key is `(rule_id, party, target)` (`CountKey`) — **per-assignee, per-asset, per-rule** (one party exhausting a limit never locks out another; the same rule on a different target counts separately). A partyless/targetless request shares the `""` bucket.
- **Atomicity / concurrency boundary (load-bearing).** The single mutating op is the **atomic** `try_consume` (the whole check-and-consume under one lock in the in-memory store) — deliberately **not** a `current()`+`increment()` pair, which would reintroduce a TOCTOU race (two concurrent exercises both reading `N-1`, both granting → over-grant). A concurrency test asserts exactly the limit is granted under 16 threads, never one more. **Multiple `odrl:count` constraints on one rule** all bind the *same* counter (`CountKey` is per `(rule, party, target)`, never per-constraint), so they are several *bounds on one budget*: the **tightest (minimum) bound governs**, and an exercise is a **single** atomic `try_consume` against that one effective limit — exactly one unit per exercise (never one-per-constraint), no read-then-consume gap. The multi-limit case is therefore as atomic as the single-limit case (a concurrency test asserts no over-grant under two constraints), closing the prior pre-check→consume window (sq-ea27). The in-memory store is atomic **in-process only**; a **distributed** deployment needs a shared atomic backend (Redis `INCR`, SQL `UPDATE … WHERE consumed < limit RETURNING`) and MUST honour the same `try_consume` atomicity contract. Cross-process atomicity is a **deferred bead**.
- **Fail-closed.** `ConsumeResult::Unavailable` (store outage / poisoned lock) or a malformed limit → **DENY** — never silently "unlimited". `count_status(&rule, &request, &store) -> CountStatus` is the side-effect-free audit surface (`Satisfied{consumed,limit}` / `DefinitelyUnsatisfied{..}` / `Unprovable` / `NotConstrained`), the count dual of `purpose_status`.
- **Deferred (honest).** The stateless ODRL→ACP **bridge does NOT wire this stateful path** — a bridged ACP grant does not self-retract on count exhaustion (ACP has no usage counter; the bridge keeps `odrl:count` one-shot/unmappable, see the mapping table). This feature is the evaluator + store seam such a bridge would build on.

## Static conflict + containment analysis (request-free) — [OPUS-4.8] sq-zabv

Where `evaluate` answers *"may THIS request go through?"*, two **request-free** functions answer questions about the policies themselves (the query-containment comparison semantics, [arXiv 2509.05139 §comparison](https://arxiv.org/html/2509.05139v1)). Both are always compiled (no feature, no deps) and both are **sound / fail-closed — never over-claimed**.

```rust,ignore
use sparq_policy::{contains, detect_conflicts, Containment, Overlap};
// Lint a policy for permission/prohibition conflicts.
for c in detect_conflicts(&policy) {
    // c.permission_id is (wholly, if c.overlap == Overlap::Certain) carved out by c.prohibition_id
}
// Does the provider's offer permit everything the requester asks?
match contains(&provider_policy, &request_policy) {
    Containment::Contains => { /* every ask is covered */ }
    Containment::NotContained => { /* a witness ask the offer denies */ }
    Containment::Unknown => { /* undecidable under the conservative comparison */ }
}
```

- **`detect_conflicts(&policy) -> Vec<Conflict>`** — every permission/prohibition pair whose request footprints overlap (a prohibition carves the permission out — deny-overrides). `Conflict { permission_id, prohibition_id, overlap, action, target }`. `overlap` is `Overlap::Certain` **only** when the structural attributes (action / target / assignee) prove an overlap AND the prohibition adds **no** constraint the permission lacks (so the carve-out covers the *whole* permission); otherwise `Overlap::Possible` (the rules *might* overlap but we cannot prove they always do). A pair that **provably never** overlaps (disjoint concrete action / target / assignee) is omitted. Conflict is strictly across the permission/prohibition divide — two permissions never conflict.
- **`contains(outer, inner) -> Containment`** — does `outer` permit everything `inner` permits (refinement / requester-vs-provider containment)? `Containment::Contains` **only** when every `inner` permission is *provably* subsumed by some `outer` permission AND no `outer` prohibition could carve into it; `Containment::NotContained` when an `inner` permission *provably* grants a request `outer` denies (disjoint concrete target/action, or `inner` leaves a dimension `outer` restricts wide open); `Containment::Unknown` otherwise. An `inner` with no permissions is contained vacuously.
- **Soundness boundary (honest).** Constraint satisfiability / query containment is undecidable in the general ODRL constraint language. This module decides only what it can *prove* from rule structure plus a conservative per-dimension constraint comparison (identical constraints; `eq v` admitted by an `outer` bound; a *tighter* same-direction order bound — `lt`/`lteq`, `gt`/`gteq` — implying a looser one). Everything else degrades to `Possible` / `Unknown` — it **never** reports `Certain` / `Contains` it cannot prove (that is the fail-OPEN failure mode: claiming an ask is covered when it is not). It also does not (yet) prove `NotContained` from a *looser* inner numeric bound reaching above a tighter outer one — that case honestly returns `Unknown`. DPV/`isPartOf` set-subset refinement is a deferred bead.

## Bridge to WAC/ACP enforcement (opt-in `odrl-bridge`) — [OPUS-4.8] sq-h3uk

`sparq-solid` can **materialize** a matched ODRL permission into its `<urn:sparq:auth>` AUTH_GRAPH so the existing graph-level WAC/ACP enforcement applies it — **no new enforcement engine**. Behind the off-by-default `odrl-bridge` cargo feature on `sparq-solid` (it pulls in `sparq-policy` only when enabled; the default solid build stays ODRL-free). This is the **single-node** bridge of epic sq-3183, **research-track**, NOT the (gated) federated/ZK-disclosure path.

```rust,ignore
// cargo: sparq-solid with --features odrl-bridge
use sparq_solid::{PodStore, Session, Mode};
use sparq_policy::Request;
let req = Request::new("http://www.w3.org/ns/odrl/2/read")
    .on("https://pod.ex/notes/n1").by("https://alice.ex/card#me");
// On a definite Permit, appends `alice auth:read n1` to the auth view, then reindexes.
let out = store.materialize_odrl_permission(&policy, &req);
assert!(out.granted);
// …now honoured by the unchanged enforcement path:
assert!(!store.accessible(&Session { agent: Some("https://alice.ex/card#me"), client: None }, Mode::Read).is_empty());
```

**Action → mode** (the ODRL *request* action; conservative — narrowest mode only): `read`/`display`/`present`/`print`/`play` → `acl:Read`; `append` → `acl:Append`; `modify`/`delete`/`write` → `acl:Write`; **anything else (incl. the `odrl:use` umbrella) is unmapped → no grant**. `use` is left unmapped because it subsumes every action (mapping it would have to pick the widest mode) — request `odrl:read` explicitly; a `use` permission still grants that concrete request.

**Fail-closed:** a grant is materialized only on a *definite Permit* AND a *mappable action* AND a *concrete party (WebID) + target graph*. A Deny, unsatisfied constraint, undischarged duty, unmapped action, or partyless/targetless request materializes **nothing**.

### Prohibitions → explicit `auth:deny*` (deny-overrides) — [OPUS-4.8] sq-w693

A matched ODRL **Prohibition** is materialized as the dual triple — `principal auth:deny<Mode> target` — via `materialize_odrl_prohibition` (or `materialize_odrl_policy`, which does both sides at once). The **same** action→mode mapping picks the mode, so the deny predicate is `auth:denyRead` / `auth:denyWrite` / `auth:denyAppend` / `auth:denyControl`.

```rust,ignore
// Prohibition side: appends `alice auth:denyWrite n1`, then reindexes.
let dreq = Request::new("http://www.w3.org/ns/odrl/2/modify")
    .on("https://pod.ex/notes/n1").by("https://alice.ex/card#me");
let out = store.materialize_odrl_prohibition(&policy, &dreq);
assert!(out.prohibited);
// Both sides of a policy at once (permit grant + matched-prohibition deny):
let out = store.materialize_odrl_policy(&policy, &req);
```

**Deny-overrides:** the deny is honoured by the **existing, unchanged** enforcement — the session layer already computes `∪ allow ∖ ∪ deny` (`AuthIndex::accessible`) and `Mode::from_pred` already parses `auth:deny*`, so a materialized deny **beats any allow grant** for the same principal+target+mode. No new enforcement engine; the bridge only emits the triple. (Within one policy the ODRL evaluator *also* applies deny-overrides upstream, so the permit allow triple is never even emitted when a prohibition carves the request out.)

**Fail-closed (deny):** a deny is materialized only when a prohibition **matches** the request (decided by `sparq_policy::matched_prohibition` — the evaluator's own conflict test, *not* `Decision.allow == false`, which conflates a carve-out with a plain no-permission deny) AND the action is mappable AND the party+target are concrete. An unmatched / unmapped / partyless / targetless prohibition materializes **nothing** — and an unmappable carve-out is *reported* in `reasons`, never silently dropped (dropping a deny would widen access).

### Persisting a constraint as a re-checked ACP condition (`materialize_odrl_permission_conditional`) — [OPUS-4.8] sq-hiz4

The one-shot `materialize_odrl_permission` *freezes* every constraint into a single allow scoped to the supplied request party. `materialize_odrl_permission_conditional` instead persists a **faithfully-mappable** constraint as an ACP `auth:ConditionalGrant` (the same `noneOf` machinery the ACP materializer emits), so the granted agent is **re-checked per session** through the unchanged enforcement path — not re-running the ODRL evaluator. A constraint with **no** faithful ACP analogue keeps the one-shot behaviour (checked once, frozen).

```rust,ignore
// The recipient constraint names carol — NOT whoever materializes the grant.
let req = Request::new("http://www.w3.org/ns/odrl/2/read")
    .on("https://pod.ex/notes/n1").by("https://alice.ex/card#me");
store.materialize_odrl_permission_conditional(&policy, &req); // policy: recipient eq carol
// Re-checked per session: carol is granted; alice (the materializer) is NOT.
```

**Constraint → ACP condition mapping table** (fail-closed: map ONLY when the ACP analogue is the *same or stricter*):

| ODRL constraint | Operator | ACP analogue | Faithful? | Behaviour |
|---|---|---|---|---|
| `odrl:recipient` / `odrl:assignee` | `eq` / `isA` | `auth:agent <webid>` on a `ConditionalGrant` (agent matcher) | ✅ recipient-of-data IS the session agent | **persisted, re-checked per session** |
| `odrl:recipient` / `odrl:assignee` | `isPartOf` (static set) | one `auth:agent` head per member (OR) | ✅ set membership = agent ∈ set | **persisted** (one grant/member) |
| `odrl:recipient` / `odrl:assignee` | `neq` ("everyone EXCEPT X") | `auth:Public` `ConditionalGrant` + `auth:exceptMatcher` carving out `X` (ACP `noneOf`) | ✅ everyone-except is exactly the ACP `noneOf` shape | **persisted, re-checked per session** ([OPUS-4.8] sq-5037) |
| `odrl:recipient` / `odrl:assignee` | order (`lt`/`gt`/…) | (none — not meaningful on a recipient) | ❌ | **one-shot** (frozen) |
| `odrl:purpose` | any (incl. `isPartOf` DPV hierarchy) | (none — a client app ≠ a purpose-of-use) | ❌ ACP session carries no purpose dimension; client-matcher would over-grant | **one-shot** (frozen) |
| `odrl:spatial` | any (incl. `isPartOf` region hierarchy) | (none — ACP session carries no region) | ❌ no spatial dimension to re-check | **one-shot** (frozen) ([OPUS-4.8] sq-wukl) |
| `odrl:dateTime` / time window | `lteq` / `lt` / `gteq` / `gt` | (none — matcher accept-sets are static; no "now") | ❌ ACP has no clock dimension to re-check | **one-shot** (frozen) |
| `odrl:count` | any | (none — ACP is stateless; no per-session usage counter) | ❌ in the bridge | **one-shot** (frozen) in the bridge — stateful enforcement lives in `sparq-policy`'s `count-enforcement` feature (`evaluate_and_exercise`), not yet wired through ACP |
| any unrecognised left-operand | any | (none) | ❌ | **one-shot** (frozen) |
| *no constraint* | — | `auth:agent auth:Public` (action/target/duties already held) | ✅ | persisted (public) |

**The `neq` / "everyone-except" → `noneOf` shape ([OPUS-4.8] sq-5037):** a `recipient neq X` rule emits a `ConditionalGrant` whose head is the positive recipient set (or `auth:Public` if there is none) plus one `auth:exceptMatcher <m>` per excluded `X`; the matcher `<m>` carries the accept-set facts the session layer reads (`solidx:acceptsAgentP <X>` + `solidx:acceptsClientP auth:AnyClient`). `AuthIndex` then suppresses the grant for any session the matcher accepts — i.e. for `X` under any client — so everyone keeps the grant **except** `X`. This is byte-for-byte the shape the ACP `noneOf` rules (`rules/acp-c.n3`) emit, re-checked by the same `cond_applies` code path. A `neq` recipient inside the reserved pair encoding **cannot** become an enforceable matcher (it would impersonate a minted pair principal), so rather than emit an exception that silently fails to bite (which would re-admit `X`), the whole rule falls back to the one-shot path — **fail-closed: never widen to a public everyone-except grant on an unenforceable exclusion**.

**Fail-safe on mixed constraints:** a persisted condition is emitted ONLY when **every** constraint on the rule maps faithfully. A rule mixing a mappable recipient with an unmappable `dateTime`/`purpose`/`count` falls back **entirely** to the one-shot path — persisting only the recipient would silently drop the time/purpose/count bound and over-grant. Recipient IRIs inside the reserved pair encoding (`urn:sparq:` / `&client=`) are dropped from the grant head (anti-impersonation). The two ODRL "any recipient" sentinels fold onto auth principals: `odrl:All`/`odrl:Group` → `auth:Public`, `odrl:AllConnections` → `auth:Authenticated`.

**Why only recipient/assignee maps:** the ACP session re-check carries exactly `(agent, client)`. The recipient-of-data is precisely the session **agent**, so an agent matcher re-checks it with identical semantics. Purpose, time, and count have **no** stateless `(agent, client)` analogue, so persisting them would require either freezing the check (= the one-shot path, already correct) or a looser approximation that could over-grant — rejected.

### Constraint-conditional DENY (`materialize_odrl_prohibition_conditional`) — [OPUS-4.8] sq-4r70

The dual of the conditional grant: a matched ODRL **prohibition** whose recipient/assignee constraints map faithfully is persisted as a re-checked `auth:ConditionalGrant` with **`auth:effect auth:Deny`** (rather than a frozen one-shot `auth:deny*`). The carve-out is re-verified per session through the SAME `accessible`/`query_as` path, and **composes with deny-overrides**: the session layer subtracts `∪ deny` from `∪ allow`, so a conditional deny that applies to a session removes the target from its accessible set — beating any allow for the same principal+target+mode.

- A prohibition `recipient eq carol` → a deny condition headed by carol (only carol's sessions are denied); `recipient neq bob` → a deny on **everyone EXCEPT bob** (an `exceptMatcher` carving bob back IN to access). Same mapping table as the allow path (recipient/assignee `eq`/`isA`/`isPartOf`/`neq` are faithful; `purpose`/`dateTime`/`count` are unmappable).
- **Fail-safe fallback:** a prohibition carrying an unmappable constraint (`purpose`/`dateTime`/`count`) falls back to the one-shot `materialize_odrl_prohibition` (frozen, materialized iff the prohibition currently matches) so the bound is still enforced — a persisted deny condition is emitted ONLY when every constraint maps faithfully. A reserved-encoded recipient cannot become a matcher, so the rule falls back to one-shot rather than emit a deny that silently fails to bite (which would FAIL OPEN — a dropped deny widens access).
- **Refresh.** Tracked as `BridgeKind::ProhibitionConditional`. On refresh the recipient carve-out is re-checked per session, so the deny is re-emitted while the prohibition still structurally names the request; a withdrawn prohibition re-emits nothing → the deny condition is **retracted** → access restored. A one-shot fallback uses the asymmetric deny-retraction rule below (re-emit on `Ambiguous`, retract only on a definite `Withdrawn`).

### Refresh / REVOCATION of bridged grants on policy change — [OPUS-4.8] sq-dpk4

The `materialize_odrl_*` calls only ever **append**. When the underlying ODRL policy changes — a permission is **withdrawn**, a **time window lapses**, or a re-evaluation now **Denies** — the previously-materialized grant would otherwise stay in the auth view, so access that should be gone persists (the sq-h3uk/#280 correctness gap). And a wholesale static WAC/ACP re-materialization rebuilds `<urn:sparq:auth>` and would drop every bridged grant. Both are reconciled by a **bridge ledger** + a refresh entry point.

- **Provenance.** Every auth triple the bridge writes into `<urn:sparq:auth>` is mirrored verbatim into a separate reserved graph `<urn:sparq:auth-bridged>` (`AUTH_BRIDGED_GRAPH`). A triple is **bridged** iff it appears there, **static** otherwise — so bridged and static grants are structurally distinguishable without inspecting predicate shape, and the enforcement reader (`AuthIndex`) is unchanged (it still reads `<urn:sparq:auth>`). The provenance graph is in the reserved `urn:sparq:` space, so a loaded dataset cannot forge it.
- **Refresh / retract.** `PodStore::refresh_odrl_grant(&new_policy, &new_request, kind)` updates the tracked grant slot `(kind, target, party)` with the new policy / request context, then rebuilds the view as `static_baseline ∪ replay(still-valid bridged entries)`: it resets `<urn:sparq:auth>` to the static baseline captured at the last `materialize_wac`/`materialize_acp`, clears the provenance graph, and re-evaluates every tracked `(policy, request)` through its original bridge entry point. An entry that no longer holds emits nothing → it is **retracted** (access gone). `refresh_odrl_grants()` (no args) replays everything as-tracked (used to reconcile after a static re-materialization, which is automatic).
- **Fail-closed (security-sensitive — access retraction).** A withdrawn / lapsed / now-Denied / now-prohibited / ambiguous re-evaluation of an **allow grant** loses access; the underlying evaluator is fail-closed, so on any doubt the grant is retracted, never left stale. A **static** WAC/ACP grant is never in the ledger, never re-evaluated, and always in the captured baseline (captured as the `install_auth_view` output verbatim, not by subtracting provenance — so a static grant byte-identical to a bridged one still survives) — refresh can neither widen nor drop it.

#### Deny RETRACTION is asymmetric to grant retraction — [OPUS-4.8] sq-2pcf

A materialized `auth:deny*` (from a `BridgeKind::Prohibition` / `Policy` entry) is **retracted on the OPPOSITE rule**: a deny carves access *out*, so retracting it *restores* access — that must happen only when the ODRL Prohibition is **definitely** withdrawn or lapsed, never on doubt. Reusing the grant rule (drop the deny whenever `matched_prohibition` no longer matches) would be **fail-OPEN**: an *ambiguous* re-eval — a prohibition still structurally naming the request but carrying a constraint the refresh request gives no evidence for — would silently restore access.

So deny retraction consults `sparq_policy::prohibition_status`, a three-valued refinement of `matched_prohibition`:

| `ProhibitionStatus` | meaning | deny on refresh |
|---|---|---|
| `Applies` | a prohibition still carves the request out | **kept** (re-emitted) |
| `Ambiguous` | still structurally names it, but a constraint is unprovable (no evidence) | **kept** (re-emitted) |
| `Withdrawn` | no prohibition names it, or every one is *definitely* false given the evidence | **retracted** (dropped) |

"Definitely false" means the refresh request supplied evidence for the dimension and the comparison failed (e.g. a `dateTime < 2026-01-01` window with an actual time of `2026-06-01` — provably lapsed). A retracted deny composes with deny-overrides: it may re-expose an allow grant for the same principal+target+mode — correct, *because the prohibition is genuinely gone*. Static (non-bridged) `auth:deny*` rules are never in the ledger and so are never re-evaluated or retracted.

```rust,ignore
// alice was bridged a read grant; the policy then WITHDRAWS the permission.
let (matched, retracted) =
    store.refresh_odrl_grant(&withdrawn_policy, &req, BridgeKind::Permission);
// matched == true, retracted == 1 → alice can no longer read (through accessible/query_as).

// A bridged DENY is the dual: retracted only when the Prohibition is DEFINITELY gone.
let (matched, retracted) =
    store.refresh_odrl_grant(&withdrawn_prohibition, &write_req, BridgeKind::Prohibition);
// definite withdrawal → retracted == 1 (deny gone, access restored if an allow exists);
// ambiguous re-eval (no constraint evidence) → retracted == 0 (deny KEPT, fail-closed).
```

## Learn more

- Crate README: [`crates/sparq-policy/README.md`](../../crates/sparq-policy/README.md)
- Design record: `research/feature-research-odrl-policy.md` (epic sq-3183)
- Sibling access-control skill: [`skills/http-server`](../http-server/SKILL.md) (Solid WAC/ACP via `sparq-solid`)
- W3C [ODRL Information Model 2.2](https://www.w3.org/TR/odrl-model/) · [Formal Semantics](https://w3c.github.io/odrl/formal-semantics/)
