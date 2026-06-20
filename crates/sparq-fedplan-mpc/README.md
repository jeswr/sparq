<!-- [OPUS-4.8] sq-2q1x / sq-fix4 / sq-i1wh2 / sq-pwr.2: internal README for a publish=false crate; full design lives in research/mpc-untrusted-planner-routing-design.md. -->
# sparq-fedplan-mpc

The opt-in seam between cost-based federated source selection (`sparq-fedplan`) and
MPC-over-federated-SPARQL routing (`sparq-mpc`) — without coupling the two upstreams
to each other. Behind the **`fedplan-mpc` cargo feature, OFF by default**; the
default build compiles an empty crate and pulls in neither upstream. Design:
[`research/mpc-untrusted-planner-routing-design.md`](../../research/mpc-untrusted-planner-routing-design.md).

## 🚀 Quickstart

Internal crate (`publish = false`); enable the feature in the workspace, select the
participating sources (Phase 2), partition the query operators disclosed-vs-hidden
(Phase 3), assemble the declared leakage envelope and dual-ratify it (Phase 4):

```rust
// behind `--features fedplan-mpc`
let selected = sparq_fedplan_mpc::select_private_sources(&bgp, &descriptors, &privacy)?;
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

- **`SourcePrivacyDescriptor`** (Phase 1, sq-2q1x) — the per-source privacy
  declaration. Posture is **default-deny**: a predicate is disclosable only when the
  source explicitly marks it `Public`; a planner cannot widen it, the source
  re-enforces fail-closed.
- **`select_private_sources`** (Phase 2, sq-fix4) — the **source-selection adapter**.
  It runs `sparq-fedplan::select_sources` over the BGP + descriptors, then **prunes**
  any source whose descriptor declares it will not participate
  (`participates == Some(false)`), and surfaces the retained per-pattern candidate set
  plus an **audit trail** (`pruned`) of what was dropped and why. The **only** prune
  rule is participation/authorisation — a source holding a *private* predicate stays a
  candidate (its in-clear-vs-MPC routing is the Phase-3 decision), so the adapter is
  recall-safe and loses no answers. Duplicate descriptors for one source id are
  refused (`SeamError::DescriptorMismatch`) rather than silently resolved.
- **`route_operators`** (Phase 3, sq-i1wh2) — the **disclosed/hidden routing pass**.
  Given the Phase-2 selection, the privacy descriptors, and the query's
  `QueryOperator`s, it classifies each operator `Disclosed` (every operand disclosable
  in the clear) or `Hidden(class)` (default-deny otherwise — routes through the MPC
  path), emitting the `Vec<OperatorRouting>` the `sparq-mpc` pipeline today receives
  hand-written. The `RoutingPolicy` knob picks the cheap default (disclose global-IRI
  operands) or the strict "hide even a public term" route. The most-private contributing
  source wins per operand. Duplicate descriptors are refused
  (`SeamError::DescriptorMismatch`).
- **`assemble_leakage_envelope` + `ratify_envelope`** (Phase 4, sq-pwr.2) — the
  **leakage-envelope assembly + dual-ratification** gate. `assemble_leakage_envelope`
  derives, from the Phase-3 routing and the `QueryOperator`s it was computed over, a
  declared `LeakageEnvelope` that **honestly enumerates** what the plan reveals — the
  operator structure (count, per-operator class + label), the disclose/hide partition,
  the operands each `Disclosed` operator exposes in the clear, and which sources
  participate — **over-counting, never under-counting** the leak; a routing/operator
  mismatch is fail-closed (`SeamError::DescriptorMismatch`). `ratify_envelope` then runs
  the dual gate: each **holder** fail-closed-rejects a plan disclosing one of its own
  private predicates (constraint C-B — the most-private holder wins), AND the
  **verifier** rejects an over-leaking envelope (distinct disclosed operands exceeding
  its declared budget). The result is a `RatificationOutcome` naming *which* ratification
  failed and *why*.

> **Internal crate — not published** (`publish = false`). **No soundness or privacy
> claim.** Phases 2–4 are **plumbing — source-selection + disclosed/hidden routing +
> leakage-accounting/ratification, not a cryptographic guarantee**: they perform **no**
> MPC, run **no** privacy-bearing cryptographic logic, open nothing, and verify nothing
> cryptographic — the descriptors and operators are the caller's own inputs, the envelope
> is a *declaration* of what the plan reveals, and the dual ratification is a *plan-time
> policy gate* (not a runtime guarantee a malicious holder/verifier honours it). The MPC
> estate (`sparq-mpc`) is **research-grade, honest-majority semi-honest only, and NOT
> externally audited** — the cryptographer sign-off (`sq-qhy4`) and coZK re-audit
> (`sq-9hrn`) are pending. <!-- privacy-claims-allow: NEGATIVE/scoped — denies any privacy/soundness property; audits sq-qhy4 / sq-9hrn pending -->
> The further privacy-bearing phases (untrusted-plan soundness re-validation,
> authenticated-input attestation) are **deferred and audit-gated**; nothing here is a
> working protocol.

**Threat-model / leakage note (honest).** Phase 2 reads each source's *own* declared
`SourcePrivacyDescriptor` to decide participation; Phase 3 reads `may_disclose` per
operand to decide the route. Per the design record's constraint C-B (§2.2), the
descriptor is the source's declaration — Phase 4's `ratify_envelope` has each holder
**re-enforce** it fail-closed and the verifier ratify the declared leakage envelope, so
a lying planner that over-discloses is **rejected here, not honoured**. This is a
*plan-time policy gate*, not a cryptographic enforcement. The plan **itself reveals**
the query's **operator structure** (count, class, order), the **disclose/hide partition**,
the **disclosed operands** of each disclosed operator, and the **participating sources**
— and the leakage envelope enumerates exactly that, honestly (over-counting). It does
**not** reveal hidden-operand values or result cardinalities (those live in the later
evaluation, not in this typed plan), and it makes no claim about what is learned once a
query executes, because no query executes here.

## 📚 Learn more

- Design: [`research/mpc-untrusted-planner-routing-design.md`](../../research/mpc-untrusted-planner-routing-design.md).
- Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
