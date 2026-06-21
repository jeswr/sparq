# SPIKE: OWL 2 EL / QL reasoning for `sparq-reason` — feasibility & scoping

<!-- [OPUS-4.8] Feasibility/scoping spike for bead sq-wmeg (GitHub #912). DESIGN-FOR-REVIEW
ONLY — no production code. jeswr verdict on #912: "Yes please spike." -->

> 🤖 **SPARQ agent** — design record for @jeswr's review. This is an exploratory spike, not
> an implementation commitment.

**Status:** SPIKE / design-for-review. **Bead:** sq-wmeg (#912). **Crate target:**
`crates/sparq-reason` (opt-in). **Decision asked of this spike:** add an OWL 2 **EL**
consequence-based classifier, an OWL 2 **QL** query-rewriting reasoner, both, or neither —
and if so, **which first**, with effort sizing and a dataset/perf target.

**Recommendation in one line:** **EL classifier first** (as an opt-in `sparq-reason-el`
capability that computes the complete class-subsumption hierarchy and emits it as
`rdfs:subClassOf` triples), **QL rewriting second** as a separate opt-in pass — because EL
fills a *capability* gap sparq cannot reach by any amount of RL rule-tuning (the complete
subsumption lattice of large biomedical ontologies), reuses the existing dict/Graph
substrate, and has a clean correctness oracle; QL is valuable but lands on a *correctness*
minefield (UCQ-containment, OPTIONAL/FILTER/path scoping) for a smaller incremental win over
the work already shipped.

---

## 0. Premise check (honesty first — what the brief got right and what it got wrong)

The brief's framing is **mostly correct and I verified it against the code**, with **one
correction**:

| Claim in the brief / bead | Verdict | Evidence |
|---|---|---|
| `Profile` enum is `{Rdfs, OwlRl}` only | **TRUE** | `crates/sparq-reason/src/lib.rs:25` — exactly two variants; `parse()` accepts `"rdfs"`/`"owl"`/`"owl-rl"`. |
| No classification / subsumption-lattice output and no query-rewriting path exist | **TRUE** | The only `rewrite_query` paths in the tree are `sparq-vectors` / `sparq-text` / `sparq-solid` (magic-pattern and access-control rewrites) — none is a TBox reasoning rewrite. No `classify`/subsumption API exists. |
| OWL 2 RL is sound but **silently incomplete** for EL/QL classification | **TRUE — and sharper than stated** (see §1) | The incompleteness is *structural to the RL profile*, not a gap in sparq's RL coverage. sparq's RL is in fact **substantially complete** for the RL/RDF rule set (`research/inference-completeness-audit.md` §2: every RL/RDF rule is ✅ or an argued by-design omission, incl. `scm-svf1/2`, `scm-avf1/2`, `scm-hv`). |
| Effort is honestly L/XL (real algorithms, not rule additions) | **TRUE** | Both EL completion and QL rewriting are new algorithm families, not new datalog rules over the existing fixpoint. |
| Perf figures (SNOMED ~350k classes, GO/ChEBI) need CANONICAL numbers; work-box is NON-canonical | **TRUE** | All figures in §3 are **external published** results on stated 2011-era hardware; none is measured here, none is baked as a sparq target. |
| **"No EL/QL reasoner exists in Rust, so this would be novel"** | **PARTLY WRONG — corrected** | A QL/PerfectRef/tree-witness rewriter in Rust: **none found** (novel ✓). But **EL reasoners in Rust DO exist**: **whelk-rs** (INCATools, experimental, MIT, on `horned-owl`) and **DEALER** (a fuzzy-EL++ reasoner, ~6.2k LOC, author blog). What does **not** exist is an *ELK-class* Rust EL reasoner (concurrent lock-free saturation, full indexing/redundancy/transitive-reduction, SNOMED-scale). The novelty is "ELK-class in Rust," not "any EL in Rust." |

The correction matters because it changes the framing from "we'd be first" to "we'd be
first *at ELK-class engineering*, with two small-scale Rust priors to learn from
(`whelk-rs`'s normalization + completion is a readable reference)."

**Where this also corrects the existing docs:** `research/inference.md` §147 already
flagged query-rewriting as a deferred future `Profile`, and
`research/feature-research-broad-sparql-vendors.md` already scored EL (impact 3, effort L)
and QL (overlaps OBDA). This spike supersedes those one-liners with an actual decision.

---

## 1. Why RL — even sparq's *complete* RL — cannot do EL or QL (the load-bearing point)

This is the crux. OWL 2 RL is **sound** but **silently incomplete** for the EL/QL tasks, and
the incompleteness is **not fixable by adding more RL rules** — it is a property of the
profile.

### 1.1 RL's completeness is scoped to *assertional* conclusions, not TBox classification

The W3C Profiles "Reasoning in OWL 2 RL and RDF Graphs using Rules" section (Theorem PR1)
proves the RL/RDF rule set sound-and-complete only for a **restricted entailment shape** and
only on ontologies meeting the §4.2 syntactic constraints; on arbitrary graphs "it is no
longer possible to guarantee that all correct answers can be returned"
(<https://www.w3.org/TR/owl2-profiles/#Reasoning_in_OWL_2_RL_and_RDF_Graphs_using_Rules>).
sparq's own audit already records this: `research/inference-completeness-audit.md` §2 notes
RL completeness "scopes completeness to assertion-style conclusions" and lists
**TBox-axiom conclusions** (e.g. `chain2trans1`) and **invented class expressions** as
**PROVABLY outside the RL/RDF rules** — they appear in sparq's RL conformance run as
*documented divergences*, not bugs.

**Class classification — computing the complete `rdfs:subClassOf` lattice — is exactly a
TBox-conclusion task.** It is the thing RL deliberately does not complete.

### 1.2 The precise mechanism: RL has no rule that reasons *through* an existential successor

The decisive primary source is Markus Krötzsch, **"The Not-So-Easy Task of Computing Class
Subsumptions in OWL RL"** (ISWC 2012,
<https://link.springer.com/chapter/10.1007/978-3-642-35176-1_18>) — the title *is* the
thesis: the standard RL/RDF rule set is **incomplete for class subsumption even on
ontologies that lie within OWL 2 RL**. His companion, "Efficient Rule-Based Inferencing for
OWL EL" (IJCAI 2011,
<https://iccl.inf.tu-dresden.de/w/images/a/a4/Kroetzsch_OWL-EL-Reasoning_IJCAI_2011.pdf>),
builds an *extra* materialisation calculus (auxiliary predicates of higher arity) precisely
because the RL-style rules cannot reason about existential successors.

The EL completion rule that RL lacks is **CR4** (Baader–Brandt–Lutz; ELK's `R⁻∃ + R⁺∃`
link rules): given `C ⊑ ∃r.D`, a derived `D' ∈ S(D)`, and an existential-LHS axiom
`∃r.D' ⊑ E`, conclude `C ⊑ E`. RL's restriction-subsumption rules (`scm-svf1/2`,
`scm-avf1/2`, `scm-hv`) only relate restriction nodes **that already appear syntactically**,
monotone in filler/property; **none introduces or reasons through a fresh existential
successor.**

### 1.3 Concrete counterexample (mechanism verified; ontology is an illustration)

```text
ax1:  A      ⊑  ∃r.B          # A SubClassOf ObjectSomeValuesFrom(r, B)
ax2:  B      ⊑  C             # B SubClassOf C
ax3:  ∃r.C   ⊑  D             # ObjectSomeValuesFrom(r, C) SubClassOf D
```

**EL entails `A ⊑ D`.** Trace: `R⁻∃`/CR3 gives the link `A →ʳ B`; CR1 gives `C ∈ S(B)`;
**CR4** (`(A,B)∈R(r)`, `C∈S(B)`, `∃r.C ⊑ D`) gives `D ∈ S(A)`, i.e. `A ⊑ D`.

**OWL 2 RL forward-chaining does NOT derive `A rdfs:subClassOf D`.** `∃r.C` (ax3) and `∃r.B`
(ax1) are *different* restriction nodes; `scm-svf*` only relate restrictions of the same
property where one named filler subsumes the other, and there is no RL rule that says "an
A-instance has an r-successor of type C, hence `A ⊑ D`." The existential successor is never
materialised, so the bridge `∃r.C ⊑ D` never fires for A.

There is a **second, syntactic** reason: `ObjectSomeValuesFrom` in a *superclass* position
(ax1's RHS) is **outside OWL 2 RL entirely** (RL forbids existentials on the RHS). So RL
cannot even *express* ax1 losslessly. Two independent reasons RL ≠ EL on classification.

**The honest one-line takeaway for users:** *Running `--reason owl` over an EL ontology like
GO or SNOMED returns a sound but **incomplete** class hierarchy — it silently omits
subsumptions that require existential reasoning. RL is not an approximation of EL you can
"tune up" with more rules; EL needs a different algorithm.* This is the capability gap the
spike exists to close.

---

## 2. The two options, honestly

### Option A — OWL 2 EL consequence-based classifier (ELK family)

**Task:** given an EL (or `EL+⊥`) TBox, compute the **complete class-subsumption hierarchy**
(and consistency / `owl:Nothing` clashes), by deterministic least-fixpoint saturation. No
non-determinism, no model case-split — EL has no disjunction/negation/value-restriction, so
a single canonical model suffices and subsumption is **PTIME-complete**
(<https://www.w3.org/TR/owl2-profiles/#OWL_2_EL>, §5).

**Algorithm (verified from primary sources).**

1. **Normalize** axioms to the four EL normal forms (Baader–Brandt–Lutz, "Pushing the EL
   Envelope," IJCAI-05,
   <https://lat.inf.tu-dresden.de/research/papers/2005/BaaderBrandtLutz-IJCAI-05.pdf>):
   `C1 ⊑ D`, `C1 ⊓ C2 ⊑ D`, `C1 ⊑ ∃r.C2`, `∃r.C1 ⊑ D`, plus RBox forms `r ⊑ s` and
   `r1 ∘ r2 ⊑ s`. Linear-time, conservative, introduces fresh names.
2. **Saturate** two mappings — `S(C)` (basic concepts subsuming C) and `R(r)` (pairs
   `(C,D)` with `C ⊑ ∃r.D`) — under completion rules CR1–CR5 (core), CR10/CR11 (role
   hierarchy + composition), CR6 (safe nominals), CR7–CR9 (concrete domains). **CR4 is the
   load-bearing existential-traversal rule** (§1.2).
3. ELK refinement (Kazakov, Krötzsch, Simančík, "The Incredible ELK," JAR 53(1) 2014,
   <https://link.springer.com/article/10.1007/s10817-013-9296-3>): existential **links**
   `C →ᴿ D`, split `R∃` into `R⁻∃`/`R⁺∃`, redundancy "blocking" (don't re-feed `R⁻∃`
   outputs as premises), three phases (index → saturate roles → saturate concepts), and a
   **transitive reduction** to emit the *direct*-subsumer Hasse diagram.

**Fit with sparq.** Excellent. The output is a class hierarchy that maps naturally onto
`rdfs:subClassOf` triples in the dict-encoded store — exactly the shape the existing RL
`scm-*` rules already emit (subClassOf edges that feed rdfs9/11 and are queryable by plain
BGP eval). So an EL classifier can plug into the *same* `(Dict, Vec<[Id;3]>)` seam as
`materialize()`: classify → emit the lattice as subClassOf triples → query as today.
Consistency clashes reuse the existing `inconsistencies()` reporting shape.

**Hard parts (verified):** (1) normalization with fresh-name bookkeeping; (2) **RBox /
property-chain saturation** (transitive roles `r∘r⊑r`, right-identity `r∘s⊑s` for SNOMED) —
classically automata over role chains, the subtle part; (3) the concurrent lock-free
saturation (ELK §6: `AtomicBool` per-context activation + work-stealing queue) — *optional*
for an MVP, single-threaded EL already beats every tableau reasoner; (4) indexing + join
caching + transitive reduction — the gap between "correct" and "ELK-fast"; (5) incremental
classification — a **separate phase**, not MVP (ELK adds it via dedicated follow-up work).

**Rust priors to lean on:** `whelk-rs` (<https://github.com/INCATools/whelk-rs>, experimental
EL on `horned-owl`) and DEALER (fuzzy-EL++, ~6.2k Rust LOC: parser 1.5k / normalizer 1k /
reasoner 1.9k / store 1.7k / taxonomy 0.4k) are readable single-threaded references for the
normalize→complete→reduce pipeline. Neither is ELK-class.

### Option B — OWL 2 QL query-rewriting reasoner (PerfectRef / tree-witness)

**Task:** given a QL (DL-Lite_R) TBox, rewrite a conjunctive SPARQL query into a **union of
conjunctive queries (UCQ)** such that evaluating the UCQ over the **unmodified ABox** (empty
TBox) returns exactly the **certain answers** — no materialization, no closure, no extra
storage. QL is **FO-rewritable**; data complexity is **AC0** (≈ plain SQL/BGP), strictly
below LogSpace (<https://www.w3.org/TR/owl2-profiles/#computational_properties>; Artale et
al., JAIR 36 (2009), <https://arxiv.org/pdf/1401.3487>).

**Algorithm.** **PerfectRef** (Calvanese, De Giacomo, Lembo, Lenzerini, Rosati, JAR 39(3),
2007) saturates a UCQ under two operations to a fixpoint: (a) **rewrite** — apply a positive
TBox inclusion *backward* to one query atom (`A1⊑A2` + atom `A2(x)` ⇒ disjunct `A1(x)`;
`∃R⊑A` + `A(x)` ⇒ `R(x,_)`; `A⊑∃R` + `R(x,_)` with `_` non-shared ⇒ `A(x)`; `R1⊑R2` +
`R2(x,y)` ⇒ `R1(x,y)`); (b) **reduce** — unify two atoms (mgu + dedup), which can turn a
shared join variable into a non-shared one and re-enable rewrites. The **applicability
condition** (an existential-introducing inclusion fires only on a *non-distinguished,
non-shared* variable position) is the #1 unsoundness trap. Terminates (finite TBox
signature); sound + complete for certain answers.

**Fit with sparq.** The seam is **already proven in-repo**: `sparq-vectors::rewrite_query`
(`crates/sparq-vectors/src/rewrite.rs`) walks the parsed `spargebra::algebra::GraphPattern`,
rewrites BGPs, and re-emits a `Query` through the `PreparedQuery: From<spargebra::Query>`
seam so the planner runs unchanged. A QL rewriter is the same shape: BGP → CQ → PerfectRef →
`Union`-folded BGPs under the original `Project`. No store changes.

**The honest cost.** Two real problems:

1. **UCQ blowup is provably unavoidable in general.** QuOnto's PerfectRef "often returned
   hundreds of thousands of CQs even for simple ontologies." For QL ontologies of depth ≥ 2,
   UCQ and nonrecursive-Datalog rewritings have **exponential worst-case blowup**, and FO
   rewritings are super-polynomial unless NP ⊆ P/poly (Kikot et al., ICALP 2012,
   <https://arxiv.org/pdf/1202.4193>). The mitigations are not optional: **tree-witness
   rewriting** (Rodriguez-Muro–Kontchakov–Zakharyaschev, ISWC 2013,
   <https://titan.dcs.bbk.ac.uk/~roman/papers/ISWC13.pdf>) over an **H-complete ABox**
   (cheap hierarchy-only closure), the **combined approach** (KR 2010, bounded ABox
   completion + filtering rewrite), and **UCQ minimization by query containment**. So the
   real target is *tree-witness*, with PerfectRef as a correctness oracle — that is what
   Ontop/Stardog ship.
2. **UCQ-containment minimization is NP-complete and correctness-critical.** Dropping a CQ
   that is *not* actually subsumed silently loses correct answers. This is the costliest and
   most bug-prone step.

**Scope gate (must be enforced or it is unsound).** The rewriting theory is for **(U)CQs
only**. The pass may fire only on `Bgp` / CQ-shaped `Join`s under `Project`/`Distinct`;
**`LeftJoin` (OPTIONAL), `Filter`, `Minus`, `Group`/aggregation, and `Path` (property
paths)** have no certain-answer-preserving UCQ rewriting and must be left untouched or the
query rejected for reasoning. Property paths in particular reintroduce the
transitivity/recursion QL deliberately excludes.

**Rust prior:** **none** — no Rust QL/PerfectRef/tree-witness rewriter exists (the ecosystem
is Java/Scala: Ontop, Mastro/QuOnto, Stardog Blackout, Requiem, Rapid). A spargebra-native
rewriter is genuinely novel.

### Side-by-side

| Dimension | EL classifier (A) | QL rewriting (B) |
|---|---|---|
| What it produces | Complete `rdfs:subClassOf` lattice + clashes (materialized) | Certain answers to CQs (no materialization) |
| Capability gap closed | **Cannot be reached by any RL tuning** (existential classification) | Overlaps tasks RL/materialization *partially* serves; the win is "no closure storage / live data" |
| Substrate fit | Reuses `(Dict, Vec<[Id;3]>)` + subClassOf-emission seam | Reuses the `spargebra` rewrite seam (`sparq-vectors` precedent) |
| Correctness oracle | Cross-check vs ELK on the same ontology (Apache-2.0, runnable) | Cross-check PerfectRef (oracle) vs tree-witness output |
| Dominant risk | RBox/chain saturation correctness (bounded, well-specified) | UCQ-containment minimization + OPTIONAL/FILTER/path scoping (open-ended, silent-answer-loss) |
| Rust prior art | whelk-rs / DEALER (small-scale, readable) | none (fully novel, no oracle code to diff against) |
| Headline external evidence | ELK classifies SNOMED CT in **~5 s** (external, 2011 laptop) | Ontop/Stardog: production OBDA, but blowup-sensitive |
| Effort (honest) | **L–XL**: MVP single-thread classifier ~3–5 wk; +RBox chains, +transitive-reduction, +concurrency, +incremental each a phase | **L–XL**: PerfectRef MVP ~3–5 wk; tree-witness + SQO another ~3–6 wk; containment minimizer is the long pole |

---

## 3. External performance evidence (NON-CANONICAL — published, on stated foreign hardware)

> These are **external published results**, cited so the maintainer can judge the target.
> **None is measured on sparq or on the work box; none is a sparq performance claim or a
> baked target.** Hardware is stated because the numbers are meaningless without it. Any
> sparq number must come from a controlled quiet box / CI, per the repo's
> no-hard-coded-perf-numbers rule.

From "The Incredible ELK" (JAR 2014), classification on **a 2011-era laptop: Intel Core
i7-2630QM 2 GHz quad-core, 6 GB RAM, Windows 7, Java 1.6, 4 GB heap, 8 workers**; SNOMED CT
≈ 300k concepts entailing ≈ 5M subsumptions:

- ELK classifies **SNOMED CT** in roughly the **single-digit-seconds** range; the paper
  reports it as "in as little as 5 seconds," vs tens of seconds (Snorocket) and ~10 minutes
  (CEL/jcel) on the same machine, and ~1 GB heap suffices.
- GO / ChEBI / FMA classify in roughly **~1 second** for ELK in the paper's table; tableau
  reasoners (FaCT++/HermiT) time out or run 10–40× slower on the same machine.
- Concurrency speedup is **sub-linear** and **larger for larger ontologies** (SNOMED ≈ 3.8×
  on the 8-worker quad-core; small ontologies plateau below 2×) — i.e. an MVP
  *single-threaded* EL classifier is already viable; concurrency is a later multiplier.

**Implication for the dataset/perf target (§5):** the right success bar is **correctness +
order-of-magnitude reasonableness**, not "beat ELK's seconds." A correct single-threaded
Rust EL classifier that classifies GO/ChEBI in the *seconds* range and SNOMED in the
*tens-of-seconds* range on a quiet box would be a strong first result; ELK-class timings are
a later optimization phase, not the MVP gate.

---

## 4. Recommendation: **EL first**, QL second — and why

**Recommend Option A (EL classifier) first, shipped as a new opt-in capability crate
`sparq-reason-el` (or a cargo feature on `sparq-reason`), QL as a later separate pass.**

Reasoning (non-sycophantic — I weighed shipping neither, and QL-first):

1. **EL closes a gap nothing else can.** §1 proves RL — *even sparq's substantially-complete
   RL* — cannot compute the EL subsumption lattice. Biomedical ontologies (GO, ChEBI, SNOMED,
   FMA) are the canonical EL use case and are exactly where a user runs `--reason owl` today
   and gets a **silently incomplete** hierarchy. EL is the only way to make that correct.
   QL, by contrast, partially overlaps what materialization already serves; its distinctive
   win (no closure storage, live-updating data) is real but a *smaller incremental* gain on
   top of the shipped incremental-maintenance machinery.
2. **EL fits the existing substrate and has a runnable oracle.** Output = subClassOf triples
   into the same dict/Graph seam the RL `scm-*` rules already use; correctness can be
   differentially checked against **ELK** (Apache-2.0) on real ontologies — the same
   differential-oracle discipline sparq already uses for parsers and SHACL. QL's hardest step
   (UCQ-containment minimization) has **no in-repo or Rust oracle** and fails by *silently
   dropping answers* — a worse failure mode to ship first.
3. **EL's risk is bounded and well-specified; QL's is open-ended.** EL's hard part
   (RBox/chain saturation) is a finite, fully-specified calculus. QL's hard parts (the
   existential-applicability condition, NP-complete containment minimization, and the
   OPTIONAL/FILTER/path scope gate) are each a silent-unsoundness trap, and the UCQ blowup
   forces the tree-witness + combined-approach machinery just to be *practical*.
4. **Both stay opt-in and zero-cost-when-off** (§6), so EL-first does not foreclose QL.

**This is a recommendation to spike-then-build EL, not a commitment to ship a full ELK
clone.** The MVP is a correct single-threaded `EL+⊥` classifier (no concurrency, no
incremental) validated against ELK; the higher-end ELK engineering (concurrency, transitive
reduction tuning, incremental) is explicitly deferred to later phases.

**Open questions that genuinely need the maintainer** (§7) could flip this — in particular,
if the *primary* near-term driver is OBDA / virtual-graph / live-data answering (the
sibling's topic), QL-first is defensible. Absent that signal, EL-first is the call.

---

## 5. Phased plan (each phase = a future bead; ordered)

EL track is the recommended path; the QL track is parked behind it. The orchestrator can
track each phase as its own bead. **The follow-up implementation bead created with this spike
is Phase E1** (below).

**EL track (recommended, do first):**

1. **E1 — EL MVP: normalize + core completion (`EL` minus RBox), single-threaded.**
   New opt-in crate `sparq-reason-el` (or `el` feature). Parse a QL/EL TBox from the graph's
   triples into normal form (the four GCI forms); implement CR1–CR5 saturation over `S(C)` /
   `R(r)`; emit the complete subClassOf lattice as triples + `owl:Nothing` clashes via the
   existing `inconsistencies()` shape. **Gate:** differential correctness vs **ELK** on a
   small EL ontology fixture; clippy + tests in both feature states; core/wasm zero-cost when
   off. *This is the bead created alongside this spike.*
2. **E2 — RBox / property-chain + transitive-role saturation (CR10/CR11, role automata).**
   Adds the SNOMED-critical right-identity/transitive role reasoning. Gate: ELK-differential
   on a chain-bearing fixture (a GO/SNOMED slice).
3. **E3 — Scale + transitive reduction (direct-subsumer Hasse diagram) + indexing/join
   caching.** Move from "correct" toward "fast"; benchmark GO/ChEBI/SNOMED on a **quiet
   box / CI** (deterministic assertion = subsumption *counts* vs ELK; timings advisory only).
4. **E4 — (optional) concurrent lock-free saturation** (ELK §6 contexts + `AtomicBool` +
   work-stealing). Multiplier; sub-linear, larger ontologies benefit most.
5. **E5 — (optional) incremental classification** under TBox edits — separate, substantial;
   only if a use case demands it.
6. **E6 — surfaces:** CLI `--reason el` / `classify` subcommand, `skills/inference/SKILL.md`
   + crate README update (per the MAINTENANCE RULE), and an honest "RL is incomplete for EL —
   use `el`" note where `--reason owl` is documented.

**QL track (second, parked):**

7. **Q1 — QL TBox ingestion + PerfectRef MVP over `spargebra` BGPs** (CQ-only, strict scope
   gate rejecting OPTIONAL/FILTER/Minus/Group/Path). Oracle: PerfectRef is its own reference;
   validate certain answers against an RL-materialize-then-query baseline on QL-expressible
   fixtures.
8. **Q2 — UCQ minimization by query containment** (syntactic subsumption fast-path →
   homomorphism check). The correctness long pole.
9. **Q3 — tree-witness rewriting over an H-complete ABox + combined-approach completion** to
   tame UCQ blowup (the production engine; PerfectRef becomes the oracle).
10. **Q4 — SQO-style pruning** using store domain/range/disjointness, and the CLI/skill
    surface + docs.

Dependencies: E1→E2→E3→(E4,E5,E6); Q1→Q2→Q3→Q4. E and Q tracks are independent of each
other (different seams), so QL could start in parallel if the maintainer greenlights it —
but the recommendation is to sequence EL ahead.

---

## 6. Opt-in / zero-cost discipline (non-negotiable constraint)

Both features MUST follow the established `sparq-reason` opt-in posture (the crate already
proves this: `default = ["parallel"]`, `explain` non-default, never in the lean wasm bundle):

- **EL** ships as a **separate crate `sparq-reason-el`** (preferred — keeps `sparq-reason`'s
  dependency surface unchanged) or a **non-default `el` cargo feature** on `sparq-reason`.
  When off, every EL type/path is `cfg`'d out — `sparq-core`/`sparq-engine` and the wasm
  build carry zero EL code, deps, or runtime cost (the same guarantee the audit records for
  `explain`). EL pulls an OWL-axiom view over the graph triples; it must not add a heavy OWL
  parser to the core.
- **QL** ships as a non-default rewriting pass (a `rewrite`-style entry point gated behind a
  feature), engaged only when the caller opts in — exactly like the `sparq-vectors` /
  `sparq-text` magic-pattern rewrites, which are zero-cost on the default query path.
- A new crate triggers the **G1 new-crate-completeness** gate (README ≤120 lines, registered
  bench or `publish=false` stub, SKILL if it is a public surface) and the **G2/G6** skill +
  config-doc gates if it adds CLI flags. Wire the EL test suite into a **`feature-matrix.yml`
  leg** (default-OFF feature coverage), per the post-batch checklist.

---

## 7. Open questions for the maintainer (genuinely need a decision)

1. **Primary near-term driver?** If it is **biomedical-ontology classification** (GO/SNOMED
   correctness), EL-first is clear. If it is **OBDA / virtual-graph / live-data certain
   answering** (the sibling's topic), QL-first is defensible and would re-order §5.
2. **EL output shape:** emit the lattice as `rdfs:subClassOf` triples into the store
   (queryable as today, recommended), **and/or** expose a typed `ClassHierarchy` API
   (direct-subsumers, equivalents)? The triple emission is the cheap default; the typed API
   is a small add.
3. **EL profile scope for the MVP:** target `EL+⊥` (⊤/⊥/⊓/∃ + role hierarchies + chains —
   what ELK does, covers SNOMED/GO) and **defer** safe-nominals (CR6) and concrete domains
   (CR7–CR9)? Recommended: yes, defer those to a later phase.
4. **Separate crate vs feature for EL** — `sparq-reason-el` (cleaner dep isolation, more
   scaffolding) vs an `el` feature on `sparq-reason` (less scaffolding, shared deps).
   Recommend the **separate crate** to keep `sparq-reason` lean, matching the
   `sparq-shacl`/`sparq-geo` capability-crate pattern.
5. **QL go/no-go and ordering** — greenlight the QL track now (in parallel behind EL), or
   hold it until EL lands and the OBDA driver is confirmed?

---

## 8. Sources (primary unless noted)

**OWL 2 profiles / complexity:**
<https://www.w3.org/TR/owl2-profiles/> (EL §2, QL §3, RL reasoning §4.3, complexity §5).

**EL / ELK:**
- Baader, Brandt, Lutz, "Pushing the EL Envelope," IJCAI-05 —
  <https://lat.inf.tu-dresden.de/research/papers/2005/BaaderBrandtLutz-IJCAI-05.pdf>
- Kazakov, Krötzsch, Simančík, "The Incredible ELK," JAR 53(1) 2014 —
  <https://link.springer.com/article/10.1007/s10817-013-9296-3>
- Kazakov, Krötzsch, Simančík, "Concurrent Classification of EL Ontologies," ISWC 2011 —
  <https://www.uni-ulm.de/fileadmin/website_uni_ulm/iui.inst.090/Publikationen/2011/KazKroSim11Concurrent_ISWC.pdf>
- Krötzsch, "Efficient Rule-Based Inferencing for OWL EL," IJCAI 2011 —
  <https://iccl.inf.tu-dresden.de/w/images/a/a4/Kroetzsch_OWL-EL-Reasoning_IJCAI_2011.pdf>
- Krötzsch, "The Not-So-Easy Task of Computing Class Subsumptions in OWL RL," ISWC 2012 —
  <https://link.springer.com/chapter/10.1007/978-3-642-35176-1_18> (paywalled; thesis = title)
- ELK reasoner (Apache-2.0) — <https://github.com/liveontologies/elk-reasoner>
- Rust priors: whelk-rs — <https://github.com/INCATools/whelk-rs>; DEALER (blog) —
  <https://www.loxation.com/blog/posts/dealer-ontology-reasoner/>

**QL / rewriting:**
- Calvanese, De Giacomo, Lembo, Lenzerini, Rosati, "Tractable Reasoning … DL-Lite Family,"
  JAR 39(3) 2007 (PerfectRef) —
  <https://www.researchgate.net/publication/221651080_Ontological_Query_Answering_via_Rewriting>
- Artale, Calvanese, Kontchakov, Zakharyaschev, "The DL-Lite Family and Relations," JAIR 36
  (2009) — <https://arxiv.org/pdf/1401.3487>
- Kontchakov, Lutz, Toman, Wolter, Zakharyaschev, "The Combined Approach to Query Answering
  in DL-Lite," KR 2010 —
  <https://aaai.org/papers/31-1282-the-combined-approach-to-query-answering-in-dl-lite/>
- Rodriguez-Muro, Kontchakov, Zakharyaschev, "Ontology-Based Data Access: Ontop of
  Databases," ISWC 2013 — <https://titan.dcs.bbk.ac.uk/~roman/papers/ISWC13.pdf>
- Calvanese et al., "Ontop: Answering SPARQL Queries over Relational Databases," SWJ 8(3)
  2017 — <https://www.semantic-web-journal.net/system/files/swj1004.pdf>;
  Ontop guide — <https://ontop-vkg.org/guide/>
- Kikot, Kontchakov, Podolskii, Zakharyaschev, "Exponential Lower Bounds and Separation for
  Query Rewriting," ICALP 2012 — <https://arxiv.org/pdf/1202.4193>
- Stardog Blackout — <https://docs.stardog.com/inference-engine/blackout-owl-support>

**In-repo:** `crates/sparq-reason/src/lib.rs` (Profile enum), `.../src/owl.rs` (RL rules,
`scm-svf/avf`), `research/inference-completeness-audit.md` (RL completeness scope),
`research/inference.md` (deferred query-rewriting note),
`research/feature-research-broad-sparql-vendors.md` (EL/QL feature scoring),
`crates/sparq-vectors/src/rewrite.rs` (the spargebra rewrite seam a QL pass would reuse).

**Verification flags:** the EL-vs-RL §1.3 counterexample's *mechanism* (CR4 absent from RL)
is verified from primary sources; the specific 3-axiom ontology is my own minimal
illustration. The exact PerfectRef applicability/reduce pseudocode is reported from
secondary expositions (the JAR'07 PDF did not text-extract cleanly) and must be re-read from
the source before coding Q1. All §3 timings are external, on the stated 2011-era laptop.
