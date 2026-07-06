# Design — the reasoner suite on the shared substrate (RL-complete / EL / QL / Direct / RIF / D)

<!-- [OPUS-4.8] Design-for-review for epic sq-pbz04 (under umbrella sq-6tykl). NO production
code in this PR. Builds on research/owl2-el-ql-reasoning-spike.md (sq-wmeg). 🤖 SPARQ agent. -->

> 🤖 **SPARQ agent** — design record for @jeswr's review. DESIGN-FOR-REVIEW only. This record
> sequences the reasoner regimes on top of the shared substrate; it does not implement them.

**Status:** DESIGN / design-for-review. **Epic:** sq-pbz04 (reasoners on the shared substrate),
depends on the substrate design (sq-qonbz, see `research/shared-eval-substrate.md`). **Prior
art in-repo:** `research/owl2-el-ql-reasoning-spike.md` (the EL/QL feasibility spike, sq-wmeg);
`research/owl2-el-ql-reasoning-spike.md` already chose **EL-first, QL-second**, which this record
adopts and extends to the full suite.

**Recommendation in one line:** ship the regimes in increasing soundness-risk order —
**(1) RL completeness hardening + (2) EL hardening** (both already exist and pass conformance) →
**(3) D-entailment** (smallest sound addition, value-space closure) → **(4) RIF-Core / N3
builtins** (existing N3 engine, expand builtins) → **(5) QL query-rewriting** (the PerfectRef /
tree-witness minefield) → **(6) OWL 2 Direct / DL** (research-grade, EL-first restraint). Each is
opt-in, each wires into the W3C entailment conformance harness, and each that does materialisation
adopts the shared join from sq-qonbz.

---

## 0. Premise check (honesty first)

What the brief says, checked against the code:

- **RL and EL already exist and pass conformance.** `crates/sparq-reason` (RDFS + OWL-RL forward
  chaining, `owl.rs` 3120 LOC, plus `incremental.rs` 3157 LOC — the brief's "3963 LOC" undercounts;
  it is larger) and `crates/sparq-reason-el` (consequence-based EL classifier with optional `rbox`
  / `hasse` features) are real, shipped, opt-in crates. The entailment harness already drives them
  (`sparq_reason::materialize(Profile::Rdfs|OwlRl, …)` in `inference/sparql_entail.rs`).
- **QL, Direct, RIF-Core, D are NOT built.** Confirmed: no QL rewriter, no tableau, no RIF-Core
  semantics, no D-entailment materialisation crate exists. These are the genuine new builds.
- **The EL/QL spike (sq-wmeg) already landed** and reached EL-first / QL-second with the PerfectRef
  trap called out. This record does not re-litigate that; it folds it into the full-suite sequence
  and adds D / RIF / Direct.

One correction to emphasise: the brief lists "RL-complete" as a regime to *build*. RL already
ships and passes 78 OWL-2-RL tests with 13 **documented** divergences (TBox-only conclusions
outside the RL/RDF completeness theorem). "RL-complete" here means *close those documented
divergences where the RL/RDF profile actually entails them*, **not** make RL complete for EL —
RL is provably, mechanistically incomplete for EL (it lacks CR4, the existential-traversal rule;
the spike has the verified counterexample `A⊑∃r.B, B⊑C, ∃r.C⊑D ⊨ A⊑D`). The honest fix for EL
completeness is the EL classifier, which already exists, not RL tuning.

---

## 1. The regimes, by soundness risk and build cost

| Regime | Status | Decidable? | Soundness oracle | Build risk |
|---|---|---|---|---|
| RDFS / OWL-RL | **shipped** | yes (PTIME, sound, RL-incomplete-for-EL) | W3C inference suite, 48/48 RDFS + RL ratchet | low — harden divergences |
| OWL 2 EL | **shipped** (`sparq-reason-el`) | yes (PTIME) | EL is its own oracle; SNOMED/GO scale | low–med — fragment gaps (nominals/concrete domains deferred) |
| D-entailment | not built | yes | datatype value-space spec | **low** — smallest sound add |
| RIF-Core | not built (N3 engine exists) | Horn-decidable for the core | RIF test suite (if vendorable) | med — builtins + safety |
| OWL 2 QL | not built | yes (AC0 data complexity) | **PerfectRef** (no Rust impl exists) | **high** — applicability trap + UCQ-containment minimisation (NP-complete) |
| OWL 2 Direct / DL | not built | **undecidable** for full OWL DL | needs a sound SAT/tableau | **highest** — research-grade |

The sequence below follows this risk gradient: bank the sound, cheap wins first; defer the
minefields with eyes open.

---

## 2. Per-regime design notes (honest scope)

### 2.1 RL completeness hardening (sound, in-scope)

Close the documented RL/RDF divergences where the profile genuinely entails the conclusion;
keep the ones that are out of the RL completeness theorem as *honest* `Divergence(reason)` entries
(never silently "pass"). Adopt the shared join (sq-qonbz phase 5) for rule firing. **No new
soundness risk** — RL stays sound; we only add derivations the profile already sanctions.

### 2.2 EL hardening (sound, in-scope)

EL already classifies via CR1–CR5 (+ optional CR10/CR11 RBox, + Hasse). The deferred fragment is
nominals / concrete domains (CR6–CR9). Honest options: (a) leave them deferred and keep surfacing
them in `Report::skipped_axioms` (current behaviour, recommended for now), or (b) add CR6–CR9 as a
later opt-in. The **perf** open item the spike flagged remains: an end-to-end SNOMED/GO benchmark
to confirm normalise + RBox + Hasse compose without a hidden quadratic. The work-box timing is
non-canonical; the gate is the conformance ratchet + a *relative* (not absolute) perf check.

### 2.3 D-entailment (sound, smallest new build — do this first of the new ones)

Datatype value-space closure: e.g. an `xsd:int` literal entails the corresponding `xsd:integer`
/ `xsd:decimal` typings, ill-typed literals are detected, and value-equal literals in different
lexical forms compare equal. The substrate's typed `Num` / `compare_values` already has most of
the value-space machinery (the harness uses it at entailment-check time today). The build is
mostly: (a) a small materialisation pass that adds sanctioned datatype triples, (b) wiring it as a
`Profile::D` into the entailment harness. **Lowest risk of the unbuilt regimes** and a clean
conformance story.

### 2.4 RIF-Core / N3 builtins (med risk)

sparq already has an N3 forward-chaining engine (`sparq-reason/src/n3`) with builtins. RIF-Core is
Horn rules + builtins; the honest path is **extend the N3 builtin set and add a RIF-Core
front-end / mapping**, not a green-field rule engine. Risk items: builtin safety (range-
restriction), and negation-as-failure is **out of RIF-Core** (it's RIF-BLD/PRD territory) — keep
the core monotone. If the official RIF test suite is not vendorable (license / format), frame the
ratchet honestly as "RIF-Core expressivity coverage," not "RIF conformance."

### 2.5 OWL 2 QL — the PerfectRef trap (HIGH risk, sequence late)

This is the regime the brief and the spike both flag hardest, and the verification confirms why:

- **No Rust PerfectRef oracle exists.** Production systems (Ontop, Stardog) are Java/Scala. Ontop
  ships **tree-witness** rewriting + the **combined approach** + SQL-side optimisation
  (confirmed against the literature: Kikot et al. tree-witnesses; Ontop's tree-witness + T-mappings).
  A pure PerfectRef MVP is a *correctness oracle*, not a production rewriter.
- **Two soundness traps** (both can *silently lose correct answers*): (1) the **applicability
  condition** — an existential-introducing inclusion may fire only on a non-distinguished,
  non-shared variable; mis-gating scope (OPTIONAL / FILTER / MINUS / paths passing through the
  rewrite) is unsound, so a **strict CQ-shape gate** that *rejects* non-CQ queries is mandatory;
  (2) **UCQ minimisation by query containment** is **NP-complete** and correctness-critical —
  dropping a CQ that is not actually subsumed loses answers.
- **Honest plan:** build PerfectRef as a *gated, CQ-only, oracle-tested* MVP first (validated
  against hand-checked DL-Lite examples since no Rust oracle exists), label it experimental, and
  **do not ship it as a general entailment regime until tree-witness + minimisation land**. The
  UCQ blow-up is theoretically unavoidable (exponential in TBox depth, Kikot et al. lower bounds),
  so production QL is a multi-phase effort, not an MVP.

### 2.6 OWL 2 Direct / DL (HIGHEST risk, research-grade)

Full OWL 2 Direct Semantics (DL) is **undecidable**; a sound implementation needs a tableau / SAT
reasoner. The spike's restraint holds: **EL-first** (PTIME, closed semantics) covers the high-
value biomedical use cases. If DL is ever attempted, it is a separate research track and must NOT
claim soundness without external review — the same discipline the ZK verifier is under (sq-qhy4
external-audit precedent). Recommend **deferring DL out of this program** and recording it as a
future research bead.

---

## 3. Substrate coupling

Every regime that **materialises** (RL, EL-as-triples, D, RIF) adopts the shared join from
sq-qonbz phase 5 in place of hand-rolled adjacency, and interns all output terms through
`sparq-core::Dict` so the single-id-space soundness keystone holds. QL is the exception: it is a
**query-rewriter**, not a materialiser — it produces a rewritten SPARQL query through a
`PreparedQuery`-style seam (the brief's "no store/planner changes needed"), so it reuses the
*engine's* execution path rather than the materialisation substrate. This split is worth stating
plainly: **materialisers share the join kernel; the rewriter shares the query engine.**

---

## 4. Conformance wiring (the harness already exists)

The entailment harness (`crates/sparq-conformance/src/inference`) already runs RDFS / OWL-RL /
N3 and reports a four-bucket scoreboard (Pass / Fail / **Divergence**(reason) / **OutOfScope**).
Each new regime adds a `Profile` and a suite arm:

- D → a `Profile::D` arm over the rdf-mt datatype tests (currently `OutOfScope`).
- RIF → an N3/RIF suite arm (if a suite is vendorable; else an expressivity ratchet).
- QL → an entailment-regime arm gated behind the experimental CQ-only rewriter, with a pinned
  floor and honest `OutOfScope` for non-CQ shapes.

Every floor is a const in the suite source, synced to `scoreboard::SUITES` by the
`scoreboard_floors.rs` guard — the established graduation pattern.

---

## 5. Phased plan (each phase = a future bead under sq-pbz04)

1. **RL completeness hardening** — close in-profile divergences; adopt shared join. *Acceptance:*
   OWL-RL ratchet rises or holds; remaining divergences stay documented; inference conformance
   green. *Depends on substrate phase 5.*
2. **EL SNOMED/GO end-to-end benchmark + fragment-gap doc** — confirm normalise+RBox+Hasse compose
   with no hidden quadratic; document deferred nominals/concrete-domains in SKILL.md. *Acceptance:*
   relative perf check passes on a fixed ontology slice; EL conformance unchanged.
3. **D-entailment materialiser** (`Profile::D`) — value-space closure via substrate `Num`.
   *Acceptance:* new datatype tests move from OutOfScope to Pass with a pinned floor; byte/bundle
   ratchets unchanged (opt-in). *Independent of QL.*
4. **RIF-Core front-end on the N3 engine** — extend builtins, add RIF-Core mapping, keep monotone.
   *Acceptance:* RIF-Core expressivity ratchet (or vendored suite) green; honest scoping note.
5. **QL PerfectRef MVP (experimental, CQ-only, oracle-tested)** — strict CQ-shape gate that
   *rejects* non-CQ queries; hand-checked DL-Lite oracle. *Acceptance:* rewrites match the oracle
   on the test set; non-CQ queries are rejected (not silently wrong); labelled experimental.
   **Large.** *Depends on nothing in this list but is sequenced late by risk.*
6. **QL tree-witness + UCQ minimisation** — the production path; bounded ABox closure +
   containment minimisation. *Acceptance:* QL entailment-regime arm graduates from experimental;
   matches oracle on the full DL-Lite suite. **Large / long-pole.** *Depends on phase 5.*
7. **(Deferred / future research bead) OWL 2 Direct / DL** — recorded, not built in this program;
   needs external soundness review if ever attempted.

---

## 6. Open questions for the maintainer

1. **QL scope for *this* program:** MVP oracle (phase 5) only, or commit to tree-witness (phase 6)
   now? The spike and I both recommend MVP-then-stop until tree-witness is resourced.
2. **RIF test suite:** is an executable RIF-Core suite vendorable (license/format), or do we frame
   the ratchet as expressivity coverage like RSP?
3. **DL/Direct:** confirm we **defer** it out of this program (record as future research), per the
   spike's EL-first restraint.
