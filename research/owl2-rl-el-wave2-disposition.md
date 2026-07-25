# OWL 2 RL + EL workstreams: wave-1 completion audit and the EL wave-2 decomposition

<!-- [FABLE-5] Design record for the sq-pbz04.1 / sq-pbz04.2 decomposition pass (epic sq-pbz04,
     program sq-6tykl). 🤖 SPARQ agent. Status: DECISION RECORD — wave-1 audit is
     implemented-and-verified state; wave-2 is DESIGNED-ONLY until its beads land. -->

## 1. Premise correction

This record answers a decomposition request for the "two empty reasoner profile workstream
sub-epics" sq-pbz04.1 (OWL 2 RL + RDFS, `sparq-reason`) and sq-pbz04.2 (OWL 2 EL,
`sparq-reason-el`). **The premise is stale: neither sub-epic is empty.** Both were decomposed
on 2026-07-03 and fully drained by 2026-07-06:

| Bead | What landed | PR |
|------|-------------|----|
| sq-pbz04.1.1 | rdfs9/`rdf:type` branch ADOPTED onto the substrate join kernel; `PropExpand` RETAINED hand-rolled with a written per-branch rationale | #1430 |
| sq-pbz04.1.2 | `sparq_substrate::compare::CompareTerm` adopted for dictionary ids — entailed-solution ordering parity with the engine (seam 3) | #1455 |
| sq-pbz04.1.3 | Per-divergence disposition over the RL suite's documented divergences — fix genuinely-entailed, document beyond-profile | #1428 |
| (sq-qonbz.2 → sq-6w7x6) | OWL-RL semi-naive Δ⋈full fixpoint migrated onto the `join::delta` `DeltaTable` seam (`owl_delta_adj.rs`), byte-identical ratchet in both feature states | #1643 |
| sq-pbz04.2.1 | CR6 safe nominals (`owl:oneOf`/`owl:hasValue`) with the real Baader–Brandt–Lutz reachability side-condition | #1427 |
| sq-pbz04.2.2 | CR7–CR9 concrete domains on `sparq_substrate::numeric` (exact tier only; honest deferral boundary documented in `cdomain.rs`) | #1434 |
| sq-pbz04.2.3 | Reasoned NON-adoption of substrate joins for the CR1–CR5 saturation (documented in `classify.rs`) | #1442 |
| sq-pbz04.2.4 | OWL 2 EL entailment-regime conformance arm + ratchet (`tests/el_suite.rs`, `EL_SUITE_FLOOR = 50`) | #1457 |

So the honest output of this pass is not a fresh greenfield plan. It is: **(a)** an audit
verdict that the RL workstream is complete as scoped (§2 — recommend closing sq-pbz04.1,
zero new beads), and **(b)** a residue analysis showing the EL workstream has a genuine,
measurable, in-profile second wave (§3–§5 — six new beads under sq-pbz04.2).

## 2. RL workstream audit (`sparq-reason`) — complete as scoped; zero new beads

### 2.1 What "honest RL-completeness" means here, and where it stands

The `owl2-rl` conformance lane (`crates/sparq-conformance/src/inference/owl_suite.rs`)
selects `test:RL` ∧ `test:RDF-BASED` Approved cases from the pinned OWL WG export and runs
each through `materialize_owl_rl` + the bnode-homomorphism entailment check. Completeness is
**measured against that corpus, never claimed beyond it**: passes ratchet a floor, and every
remaining fail must carry a pinned per-case rationale in `DOCUMENTED_DIVERGENCES` (a
divergence-listed case that starts passing is flagged as a stale entry).

After the #1428 disposition pass, **all 13 remaining divergences are PERMANENT**, each with a
corpus-grounded rationale falling into four mechanisms:

1. **No rule head constructs class expressions or reified structures** — the RL/RDF rules'
   PR1 completeness theorem is assertion-only; conclusions that assert anonymous
   `owl:complementOf`/`owl:unionOf` classes or reified `owl:AllDifferent` structures are
   underivable by design (`DisjointClasses-001/-003`, `New-Feature-ObjectQCR-002`,
   `WebOnt-I5.5-005`, `New-Feature-DisjointDataProperties-002`,
   `New-Feature-DisjointObjectProperties-001/-002`).
2. **Contrapositive derivations** — `owl:differentFrom` between individuals via the
   contrapositives of prp-fp/prp-ifp/prp-pdw has no producing rule
   (`owl2-rl-rules-fp-differentFrom`, `owl2-rl-rules-ifp-differentFrom`, and the
   DisjointObjectProperties pair above).
3. **Profile-excluded input** — `ReflexiveObjectProperty` is excluded from the RL grammar
   (Profiles §4.2), so no prp-rfx rule exists (`New-Feature-ReflexiveProperty-001`).
4. **Datatype value-space intersection** — TBox `rdfs:range` conclusions needing
   `xsd` value-space intersection are beyond the dt-\*/scm-\* rules
   (`WebOnt-I5.8-008/-009`, plus `chain2trans1`'s TBox `owl:TransitiveProperty` conclusion).

Fixing any of these would require rule heads the RL/RDF rule set does not license — i.e. it
would be **unsound relative to the profile spec**, not a completeness win. The honest
RL-completeness push is therefore **done**: measured floor + 13 documented-permanent
divergences, none re-classifiable as fixable.

### 2.2 Substrate delta-seam adoption — done, with one documented permanent retention

`substrate_join.rs` records the full disposition:

- rdfs2/rdfs3/rdfs7 predicate joins: **adopted** (sq-yk6or, #1301);
- rdfs9/`rdf:type` subclass typing: **adopted** (sq-pbz04.1.1, #1430 — a uniform join after
  all, expressed via `JoinKeys` column pairs);
- `PropExpand` (inverseOf/Symmetric/equivalentProperty rewrite): **retained hand-rolled,
  permanently** — its per-match combine is data-dependent (each build row selects its own
  output orientation) and fans into a cascaded second join over a derived column; adoption
  would rebuild the rule structure around the kernel to share only its innermost map lookup;
- the OWL-RL semi-naive Δ⋈full fixpoint: **migrated onto `join::delta`**
  (`owl_delta_adj.rs`, #1643) — persistent extend/probe tables, UnionFind sameAs kept,
  byte-identical OWL-RL ratchet in both feature states;
- `CompareTerm` (seam 3, #1455) and `Num::cmp_relational` (sq-v5evr, #1646) wired.

### 2.3 No-perf-regression guard (held; nothing further to bead)

All substrate adoption sits behind the default-off `substrate-join` feature with
identical-output cross-assert tests against the plain branch; `scripts/check-no-dyn-dispatch.py`
guards the substrate hot paths plus `sparq-reason/src/compare.rs` (clean on main); the wasm
bundle / store / dict ratchet floors are unchanged with features off. Timings live in PR
bodies only (work-box numbers are non-canonical) — no committed performance numbers.

### 2.4 Considered and NOT beaded

- **OWL functional-syntax premise parsing** and **`owl:imports` dereferencing** would
  graduate some `OutOfScope` corpus rows, but that is harness/corpus scope (a whole new
  parser / a fetch policy), not RL reasoning completeness — poor value for the sub-epic.
- Further `PropExpand` kernel adoption — re-litigating a written permanent disposition with
  a pinned red/green harness; no new information since #1430.

**Verdict: recommend closing sq-pbz04.1** (all children complete, the delta-seam long pole
landed via sq-qonbz.2, residue empty).

## 3. EL workstream audit (`sparq-reason-el`) — scoped items complete; a real wave 2 exists

### 3.1 Where wave 1 ended

The crate is a consequence-based classifier: CR1–CR5 saturation, CR6 safe nominals with the
genuine reachability side-condition (#1427), CR7–CR9 concrete domains on the shared exact
numeric tier with a precisely documented deferral boundary (#1434), optional `rbox`
(CR10/CR11 role automaton, internal-only) and `hasse` features, and honest
`Report::skipped_axioms` accounting for everything outside the implemented fragment. The
substrate question is settled: reasoned non-adoption for the CR1–CR5 joins (#1442), while
`cdomain` **is** the crate's first real substrate consumer (`sparq_substrate::numeric`).

The conformance arm (#1457) pins `EL_SUITE_FLOOR = 50` with the composition
45 consistency / 0 inconsistency / 2 positive-entailment / 3 negative-entailment, and 28
audited PERMANENT divergences. Two facts in that audit define wave 2:

1. **The lane runs the classifier with NO features** (`el-suite = ["dep:sparq-reason-el"]`
   in `crates/sparq-conformance/Cargo.toml`) — the landed `rbox` and `cdomain` code paths
   are never exercised by conformance.
2. **The 28 divergences fall into three mechanisms** (per the audit in
   `tests/el_suite.rs`): ABox/instance reasoning (~19 cases: the classifier is TBox-only and
   internalizes no assertions), RBox-off (~5 cases), and output-vocabulary (~2 cases:
   mutual `rdfs:subClassOf` emitted but the conclusion asserts `owl:equivalentClass`), plus
   `WebOnt-I5.5-005` (`owl:unionOf` — genuinely outside EL).

### 3.2 The in-profile gap (a label correction)

`extract.rs` lumps `hasSelf` into `NON_EL_MARKERS` under the comment "outside EL". That is
imprecise: **`ObjectHasSelf`, `HasKey`, `ReflexiveObjectProperty`, negative property
assertions, `DifferentIndividuals`, and class/property assertions are all inside the W3C
OWL 2 EL profile grammar** — outside only the crate's *implemented fragment*. (The deferral
itself is honest — counted skips, never mis-derivation — only the label overstates the
boundary.) `owl:unionOf`/`complementOf`/`allValuesFrom`/cardinality are genuinely outside
EL. So a wave-2 build-out of ABox + self-restrictions + keys is *profile-completing* work
squarely inside the parent epic's "verify+complete OWL-EL" mandate — and the CR6 nominal
machinery that landed in wave 1 is exactly the foundation ABox internalization needs
(assertions become safe-nominal axioms: `a rdf:type C` ⇒ `{a} ⊑ C`,
`a p b` ⇒ `{a} ⊑ ∃p.{b}`).

### 3.3 Options considered

- **A. Close sq-pbz04.2 too; no wave 2.** Rejected: ~24 of 28 pinned divergences are
  addressable inside the EL profile, the floor's own composition note admits only 2 rows
  exercise real derivation, and the parent epic explicitly asks for EL completion.
- **B. Route EL ABox through the RL materialiser.** Rejected: RL is the crate's documented
  complement — it has no rule reasoning *through* existential successors, which is the whole
  reason `sparq-reason-el` exists; instance realisation for EL ontologies belongs in the
  consequence-based calculus where the nominal rule lives.
- **C. One mega-bead.** Rejected: four soundness-sensitive extensions plus two conformance
  re-pins in one diff defeats tier economy and review scope.
- **D. Six beads in two file-disjoint lanes (chosen)** — a sequenced `sparq-reason-el`
  chain (same-crate files overlap, so NON-parallel by `bd dep` edges) and a
  `sparq-conformance` lane that runs in parallel with the chain head.

## 4. The wave-2 decomposition (six beads under sq-pbz04.2)

Lane 1 — `sparq-reason-el` (chain; one bead in flight at a time; each bead also syncs
`README.md` + `skills/inference/SKILL.md` for its additive public API, safe because the
chain is sequenced):

1. **E1 — ABox internalization + realisation + whole-ontology consistency** (new opt-in
   `abox` feature; `src/abox.rs` new, `src/extract.rs`, `src/lib.rs`). Assertions encoded as
   safe-nominal axioms over the landed CR6 machinery; readoff `{a} ⊑ C` ⇒ typing,
   `{a} ⊑ {b}` ⇒ `owl:sameAs`, `{a} ⊑ ⊥` or `⊤ ⊑ ⊥` ⇒ inconsistent. Data-property
   assertions defer to skips except the `cdomain` point-range forms. tier: opus.
2. **E2 — `ObjectHasSelf` completion rules** (`src/extract.rs`, `src/normal.rs`,
   `src/classify.rs`, `src/lib.rs`; corrects the `NON_EL_MARKERS` label). The two EL++
   self-restriction rules (`X ⊑ ∃r.Self ⇒ (X,X) ∈ R(r)`; `(X,X) ∈ R(r), ∃r.Self ⊑ D ⇒
   X ⊑ D`) with exact side-conditions. tier: opus.
3. **E3 — role-lattice readoff under `rbox`** (`src/rbox.rs`, `src/lib.rs`). Emit the
   already-computed reflexive-transitive told-inclusion closure as `rdfs:subPropertyOf`
   rows (readoff only — no new saturation). tier: sonnet.
4. **E4 — `HasKey` + negative property assertions + `differentFrom`** (`src/abox.rs`,
   `src/extract.rs`, `src/classify.rs`, `src/lib.rs`). Keys fire only when BOTH individuals
   have derivable values on EVERY key property (the classic over-derivation trap); NPA
   clashes only against a derived/asserted positive assertion; `differentFrom` readoff only
   from a derived `{a} ⊓ {b} ⊑ ⊥`-style clash or an asserted-inequality-vs-derived-sameAs
   contradiction. tier: opus.

Lane 2 — `sparq-conformance` (parallel with E1; C2 sequenced after C1 on the same files):

1. **C1 — feature-ON lane + `owl:equivalentClass` output-vocabulary completion**
   (`Cargo.toml`, `tests/el_suite.rs`). Make `el-suite` pull `sparq-reason-el/rbox` +
   `/cdomain`; add the semantically exact mutual-subsumption ⇒ `owl:equivalentClass`
   augmentation before the homomorphism check (mirroring the lane's existing
   `augment_datatypes` precedent); re-audit and re-pin with measured evidence
   (`WebOnt-equivalentClass-003` is expected to graduate). tier: sonnet.
2. **C2 — ABox-mechanism graduation re-pin** (same two files; depends on E4 + C1). Enable
   `abox` in the lane, prune ONLY pins observed to pass, raise the floor to the measured
   value, refresh the remaining divergences' mechanism rationales. tier: sonnet.

Dependency edges: E1 → E2 → E3 → E4 (file-overlap sequencing; E2's and E4's corpus
graduations also semantically need E1), C1 independent, C2 ← {E4, C1}.
**Dispatchable now: E1 and C1** (disjoint crates).

## 5. Soundness and honesty constraints (binding on every wave-2 bead)

- **Soundness over completeness.** Every emitted typing / `sameAs` / `subPropertyOf` /
  inconsistency verdict must hold in every model of the input; residual incompleteness stays
  honestly pinned as divergences, never papered over. The CR6 reachability side-condition is
  load-bearing (merging on `X ⊑ {a}, Y ⊑ {a}` alone is unsound — see `classify.rs`).
- **Fail-closed skips.** Any assertion/key/facet shape outside a bead's exactly-specified
  fragment stays in `Report::skipped_axioms` — never guessed at.
- **Floors only rise with measured evidence.** A divergence pin is pruned only when the lane
  observes the case passing (the stale-entry assert enforces this); `EL_SUITE_FLOOR` moves
  only to the newly measured count. No aspirational floors.
- **Opt-in posture unchanged.** `abox` is default-off; feature-off builds must be
  byte-identical in behaviour (the default `--workspace` shards still link no EL code); the
  lean wasm posture and existing ratchet floors are untouched.
- **No committed performance numbers**; work-box timings in PR bodies only.

## 6. Relationship to open work

- **sq-qcnn.23** (coverage-floor seeding for `sparq-reason-el` + `sparq-reason-ql`) remains
  open under the test-quality epic; it touches `scripts/coverage.sh` + ratchet JSONs —
  file-disjoint from every wave-2 bead. Not duplicated here.
- The RIF (sq-pbz04.5), D-entailment (sq-pbz04.6), QL (sq-pbz04.3) and Direct-Semantics
  (sq-pbz04.4) workstreams are separate sub-epics, untouched by this record.
