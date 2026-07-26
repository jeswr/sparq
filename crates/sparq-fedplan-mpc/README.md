<!-- [OPUS-4.8] sq-2q1x / sq-fix4 / sq-i1wh2 / sq-pwr.2 / sq-pwr.3 / sq-xkrt: internal README for a publish=false crate; full design lives in research/mpc-untrusted-planner-routing-design.md. -->
# sparq-fedplan-mpc

The opt-in seam between cost-based federated source selection (`sparq-fedplan`) and
MPC-over-federated-SPARQL routing (`sparq-mpc`) — without coupling the two upstreams
to each other. Behind the **`fedplan-mpc` cargo feature, OFF by default**; the
default build compiles an empty crate and pulls in neither upstream. Design:
[`research/mpc-untrusted-planner-routing-design.md`](../../research/mpc-untrusted-planner-routing-design.md).

## 🚀 Quickstart

Internal crate (`publish = false`); enable the feature in the workspace, select the participating
sources (Phase 2), result-aware-prune the source combinations (Phase 6), partition the operators
disclosed-vs-hidden (Phase 3), assemble + dual-ratify the declared leakage envelope (Phase 4):

```rust
// behind `--features fedplan-mpc`
let selected = sparq_fedplan_mpc::select_private_sources(&bgp, &descriptors, &privacy)?;
// Phase 6 — result-aware combination prune. Rule C1 alone (no summaries needed):
let combos = sparq_fedplan_mpc::prune_source_combinations(&bgp, &descriptors, &selected)?;
// …or C1 + the quotient-summary provenance prune, when sources published summaries:
let combos = sparq_fedplan_mpc::prune_source_combinations_with_summaries(
    &bgp, &descriptors, &selected, &summaries, CombinationBudget::default(),
)?;
if combos.is_bgp_dead() { /* no combination can answer — skip the MPC path entirely */ }
// combos.live_combinations — the reduced set still worth routing toward MPC
let routing = sparq_fedplan_mpc::route_operators(
    &selected, &privacy, &operators, RoutingPolicy::Default,
)?;
// routing.routing — Vec<OperatorRouting> the sparq-mpc pipeline consumes
let envelope = sparq_fedplan_mpc::assemble_leakage_envelope(&routing, &operators, &selected)?;
// dual ratification: each holder fail-closed + the verifier's disclosed-operand budget
match sparq_fedplan_mpc::ratify_envelope(&envelope, &privacy, Some(budget)) {
    RatificationOutcome::Ratified => { /* cleared for execution */ }
    rejected => { /* a holder vetoed a disclosure, or the verifier rejected an over-leak */ }
}
```

## ✨ Features

- **`SourcePrivacyDescriptor`** (Phase 1, sq-2q1x) — the per-source privacy declaration.
  Posture is **default-deny**: a predicate is disclosable only when the source explicitly
  marks it `Public`; a planner cannot widen it, the source re-enforces fail-closed.
- **`select_private_sources`** (Phase 2, sq-fix4) — the **source-selection adapter**: runs
  `sparq-fedplan::select_sources`, then **prunes** any source whose descriptor declares it will
  not participate (`participates == Some(false)`), surfacing the retained per-pattern candidates
  plus an **audit trail** (`pruned`). Participation is the **only** prune rule — a source holding
  a *private* predicate stays a candidate (Phase 3 routes it), so the adapter is recall-safe.
- **`route_operators`** (Phase 3, sq-i1wh2) — the **disclosed/hidden routing pass**: from the
  Phase-2 selection, the descriptors and the query's `QueryOperator`s it classifies each operator
  `Disclosed` (every operand disclosable in the clear) or `Hidden(class)` (default-deny
  otherwise), emitting the `Vec<OperatorRouting>` the `sparq-mpc` pipeline today receives
  hand-written. `RoutingPolicy` picks the cheap default or the strict "hide even a public term";
  the most-private contributing source wins per operand.
- **`assemble_leakage_envelope` + `ratify_envelope`** (Phase 4, sq-pwr.2) — **leakage-envelope
  assembly + dual ratification**. The envelope **honestly enumerates** what the plan reveals —
  operator structure (count/class/label), the disclose/hide partition, the operands each
  `Disclosed` operator exposes, the participating sources — **over-counting, never
  under-counting**; a mismatch is fail-closed. Each **holder** then fail-closed-rejects a plan
  disclosing one of its own private predicates (constraint C-B) AND the **verifier** rejects an
  over-budget leak; `RatificationOutcome` names which ratification failed, and why.
- **`prune_source_combinations`** (Phase 6 Rule C1, sq-pwr.3) — the **result-aware
  source-combination prune**. Secure-join cost grows with the number of source *combinations*
  (one source per pattern) the seam considers, so this pass surfaces those that **provably
  contribute no answer**. The rule expressible from the `SourceDescriptor` summary alone is
  **unsatisfiable-conjunct collapse**: an empty Phase-2 candidate list proves that conjunct
  empty, so the conjunction is unsatisfiable and **every** combination dead (`∅ ⋈ R = ∅`). The
  selection is carried through **unchanged** (advisory + auditable) with `bgp_satisfiable`, the
  empty-pattern witnesses, the join components and an audit trail; a misaligned selection is
  fail-closed. A value-overlap prune *from the `SourceDescriptor`* is **deliberately declined** —
  not recall-safely expressible from its single-IRI `may_hold_authority` — so nobody builds it.
- **`prune_source_combinations_with_summaries` + `SourceQuotientSummary`** (Phase 6 Rule C2,
  sq-xkrt) — the FedUP-style (WWW'24) **provenance over quotient summaries** prune, the design
  record's §3 highest-leverage pre-MPC cost win. Past the declined non-rule lies a **different
  input**: a source may publish a `SourceQuotientSummary` — its graph under the **authority
  quotient** (an authority-bearing IRI collapses to `scheme://authority`; an authority-less one
  — `urn:…`, relative — is kept **verbatim**; **literal values are never recorded**) — and
  declare it a *complete over-approximation*. Evaluating the BGP at the quotient level once per
  combination then **provably** kills every combination evaluating empty, including those
  individually non-empty but *jointly* unsatisfiable (disjoint join authorities) — exactly what
  C1 cannot see. **Recall-safe in three layers**: a source without a complete summary is never
  why a combination dies; the over-approximation makes an empty evaluation a *proof*; and every
  indecisive case (a variable spanning the predicate and term domains, an over-budget
  `CombinationBudget`, an over-wide join) **declines** and prunes nothing. It reports
  `live_combinations`, a `SummaryPruneOutcome` and `EmptyProvenance` audit entries; duplicate
  summaries for one source id are refused.

> **Internal crate — not published** (`publish = false`). **No soundness or privacy claim.**
> Phases 2–4 + 6 are **plumbing — source selection, result-aware combination pruning,
> disclosed/hidden routing and leakage-accounting/ratification, not a cryptographic guarantee**:
> they perform **no** MPC, run **no** privacy-bearing cryptographic logic, open nothing and
> verify nothing cryptographic. The envelope is a *declaration* of what the plan reveals and the
> dual ratification a *plan-time policy gate* — not a runtime guarantee a malicious
> holder/verifier honours it. The MPC estate (`sparq-mpc`) is **research-grade, honest-majority
> semi-honest only, and NOT externally audited** — the cryptographer sign-off (`sq-qhy4`) and
> coZK re-audit (`sq-9hrn`) are pending. <!-- privacy-claims-allow: NEGATIVE/scoped — denies any privacy/soundness property; audits sq-qhy4 / sq-9hrn pending -->
> The further privacy-bearing phases (untrusted-plan soundness re-validation, authenticated-input
> attestation) are **deferred and audit-gated**; nothing here is a working protocol.

**Threat-model / leakage note (honest).** Every input read here is already public to this crate:
each source's *own* declared `SourcePrivacyDescriptor`, the Phase-2 selection it yields, and the
source-published quotient summaries. Publishing a `SourceQuotientSummary` **is itself a
disclosure** the source opts into — it reveals which **authorities** its subjects/objects come
from, per predicate, though not literal values (collapsed to one class) or cardinalities (it is a
set); an IRI with **no** extractable authority (`urn:…`, relative) is its own quotient class and
is therefore disclosed **verbatim**, so a source unwilling to reveal even that publishes nothing
and is never pruned. The plan itself reveals the operator structure, the disclose/hide partition,
the disclosed operands and the participating sources — enumerated honestly (over-counting) in the
envelope — but **not** hidden-operand values or result cardinalities, and says nothing about what
a query execution leaks, because no query executes here.

## 📚 Learn more

- Design: [`research/mpc-untrusted-planner-routing-design.md`](../../research/mpc-untrusted-planner-routing-design.md).
- Per-item detail (invariants, error phases, decline reasons): the crate rustdoc.
- Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
