# Foundational-Ontology choice for the sparq PKG — a benchmark design (design-for-review)

> 🤖 **SPARQ agent.** This is a research/design record produced for the maintainer to
> review. It does **not** ship code. It synthesises four research streams (landscape,
> empirical prior art, LLM-fluency, PKG-fit) into a runnable benchmark design that can
> answer: *does any foundational ontology (FO) beat gUFO — and a no-FO baseline — on the
> knowledge-management (KM) tasks this repo's Project-Knowledge-Graph (PKG) actually does?*

**Status:** design record (no implementation shipped). [OPUS-4.8]
**Bead:** sq-yqhj8 (epic sq-mztg8; under PKG epic sq-2m6zm).
**Honest prior (pre-registered):** FO choice is expected to be **NEUTRAL** for the PKG's
KM tasks. The design is built to be *able to conclude* "no FO robustly beats no-FO/gUFO",
and that is the most likely outcome on the current corpus. See §7.

---

## 0. Premise correction (verified against the code, not taken on faith)

Two claims that must be corrected before any FO comparison is scoped — both checked
against the actual repository this session:

1. **gUFO is NOT the shipped baseline.** The brief (and the epic title) treat gUFO as
   "the current baseline". It is not. `crates/sparq-kb/ontology/pkg/pkg.ttl` imports **no
   foundational/upper ontology at all** (`grep -ri gufo crates/sparq-kb` → nothing). The
   shipped PKG is **reuse-first over domain vocabularies**: each core class is given a
   parent to a *domain* vocabulary, not an FO —
   `pkg:Task ⊑ schema:Action`, `pkg:Finding ⊑ sig-impl:Assertion , skos:Concept`,
   `pkg:Source ⊑ fabio:Expression , dcat:CatalogRecord`, `pkg:Technique ⊑ skos:Concept`
   (verified in `pkg.ttl`; reuse table in `crates/sparq-kb/ontology/pkg/PROVENANCE.md`).
   So **the de-facto shipped baseline IS the "no-FO" arm** of this benchmark; gUFO is a
   *named candidate*, not the status quo. This makes the no-FO arm the incumbent, not a
   strawman — a point that materially shifts the burden of proof onto any FO.

2. **The PKG-dogfood harness is already on `main` and was already RUN.** The pkg-fit
   stream said `bench/pkg-dogfood/` is "not yet on main" — it is now
   (`bench/pkg-dogfood/{PREREG.md,run.py,grade.py,stats.py,tokens_real.py,tasks/tasks.jsonl}`,
   12 tasks). The N=30 cache-discounted 3-arm token A/B was executed and recorded a
   **positive** verdict for the PKG-answerable question class
   (`bench/pkg-dogfood/RESULTS.md`; status note at the top of
   `research/dogfooding-sparq-knowledge-graph.md`). That matters here because it means the
   harness this design extends is **proven and at the N=30 power floor already** — the FO
   benchmark inherits a working substrate, not a PoC.

Everything below is written against that corrected premise.

---

## 1. Problem framing

The PKG stores four entity classes — **Task**, **Finding**, **Source**, **Technique**
(verified instance scale in `crates/sparq-kb/ingest/pkg-instances.ttl`: Tasks dominate by
~100×, with a handful each of Findings/Sources/Techniques). Agents query it for "what
depends on X / status of Y / what's the provenance of finding Z / which sources are
unexplored", via 11 canned queries (`crates/sparq-kb/src/query/canned.rs`) and the
`pkg-query` introspect→ground→ask helper.

A *foundational* (upper) ontology imposes a top-level category structure
(endurant/perdurant, continuant/occurrent, proposition vs information-bearer, rigid-kind
vs anti-rigid-phase, …) above the domain vocabulary. The hypothesis under test (epic
sq-mztg8) is that adopting one — gUFO, BFO, DOLCE-DUL, SUMO, schema.org-as-top, or none —
**changes downstream KM-task performance**.

**The single lever an FO can pull.** The engine has an **opt-in `close` feature**
(`crates/sparq-kb/Cargo.toml`: `close = ["query", "dep:sparq-reason"]`; gated call at
`crates/sparq-kb/src/lib.rs:137`) that forward-chains RDFS/OWL-RL closure, materialising
`rdfs:subClassOf`/`subPropertyOf` entailments and the `pkg:dependsOn` ⇄ `pkg:blockedBy`
`owl:inverseOf` pair. **An FO can only pay off through this feature** — by injecting
entailed `rdf:type`/superclass triples (or inverse/role triples) that the no-FO graph
lacks, on queries that traverse the new hierarchy. If a query reads `pkg:`-direct terms
that are already asserted on every instance, an FO superclass adds zero rows. This is the
crux of the neutral prior (§7).

---

## 2. Candidate FO landscape — ranked comparison

Ranked by a four-factor product: **formal fit × LLM-fluency × reasoning value × tooling**
for *this* (tasks + findings + sources) agentic KG. Sources are cited inline; sizes for
gUFO/BFO were measured by fetching the artifacts (landscape stream).

| FO | Formal fit (this domain) | LLM-fluency (training-data prevalence) | Reasoning value here | Tooling / loadable OWL-DL | Verdict |
|---|---|---|---|---|---|
| **schema.org-as-top** | Low metaphysics: `Thing`→`CreativeWork`/`Action`/`Event`/… No endurant/perdurant or continuant/occurrent axis. Already partly in the PKG (`schema:Action`). | **Highest.** Massively over-represented (JSON-LD web, Google guidance); agents emit/ground it fluently. | Low: not OWL-2-DL (can't cleanly partition object/data props, no disjointness — Patel-Schneider; Harmse). RDFS-soft, ~no consistency checking. | Best tooling familiarity; trivially loadable; weak as a reasoning layer. | **Pragmatic baseline-adjacent.** Best fluency, near-zero reasoning guarantee. |
| **gUFO** (named baseline) | **Strong.** OWL-safe cut of UFO. Type taxonomy (`Kind`/`Phase`/`Role`/`Category`) maps onto status-as-phase, finding-as-reified-claim. But no built-in doc/task/source vocab. | **Lowest.** Newest + rarest of the set; NEMO/UFES research community; distinctive type-of-types machinery least-represented in training data. | **High in principle, low in practice here:** its rigid/anti-rigid axis is what the repo's own KGE ablation slice exercises — but that benefits the *reasoner*, not the LLM. | Decidable OWL 2 DL; single ~self-contained TTL; OntoUML/Tonto transform tooling. | **Best formal fit of the rich options; worst LLM-fluency.** The thing to beat. |
| **BFO** (ISO/IEC 21838-2) | Moderate: realist continuant/occurrent. Awkward for *claims/findings* (information artifacts) — needs IAO bolted on; no doc/task vocab. | Third (cautionary): the most LLM-tooling activity (Beverley et al.), WordNet-distant; "My Ontologist" shows GPT-4 needs heavy scaffolding + a GPT-4o regression. | Moderate: clean continuant/occurrent + GDC/bearer split, but you still bolt PROV/DCAT/IAO on top. | Most mature/standardised (OBO Foundry 650+, DoD/IC); `bfo-core` is OWL 2 DL (above EL — inverse+transitive part-of). | **Only if external OBO/DoD interop is a hard requirement** (it is not for a self-contained PKG). |
| **DOLCE-DUL** | **Best out-of-the-box vocab coverage:** native `dul:Task`, `dul:Plan`, `dul:Role`, `dul:InformationObject`, `dul:Description`/`Situation`, `dul:Method`. Directly analogous to the four PKG classes. | Fourth: comparable rarity to BFO, *less* dedicated LLM tooling; no reliable zero-shot DOLCE grounding study found. | Moderate-high: the IO/realization split + DnS reification fit findings/sources cleanly. | Decidable OWL 2; `DUL.owl` + modular patterns; "DOLCE for the Semantic Web". | **Strongest practical upgrade IF an FO is wanted** — most vocab for free; cost is a large pattern framework that overlaps existing PROV/reification. |
| **SUMO** | Broad term coverage but value lives in **SUO-KIF FOL axioms that do not survive the OWL export** ("provisional + lossy"). | Second (distant): larger footprint than BFO/DOLCE; uniquely mapped to the *entire* WordNet lexicon (the most LLM-familiar lexical resource) — but KIF syntax is rare. | Low: the loadable OWL shell is the lossy part; ~20k terms add noise; no clean tasks/findings subset. | OWL export exists but lossy; KIF authoritative; not decidable as KIF. | **Reject** for a reasoning layer; the WordNet bridge is its only edge for NL grounding. |
| **Full UFO** | n/a — a modal/HOL theory realised via OntoUML, not a loadable OWL artifact. | n/a | n/a | **Its OWL form IS gUFO.** | **Not a runnable candidate** — use gUFO. |
| **DOLCE / DOLCE-Lite (full)** | Descriptive stance fits human-conceptualised project knowledge; full DOLCE is FOL, DOLCE-Lite is the weakened OWL cut. | Like BFO/DOLCE (rare). | Moderate but lacks source/task vocab alone. | DOLCE-Lite.owl available; usually consumed via DUL. | **Subsumed by DUL** for this use — benchmark DUL, not bare DOLCE-Lite. |
| **no-FO (the SHIPPED baseline)** | Author the exact profile you need; current PKG is OWL-2-DL-safe (only `owl:inverseOf`/`SymmetricProperty`/subclass/subproperty). | **Highest indirectly** — you reuse the vocabularies (`schema`/`prov`/`skos`/`dcat`/`fabio`/`cito`) the model already knows. | Whatever you author; today exactly the inverse + subclass closure the canned queries need. | Minimal (`pkg.ttl` ~16 KB, ~9 classes); fully under your control; SHACL-gated. | **Incumbent / default.** Lowest cost, mainstream Linked-Data practice, matches the maintainer's `sec-prop` "mint almost no net-new terms" discipline. |

**Landscape conclusions.**

- **Decidability cliff:** gUFO, BFO-core, DOLCE-Lite, DUL are loadable decidable OWL 2 DL.
  SUMO (lossy HOL export) and full UFO (theory; OWL form = gUFO) are **not** usable
  OWL-DL artifacts. schema.org is **not OWL 2 DL at all**. The no-FO baseline is whatever
  you author (currently DL-safe).
- **Vocab coverage for THIS domain (most-for-free → least):** DUL > schema.org > gUFO >
  BFO/DOLCE/SUMO/UFO (general categories only — you bolt PROV/DCAT on regardless).
- **LLM-fluency follows a prevalence law, not formal quality** (LLMs4OL gradient: WordNet
  ~0.99 F1 vs DBpedia-ontology / Gene-Ontology far lower; ID-prevalence predicts mapping
  accuracy — `arXiv:2409.13746`, `arXiv:2409.10146`). schema.org/no-FO sit at the
  fluent end; gUFO/DOLCE/BFO/SUMO at the rare end.

---

## 3. Shortlist — what to benchmark

Pre-registered shortlist of **FOs to instrument**, plus the two **mandatory baselines**:

| Arm | Why it's in | Role |
|---|---|---|
| **no-FO** | The actual shipped baseline (§0). | **MANDATORY baseline (incumbent).** |
| **gUFO** | The named candidate the epic asks to beat; best formal fit of the rich options; already the synthetic-slice the KGE ablation uses. | **MANDATORY baseline (named).** |
| **DOLCE-DUL** | Best out-of-the-box task/finding/source/method vocab; strongest practical upgrade if an FO helps at all. | Candidate #1. |
| **schema.org-as-top** | Highest LLM-fluency, already partly in the PKG; the "fluent but metaphysics-free" pole — isolates whether *any* top-level typing (vs none) matters. | Candidate #2. |
| **BFO** *(stretch — include only if budget allows)* | The standardised/most-tooled FO; tests whether realist continuant/occurrent + IAO buys anything; the only one with external-interop value. | Candidate #3 (optional). |

**Excluded with reasons (non-sycophantic):** **SUMO** (the value is in FOL axioms the OWL
export drops — benchmarking the lossy shell would mislead); **full UFO** (no loadable OWL
artifact — its OWL form is gUFO, already in); **bare DOLCE-Lite** (subsumed by DUL).

So the benchmark runs **4 arms minimum (no-FO, gUFO, DOLCE-DUL, schema.org-as-top)**, +BFO
as an optional fifth. This satisfies "2-4 FOs + mandatory gUFO + no-FO".

---

## 4. PKG-typing-per-FO mapping (the concrete instrument)

Each arm is produced by **adding an FO-overlay TTL** that types the four PKG classes under
the FO's top categories (the domain-vocab parents in `pkg.ttl` stay; the overlay adds a
*foundational* superclass + any FO-native role/phase axioms). Verified mappings:

| pkg class (asserted parent today) | **no-FO** (status quo) | **gUFO** | **BFO 2.0** | **DOLCE-DUL** | **schema.org-as-top** |
|---|---|---|---|---|---|
| `pkg:Task` (`⊑ schema:Action`) | (none added) | `gufo:Event` (perdurant work-occurrence); open/closed `status` as a `gufo:Phase` | `bfo:Process` (occurrent); `realizes` a plan-spec | `dul:Action ⊑ dul:Event`; the plan-`dul:Task` it executes | `schema:Action` (already there) |
| `pkg:Finding` (`⊑ sig-impl:Assertion , skos:Concept`) | (none) | `gufo:Proposition` (abstract truth-bearer); the *finding act* = `gufo:Event` | IAO `information content entity ⊑ bfo:GenericallyDependentContinuant`, `is-about` an entity | `dul:Description` / `dul:InformationObject` that `expresses` a `dul:Concept` | `schema:Claim` / `schema:CreativeWork` |
| `pkg:Source` (`⊑ fabio:Expression , dcat:CatalogRecord`) | (none) | `gufo:Object` carrying a `gufo:Proposition` (info artifact = endurant + content) | IAO ICE (content, GDC) **borne by** a `bfo:material entity` (the file) — clean content/bearer split | `dul:InformationObject` (abstract) realised by `dul:InformationRealization` (the file) | `schema:CreativeWork` / `schema:DigitalDocument` |
| `pkg:Technique` (`⊑ skos:Concept`) | (none) | `gufo:Type` (a kind) or abstract method-spec | IAO `directive information entity` (a plan/algorithm spec) | `dul:Method ⊑ dul:Description` | `schema:HowTo` / `schema:SoftwareSourceCode` |

**The one categorial distinction that matters for this corpus** (drawn by every FO except
schema.org, NOT by the status quo): the **occurrent/perdurant (Task) vs
continuant/information-artifact (Finding, Source, Technique)** split, plus the
**proposition (Finding) vs information-bearer (Source) vs descriptive-content (Technique)**
split. gUFO additionally captures **status as an anti-rigid phase** (Open→InProgress→Closed
of a rigid Task; Unexplored→Explored of a Source) — exactly the rigid-kind-on-nobody /
anti-rigid-phase-asserted-directly structure the KGE synthetic slice is built on
(`crates/sparq-vectors/src/eval.rs` `synthetic_gufo_ttl_sized`:
`ex:Person a gufo:Kind` asserted on nobody, `ex:Student a gufo:Role`, `ex:Child a gufo:Phase`
asserted directly, closure must derive the kind).

**Construction discipline.** The overlay TTL is checked into the benchmark fixture per arm;
the gold answer for each task is computed **twice** — once over the plain graph, once over
`--close owl-rl` with the overlay loaded — and a task only counts as an "FO win" when the
no-FO arm genuinely **cannot** answer it without manual per-class enumeration or query
rewriting (§5, ER1).

---

## 5. The KM benchmark task set (tasks that EXPLOIT foundational typing)

These are **new** tasks, distinct from the 12 direct-`pkg:`-lookup tasks in
`bench/pkg-dogfood/tasks/tasks.jsonl` (none of which benefit from an FO — §7). They slot
into the same harness and are stratified into three families, each constructed so the no-FO
arm returns fewer/no rows while an FO-typed arm under `--close owl-rl` answers via
entailment.

**TH — type-hierarchy queries (need entailed `rdf:type` up the FO tree):**

- **TH1 — "List every *information artifact* in the KB."** no-FO has no common superclass
  over Source/Finding/Technique → must hand-enumerate three classes (and silently miss a
  *future* artifact class). FO: `?x a fo:InformationContentEntity` (BFO/IAO) /
  `gufo:Proposition` / `dul:InformationObject` returns the union via closure. **The
  canonical FO win: open-world extensibility.**
- **TH2 — "List every occurrent/process the project tracks (vs every continuant)."** no-FO:
  the partition is un-queryable (nothing says Task is process-y). FO: `?x a bfo:Process` /
  `gufo:Event` ⇒ Tasks; `bfo:Continuant` ⇒ the rest.
- **TH3 — "Which entities are truth-bearers (propositions) vs information-bearers
  (documents)?"** Distinguishes Finding from Source — collapsed in the status quo.

**ER — entailment/reasoning-dependent lookups (answer exists only after closure):**

- **ER1 — "What is blocked-by task X?"** Query `?d pkg:blockedBy <X>` **directly** (not the
  `dependsOn`-inverse rewrite the canned `task-blocks` query uses). no-FO **without**
  closure: 0 rows (only `dependsOn` is asserted). FO/closure: `owl:inverseOf` materialises
  them. **Isolates the reasoning contribution cleanly** — and the existing canned query's
  hand-rewrite to `dependsOn` is itself proof of the point (without inverse entailment you
  must rewrite the query).
- **ER2 — "Find all Findings *about* anything classified as a Technique, transitively."**
  Needs `dcterms:subject` chained with `subClassOf` closure over the subject's FO type.
- **ER3 — "Which Tasks implement a method that supersedes a deprecated Technique?"** A
  3-hop cross-property lookup (`Task →implements→ Technique →dcterms:replaces→ Technique`).

**CC — cross-category queries (the FO's whole point — relate across the top split):**

- **CC1 — "For each information artifact, what process produced it and what agent enacted
  that process?"** A PROV-style join that type-checks cleanly only when continuants
  (Source/Finding) + occurrent (Task) + PR-activity sit in one FO frame. no-FO: three
  unrelated vocab silos (fabio / sig-impl / schema:Action) with no top join.
- **CC2 — "Rank entities by epistemic status across categories."** Unify `pkg:assurance`
  (Finding), `pkg:exploredStatus` (Source), `pkg:status` (Task) as phases of a common FO
  state-type (`gufo:Phase` / `dul:Phase`) so one query orders "how settled is each thing"
  regardless of class. no-FO: three incompatible enums, three queries.
- **CC3 — "Show the lifecycle phases of any entity"** (gUFO-specific anti-rigid phase
  query: `?x a/⊑ gufo:Phase` ⇒ Open/Closed, Unexplored/Explored, assurance levels
  uniformly).

**Power requirement.** The existing PREREG declares N=12 underpowered by construction and
sets the bar at **≥30 tasks + Wilcoxon p<0.05 + ≥20% paired-median reduction + bootstrap
95% CI lower-bound > 0** (`bench/pkg-dogfood/PREREG.md` §"Kill criteria"). The FO benchmark
must add the **TH/ER/CC strata to reach ≥30 FO-exercising tasks** before any
`recommend_adopt = true` for an FO arm is even reachable.

### 5.1 The two metrics — split by where they can run

#### Metric 1 — agent NL→tool/pkg-query accuracy + real token cost (RUNNABLE ON THIS BOX)

**Machinery (already on `main`):** `bench/pkg-dogfood/` —
`run.py` + `tokens.py` (char/token-proxy read-payload A/B, no API key needed;
cache-discounted `effective_input = 1.0·fresh + 0.1·cache_read + 1.25·cache_write`),
`tokens_real.py` (mines real `message.usage` from sub-agent transcripts tagged
`[ABM task=<id> arm=<…>]`), `grade.py` (answer correctness: `must_include` substrings +
`row_count`), `stats.py` (Wilcoxon signed-rank + bootstrap CI on the paired median delta).

**Arms per TH/ER/CC task:** **Arm-noFO** (`pkg-query` on the plain graph — must enumerate
classes / returns empty-or-wrong) vs **Arm-FO** (`pkg-query --close owl-rl` over the
FO-typed graph — single typed query), one FO overlay per benchmarked FO. Two registerable
outcomes: **(i) accuracy** (right row-set?) and **(ii) token cost** (does FO typing let a
*shorter* query + *fewer* rows answer it, or does closure inflate the read payload?).

**Honesty posture (carried verbatim from PREREG):** the `close` feature's closure-build
cost (CPU/wall) is **non-canonical and must NOT be charged as a token cost** — only the
durable artifact-size delta + read-payload, per the existing `ingest_build_chars()`
convention. The GOLD full-session A/B needs ≥2 isolated Claude Code sessions per task +
`ANTHROPIC_API_KEY` for the exact `count_tokens` tokenizer — **neither is in this env** —
so on this box Metric 1 runs at char-proxy fidelity always, real-`usage` fidelity if a
workflow fan-out generates transcripts. Both are **NON-CANONICAL** and never frozen into
committed markdown (the no-perf-numbers gate; the numbers live in `bench/pkg-dogfood/`).

#### Metric 2 — KGE closure-prior MRR (NEEDS A CANONICAL / EC2 BOX)

**Machinery:** `crates/sparq-vectors/src/eval.rs` (`kge` feature) —
`run_ablation_multiseed_paired` returns per-axis `PairedDelta { mean, std, se, n,
significant_at(k) }` (mean − k·SE > 0), driven by `examples/kge_ablation.rs`. The
ablation is `{closure on/off} × {type-constrained negatives on/off}` × seeds, with an
`AblationCell::gufo_prior` axis that is **declared but currently a no-op stub** (verified:
`gufo_prior` is `false` in every cell — `eval.rs:542`, asserted at `eval.rs:1222`).
**Wiring that axis live per-FO is exactly what this benchmark would do.**

**What it measures:** does materialising the FO closure (+ FO-typed negatives) **before**
vectorising firm up link-prediction MRR/Hits@k? Uses the **paired per-seed delta**
`Δ_s = MRR(closure ON) − MRR(closure OFF)` (common-random-numbers across the 4 cells →
shared variance cancels — the variance-reduction fix in the KGE history). Honest input
graph is `synthetic_gufo_ttl_sized(n, density, seed)` (rigid `Person` kind asserted on
nobody; anti-rigid Student/Child phases asserted directly; decoys preserved — anti-overfit).

**Why it needs a canonical/EC2 box:** `examples/kge_ablation.rs` self-documents as
INDICATIVE-only NON-CANONICAL on a work box; a firm verdict needs many seeds × hundreds of
epochs × an LR sweep × a **real schema-bearing typed KG** (`SPARQ_KGE_DATASET=…`). That is
the self-terminating EC2 bench-instance job (per the EC2-benchmark + work-box-non-canonical
charter rules), **not this box**.

---

## 6. Recommendation

1. **Default to the no-FO incumbent.** It is the shipped baseline, lowest-cost, fully
   under your control, SHACL-gated, DL-safe, and reuses the vocabularies LLMs already know.
   The empirical prior (§7) says it is unlikely to be beaten on the PKG's KM tasks.
2. **If an FO ever proves its keep, prefer DOLCE-DUL** (most task/finding/source/method
   vocab for free), with **gUFO** the choice if OntoUML/UFO-tooling alignment matters and
   **BFO** only if external OBO/DoD interop becomes a hard requirement. **Reject SUMO**
   (lossy OWL export) and don't benchmark full UFO (= gUFO) or bare DOLCE-Lite (= DUL).
3. **Put FO value where it empirically pays — the symbolic layer, not the LLM prompt.**
   Across the evidence, ontology info helps as a **post-hoc reasoner/validator** (closure +
   SHACL + validate/repair — `arXiv:2405.11706`), not as a typing discipline injected into
   the model (which the model grounds to unreliably and which costs tokens — `arXiv:2410.09244`
   found only the *classes* schema segment helped; ranges/property detail did not). The
   repo already holds this thesis: "an effective schema mined from instance data beats the
   declared ontology for NL→SPARQL grounding" (`research/genai-ontology-introspection.md:17`).
4. **Run the benchmark anyway** — but pre-registered to be able to return the null. The
   value of running it is a *defensible* "no FO robustly beats no-FO/gUFO on our KM tasks"
   verdict (a result the literature does not yet contain — see §7), not a presumed FO win.

---

## 7. Pre-registered honesty: the prior is NEUTRAL, and why

This benchmark **must be powered to fail to reject** "no FO measurably helps the PKG's KM
tasks", and report that verdict if true. The convergent reasons:

- **The current query surface can't exploit an FO.** All 12 PREREG tasks + 11 canned
  queries are **direct `pkg:`-term lookups** (verified): t01–t04/t09/t10 match
  `?f a pkg:Finding` / `?source a pkg:Source` directly (an upper superclass adds zero rows
  since the `pkg:` type is already asserted on every instance); t05/t06/t08/t11 traverse
  asserted `pkg:dependsOn`; t07 (`task-blocks`) **already hand-rewrites** the inverse to
  `dependsOn` (sidestepping the one place entailment would help); t12 (ready-frontier) is
  asserted-data aggregation whose real gap is **staleness vs live bd — which no ontology
  fixes**; and `schema-classes`/`schema-properties` would get *noisier* under closure
  (entailed superclasses inflate the class list — a small regression for agent grounding).
  Any FO benefit must come from the **new** TH/ER/CC families (§5), which the current corpus
  deliberately avoids.
- **The PKG resembles the schema-free regime where the closure prior was a no-op.** The
  repo's own KGE ablation (verdict on `origin/main` against sq-0wo9e.9;
  `research/dogfooding-sparq-knowledge-graph.md` §3.2) found structure priors are **no-ops
  on plain triple sets** and fire only on the gUFO/ontology-rich slice — but there the
  **per-seed variance was on the order of the mean** (the effect was *not statistically
  firm*), and type-constrained negatives **depress** the gUFO result (REJECTED, default off).
  The brief's "sign-unstable / not robustly positive" framing is consistent with this
  recorded verdict. Metric 2's pre-registered expectation is therefore **the null**: a
  gUFO/FO closure prior is adopted **only if** `PairedDelta.significant_at(k)` clears on a
  **schema-bearing real KG** across seeds.
- **The empirical literature does not show FO *choice* moving a downstream metric.** The
  big, repeatable effect is *structuring retrieval with a (domain) ontology vs nothing*
  (OG-RAG +large; schema-linking gains — `arXiv:2412.15235`, `arXiv:2508.01815`). The one
  clean *source*-comparison (`arXiv:2511.05991`, N=20) found ontology source **neutral** on
  RAG; size-controlled work (PLOS 2011) found **richer ≠ better**; manual upper-ontology
  classification is **unreliable even for BFO experts** (`arXiv:1810.05093`); and in LLM
  contexts **verbose FO detail hurts unless filtered** (`arXiv:2507.22619`). **No controlled
  benchmark fixes task+data and swaps BFO↔DOLCE↔SUMO↔gist↔none with a reported effect
  size** — running ours would be novel, with the null as the most likely result.
- **LLM-fluency argues against agent-facing FO typing.** Expected fluency tracks
  training-data prevalence, not formal quality (§2). gUFO/DOLCE/BFO/SUMO sit at the rare
  end; injecting them into the agent's grounding surface adds tokens + a vocabulary the
  model mis-types confidently (poisoning downstream queries) for a payoff that accrues to
  the reasoner/human-interop, not the LLM's task accuracy.

### 7.1 Pre-registered KILL-CRITERIA (frozen before any run)

Inherited from `bench/pkg-dogfood/PREREG.md` and specialised to the FO question. An FO arm
is declared a winner over the relevant baseline **only if it clears ALL** of its metric's
criteria; otherwise the verdict is **"no FO robustly beats no-FO/gUFO"**.

- **KILL-A (underpowered):** fewer than **≥30** FO-exercising (TH/ER/CC) tasks → no
  `recommend_adopt` is reachable; report underpowered.
- **KILL-B (Metric 1 — accuracy):** an FO arm whose paired answer-correctness is **not >**
  the no-FO arm (and gUFO arm) at Wilcoxon **p < 0.05** with a bootstrap 95% CI lower-bound
  **> 0** → that FO does not win on accuracy.
- **KILL-C (Metric 1 — token cost):** an FO arm whose paired-median **effective-input
  reduction < 20%**, OR p ≥ 0.05, OR the bootstrap median-delta CI **includes 0**, OR the
  saving is entirely closure-build cost (non-canonical, not chargeable) → no token win.
  (Closure that *inflates* the read payload is a token *loss* — explicitly a kill.)
- **KILL-D (Metric 2 — MRR):** an FO closure/typed-negative prior whose
  `PairedDelta.significant_at(k)` does **not** clear on a **schema-bearing real KG** under
  the asymmetric ComplEx model across seeds (sign-unstable / SD ≳ mean) → no robust MRR
  lift; report the null (consistent with the existing KGE verdict).
- **KILL-E (cost-benefit):** even if an FO clears B–D, if its modelling/maintenance cost
  (overlay authorship + the LLM-mis-typing risk + the verbose-schema token tax) exceeds the
  measured benefit on the PKG's actual question mix → **do not adopt** (recommend keeping
  no-FO; capture the FO as a documented option, not a migration).
- **Global stop:** if **all** FO arms fail B–D, the pre-registered conclusion is
  **"no FO robustly beats the no-FO/gUFO baselines on this repo's KM tasks"** — and that is
  a publishable, honest result, not a failure of the benchmark.

---

## 8. Phased plan (each phase = a future bead under epic sq-mztg8)

1. **Author the per-arm FO overlay TTLs** (no-FO is null; gUFO, DOLCE-DUL, schema.org-as-top,
   +BFO optional), each typing the four PKG classes per §4, checked into the benchmark
   fixture. Verify each overlay loads + closes under `--close owl-rl` and stays OWL-2-DL.
2. **Add the TH/ER/CC task strata to the harness** to reach **≥30 FO-exercising tasks**,
   each shipping a gold row-set computed two ways (plain vs `--close owl-rl` per arm), with
   `must_include`/`row_count` graders (`grade.py`) — and confirm each is a genuine FO-win
   construction (no-FO cannot answer without hand-enumeration/rewrite).
3. **Run Metric 1 on this box** (char-proxy always; real-`usage` if a workflow fan-out
   produces transcripts), per FO arm vs no-FO/gUFO; apply KILL-A/B/C; record results in
   `bench/pkg-dogfood/` (never frozen into markdown).
4. **Wire the `gufo_prior` / FO-prior axis live in `eval.rs`** (replace the no-op stub) so
   `run_ablation_multiseed_paired` can ablate FO-typed closure + FO-typed negatives per FO.
5. **Run Metric 2 on a canonical/EC2 self-terminating bench instance** over a real
   schema-bearing typed KG (many seeds × LR sweep × asymmetric ComplEx); apply KILL-D;
   record `PairedDelta`s in `bench/`.
6. **Synthesise the verdict** — rank the FO arms by the four KILL gates; either recommend
   an FO (only if it clears B–E vs both baselines) or record the pre-registered null. If
   null, graduate this design record's recommendation into the PKG architecture note and
   close the epic with "no-FO retained, FO options documented".

---

## 9. Open questions that genuinely need the maintainer

1. **Is external interop a real requirement?** The only strong reason to take BFO is
   OBO-Foundry/DoD-IC interop. If the PKG stays self-contained, BFO can be dropped from the
   shortlist (down to 3 arms). Maintainer call.
2. **Acceptable FO-overlay maintenance cost?** Authoring + keeping 3–4 overlay TTLs current
   against `pkg.ttl` is ongoing cost; is that justified to *measure* a likely-null result,
   or is a smaller 3-arm (no-FO, gUFO, DOLCE-DUL) run sufficient?
3. **Does the PKG's *future* query mix warrant the TH/ER/CC tasks at all?** These tasks are
   synthetic constructions to give an FO something to do. If the agent workflow will never
   ask "list every information artifact / rank by epistemic status across categories", the
   honest answer may be to skip the benchmark and record the neutral prior as the decision.
4. **gUFO self-assessment.** The maintainer's prior "could be crap" self-assessments have
   been unreliable (per the vendored ZKP-SPARQL ontology note) — gUFO's formal fit here is
   genuinely the best of the rich options; confirm before discounting it.

---

## 10. Sources

Landscape: gUFO <https://nemo-ufes.github.io/gufo/>; BFO/ISO 21838-2
<https://github.com/BFO-ontology/BFO-2020>; DOLCE/DUL <http://www.ontologydesignpatterns.org/ont/dul/DUL.owl>;
SUMO <https://github.com/ontologyportal/sumo>; schema.org-vs-OWL (Patel-Schneider
"Analyzing Schema.org"; Harmse). Empirical: ontology-source-neutral-on-RAG
<https://arxiv.org/abs/2511.05991>; OG-RAG <https://arxiv.org/abs/2412.15235>; bigger≠better
<https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1001055>;
manual-FO-classification-unreliable <https://arxiv.org/pdf/1810.05093>; Text2KGBench
<https://arxiv.org/abs/2308.02357>; verbose-schema-hurts <https://arxiv.org/pdf/2507.22619>.
LLM-fluency: LLMs4OL <https://arxiv.org/html/2409.10146v1>; ID-prevalence→accuracy
<https://arxiv.org/pdf/2409.13746>; My-Ontologist(BFO) <https://arxiv.org/abs/2407.17657>;
progressive-ontology ablation <https://arxiv.org/pdf/2410.09244>; validate+repair
<https://arxiv.org/html/2405.11706v1>. In-repo: `crates/sparq-kb/ontology/pkg/pkg.ttl`;
`crates/sparq-kb/ontology/pkg/PROVENANCE.md`; `bench/pkg-dogfood/` (PREREG + RESULTS +
harness); `crates/sparq-vectors/src/eval.rs` (`run_ablation_multiseed_paired`, `PairedDelta`,
`synthetic_gufo_ttl_sized`); `crates/sparq-vectors/examples/kge_ablation.rs`;
`research/dogfooding-sparq-knowledge-graph.md`; `research/genai-ontology-introspection.md`.
