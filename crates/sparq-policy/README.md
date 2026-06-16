# sparq-policy

<p>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**ODRL usage-control** over the [sparq](../../README.md) engine — the declarative
policy layer *above* access control.

`sparq-solid` answers "may this agent **read** graph G?"; `sparq-policy` answers
"may this party **use** this asset *for purpose P, with obligation O, until time
T, disclosing only to recipient R*?" It parses a [W3C ODRL
2.2](https://www.w3.org/TR/odrl-model/) policy from RDF into a typed model
(Permission / Prohibition / Duty / Action / Constraint) and **evaluates** an
access request to a **fail-closed** ALLOW/DENY [`Decision`]. This is the
**single-node base case**: ODRL over one node's data, reducing to the same
allow/deny shape `sparq-solid` enforces. It is a dependency of nothing in the
workspace (opt-in; `publish = false`).

## 🚀 Quickstart

```rust
# fn main() -> Result<(), String> {
use sparq_policy::{evaluate, parse_policy_str, Request, Value};

// An ODRL Set: alice MAY read asset-X, but only on or before 2026-12-31.
let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/1> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <urn:asset/x> ;
    odrl:assignee <https://alice.ex/me> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ;
                      odrl:operator odrl:lteq ;
                      odrl:rightOperand "2026-12-31T00:00:00Z"^^<http://www.w3.org/2001/XMLSchema#dateTime> ] ] .
"#;
let policy = parse_policy_str(ttl, "turtle")?;

// In-window request by alice → ALLOW.
let inside = Request::new("http://www.w3.org/ns/odrl/2/read")
    .on("urn:asset/x").by("https://alice.ex/me")
    .with("http://www.w3.org/ns/odrl/2/dateTime",
          Value::DateTime("2026-06-16T09:00:00Z".into()));
assert!(evaluate(&policy, &inside).allow);

// Out-of-window request → DENY (fail-closed; the constraint is unsatisfied).
let outside = inside.clone()
    .with("http://www.w3.org/ns/odrl/2/dateTime",
          Value::DateTime("2027-01-01T00:00:00Z".into()));
assert!(!evaluate(&policy, &outside).allow);
# Ok(()) }
```

## ✨ Features

- **ODRL model from RDF** — `parse_policy_str` / `parse_policy` extract a typed
  `Policy → {permissions, prohibitions}`, each `Rule` carrying an `Action`,
  `target`, `assignee`/`assigner`, `Constraint`s and `Duty`s. Rule, constraint
  and duty nodes are matched as **blank nodes or IRIs** (real ODRL uses blank
  nodes). Built on the engine sparq already ships — ODRL eval *is* a SPARQL
  workload.
- **Fail-closed evaluator** — `evaluate(&policy, &request) -> Decision { allow,
  matched_rules, unmet_constraints }`. No matching permission **or** a matching
  prohibition ⇒ DENY; a permission grants only when matched **and** all its
  duties are discharged. Prohibitions carve out permissions (ODRL conflict
  default). A malformed/unknown constraint becomes an unsatisfiable guard.
  `matched_prohibition(&policy, &request) -> Option<&Rule>` exposes the
  evaluator's own conflict test (the first prohibition that carves the request
  out), so a downstream materializer can distinguish a *carve-out* deny from a
  plain no-permission deny — `sparq-solid`'s ODRL bridge uses it to emit explicit
  `auth:deny*` triples (deny-overrides). [OPUS-4.8] sq-w693.
  `prohibition_status(&policy, &request) -> ProhibitionStatus` is the three-valued
  *deny-retraction* dual ([OPUS-4.8] sq-2pcf): `Applies` (a prohibition still carves
  the request out), `Ambiguous` (one still structurally names the request but a
  constraint is unprovable for lack of evidence), or `Withdrawn` (no prohibition names
  the request, or every one is *definitely* false given the supplied evidence). The
  bridge retracts a materialized deny **only** on `Withdrawn` — keeping it on `Ambiguous`
  so access is never restored on missing evidence (`matched_prohibition().is_none()`
  alone conflates the two, which would be fail-OPEN for a deny).
- **Constraint operators** — `eq`, `neq`, `lt`, `lteq`, `gt`, `gteq`,
  `isPartOf` (set membership), `isA`. Numeric (`xsd:integer`/`decimal`/…) and
  `xsd:dateTime`/`date` operands compare by magnitude/instant; everything else
  by IRI/string value. Constraints over `purpose` / `recipient` / `dateTime` /
  `count` / `spatial` left-operands are all supported.
- **`odrl:purpose` enforcement (faithful, fail-closed)** — [OPUS-4.8] sq-q56r. A
  purpose constraint restricts a rule to a stated *purpose of use*. A request carries
  its purpose evidence via `Request::for_purpose(Value)` (sugar over
  `.with(ODRL_PURPOSE, ..)`), readable back via `Request::purpose()`. The purpose is
  gated through the **same** `evaluate` constraint path as every other dimension, so it
  is actually checked end-to-end — never claimed-but-unchecked:
  - **Match → grant; mismatch → deny; missing purpose → fail-closed.** A request that
    states **no** purpose is *unprovable*, so a purpose-gated permission does **not**
    grant and a purpose-gated prohibition is **not** withdrawn — "no purpose stated" is
    **never** read as "any purpose allowed".
  - **Match semantics (the boundary — not over-claimed):** **exact** IRI/string
    equality (`eq`/`isA`), or membership in the explicit `isPartOf` purpose *set* the
    constraint names, or `neq` (purpose ≠ the named one). There is **no** purpose
    hierarchy / DPV subsumption: a narrower or broader purpose IRI is *not* matched
    against a constraint that names a different one. `neq` still requires a stated
    purpose (missing → fail-closed). A DPV-style purpose taxonomy / `isPartOf`-over-a-
    hierarchy is a deferred bead.
  - `purpose_status(&rule, &request) -> PurposeMatch` reports exactly what the evaluator
    checks for a rule's purpose constraints — `Satisfied` / `DefinitelyUnsatisfied` /
    `Unprovable` / `NotConstrained` — the auditable surface of this enforcement.
- **`odrl:recipient` enforcement + `neq` / "everyone-except"** — [OPUS-4.8] sq-5037.
  A `recipient` constraint restricts **who the data is disclosed to**. The
  recipient-of-data is the requesting party, so a request that names a party (`.by(..)`)
  but supplies no explicit `odrl:recipient` context is read as `recipient = party` — a
  `recipient` rule gates on *who is asking*, through the **same** `evaluate` constraint
  path. An explicit `.with(ODRL_RECIPIENT, ..)` still takes precedence.
  - **`recipient neq X` ("everyone EXCEPT X"):** grants/forbids for any recipient that is
    **not** `X`; the recipient `X` is the carve-out. **Missing identity (no
    `odrl:recipient` AND no party) is *unprovable* → fail-closed:** a `neq` permission
    does **not** grant to an unknown recipient, and a `neq` prohibition is **not**
    withdrawn. `eq`/`isA` = recipient IS the named party; `isPartOf` = recipient ∈ set.
    Match is **exact** IRI/string equality (no recipient hierarchy).
  - `recipient_status(&rule, &request) -> RecipientMatch` reports exactly what the
    evaluator checks — `Satisfied` / `DefinitelyUnsatisfied` / `Unprovable` /
    `NotConstrained` — the recipient dual of `purpose_status`. (In the `sparq-solid`
    bridge, `recipient neq X` maps to an ACP `noneOf` exception, re-checked per session.)
  - **Combined `recipient eq A AND neq B` (one rule)** — the constraints are AND-combined (the
    recipient must BE `A` and must NOT BE `B`); in the bridge this emits one
    `ConditionalGrant` headed by `A` carrying an `exceptMatcher` carving out `B`.
- **`odrl:dateTime` time-window enforcement (faithful, fail-closed)** — [OPUS-4.8] sq-idnv.
  A `dateTime` constraint gates a rule to a **time window** (`lteq T` / `lt T` / `gteq T`
  / `gt T`, or a two-sided `gteq lower` + `lteq upper`, AND-combined). The actual instant is the
  request's evaluation time, supplied via the first-class `Request::at(instant)` sugar
  (over `.with(ODRL_DATETIME, Value::DateTime(..))`; read back via `req.request_time()`).
  Instants compare by magnitude. **Missing time → *unprovable* → fail-closed:** a
  time-gated permission does not grant on an unknown clock; a time-gated prohibition is
  not withdrawn.
  - `datetime_status(&rule, &request) -> DateTimeMatch` reports exactly what the evaluator
    checks — `Satisfied` / `DefinitelyUnsatisfied` / `Unprovable` / `NotConstrained` — the
    temporal dual of `purpose_status`/`recipient_status`. (In the `sparq-solid` bridge,
    `dateTime` stays **one-shot** — ACP has no "now" dimension to re-check.)
- **`odrl:count` enforcement (stateful, opt-in, fail-closed)** — [OPUS-4.8] sq-zi5w.
  `odrl:count` limits the **number of times** a permission may be exercised ("may read
  at most 5 times"). Unlike the stateless constraints above, this is **stateful** — the
  count lives in a usage counter that persists across requests. Behind the
  `count-enforcement` feature (OFF by default; the default build is unchanged and treats
  `odrl:count` as the stateless numeric comparison), `evaluate_and_exercise(&policy,
  &request, &store) -> ExerciseDecision` runs the **real** `evaluate` decision and, on a
  grant, **atomically consumes** one unit of the applicable count budget:
  - **First *N* exercises grant; the *(N+1)*th denies.** A *denied* request burns **no**
    budget. `lteq N`/`eq N` = at most *N*; `lt N` = at most *N-1*.
  - **Counter-store seam.** `UsageCounterStore` is an injectable trait; the in-memory
    default `InMemoryCounterStore` is the reference impl. The budget key is
    `(rule_id, party, target)` (`CountKey`) — per-assignee, per-asset, per-rule.
  - **Atomicity / concurrency boundary.** The single mutating op is the atomic
    `try_consume` (check-and-consume under one lock in the in-memory store), **not** a
    read-then-increment — so the in-process TOCTOU race is closed (a concurrency test
    asserts exactly the limit is granted, never one more). A **distributed** store
    (Redis `INCR`, SQL `UPDATE … WHERE consumed < limit`) is the operator's concern and
    MUST provide the same atomicity; cross-process atomicity is a deferred bead.
  - **Fail-closed.** An *unavailable* counter (`ConsumeResult::Unavailable`) or a
    *malformed* limit denies — never silently treated as "unlimited".
    `count_status(&rule, &request, &store) -> CountStatus` is the side-effect-free audit
    surface (`Satisfied{consumed,limit}` / `DefinitelyUnsatisfied{..}` / `Unprovable` /
    `NotConstrained`), the count dual of `purpose_status`.
  - **Deferred (honest):** the stateless `sparq-solid` ODRL→ACP bridge does **not** wire
    this stateful path — a bridged ACP grant does not self-retract on count exhaustion;
    the bridge keeps `odrl:count` one-shot/unmappable. This crate provides the
    evaluator + store seam such a bridge would build on.
- **Duties as obligations** — a permission's `odrl:duty` must appear in the
  request's discharged-duty set, or the permission is denied (the usage-control
  kernel pure access control lacks).
- **Opt-in, lean** — no `unsafe`; depends only on `sparq-core` + `sparq-engine`
  + `oxrdf`; pulled into no core build (`cargo tree -p sparq-core` never shows it).

### Scope & caveats

Single-node only. The headline **federated-disclosure** / ODRL→MPC composition
(per-node ODRL drives the `sparq-mpc` disclosed-vs-hidden split; ODRL `Duty` →
ZK proof obligation) is **deferred** — it inherits the MPC honest-majority/LAN
envelope and the open ZK-soundness remediation. `dateTime` ordering compares the
lexical form; mixed-offset normalization, DPV `purpose` hierarchies, and
`Duty → proof-manifest` discharge are tracked as follow-on beads. See
`research/feature-research-odrl-policy.md`.

**Constraint persistence vs. one-shot** (sq-hiz4 / sq-5037, in `sparq-solid`'s opt-in
bridge): `materialize_odrl_permission_conditional` persists a
`odrl:recipient`/`odrl:assignee` constraint as a **re-checked** ACP
`auth:ConditionalGrant` — `eq`/`isA`/`isPartOf` as an agent matcher, and **`neq`
("everyone EXCEPT X") as an ACP `noneOf` exception** (a public grant + an
`auth:exceptMatcher` carving out `X`). These are the only constraints with a faithful
stateless `(agent, client)` analogue. `odrl:purpose`/`dateTime`/`count` have no
*stateless* ACP analogue and stay
**one-shot** in the bridge (checked once at materialization). Stateful `odrl:count`
enforcement itself (the usage counter) lives in this crate's `count-enforcement`
feature (`evaluate_and_exercise` + `UsageCounterStore`); wiring it *through* the
stateless ACP bridge so a bridged grant self-retracts on exhaustion is a deferred bead.
Mapping table: the
[`usage-control-policy`](../../skills/usage-control-policy/SKILL.md) skill +
[`sparq-solid` README](../sparq-solid/README.md).

## 📚 Learn more

- Skill: [`skills/usage-control-policy/SKILL.md`](../../skills/usage-control-policy/SKILL.md)
- Design record: `research/feature-research-odrl-policy.md` (epic sq-3183)
- W3C [ODRL Information Model 2.2](https://www.w3.org/TR/odrl-model/) ·
  [Formal Semantics](https://w3c.github.io/odrl/formal-semantics/)
- Sibling access-control crate: [`sparq-solid`](../sparq-solid/README.md)

## License

MIT — see [LICENSE](../../LICENSE).
