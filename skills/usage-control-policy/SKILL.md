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

`eq`, `neq`, `lt`, `lteq`, `gt`, `gteq`, `isPartOf` (set membership: `right` is a `|`/space/comma-separated set), `isA` (identity, ≈ `eq`). Numeric (`xsd:integer`/`decimal`/`double`/…) and `xsd:dateTime`/`date` operands compare by magnitude/instant; everything else by IRI/string value. Order comparison on non-orderable values is `false` (fail-closed). `dateTime` ordering compares the lexical form — mixed-offset normalization is a deferred bead.

## Building the request

`Request::new(action_iri)` then chain `.on(target)`, `.by(party)`, `.with(left_operand_iri, Value)` for each context dimension (`dateTime`, `purpose`, `recipient`, `count`, `spatial`, …), and `.discharge(duty_action_iri)` per discharged duty. `Value` is `Iri` | `Str` | `Num(f64)` | `DateTime(String)`.

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
| `odrl:recipient` / `odrl:assignee` | `neq` / order | "everyone EXCEPT" needs a per-session `noneOf` | ❌ no faithful single-grant analogue | **one-shot** (frozen) |
| `odrl:purpose` | any | (none — a client app ≠ a purpose-of-use) | ❌ ACP session carries no purpose dimension; client-matcher would over-grant | **one-shot** (frozen) |
| `odrl:dateTime` / time window | `lteq` / `lt` / `gteq` / `gt` | (none — matcher accept-sets are static; no "now") | ❌ ACP has no clock dimension to re-check | **one-shot** (frozen) |
| `odrl:count` | any | (none — ACP is stateless) | ❌ no usage counter exists | **one-shot** (frozen) |
| any unrecognised left-operand | any | (none) | ❌ | **one-shot** (frozen) |
| *no constraint* | — | `auth:agent auth:Public` (action/target/duties already held) | ✅ | persisted (public) |

**Fail-safe on mixed constraints:** a persisted condition is emitted ONLY when **every** constraint on the rule maps faithfully. A rule mixing a mappable recipient with an unmappable `dateTime`/`purpose`/`count` falls back **entirely** to the one-shot path — persisting only the recipient would silently drop the time/purpose/count bound and over-grant. Recipient IRIs inside the reserved pair encoding (`urn:sparq:` / `&client=`) are dropped from the grant head (anti-impersonation). The two ODRL "any recipient" sentinels fold onto auth principals: `odrl:All`/`odrl:Group` → `auth:Public`, `odrl:AllConnections` → `auth:Authenticated`.

**Why only recipient/assignee maps:** the ACP session re-check carries exactly `(agent, client)`. The recipient-of-data is precisely the session **agent**, so an agent matcher re-checks it with identical semantics. Purpose, time, and count have **no** stateless `(agent, client)` analogue, so persisting them would require either freezing the check (= the one-shot path, already correct) or a looser approximation that could over-grant — rejected.

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
