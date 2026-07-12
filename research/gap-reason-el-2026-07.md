<!-- [FABLE-5] sq-hmd7l.8 — OWL 2 EL classification competitor gap record.
Harness description + first-read methodology for the sparq-reason-el vs ELK same-box
comparison on real ontologies. No fabricated numbers; NO hard-coded performance numbers
presented as canonical. Every timing row this harness collects is flagged NON-canonical
(canonical:false) until a dedicated quiet EC2 run re-measures it. -->

# OWL 2 EL classification competitor gap — 2026-07

**Status:** harness record / first-read methodology description (NON-canonical).
**Date:** 2026-07-07.
**Bead:** sq-hmd7l.8.
**Epic:** sq-hmd7l (comparative-benchmarking-everything).
**Competitor:** `elk` — ELK, the reference consequence-based OWL 2 EL classifier
(Apache-2.0; `bench/competitors.json`, suite `reason-el-real`).
**Canonical run:** deferred to a dedicated quiet-box wave (`quiet_box_sensitive = true`).

---

## 0. Prior state

`sparq-reason-el`'s only bench was `examples/snomed_go_scale_bench.rs` — a **synthetic**,
in-process, closed-form scaling probe (a no-hidden-quadratic doubling-ratio assertion on a
generated SNOMED/GO-shaped slice). It never touches a real ontology and has no competitor
column. `bench/benchmarks.toml` carried a `reason-el-real` registry stub (from sq-hmd7l.1)
with every field `TBD`. This bead delivers the durable comparison harness + a real-ontology
classification example + a pinned hermetic acceptance fixture, and fills the stub in.

---

## 1. Metric — the subsumption-count ORACLE

The comparison metric is **proper named subsumption pairs**: the *complete transitive
closure* of `C ⊑ D` over **named** classes, with `C ≠ D`, `D ≠ owl:Thing`, and both
endpoints named IRIs (anonymous restriction classes and `⊤`/`⊥` excluded). It is the
classification's observable content that both engines can produce and that is invariant to
internal representation.

- **sparq** reports it directly: `examples/reason_el_real_bench` runs `classify_graph`
  (which materialises the *complete* closure as `rdfs:subClassOf` triples) and counts the
  distinct named-endpoint `subClassOf` pairs.
- **ELK** emits an inferred **direct** taxonomy; the harness transitively closes ELK's
  named `subClassOf` edges in the count step so both engines are compared on the *same*
  closure notion.

**ORACLE-BEFORE-TIMING (the sq-hmd7l.8 invariant).** For every ontology both counts are
recorded and cross-checked **before** any timing row is emitted; a timing row is emitted
only when a count is present. **Neither engine is ground truth** — a disagreement is
recorded as `counts_agree=false` and investigated, **never adjusted** to match. A missing
count (engine error/absent) records `counts_agree=n/a`, and the timing carries no agreement
flag. This is enforced structurally in `scripts/bench/reason-el-same-box.sh` (§3) and in the
envelope's `oracle_before_timing` block.

---

## 2. Corpora

Real ontologies are **gather-only** (fetched to `/tmp`, NOT committed — large corpora +
engines stay out of git per AGENTS.md). Both are freely licensed so the numbers are
publishable:

- **Gene Ontology** (`go-basic.owl`, CC-BY 4.0) — the canonical large biomedical EL
  ontology. `GO_OWL_URL` overridable; the release SHA-256 is recorded into each envelope at
  gather time (`ontology_sha256`).
- **OpenGALEN** (OpenGALEN 8, GALEN open-source licence) — a large clinical-terminology EL
  ontology with deep part-of role structure.
- **SNOMED CT is LICENSE-GATED — skipped** (free subsets only, if any).
- **ORE 2014/2015 EL track** — a stretch goal, not implemented here.

Each OWL is converted to N-Triples offline via Apache Jena `riot` at gather time
(`Classifier::classify` / `classify_graph` consume RDF triples). The sparq leg is built
`--features rbox` so real part-of role chains (GO's `part_of` transitivity, SNOMED-style
right-identity) are classified faithfully; the metric is unchanged in feature-OFF, `rbox`
only *adds* the edges the ontology's role axioms entail.

---

## 3. Harness (`scripts/bench/reason-el-same-box.sh`)

Built on the `scripts/bench/shacl-same-box.sh` template; emits one competitor-results-shaped
envelope JSON per ontology.

- **`--smoke` (ONLY=sparq)** — the fast, hermetic **acceptance path**: build + run
  `examples/reason_el_real_bench --smoke` on the small **vendored** fixture
  `crates/sparq-reason-el/examples/data/el_smoke.ttl`, asserting its pinned subsumption
  count. No network, no JVM, no downloads. This is the CI-runnable gate.
- **Full mode** — for each ontology: download OWL → `riot` OWL→NT → sparq classify (count +
  time) and ELK classify (taxonomy → transitively-closed count + time) → record both counts
  + agreement → emit envelope. ELK/Jena/OWL are gather-only `/tmp` deps. An engine failure
  or timeout degrades to an honest ERROR row, never a fabricated number.

### 3.1 The vendored acceptance fixture

`examples/data/el_smoke.ttl` is a **hand-built, hermetic EL+⊥ fixture** shaped like a
biomedical anatomy sub-hierarchy — it is **NOT** a slice of GO/OpenGALEN (those are fetched
gather-only). It exists so the acceptance path has a closed-form, hand-verified count to
pin. It deliberately includes a **CR4 existential subsumption OWL 2 RL cannot reach**
(`Neuron ⊑ NucleatedCell`, derived only through `∃hasPart.Nucleus`), so the pinned count is
**non-vacuous**: a consequence-based-rule regression drops the count 9→8 and fails loudly.
Pinned oracle (hand-derived in `examples/data/el_smoke.ttl` + the example's constants):
**6 lattice classes, 9 proper named subsumptions** (5 told + 4 derived, incl. the CR4 edge).
Using only is-a + role-*equality* existentials (no chains), the closure is identical with
or without `rbox`, so the pinned assertion holds in **both** feature states.

---

## 4. Canonicity

**Every** row this harness collects on the shared work box is **NON-canonical**
(`canonical:false` in the envelope, always). Work-box timings are directional only — do NOT
bake them into docs/dashboards. The harness is the durable deliverable; a future
`CANONICAL=1` run on a dedicated quiet EC2 box produces citable numbers. No wall-clock
number is pinned or asserted anywhere in this bead — only the dimensionless subsumption
**count** is an oracle.

---

## 5. Follow-ups

- Canonical quiet-box gather (GO + OpenGALEN, both engines) → publish the first
  sparq-vs-ELK subsumption-count agreement + timing envelope.
- ORE 2014/2015 EL-track corpus columns (stretch).
- If a real ontology's sparq/ELK counts **disagree**, investigate the cause (fragment
  coverage: a construct sparq's extractor skips — `skipped_axioms > 0` is the honest signal —
  vs an ELK-specific normalisation) and file a bead; never adjust either count to match.
