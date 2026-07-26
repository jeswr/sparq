<!-- [OPUS-4.8] sq-2q1x / sq-fix4 / sq-i1wh2 / sq-pwr.2 / sq-pwr.3 / sq-xkrt: internal README for a publish=false crate; full design lives in research/mpc-untrusted-planner-routing-design.md. -->
# sparq-fedplan-mpc

The opt-in seam between cost-based federated source selection (`sparq-fedplan`) and
MPC-over-federated-SPARQL routing (`sparq-mpc`) — without coupling the two upstreams
to each other. Behind the **`fedplan-mpc` cargo feature, OFF by default**; the
default build compiles an empty crate and pulls in neither upstream. Design:
[`research/mpc-untrusted-planner-routing-design.md`](../../research/mpc-untrusted-planner-routing-design.md).

## 🚀 Quickstart

Internal crate (`publish = false`); enable the feature in the workspace, select the
participating sources (Phase 2), result-aware-prune the source combinations (Phase 6),
partition the operators disclosed-vs-hidden (Phase 3), assemble + dual-ratify the
declared leakage envelope (Phase 4):

```rust
// behind `--features fedplan-mpc`
let selected = sparq_fedplan_mpc::select_private_sources(&bgp, &descriptors, &privacy)?;
// Phase 6 — result-aware combination prune: is any full-BGP combination feasible?
// Rule C1 only (no summaries needed):
let combos = sparq_fedplan_mpc::prune_source_combinations(&bgp, &descriptors, &selected)?;
// …or Rule C1 + the FedUP-style quotient-summary provenance prune, when sources published one:
let combos = sparq_fedplan_mpc::prune_source_combinations_with_summaries(
    &bgp, &descriptors, &selected, &summaries, CombinationBudget::default(),
)?;
if combos.is_bgp_dead() {
    // no source-combination can answer — skip the MPC path entirely
}
// combos.live_combinations — the reduced set still worth routing toward MPC
let routing = sparq_fedplan_mpc::route_operators(
    &selected, &privacy, &operators, RoutingPolicy::Default,
)?;
// routing.routing — Vec<OperatorRouting> the sparq-mpc pipeline consumes
let envelope = sparq_fedplan_mpc::assemble_leakage_envelope(&routing, &operators, &selected)?;
// dual ratification: each holder fail-closed + the verifier's disclosed-operand budget
match sparq_fedplan_mpc::ratify_envelope(&envelope, &privacy, Some(budget)) {
    RatificationOutcome::Ratified => { /* cleared for execution */ }
    rejected => { /* a holder vetoed a private-term disclosure, or the verifier rejected an over-leak */ }
}
```

## ✨ Features

- **`SourcePrivacyDescriptor`** (Phase 1, sq-2q1x) — the per-source privacy declaration.
  Posture is **default-deny**: a predicate is disclosable only when the source explicitly
  marks it `Public`; a planner cannot widen it, the source re-enforces fail-closed.
- **`select_private_sources`** (Phase 2, sq-fix4) — the **source-selection adapter**.
  It runs `sparq-fedplan::select_sources` over the BGP + descriptors, then **prunes** any
  source whose descriptor declares it will not participate (`participates == Some(false)`),
  surfacing the retained per-pattern candidate set plus an **audit trail** (`pruned`) of
  what was dropped and why. The **only** prune rule is participation/authorisation — a
  source holding a *private* predicate stays a candidate (its in-clear-vs-MPC routing is the
  Phase-3 decision), so the adapter is recall-safe and loses no answers. Duplicate
  descriptors for one source id are refused (`SeamError::DescriptorMismatch`).
- **`route_operators`** (Phase 3, sq-i1wh2) — the **disclosed/hidden routing pass**.
  Given the Phase-2 selection, the privacy descriptors, and the query's `QueryOperator`s,
  it classifies each operator `Disclosed` (every operand disclosable in the clear) or
  `Hidden(class)` (default-deny otherwise — routes through the MPC path), emitting the
  `Vec<OperatorRouting>` the `sparq-mpc` pipeline today receives hand-written. The
  `RoutingPolicy` knob picks the cheap default (disclose global-IRI operands) or the strict
  "hide even a public term" route; the most-private contributing source wins per operand.
  Duplicate descriptors are refused (`SeamError::DescriptorMismatch`).
- **`assemble_leakage_envelope` + `ratify_envelope`** (Phase 4, sq-pwr.2) — the
  **leakage-envelope assembly + dual-ratification** gate. `assemble_leakage_envelope`
  derives a declared `LeakageEnvelope` that **honestly enumerates** what the plan reveals
  — the operator structure (count/class/label), the disclose/hide partition, the operands
  each `Disclosed` operator exposes, and which sources participate — **over-counting, never
  under-counting**; a routing/operator mismatch is fail-closed. `ratify_envelope` then runs
  the dual gate: each **holder** fail-closed-rejects a plan disclosing one of its own
  private predicates (constraint C-B — the most-private holder wins), AND the **verifier**
  rejects an over-leaking envelope (distinct disclosed operands exceeding its budget),
  returning a `RatificationOutcome` naming *which* ratification failed and *why*.
- **`prune_source_combinations`** (Phase 6 Rule C1, sq-pwr.3) — the **result-aware
  source-combination prune** over the Phase-2 selection. A federated query executes a
  *conjunction* of patterns, and the seam's secure-join cost grows with the number of
  source *combinations* (one source per pattern) it considers; this pass surfaces which
  combinations **provably contribute no answer** so the seam routes fewer toward MPC. The
  rule expressible from the `SourceDescriptor` summary alone is **unsatisfiable-conjunct
  collapse**: if any pattern's Phase-2 candidate list is empty, that conjunct is
  proved-empty, so the whole conjunction is unsatisfiable and **every** combination is dead
  (`∅ ⋈ R = ∅`). It carries the Phase-2 selection through **unchanged** (advisory +
  auditable), reporting `bgp_satisfiable`, the empty-pattern witnesses, the BGP join
  components, and a combination-dead audit trail. A value-overlap prune *from the
  `SourceDescriptor`* is **deliberately declined** — not recall-safely expressible from it
  (single-IRI `may_hold_authority`, no authority-set enumerator) — so a future contributor
  does not build an unsound prune. A length mismatch is fail-closed
  (`SeamError::DescriptorMismatch`, phase `SourceCombination`).
- **`prune_source_combinations_with_summaries` + `SourceQuotientSummary`** (Phase 6 Rule C2,
  sq-xkrt) — the FedUP-style (WWW'24) **provenance over quotient summaries** prune, the lever
  the design record's §3 names as the highest-leverage pre-MPC cost win. The way past the
  declined value-overlap non-rule is not a cleverer reading of `SourceDescriptor` but a
  **different input**: a source may publish a `SourceQuotientSummary` — its graph under the
  **authority quotient** (IRIs collapse to `scheme://authority`, **literal values are never
  recorded**) — and declare it a *complete over-approximation*. The pass then evaluates the
  BGP at the quotient level **once per source-combination** and drops those whose evaluation
  is empty, which **provably** produce no concrete answer. That kills combinations whose
  patterns are individually non-empty but *jointly* unsatisfiable (e.g. two sources holding
  the join predicate over disjoint authorities) — exactly what Rule C1 cannot see. It reports
  `live_combinations` (the reduced set to route), a `SummaryPruneOutcome`, and
  `EmptyProvenance` audit entries; if **no** combination survives, `bgp_satisfiable` goes
  false with no empty conjunct. **Recall-safe in three layers**: a source with no complete
  summary constrains nothing and is never the reason a combination dies; the
  over-approximation makes an empty evaluation a *proof*; and every indecisive case (a
  variable spanning the predicate and subject/object domains, an over-budget combination
  space via `CombinationBudget`, an over-wide join) **declines** and prunes nothing. Duplicate
  summaries for one source id are refused (`SeamError::DescriptorMismatch`).

> **Internal crate — not published** (`publish = false`). **No soundness or privacy
> claim.** Phases 2–4 + 6 are **plumbing — source-selection + result-aware combination
> pruning + disclosed/hidden routing + leakage-accounting/ratification, not a cryptographic
> guarantee**: they perform **no** MPC, run **no** privacy-bearing cryptographic logic, open
> nothing, and verify nothing cryptographic — the descriptors and operators are the caller's
> own inputs, the combination prune reads only the already-public Phase-2 selection, the
> envelope is a *declaration* of what the plan reveals, and the dual ratification is a
> *plan-time policy gate* (not a runtime guarantee a malicious holder/verifier honours it).
> The MPC estate (`sparq-mpc`) is **research-grade, honest-majority semi-honest only, and NOT
> externally audited** — the cryptographer sign-off (`sq-qhy4`) and coZK re-audit
> (`sq-9hrn`) are pending. <!-- privacy-claims-allow: NEGATIVE/scoped — denies any privacy/soundness property; audits sq-qhy4 / sq-9hrn pending -->
> The further privacy-bearing phases (untrusted-plan soundness re-validation,
> authenticated-input attestation) are **deferred and audit-gated**; nothing here is a
> working protocol.

**Threat-model / leakage note (honest).** Phase 2 reads each source's *own* declared
`SourcePrivacyDescriptor` to decide participation; Phase 6 reads only the (public) Phase-2
selection and the (public, source-published) quotient summaries to decide which combinations
are infeasible, revealing nothing beyond what those inputs already imply. Publishing a
`SourceQuotientSummary` **is itself a disclosure** the source opts into: it reveals which
**authorities** its subjects/objects come from, per predicate. It does *not* reveal literal
values (collapsed to one class), individual IRIs (collapsed to their authority), or
cardinalities (the summary is a set); a source that considers even the authority-level shape
sensitive publishes nothing and is simply never pruned. Phase 3
reads `may_disclose` per operand to decide the route. Per constraint C-B (§2.2) the
descriptor is the source's declaration — Phase 4's `ratify_envelope` has each holder
**re-enforce** it fail-closed and the verifier ratify the leakage envelope, so a lying
planner that over-discloses is **rejected here, not honoured**. This is a *plan-time policy
gate*, not a cryptographic enforcement. The plan **itself reveals** the **operator
structure** (count, class, order), the **disclose/hide partition**, the **disclosed
operands**, and the **participating sources** — enumerated honestly (over-counting) in the
envelope. It does **not** reveal hidden-operand values or result cardinalities, and makes no
claim about what is learned once a query executes, because no query executes here.

## 📚 Learn more

- Design: [`research/mpc-untrusted-planner-routing-design.md`](../../research/mpc-untrusted-planner-routing-design.md).
- Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
