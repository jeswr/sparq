# OWL 2 Direct Semantics for sparq — sound-and-decidable fragment scoping (sq-pbz04.4) [FABLE-5]

> 🤖 **SPARQ agent** — decomposition design record for @jeswr's review, authored by Claude
> Fable 5 as the Fable-tier architect stage. DESIGN + DECOMPOSITION ONLY — no implementation
> in this PR. Epic: **sq-pbz04.4** (parent sq-pbz04, program sq-6tykl). Per the standing
> `proceed-and-document` rule the child implementation beads are created alongside this
> record but are **dependency-gated behind wave order**; a steering issue lets the maintainer
> amend or veto before wave 1 starts.

**Status:** DESIGN — to be ratified. **Honest starting point:** GREENFIELD. No tableau, no
OWL structural-axiom model, no profile-membership checker, and no OWL functional-syntax
parser exists anywhere in the workspace (verified against the code, not taken on faith —
see §1). Every Direct-Semantics conformance test is `OutOfScope` today.

**Decision in one line:** do **not** attempt a SROIQ(D) tableau. Ship a **layered,
fail-closed Direct-Semantics checker** in a new opt-in crate `sparq-reason-dl`: a structural
OWL model + reverse RDF mapping (L1), a purely syntactic EL/QL/RL **profile-membership
checker** (L2 — the single largest conformance-corpus win, with zero semantic-reasoning
risk), a **terminating ALCH tableau** for consistency/satisfiability with a short, reviewable
subset-blocking termination argument (L3), and a **fragment-dispatch** checker that composes
L3 with the *existing* RL/EL machinery under explicit completeness guards (L4), wired into
`sparq-conformance` as a separately-tallied sparq-extension lane with tri-state
(decided / failed / out-of-fragment) accounting (L5). Everything outside the argued fragment
returns `Unknown` — never a guessed verdict.

---

## 1. Premise check — the estate, verified

| Claim | Verdict | Evidence |
|---|---|---|
| Nothing Direct-Semantics-shaped exists in-tree | **TRUE** | No structural `Axiom`/`ClassExpression` model anywhere; the only class-expression enum is `sparq-reason-el`'s 3-variant `Expr { Atom, And, Exists }` (`crates/sparq-reason-el/src/normal.rs`). No satisfiability checker beyond the RL `inconsistencies()` clash detector (materialisation-based, not a decision procedure). |
| Direct-Semantics tests are all skipped today | **TRUE** | `crates/sparq-conformance/src/inference/owl_suite.rs` runs only `test:profile test:RL` ∧ `test:semantics test:RDF-BASED`; `test:DIRECT`-only cases are counted in `not_rdf_based` and never run. In `sparql11/entailment`, `OWL-Direct`-only regimes are `OutOfScope`; dual-tagged tests run via the *RDF-Based* side (`Profile::OwlRl`). |
| No OWL functional-syntax parser exists | **TRUE** | Three `OutOfScope("… only available in functional syntax")` sites in `owl_suite.rs`. All OWL input is RDF-encoded. |
| No profile-membership checker exists | **TRUE** | `sparq-reason-el`/`-ql` *extract what they can* and skip the rest (`skipped_axioms`, `TBox::skipped`); neither implements the owl2-profiles membership grammars, and nothing checks RL membership at all. |
| The profile reasoners cover real fragments already | **TRUE** | RL: forward-chaining materialiser, sound, RL-incomplete with 13 documented divergences (`DOCUMENTED_DIVERGENCES` in `owl_suite.rs`). EL: CR1–CR9 (+CR10/11 behind `rbox`) classifier with honest `skipped_axioms`. QL: PerfectRef rewriter, **explicitly no consistency checking** (deferred behind sq-pbz04.3.4). |

### 1.1 The corpus, quantified (drives the fragment choice)

The OWL WG export (`tests/w3c/owl2/all.rdf`, fetched by `scripts/fetch-inference-suites.sh`; 493 test cases) contains **485
Direct-Semantics-sanctioned** cases (6 DIRECT-only, 479 dual-tagged DIRECT+RDF-BASED).
Status: 355 Approved / 79 Proposed / 19 Extracredit. By (overlapping) type among the 485:

| Test type | Count | What it needs |
|---|---|---|
| `ProfileIdentificationTest` | **414** | a **syntactic** EL/QL/RL membership checker — no reasoning at all |
| `ConsistencyTest` | 351 | a consistency decision procedure |
| `PositiveEntailmentTest` | 207 | entailment (reducible to inconsistency by refutation) |
| `InconsistencyTest` | 127 | a consistency decision procedure |
| `NegativeEntailmentTest` | 19 | a *complete* fragment (a definitive "consistent" verdict) |

Construct frequency over the DIRECT-sanctioned premise ontologies (top of the tally):
`someValuesFrom` 123 · `intersectionOf` 122 · `maxCardinality` 79 · `allValuesFrom` 72 ·
`inverseOf` 65 · `complementOf` 64 · `minCardinality` 63 · `cardinality` 38 · `unionOf` 35 ·
`oneOf` 34 · `FunctionalProperty` 33 · `disjointWith` 32 · `sameAs` 17 · `differentFrom` 15.
27 of the 485 have no RDF/XML input at all (functional-syntax-only → stay `OutOfScope`).

Three consequences fall straight out of the data:

1. **The profile-identification lane is the dominant graduation** (414 cases) and needs only
   syntactic grammar checks — terminating by construction, no soundness trap. Nothing
   in-tree can serve it today.
2. **Cardinalities and inverses are frequent** in the consistency corpus. Any honest v1
   tableau that guarantees termination cheaply (no pairwise/dynamic blocking) will leave a
   large slice of consistency tests out-of-fragment. That is expected and documented, not a
   failure: a smaller sound fragment beats a bigger unsound one.
3. **The boolean heart of DL** (`complementOf`/`unionOf`/`allValuesFrom` in arbitrary
   positions — 64/35/72 premises) is untouchable by all three profile reasoners *by
   profile definition* (EL has no ⊔/¬/∀; RL restricts them by polarity/position; QL is
   sub-boolean). This is the genuine capability gap a tableau closes.

---

## 2. Options considered

### Option A — full SROIQ(D) tableau ("real OWL 2 DL"). REJECTED for v1.

SROIQ satisfiability is **2NEXPTIME-complete** (Kazakov, KR 2008). The implementation
hazards are exactly the epic's #1 trap: the nominals + inverse-roles + qualified-cardinality
interaction forces pairwise/dynamic blocking and merging with NI-rule bookkeeping, where a
subtly wrong blocking condition is *silently unsound or non-terminating* — the two failure
modes this workstream is mandated to avoid. Verifying such an engine needs a
differential-oracle investment (HermiT/Openllet-class) far beyond a first increment. Every
production DL reasoner took years to get this right; a fleet-implementable bead cannot.

### Option B — profile-dispatch only (delegate everything to RL/EL/QL). REJECTED as the *whole* answer, ADOPTED as a layer.

Sound and cheap, but: (i) QL contributes nothing to consistency today (explicitly deferred,
sq-pbz04.3.4); (ii) the RL branch's "consistent" verdicts are only trustworthy where the
implemented rule set is complete — the 13 documented divergences must gate them; (iii) it
leaves the boolean-DL gap (§1.1 point 3) untouched, so "Direct Semantics support" would be
a relabelling of what already exists. Dispatch is the right *composition* mechanism (L4),
not the whole checker.

### Option C — bounded-model / model-finder consistency checker. REJECTED.

One-sided by construction: finding a model within a bound proves consistency, but bound
exhaustion proves nothing, and claiming "inconsistent" on exhaustion would be unsound.
Verdicts would be budget-shaped rather than fragment-shaped, which makes conformance floors
non-deterministic and the honesty story mushy. No layer of this design uses it.

### Option D — layered fail-closed checker over an argued fragment. **CHOSEN.**

The five layers, each independently useful, each with its own soundness story:

| Layer | What | Soundness/termination story | Corpus effect |
|---|---|---|---|
| L1 | Structural OWL model (`Axiom`/`ClassExpression` enums) + **reverse RDF mapping** with a fail-closed `UnsupportedConstruct` taxonomy | purely syntactic; every RDF pattern either maps per the OWL 2 *Mapping to RDF Graphs* tables or returns a structured error | prerequisite for everything |
| L2 | Syntactic **EL/QL/RL profile-membership checker** (owl2-profiles §2/§3/§4 grammars) | terminating by construction (grammar walk); no semantic claims | the 414-case profile-identification lane |
| L3 | **ALCH tableau** — consistency + class-satisfiability for GCIs + role hierarchies + ground ABox, ancestor **subset blocking** | §3 below — the textbook terminating configuration | boolean/∃/∀ consistency + the class-satisfiability user capability |
| L4 | **Fragment dispatch** + entailment-by-refutation | each branch guarded by an explicit completeness precondition (§4); otherwise `Unknown` | consistency + entailment lanes |
| L5 | Conformance arm (opt-in feature, sparq-extension family, tri-state accounting) | abstention never counted as pass; deterministic count-based budgets | pinned floors |

Why this is the right first increment: it graduates the largest honest slice of the corpus
(L2) with zero reasoning risk; it adds the one semantic capability no profile reasoner can
reach (L3, boolean DL) inside the *smallest well-understood terminating fragment*; it reuses
rather than reinvents RL/EL (L4); and every deferral in §5 has a named, reviewable reason.

---

## 3. The L3 fragment and its soundness / decidability / termination argument

**Fragment (exactly):** ALCH with general TBoxes and a ground ABox —

- Class expressions: named classes, `owl:Thing`, `owl:Nothing`, `⊓` (`intersectionOf`),
  `⊔` (`unionOf`), `¬` (`complementOf`), `∃R.C` (`someValuesFrom`), `∀R.C`
  (`allValuesFrom`), over **named object properties only**.
- TBox: arbitrary GCIs `C ⊑ D` (hence `equivalentClass`, `disjointWith` by desugaring);
  `rdfs:domain`/`rdfs:range` desugared as `∃R.⊤ ⊑ C` / `⊤ ⊑ ∀R.C`.
- RBox: `rdfs:subPropertyOf` hierarchies over named object properties (`⊑*` its
  reflexive-transitive closure). **No** inverses, transitivity, chains, functionality,
  cardinality, (a)symmetry, (ir)reflexivity, disjointness.
- ABox: ground class assertions `C(a)` and role assertions `R(a,b)` over named individuals.
  **No** `sameAs`/`differentFrom` (fail-closed in this path; the RL branch covers them for
  RL ontologies), no nominals in class expressions (`oneOf`/`hasValue` out), no data
  properties / datatypes.

**Decidability/complexity (known results, not claims of novelty):** concept satisfiability
w.r.t. general TBoxes in ALC is EXPTIME-complete (Schild's PDL correspondence; see Baader &
Sattler, *An Overview of Tableau Algorithms for Description Logics*, Studia Logica 69,
2001, and the Description Logic Handbook ch. 2–3). Adding role hierarchies without inverses
(ALCH) does not change the tableau's blocking requirements.

**The procedure:** negation normal form; `sub(O)` = the closure of subexpressions of the
NNF'd input (finite, linear in input size); a completion **forest** rooted at the ABox
individuals; node labels `L(x) ⊆ sub(O)`; rules ⊓/⊔/∃/∀/GCI, with the ∀-rule and ∃-rule
matching modulo `⊑*` on roles; **ancestor subset blocking**: a node `y` is blocked by an
ancestor `x` in the same tree when `L(y) ⊆ L(x)`; the ∃-rule never expands a blocked node.
The ⊔-rule and the GCI internalisation (`⊤ ⊑ NNF(¬C ⊔ D)` added to every node) introduce
the nondeterminism, explored by backtracking search.

**Termination argument** (to be reproduced in the module docs of `tableau.rs` and reviewed
as part of the bead's acceptance):

1. `sub(O)` is finite and fixed; every label is a subset of it, so there are at most
   `2^|sub(O)|` distinct labels.
2. Rules only **add** concepts to labels or **add** nodes — labels grow monotonically and
   nodes are never relabelled downward, so each node admits finitely many rule firings.
3. On any root-to-leaf path, if the path length exceeds `2^|sub(O)|` then two nodes on it
   have `L(y) ⊆ L(x)` with `x` an ancestor (pigeonhole on the label lattice), so `y` is
   blocked and the path stops growing. Depth is therefore bounded.
4. Out-degree is bounded by the number of `∃R.C ∈ sub(O)` plus the ABox role assertions.
   A bounded-depth, bounded-out-degree forest is finite; with (2), the rule system
   terminates on every branch of the nondeterministic search, hence the procedure
   terminates. ∎

**Soundness** (an "Inconsistent" verdict is true): each rule preserves satisfiability of
the constraint system, so a closed search space (clash — `{A, ¬A} ⊆ L(x)` or
`Nothing ∈ L(x)` — on every branch) implies unsatisfiability. **Completeness** for this
fragment (a "Consistent" verdict is true): from a clash-free fully-expanded forest, build a
model by unravelling blocked nodes to their blockers — the standard construction, valid for
subset blocking precisely **because ALCH has no inverse roles** (nothing propagates
constraints from a child back up into a blocked node's context). This "why subset blocking
suffices *here*" note is load-bearing and must appear in the module docs: it is exactly the
condition that fails when inverses are added, which is why inverses are deferred (§5).

**Budgets:** the tableau accepts a deterministic **count-based** budget (max nodes / max
rule applications). Budget exhaustion returns `Unknown(ResourceBudget)` — never a verdict —
and the conformance lane pins floors only on budget-independent outcomes (a case that
decides within the fixed CI budget is asserted decided; anything else is out-of-fragment
for floor purposes). Wall-clock budgets are banned (non-deterministic floors).

---

## 4. The L4 dispatch — composing with the existing reasoners, honestly

Given an ontology (structural model from L1), dispatch in order:

1. **RL branch** (if L2 says the ontology is in RL): run the existing
   `sparq_reason::materialize(Profile::OwlRl)` + `inconsistencies()`.
   - An **Inconsistent** verdict is always sound (the implemented rules are sound, and for
     RL ontologies Theorem PR1 of owl2-profiles ties rule-derived inconsistency to
     Direct-Semantics inconsistency). The PR1 *preconditions* are checked, not assumed.
   - A **Consistent** verdict ("no clash") additionally requires rule-set *completeness*,
     which sparq's RL does not globally have (13 documented divergences). Gate: the
     dispatch derives a **divergence guard** from the constructs implicated in
     `DOCUMENTED_DIVERGENCES`; if the input touches any implicated construct, the branch
     returns `Unknown(RlDivergenceGuard)` instead of "consistent".
2. **EL branch** (if in EL): run `sparq_reason_el::classify_graph`; verdicts are emitted
   **only when `Report::skipped_axioms == 0`** (a skipped axiom could be the inconsistency);
   otherwise `Unknown(ElSkippedAxioms)`.
3. **QL branch**: always `Unknown(QlConsistencyPending)` — DL-Lite_R consistency is
   explicitly deferred in the QL workstream (sq-pbz04.3.4's pending-consistency tally).
   Revisit when that lands; do **not** duplicate it here.
4. **ALCH branch** (if the whole ontology is inside the §3 fragment): the L3 tableau,
   yielding a definitive verdict (complete for the fragment).
5. Otherwise: `Unknown(OutOfFragment(constructs…))`.

**Entailment by refutation** (`PositiveEntailmentTest`): `O ⊨ α` is checked per
conclusion-axiom, each with an explicit sound encoding into a consistency question —
`SubClassOf(C,D)` → `O ∪ {(C ⊓ ¬D)(x_fresh)}` inconsistent; `ClassAssertion(C,a)` →
`O ∪ {(¬C)(a)}` inconsistent; `ObjectPropertyAssertion(R,a,b)` → `O ∪ {B(b), (∀R.¬B)(a)}`
inconsistent with `B` a **fresh** class name (sound *and* complete: a model of `O` lacking
`R(a,b)` extends to one interpreting `B` as `{b}`, and conversely `b ∈ B` forces the clash
exactly when every model has `R(a,b)`); `SubObjectPropertyOf(R,S)` →
`O ∪ {R(a,b), B(b), (∀S.¬B)(a)}` inconsistent with `a`, `b` **fresh** individuals and `B` a
**fresh** class name (**§4 amendment, sq-pbz04.4.9** — the role-subsumption lift of the
fresh-class trick; the original record deferred property-axiom conclusions). Soundness: if
`O ⊨ R ⊑ S`, any model of the union has `(a,b) ∈ R ⊆ S`, so `b` is an `S`-successor of `a`
and `(∀S.¬B)(a)` forces `b ∈ ¬B`, clashing with `B(b)` — no model exists. Completeness: a
model `I` of `O` with `(u,v) ∈ Rᴵ ∖ Sᴵ` extends to the union via `aᴵ = u`, `bᴵ = v`,
`Bᴵ = {v}` — the fresh names occur nowhere in `O`, and `(u,v) ∉ Sᴵ` means no `S`-successor
of `u` is `v` (the sole member of `B`), so `(∀S.¬B)(a)` holds. The decision procedure is
faithful to this semantics because the L3 tableau's `∀`-rule fires modulo the
reflexive-transitive role hierarchy, and all three additions stay inside the §3 fragment.
Conclusion-axiom kinds without an argued encoding (keys, …; none currently expressible in
the L1 model) → `Unknown`. **Negative entailment** verdicts are
emitted only when the refutation check lands in a branch that is *complete* (EL-guarded,
ALCH, or RL-with-guard) — a definitive "consistent" is what certifies non-entailment.

**Double-counting discipline:** the dual-tagged (DIRECT + RDF-BASED) tests keep their
RDF-Based run in the existing standards-conformance lane; the new DIRECT arm is a separate
sparq-extension row. One test may legitimately appear in both tallies because they test
different semantics; the scoreboard note says so.

**Profile-identification lane semantics:** the lane asserts positive membership for tagged
profiles and non-membership for untagged ones among {EL, QL, RL} — *after* validating the
absence-is-negative reading against known corpus cases during implementation; if that
reading proves wrong, the floor pins positive-membership checks only, and says so. The
OWL 2 **DL species check** (global restrictions: regularity, simple-role constraints) is
**deferred** — the corpus tags 319 cases as species DL, but validating the species claim
needs machinery (role-hierarchy regularity analysis) that the v1 fragment does not, and a
wrong species check would poison the lane. Untagged species assertions are not checked.

---

## 5. Deferral ledger (what is OUT, and why)

| Construct / capability | Deferred because | Unlock path |
|---|---|---|
| Inverse roles (`inverseOf`, I) | subset blocking becomes **incomplete** (constraints propagate back into blocked contexts); needs equality blocking | dedicated bead post-v1, with a new blocking argument |
| Transitive roles (S) | **IMPLEMENTED (opt-in)** — bead sq-zfwzq [GPT-5.6] shipped the first-in-line extension behind the OFF-by-default `dl_transitive` cargo feature: L1 recognises `owl:TransitiveProperty` (→ `TransitiveObjectProperty`), the L3 tableau adds the ∀₊-propagation rule, and the extended termination/soundness/completeness argument (Horrocks & Sattler 1999: *no* blocking change absent inverses — the `E(R) ∪ ⋃ E(T)⁺` model construction) is WRITTEN OUT in `tableau.rs` module docs §5a, per this ledger's discipline (deferrals are argued, never silently added). L4 dispatch routes transitive ontologies straight to the tableau (the only transitivity-complete branch); the RL/EL guards recognise the axiom kind fail-closed as defence in depth. With the feature OFF, behaviour is byte-identical to v1 (fail-closed refusal). | done (sq-zfwzq); inverses/cardinality/nominals below remain the next rows |
| Cardinalities / functionality (N/Q/F) | need the choose-rule + node merging; merging + blocking interaction is the classic unsoundness trap; with inverses added later this escalates to pairwise blocking | post-v1, only with a written argument |
| Nominals in class expressions (`oneOf`, `hasValue`, O) | the O+I+Q interaction is where NEXPTIME bites and where naive blocking is unsound; a "fresh-concept" approximation is sound but **incomplete** and would blur the completeness ledger | revisit only after I and Q are argued |
| Datatypes / data properties (D) | a datatype-aware tableau needs a concrete-domain satisfiability oracle; sparq already has the seam (`sparq-substrate` numeric tower; `dtype.rs`; EL `cdomain`) but wiring it into a tableau is its own design problem. **sq-pbz04.4.9 hardened the L1 boundary here:** a bare OWL 2 datatype-map IRI (`xsd:*`, `rdfs:Literal`, `rdf:PlainLiteral`/`XMLLiteral`, `owl:real`/`rational`) reaching ANY class-expression position — including an `rdfs:range`/`rdfs:domain` object — now refuses extraction as a `DataConstruct` rather than reading as an opaque object class (which silently dropped the value-space meaning; two disjoint datatype ranges were invisibly unrelated, e.g. WebOnt-I5.3-015). Uniform position check, same discipline as the literal / `owl:onDatatype` refusals. | future record; reuse the shared tower, never a private one |
| `sameAs` / `differentFrom` in the tableau path | equality reasoning = merging (see cardinalities); RL branch already covers them for RL ontologies | with N/Q |
| Keys, property chains, `hasSelf`, property disjointness/(a)symmetry/(ir)reflexivity | outside ALCH; RL branch handles their assertional consequences for RL ontologies | per-construct evaluation later |
| OWL 2 DL species validation (global restrictions) | needs regularity/simple-role machinery; wrong checks poison the profile lane | post-v1 bead if the lane's value warrants it |
| `owl:imports` | no dereferencing in the conformance harness (existing posture) | unchanged |
| Functional-syntax-only tests (27 of 485) | no `.ofn` parser in-tree; a parser is mechanical but out of this epic's scope | possible future haiku-tier bead if the 27 cases justify it |
| Full SROIQ(D) | §2 Option A | not planned; would be its own program |

Honest floor expectation (no numbers promised): the profile-identification lane should
graduate the bulk of its 414 cases; the consistency/entailment lanes will graduate the
boolean/∃/∀-shaped and RL/EL-profile subsets and leave the cardinality/inverse-heavy rest
out-of-fragment. The scoreboard row is labelled **"scoped fragment — NOT full OWL 2 DL"**
and the lane never counts an abstention as a pass.

---

## 6. Placement, gating, wiring

- **New crate `crates/sparq-reason-dl`** (workspace member, `publish = false` initially,
  README per the readme-template HARD gate). Separate crate — same isolation rationale as
  `sparq-reason-el`/`-ql`; `sparq-core`/`sparq-engine` gain nothing.
- Crate features: `default = []`; `dispatch = ["dep:sparq-reason", "dep:sparq-reason-el"]`
  (L4 pulls the profile reasoners; L1–L3 are dependency-free beyond `sparq-core`).
- **Conformance wiring** mirrors the `ql-experimental` precedent exactly: optional dep in
  `sparq-conformance/Cargo.toml` behind a `dl-direct` feature; a new
  `src/inference/dl_suite.rs` runner over the DIRECT arm of `tests/w3c/owl2/all.rdf`;
  `family = "sparq extension"` scoreboard rows (profile-identification lane + Direct
  consistency/entailment lane); a self-skipping `tests/dl_suite.rs` with pinned floors.
- Feature-gated intra-doc links use code spans (the recurring rustdoc trap); every new
  public fn gets one direct unit test (coverage-ratchet floor).

## 7. Child beads (created with this record, dependency-gated)

All are `--parent sq-pbz04.4`, exclusive-files disjoint within a wave; cross-wave overlaps
(lib.rs/Cargo.toml owned by D1, later consumed by D4) are serialized by `bd dep`.

| Bead | Wave | Tier | Scope (exclusive files) |
|---|---|---|---|
| sq-pbz04.4.1 — L1 structural model + fail-closed reverse RDF mapping; crate scaffold **pre-declares stub modules** `profile`/`nnf`/`tableau`/`check` so later beads never touch `lib.rs` | 1 | sonnet | root `Cargo.toml`; `crates/sparq-reason-dl/{Cargo.toml, README.md, src/lib.rs, src/model.rs, src/extract.rs, src/profile.rs(stub), src/nnf.rs(stub), src/tableau.rs(stub), src/check.rs(stub), tests/extract.rs}` |
| sq-pbz04.4.2 — L2 syntactic EL/QL/RL profile-membership checker | 2 | sonnet | `src/profile.rs`, `tests/profile.rs` |
| sq-pbz04.4.3 — L3 ALCH tableau (NNF + forest + subset blocking + count budgets), termination/soundness argument in module docs | 2 | opus | `src/nnf.rs`, `src/tableau.rs`, `tests/tableau.rs` |
| sq-pbz04.4.4 — L4 dispatch + entailment-by-refutation (RL divergence guard, EL skipped-axioms guard, QL pending, refutation encodings) | 3 | opus | `src/check.rs`, `tests/check.rs`, plus the D1-owned `Cargo.toml`/`lib.rs` feature/re-export lines (serialized, sole bead in its wave) |
| sq-pbz04.4.5 — L5 conformance arm: `dl-direct` feature, DIRECT-arm runner, tri-state accounting, pinned extension floors | 4 | sonnet | `crates/sparq-conformance/{Cargo.toml, src/inference/dl_suite.rs, src/inference/mod.rs, src/scoreboard.rs, tests/dl_suite.rs}` |
| sq-pbz04.4.6 — docs: `skills/inference/SKILL.md` fragment table + deferral-ledger pointer, honest not-full-DL qualifier | 5 | haiku | `skills/inference/SKILL.md` (cross-EPIC contention with sq-pbz04.5.6 / sq-pbz04.6.5 — run after they land or rebase) |

Deps: `.2 ← .1`, `.3 ← .1`, `.4 ← {.2, .3}`, `.5 ← .4`, `.6 ← .5`. The only parallel pair
is `.2 ∥ .3`, which share no file (stubs pre-created by `.1`).

Soundness-critical tiering: the tableau (`.3`) and the dispatch/guards (`.4`) are
**opus-tier** — their invariants are exactly the "unsound blocking / fake completeness"
traps; the mechanical spec-table work (`.1`, `.2`, `.5`) is sonnet; docs are haiku.

## 8. Sources

- W3C OWL 2 Profiles: <https://www.w3.org/TR/owl2-profiles/> (EL §2, QL §3, RL §4 + Theorem PR1, complexity §5).
- W3C OWL 2 Mapping to RDF Graphs: <https://www.w3.org/TR/owl2-mapping-to-rdf/> (the L1 reverse-mapping tables).
- W3C OWL 2 Structural Specification (incl. §11 global restrictions — deferred species check): <https://www.w3.org/TR/owl2-syntax/>.
- Baader & Sattler, *An Overview of Tableau Algorithms for Description Logics*, Studia Logica 69, 2001 (subset blocking; ALC+GCI termination/completeness).
- Baader, Calvanese, McGuinness, Nardi, Patel-Schneider (eds.), *The Description Logic Handbook*, 2nd ed. (ALC EXPTIME-completeness; blocking taxonomy).
- Horrocks & Sattler, *A Description Logic with Transitive and Inverse Roles and Role Hierarchies*, J. Logic & Computation 9(3), 1999 (why inverses force stronger blocking; the S-extension path).
- Kazakov, *RIQ and SROIQ Are Harder than SHOIQ*, KR 2008 (SROIQ 2NEXPTIME-completeness).
- In-repo: `research/reasoner-federation-program.md` (program frame; "Direct last, gated on its design record"); `research/owl2-el-ql-reasoning-spike.md` (profile-reasoner scoping precedent); `crates/sparq-reason{,-el,-ql}` and `crates/sparq-conformance/src/inference/*` as surveyed in §1; `tests/w3c/owl2/all.rdf` (corpus quantification in §1.1, measured by script over the file fetched by `scripts/fetch-inference-suites.sh`).

<!-- [FABLE-5] sq-pbz04.4 decomposition record. No performance numbers by design; floors live in CI ratchets. -->
