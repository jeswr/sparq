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
  `isPartOf` (set membership **or**, for the taxonomic dimensions `purpose`/`spatial`,
  a transitive subsumption match — see below), `isA`. Numeric (`xsd:integer`/`decimal`/…) and
  `xsd:dateTime`/`date` operands compare by magnitude/instant; everything else
  by IRI/string value. Constraints over `purpose` / `recipient` / `dateTime` /
  `count` / `spatial` left-operands are all supported.
- **`odrl:spatial` region enforcement + region `isPartOf` trees** — [OPUS-4.8] sq-wukl.
  A `spatial` constraint gates a rule to a **geographic region** (`spatial isPartOf
  <country/EU>`, "anywhere in the EU"). A request supplies its region as `odrl:spatial`
  evidence (`.with(ODRL_SPATIAL, ..)`), gated through the **same** `evaluate` constraint
  path as every other dimension.
  - **Region `isPartOf` tree (subsumption).** The spatial dimension is taxonomic, so it
    reuses the **same** caller-supplied subsumption closure the DPV purpose taxonomy uses
    (`Request::with_purpose_subsumption(narrower, broader)` / `with_purpose_taxonomy(edges)`
    — there is *no* separate spatial evidence channel). A `spatial isPartOf <EU>`
    constraint is then satisfied by a request whose stated region is the EU **or
    transitively part-of** it — `Berlin ⊑ DEU ⊑ EU`. **Honesty / fail-closed:** the tree is
    *the requester's asserted subsumption*, never invented — with **no** edge supplied
    `spatial` matching is exact membership (fully backward compatible), and a sub-region
    does **not** grant a broad-region permission unless the request asserts the edge (a
    missing edge fails closed, like a missing context value). Cycle-safe (the closure
    tolerates a malformed `A ⊑ B ⊑ A`).
  - `spatial_status(&rule, &request) -> SpatialMatch` reports exactly what the evaluator
    checks for a rule's spatial constraints — `Satisfied` / `DefinitelyUnsatisfied` /
    `Unprovable` / `NotConstrained` — the spatial twin of `purpose_status` (it runs the
    same subsumption-aware path `evaluate` does). (Through the `sparq-solid` bridge
    `purpose`/`spatial` stay **one-shot** — ACP has no purpose/region dimension to
    re-check; see the mapping table.)
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
    constraint names, or `neq` (purpose ≠ the named one). `neq` still requires a stated
    purpose (missing → fail-closed).
  - **DPV / purpose-taxonomy subsumption** — [OPUS-4.8] sq-z3ve. Supply the request a
    purpose taxonomy via `Request::with_purpose_subsumption(narrower, broader)` (one
    `skos:broader`/`rdfs:subClassOf`/`dpv:isSubTypeOf` edge) or the bulk
    `with_purpose_taxonomy(edges)`, and a purpose constraint naming a purpose `B` is then
    also satisfied by a request whose stated purpose `P` is **transitively narrower than
    `B`** (`P ⊑ B`): a permission gated on the broad `research` purpose covers a request
    for the narrow `clinical-research` sub-purpose, and a `neq research` carve-out *also*
    excludes that sub-purpose (a sub-purpose **is** a research purpose). **Sound, never
    over-claimed:** the `⊑` relation is the **caller-supplied transitive closure only** —
    it is *never* inferred from IRI string structure — so with no taxonomy supplied
    matching is byte-for-byte the exact-IRI base case, the broader-under-narrower
    direction never matches, and access is never widened on an unproven relation. The
    edges form the closure incrementally (order-independent). The **same** closure also
    drives the `odrl:spatial` region tree (see the spatial bullet above) — one subsumption
    evidence channel for both taxonomic dimensions.
  - `purpose_status(&rule, &request) -> PurposeMatch` reports exactly what the evaluator
    checks for a rule's purpose constraints — `Satisfied` / `DefinitelyUnsatisfied` /
    `Unprovable` / `NotConstrained` — the auditable surface of this enforcement (it runs
    the same subsumption-aware path `evaluate` does).
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
  Instants compare by the **UTC point they denote, not their lexical form** — a
  mixed-offset pair such as `2026-06-16T13:00:00+02:00` (= 11:00Z) and `…T12:00:00Z`
  orders correctly, and `…T12:00:00Z` equals `…T14:00:00+02:00` under `eq` ([OPUS-4.8]
  sq-qj2q). Normalization is self-contained (std-only — the crate carries no `chrono`/
  `time` dependency): `xsd:dateTime`/`xsd:date` lexical forms are parsed to a UTC instant
  (`Z`/`±hh:mm`/no-tz, fractional seconds, leap-second clamp), and an **unparseable**
  operand compares fail-closed (order is undefined → constraint not satisfied).
  **Missing time → *unprovable* → fail-closed:** a time-gated permission does not grant
  on an unknown clock; a time-gated prohibition is not withdrawn.
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
    default `InMemoryCounterStore` is the single-process reference impl and
    `FileCounterStore` ([OPUS-4.8] sq-5z1q) is a **cross-process** impl (below). The
    budget key is `(rule_id, party, target)` (`CountKey`) — per-assignee, per-asset,
    per-rule.
  - **Atomicity / concurrency boundary.** The single mutating op is the atomic
    `try_consume` (check-and-consume under one lock in the in-memory store), **not** a
    read-then-increment — so the in-process TOCTOU race is closed (a concurrency test
    asserts exactly the limit is granted, never one more).
  - **Cross-process counting (`FileCounterStore`)** — [OPUS-4.8] sq-5z1q. The in-memory
    store is atomic in-process only; in a multi-process deployment each process would get
    its own full budget. `FileCounterStore::new(dir)` persists all budgets in one file and
    serializes the **whole** compare-and-increment with an OS-level lockfile (`O_EXCL`
    `create_new` — the cross-process analogue of the in-memory `Mutex`), so every process
    that opens the same `dir` shares one budget. It is a drop-in for the same trait seam
    (identical `try_consume` contract / `evaluate_and_exercise` semantics), **std-only —
    no new deps, no `unsafe`**, and a cross-process test (6 worker subprocesses, 240
    attempts, budget 60) asserts exactly the limit is granted across all processes.
    **Boundary (honest):** `O_EXCL` create is atomic on a local FS and on modern NFS, not
    on old/misconfigured NFS — for a multi-*host* deployment over a questionable mount,
    prefer a Redis `INCR` / SQL `UPDATE … WHERE consumed < limit RETURNING` store against
    the same trait.
  - **Fail-closed.** An *unavailable* counter (`ConsumeResult::Unavailable`) or a
    *malformed* limit denies — never silently treated as "unlimited".
    `count_status(&rule, &request, &store) -> CountStatus` is the side-effect-free audit
    surface (`Satisfied{consumed,limit}` / `DefinitelyUnsatisfied{..}` / `Unprovable` /
    `NotConstrained`), the count dual of `purpose_status`.
  - **Deferred (honest):** the stateless `sparq-solid` ODRL→ACP bridge does **not** wire
    this stateful path — a bridged ACP grant does not self-retract on count exhaustion;
    the bridge keeps `odrl:count` one-shot/unmappable. This crate provides the
    evaluator + store seam such a bridge would build on.
- **Static conflict + containment analysis (request-free, sound)** — [OPUS-4.8]
  sq-zabv. Two policy-vs-policy lints on the query-containment comparison semantics,
  always compiled (no feature, no deps):
  - `detect_conflicts(&policy) -> Vec<Conflict>` flags every
    permission/prohibition pair whose request footprints overlap (a prohibition
    carves the permission out — deny-overrides). `Overlap::Certain` when the
    prohibition carves out the **whole** permission (structural overlap + it adds no
    constraint the permission lacks), else `Overlap::Possible`; a pair that
    *provably never* overlaps is omitted.
  - `contains(outer, inner) -> Containment` answers *"does `outer` permit everything
    `inner` permits?"* (refinement / requester-vs-provider containment):
    `Contains` / `NotContained` / `Unknown`.
  - **Sound, never over-claimed.** Constraint satisfiability / query containment is
    undecidable in general, so this decides only what it can *prove* (identical
    constraints, `eq` admitted by an outer bound, a tighter same-direction order
    bound implying a looser one); everything else degrades to `Possible` / `Unknown`
    — it never reports `Certain` / `Contains` it cannot prove (the fail-OPEN failure
    mode). DPV/`isPartOf` set-subset refinement is a deferred bead.
- **Duties as obligations** — a permission's `odrl:duty` must appear in the
  request's discharged-duty set, or the permission is denied (the usage-control
  kernel pure access control lacks).
- **Opt-in, lean** — no `unsafe`; depends only on `sparq-core` + `sparq-engine`
  + `oxrdf`; pulled into no core build (`cargo tree -p sparq-core` never shows it).

### Scope & caveats

Single-node only. The headline **federated-disclosure** / ODRL→MPC composition
(per-node ODRL drives the `sparq-mpc` disclosed-vs-hidden split; ODRL `Duty` →
ZK proof obligation) is **deferred** — it inherits the MPC honest-majority/LAN
envelope and the open ZK-soundness remediation. `dateTime` ordering normalizes
mixed timezone offsets to the UTC instant before comparing ([OPUS-4.8] sq-qj2q).
DPV-`purpose`-taxonomy subsumption ([OPUS-4.8] sq-z3ve — `Request::with_purpose_subsumption`
/ `with_purpose_taxonomy`) and the `odrl:spatial` region `isPartOf` tree ([OPUS-4.8]
sq-wukl — the same caller-supplied closure) are both evaluated at request time when the
request supplies the edges. The *static* (request-free) `contains` refinement over
`isPartOf` set-subset, and `Duty → proof-manifest` discharge, remain follow-on beads.
See `research/feature-research-odrl-policy.md`.

**Constraint persistence vs. one-shot** (sq-hiz4 / sq-5037, in `sparq-solid`'s opt-in
bridge): `materialize_odrl_permission_conditional` persists a
`odrl:recipient`/`odrl:assignee` constraint as a **re-checked** ACP
`auth:ConditionalGrant` — `eq`/`isA`/`isPartOf` as an agent matcher, and **`neq`
("everyone EXCEPT X") as an ACP `noneOf` exception** (a public grant + an
`auth:exceptMatcher` carving out `X`). These are the only constraints with a faithful
stateless `(agent, client)` analogue. `odrl:purpose`/`spatial`/`dateTime`/`count` have no
*stateless* ACP analogue (ACP carries no purpose/region/clock dimension) and stay
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
