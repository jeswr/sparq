<!-- [OPUS-4.8] Design record (provisional). Authored under the empirical-honesty mandate:
     every load-bearing literature claim is cited; provable properties and benchmarked-only
     properties are kept in SEPARATE columns; no benchmark numbers appear (none exist yet);
     no ZK/MPC privacy or soundness claim is made. Graduate per AGENTS.md when shipped. -->

# Structure-aware vectorisation and flexible grounding for sparq

**Status:** design record, 2026-06-20. Validated against an adversarial literature-grounding
review (verdict: ARM with must-fix edits — all applied here). This document is the plan of
record; the cited reports and crate sources are the references. **No benchmark numbers appear
anywhere in this document — none exist yet; every empirical claim is gated on a measurement
that has not been run.**

This sits alongside the existing GenAI design records — [`genai-design.md`](genai-design.md)
(the four-crate plan: `sparq-sim`, `sparq-introspect`, `sparq-nlq`, `sparq-vectors`),
[`genai-kg-embeddings-vectorindex.md`](genai-kg-embeddings-vectorindex.md) (the storage +
compute layer), and [`genai-ontology-introspection.md`](genai-ontology-introspection.md) — and
extends them in one specific direction: **using the schema and ontology sparq already holds as
priors over the embedding manifold, not as post-hoc filters**, and making the resulting vector a
**structured object** that can be *grounded* back into whatever modality a query or tool needs.

---

## 0. The maintainer's request (verbatim)

This design answers four questions, posed in sequence. They are reproduced verbatim so the
design can be checked against what was actually asked.

**Message 1 — the structural-vectorisation question.**

> "How would you approach vectorising a knowledge graph in a way that takes account of
> its structural and ontological properties — e.g. that some predicate ranges are enumerations,
> some are integers, some have a declared domain/range, there is a subclass hierarchy, etc.?"

**Message 2 — quality assessment.**

> "Assess the quality of the approach — via formal results and/or benchmarks."

**Message 3 — grounding need not be a subgraph.**

> "When such a vectorisation is used for grounding (e.g. an LLM tool call), the grounding
> need not be a subgraph — it could be a vector, an NL string, or some other object the consumer
> needs. Consider that."

**Message 4 — inference interplay.**

> "Consider how different inference techniques — structured (deductive / constraint) and
> unstructured (neural / similarity) — interplay here."

The four map onto the four substantive sections below: **vectorisation design** (§3), **quality
assessment** (§6), **flexible grounding** (§4), **inference interplay** (§5). §1 states the
thesis, §2 the structural-priors table, §7 the sparq integration and phased plan, §8 a literature
appendix, §9 a frank limitations section.

---

## 1. Thesis

**Structural and ontological properties of a KG are PRIORS over the embedding manifold, not
post-hoc filters.** sparq already holds the schema that mainstream KGE pipelines discard:

- `xsd` datatype tags and inline order-preserving canonical integers in **sparq-core**
  (`crates/sparq-core/src/dict.rs`: inline `xsd:integer` ids that *sort by value* in the
  permutations; `crates/sparq-core/src/temporal.rs`: `Timeline` epoch-seconds value cells for
  `xsd:date`/`xsd:dateTime`);
- characteristic sets, observed-and-declared domain and range in **sparq-introspect**
  (`crates/sparq-introspect/src/lib.rs`: `CharacteristicSet(s)`, `inferred_domains/ranges`,
  `declared_domains/ranges`);
- enumerations via `owl:oneOf` and `sh:in`, cardinality, `owl:disjointWith`/`sh:not`, and the
  `rdfs:subClassOf` hierarchy in **sparq-reason** (`crates/sparq-reason/src/rdfs.rs` materialises
  the `subClassOf` closure; `crates/sparq-reason/src/owl.rs` handles `FunctionalProperty`,
  `disjointWith`/`cax-dw`) and **sparq-shacl** (`crates/sparq-shacl/src/model.rs`: `Datatype`,
  `MinCount`/`MaxCount`, `sh:in`, `sh:not`).

The core move is to make a node vector a **STRUCTURED PARTITIONED OBJECT**: a concatenation of
typed sub-vectors whose geometry is *chosen by the structure* (Euclidean for free attributes,
hyperbolic or box for taxonomy, order-preserving for magnitudes, codebook for enums, sign for
booleans) and whose training is *regularised by the closed and declared axioms* — rather than one
opaque blob learned from triple structure alone.

This is principled, not decorative, because **the score-function algebra fixes which relation
patterns a model can express** — DistMult is symmetric-only, ComplEx adds antisymmetry, RotatE
adds inversion and composition (Sun et al., RotatE, ICLR 2019) — and because **flat Euclidean
vectors compress trees badly while hyperbolic and box geometry embed subsumption at low
distortion** (Nickel & Kiela, NeurIPS 2017; Chami et al., ACL 2020; Vilnis et al. box
embeddings).

The same structured object is the unit of **FLEXIBLE GROUNDING**: return the
*minimal-and-complete* object in whatever modality the query or tool needs (a typed sub-vector for
ANN, a subgraph for exact answers, an enum- or unit-typed value for a tool slot, an NL
verbalisation for an LLM). All of this is under the inherited, non-negotiable invariant that
**structure-aware vectors are opt-in, never serve exact BGP answers, and only propose candidates
the exact engine re-validates** ([`genai-kg-embeddings-vectorindex.md`](genai-kg-embeddings-vectorindex.md)
§0; `research/data-structures.md` §R6/R7).

**What is provable vs what is empirical (stated up front, honestly).** Order-preservation of the
numeric encoder, enum/boolean exactness, and *profile-relative* closure completeness are
**formally provable**. Embedding *quality* (does any of this raise link-prediction or end-task
retrieval accuracy?) is **empirical and dataset-dependent**, and the literal- and type-aware KGE
literature is explicit that the gains are **inconsistent** (Gesese et al., Semantic Web Journal
2021; LiteralE's own ISWC-2019 numbers help FB15k but not FB15k-237). This document never asserts
an empirical win; it specifies the ablations that would have to measure one.

---

## 2. Structural priors — what sparq already holds and how each becomes a prior

Each row is a property sparq's engine *already computes or stores*, mapped to a concrete encoder
or training/serving prior. The lowest-risk, highest-leverage rows are flagged.

| Structural fact (where sparq has it) | Becomes this prior |
|---|---|
| **`xsd` datatype tag** (deduplicated in the Dict) | A **typed sub-vector ROUTER**: the datatype selects the encoder and geometric block — numeric → order-preserving block, date → `Timeline` epoch block, string → text block, boolean → sign block, IRI → entity block. Exact, free, no learning. |
| **`xsd:integer` in range** (sparq inlines canonical `xsd:integer` into ids that *sort by value*) | An **ORDER-PRESERVING magnitude encoder** (see §3.1 for the precise, provable form). Per-predicate quantile-normalise over the observed range, then a **strictly-monotone scalar-to-vector** encoding (thermometer / cumulative / single-axis projection) whose cosine to a fixed query is monotone in value over the predicate's range. Closes the gap that *plain relational KGEs* (TransE/DistMult/ComplEx/RotatE) do not order numbers — see the scoping note below. |
| **`xsd:boolean`** | A **SIGN prior**: one reserved ±1 dimension, optionally a learned per-predicate offset so `true`/`false` are antipodal; never a free float. |
| **`owl:oneOf` / `sh:in` enum** (sparq-shacl evaluates `sh:in`) | A **CATEGORICAL CODEBOOK** sub-vector of width = enum size (one-hot or learned), closed-world: out-of-enum is a reserved invalid code SHACL rejects. **Enum equality is a slot match, not a cosine threshold** (no recall loss). |
| **Cardinality** (`owl:FunctionalProperty`, `sh:maxCount=1`, `sh:minCount`, per-CS multiplicity) | A **POOLING RULE**: functional predicates → one deterministic slot; multi-valued predicates → a permutation-invariant pooled block; the degree/multiplicity vector (free from permutation prefix counts) → a struc2vec-style structural-role sub-vector. |
| **`rdfs:domain`/`range` + `rdf:type` + `rdfs:subClassOf` hierarchy** (introspect mines observed+declared; sparq-reason materialises the `subClassOf` closure) | Two priors: **(a) TYPE-CONSTRAINED NEGATIVE SAMPLING** — restrict KGE corruptions to domain/range-consistent entities (the **cheapest, lowest-risk win**; Krompass et al. 2015); **(b) a HYPERBOLIC or BOX taxonomy sub-vector** where subsumption is containment, switched on **only after a measured-distortion gate** on the actual `subClassOf` structure (see §3.3 and the must-fix note — the low-distortion result is for *tree/DAG transitive closures*, not arbitrary noisy `rdfs:subClassOf`). |
| **QUDT / unit annotations** (`qudt:unit`) | **UNIT-NORMALISE-BEFORE-MAGNITUDE**: convert to canonical SI via a bundled table, then apply the order-preserving encoder so `1000 m` and `1 km` share a code; quantity-kind is a routing slot; unit mismatch is **SHACL-detectable, not silent noise**. |
| **gUFO rigidity and roles** (rigid `gufo:Kind` vs anti-rigid `gufo:Role`/`Phase`) | A **SPLIT type sub-vector**: a stable rigid block (persistent, good cold-start anchor) and a volatile role/phase block refreshed by the structural-sketch channel without disturbing identity. **Speculative — most datasets carry no gUFO annotations; last/optional prior (Phase 3).** |
| **`owl:disjointWith` / `sh:not` / disjoint enums** (sparq-reason flags `cax-dw`; sparq-shacl evaluates `sh:not`) | A **REPULSION + MASK prior**: train-time margin pushing disjoint centroids apart; serve-time **hard mask** dropping any candidate the closure *proves* disjoint from the query type. **Answer-safe — removes only provably-wrong neighbours.** |

**Scoping note (must-fix from review).** The "KGEs do not order numbers" gap is **specific to
plain relational KGEs** (TransE/TransH/TransR, DistMult, ComplEx, SimplE, RotatE, QuatE, RESCAL),
which treat a literal as an opaque entity or drop it. The **literal-aware** line already injects
numerics — LiteralE (Kristiadi et al., ISWC 2019), MTKGNN, KBLRN, ReaLitE — so the order-preserving
block is **competing with existing-but-inconsistent prior art, not filling an empty gap**. Its
value over LiteralE-style gating is an *open empirical question* (§6.B), not an assumed win.

---

## 3. Vectorisation design

A node vector is a **STRUCTURED PARTITIONED OBJECT**: a fixed-layout concatenation of typed
sub-vectors, each from the encoder its structure selects, packed into the existing `.spqv`
fixed-stride row so the store format and O(1) id→offset lookup are **unchanged**
(`crates/sparq-vectors/src/store.rs`).

**Layout.** relational block · type/taxonomy block · numeric-magnitude block(s) · temporal block
· enum codebook block(s) · boolean-sign block · text block · structural-sketch block, **plus a
per-store SCHEMA HEADER** recording which predicate/datatype maps to which block offset, geometry,
and metric. The header is load-bearing: it lets a reader know that a block is order-preserving
normalised `xsd:integer` under a given predicate, or a Poincaré-ball taxonomy block under a
hyperbolic metric — **so search never applies cosine to a non-Euclidean block** (see §6's
metric-correctness guard).

**Each block carries a fusion weight** so a query can up-weight the modality it needs (consumed
by the existing `fuse_scores` / `fuse_rrf_weighted` / `hybrid_search` late-fusion path,
`crates/sparq-vectors/src/fuse.rs`).

### 3.1 The numeric-magnitude encoder — what is, and is NOT, provable (must-fix applied)

The original framing called this a "monotone Fourier-feature map". **That phrasing was withdrawn
after review and is not used here.** Sinusoidal / Fourier-feature codes are **periodic** and
**non-monotone under cosine** (the saturation/aliasing problem that motivated RoPE/ALiBi: two
far-apart values can alias to similar encodings when their difference hits a period). A generic
Fourier-feature family therefore **does not** give a globally order-preserving cosine, and
"monotone Fourier-feature map" fused two properties that do not compose.

**The provable invariant holds only for a restricted encoder:**

1. **Per-predicate quantile-normalise** the literal value over the predicate's observed range
   (a permutation scan gives the empirical CDF for free).
2. Map the normalised scalar through a **strictly-monotone scalar-to-vector encoding** — e.g. a
   thermometer / cumulative code, or a single-axis projection — whose **cosine to any fixed query
   magnitude is monotone in the value over the predicate's range**.

Under (1)+(2), for a fixed query magnitude, a lower value is closer than a higher value across the
*whole* observed range: 30 is nearer 31 than to 70, globally, not just locally. This is
**metamorphic-testable** and the test must assert **global monotonicity over the full observed
range**, not merely local neighbours.

If Fourier features are wanted for *relational expressiveness* (richer interaction in the
relational block), they may be retained there **but they MUST NOT carry the order-preservation
claim**, and the metamorphic monotonicity test will (correctly) FAIL on a periodic code — which is
exactly why the order-preserving block is a *separate, restricted* encoder.

### 3.2 The other literal encoders (pure functions keyed by datatype id)

- **date/dateTime** → sparq-core `Timeline` epoch-seconds, normalised (XPath comparison semantics
  already implemented).
- **boolean** → the sign dimension (§2).
- **enum** → one-hot or learned codebook; slot-equality decides membership.
- **string** → the existing out-of-process text embedder over the verbalisation
  (`crates/sparq-vectors/src/verbalize.rs`), which already keeps raw numbers *out of* the text;
  the numeric block now carries them.

### 3.3 The relational and taxonomy blocks (where the score-function algebra and geometry bite)

- **Relational block** — a chosen shallow KGE: **ComplEx default** (antisymmetry at 2d floats),
  **RotatE** when inversion and composition matter, **DistMult only if symmetry-dominated**.
  Trained with **type-constrained negatives** and **disjointness repulsion**. (The
  pattern→model map is the core structural fact, see §8.)
- **Taxonomy block** — **Euclidean by default**. Switchable to **hyperbolic or box ONLY after a
  measured-distortion gate** on the *actual* graph's `subClassOf` structure (Phase 3). The
  Nickel–Kiela / box low-distortion guarantee is for **tree/DAG transitive closures**; real
  `rdfs:subClassOf` is often a noisy, multiply-inheriting DAG, so hyperbolic is **not** adopted on
  a density heuristic alone — it is adopted only when the measured embedding distortion on the
  real hierarchy beats Euclidean. The header tags the block's metric so search never applies
  cosine to a Poincaré-ball block.

### 3.4 What is different from plain embeddings

Literal value, unit, enum-membership, and datatype are **first-class inputs**, not dropped. Order,
sign, and enum-exactness are **encoder invariants**, not hoped-for emergent properties. Geometry
is **per-block**, not one global Euclidean assumption. The partition makes **typed sub-vector
retrieval** possible (§4).

### 3.5 Truly-incremental cold-start fallback

The **structural-sketch block** — degree signature, characteristic-set membership, dense-predicate
Roaring bitmap, all already engine-computed — is computable for a **brand-new entity with no
training**: a cold-start vector the transductive trained blocks cannot give. This is the
incremental story for `.spqv`'s delta sidecar (`crates/sparq-vectors/src/delta.rs`).

---

## 4. Flexible grounding — modality chosen per request, minimal AND complete

**Grounding is a function from `(query | tool, graph)` to a minimal-and-complete OBJECT whose
MODALITY is chosen per request** — the same structured node object projected into each modality.
A small dispatcher on the tool's declared output type selects the modality.

1. **Subgraph** — when the tool needs verifiable facts or an exact answer. Structure-aware ANN
   proposes candidate dict-ids; the **exact engine evaluates the surrounding BGP** over them (the
   existing `vec-predicate` + `IdMask` filtered-ANN path, `crates/sparq-vectors/src/filter.rs` +
   `rewrite.rs`); the object is the **smallest connected sub-BGP entailing the answer**, checked
   against the closure.
2. **Vector or typed sub-vector** — when the consumer is a vector tool: return *only the relevant
   blocks* (numeric for similar-age/price, text for lexical, relational for fills-this-hole).
   Minimal by construction.
3. **NL string** — when the consumer is an LLM: verbalise the minimal subgraph plus typed values
   via the introspect token-budgeted schema-card machinery, **extended to render unit-normalised
   quantities and enum labels** (`verbalize.rs` + the introspect schema-card path).
4. **Structured value / tool payload** — when filling a typed slot: emit the **enum member**,
   **unit-typed quantity**, or **boolean** directly (the enum/unit/boolean blocks make this
   exact).

**Minimality** via characteristic-set + SHACL-shape selection (ABSTAT-style minimal type
patterns — only the predicates the effective type actually has; Spahiu et al., ESWC 2016), token
and `k` budgets, and typed-sub-vector projection. **Minimality is provable only relative to a
stated criterion** (smallest sub-BGP under a named entailment; fewest predicates under the
effective type) — not an absolute.

**Completeness** — stated precisely, **relative to a profile** (must-fix): via *materialising the
deductive closure before retrieval* (entailed-but-not-asserted facts present), SHACL conformance
(every *declared*-required property present), and disjointness masking. This is **completeness
relative to the materialised entailment profile (RDFS / OWL-RL / N3) and the declared SHACL
shapes** — silent outside that profile and outside whatever axioms the dataset actually declares.
It is **NOT** end-task answer-completeness, and §6's "completeness fraction" metric is explicitly
**profile-relative**.

**Default for ambiguous NL queries:** subgraph-grounding (exact, re-validated), because
approximate signals never serve as the final answer.

---

## 5. Structured ↔ unstructured inference interplay

Three composition patterns, with an honest completeness-vs-recall split between the deductive side
(sparq-reason) and the neural side (sparq-vectors / LLM).

**(A) Materialise-closure-before-vectorise.** Run sparq-reason (RDFS, OWL-RL, or N3) to
forward-chain the closure, **then** train/encode over the *closed* graph — so an entailed type via
`subClassOf`, or an inverse/transitive edge never asserted, now exists as a *real triple* the
type/taxonomy/relational blocks see, and the type-constrained negatives + disjointness repulsion
read the closed facts. **Provable:** any fact in the closure is materialised before vectorisation
(the reasoner is sound and complete for its profile, and sparq-reason's incremental closure is
property-tested equal to from-scratch). **This is the cheapest, lowest-risk, highest-confidence
pattern** and needs no `.spqv` format change.

**(B) Neuro-symbolic propose-then-verify.** The neural side (ANN or LLM) proposes high-recall
candidates *with no guarantee*; the deductive + constraint side **verifies** — the exact engine
re-evaluates the BGP (a proposed-but-absent triple is dropped), SHACL checks shape conformance
(datatype, cardinality, enum, unit), and closure disjointness + range facts mask provably-wrong
candidates. **The candidate set can only SHRINK to a sound subset** — generalising the existing
"candidates flow into the exact engine" Diverge invariant to SHACL+closure verification.

**(C) Planner-only answer-safe hook.** Relation-pattern structure (RotatE phase-composition,
ComplEx conjugate-inversion) plus CS multiplicities feed GNCE-style cardinality estimates —
**provably planner-only, never touching answers** (`research/data-structures.md` §S7).

**Completeness-vs-recall, honestly.** Deduction gives **completeness over its profile** (RDFS /
OWL-RL is sound and complete for that fragment) but is **silent outside it**. Neural gives
**recall** (analogy, lexical, soft type) with **no soundness or completeness guarantee**. The
discipline: use deduction to bound what **MUST** be returned (the complete entailed core), use
neural similarity to widen what **MAY** be returned (the approximate periphery), then **verify the
periphery deductively before it enters an exact answer**. Provable here: closure correctness,
verify-shrinks-to-sound, planner-only-safety. **Benchmarked-only:** whether the neural periphery
actually raises end-task recall.

**Honesty line (must-fix).** GraphRAG / KG-RAG does **NOT** uniformly beat vector RAG: published
benchmarks show it wins on multi-hop / hard questions, **loses or ties on easy single-hop**, and
is domain-dependent. The grounding end-task is therefore framed as a **measured comparison**, not
an assumed improvement.

---

## 6. Quality assessment — provable vs benchmarked, kept separate

Two classes, deliberately **not** mixed.

### 6.A Formally provable (prove, do not benchmark)

| Property | Statement | How verified |
|---|---|---|
| **Order-preservation** | The numeric encoder (§3.1: quantile-normalise + strictly-monotone scalar-to-vector) is a monotone map, so per predicate a lower value is closer to any fixed query magnitude than a higher value, **over the full observed range**. | **Metamorphic test** asserting *global* monotonicity (not local); a periodic Fourier code FAILS it, by design. |
| **Enum / boolean exactness** | Enum and boolean slots are exactly representable; slot-equality decides membership; **no recall loss**. | Property test: slot round-trip + out-of-enum → reserved invalid code SHACL rejects. |
| **Closure completeness (profile-relative)** | Encoding *after* sparq-reason means every fact in the **RDFS / OWL-RL / N3** closure is present pre-encoding. Reasoner sound+complete for its profile; incremental closure proven equal to from-scratch. **Relative to the materialised profile + declared shapes only — NOT answer-completeness.** | Existing sparq-reason property tests (incremental == from-scratch) + a new "every closed triple is visible to the encoder" assertion. |
| **Verify-soundness / answer-safety** | Propose-then-verify **only removes** candidates, so any returned exact answer is a real BGP solution. Extends the existing Diverge + filtered-equals-post-filter guarantees with disjointness + SHACL masking that drops **only provably-wrong** nodes. | Existing answer-safety tests extended; differential: ANN-proposed answer set ⊆ exact answer set after verify. |
| **Planner-only safety** | The cardinality hook provably never enters the answer path. | Existing GNCE-style planner-only test. |
| **Metric-correctness guard** | The per-block metric tag makes "cosine on a hyperbolic block" a **detectable error**, not silent corruption. | Test: a block tagged non-Euclidean rejects cosine search. |

### 6.B Benchmarked-only (empirical, dataset-dependent — gated, not assumed)

Every prior ships **behind an on/off ablation** and is adopted **only on measured lift**. **No
numbers exist yet; none are stated.** The harness must measure, per prior, on standard datasets
(FB15k-237, WN18RR, YAGO3-10 for link prediction; a taxonomy-heavy set — e.g. a biomedical
ontology — for the taxonomy block; a numeric-literal-rich set for the magnitude block):

- **Link-prediction** (Hits@k, MRR) with each prior on vs off — the honest, isolated ablation.
- **Type-constrained negatives** ablation (the lowest-risk win; the literature predicts a lift,
  but we measure it here).
- **Numeric-magnitude block** vs a **LiteralE-style gating baseline** (§2 scoping note) — the
  open question is whether the order-preserving block beats existing literal-aware methods, not
  whether it beats *no* literal handling.
- **Taxonomy block:** measured embedding **distortion** of Euclidean vs hyperbolic vs box on the
  *actual* `subClassOf` DAG (the Phase-3 distortion gate), then downstream Hits@k.
- **Grounding end-task:** structure-aware grounding vs plain-vector RAG vs subgraph-only, on a
  multi-hop and a single-hop QA split — reported as a **comparison**, expecting wins on multi-hop
  and possible ties/losses on single-hop (§5 honesty line).
- **Completeness fraction** = (entailed required facts present in the grounded object) / (entailed
  required facts), **explicitly profile-relative** — a sanity check on §4's completeness claim,
  **not** an answer-completeness metric.

**Eval harness.** Record/replay LLM cache for offline CI (reusing the `sparq-nlq` pattern); fixed
seeds; per-prior on/off matrix; the formal properties (6.A) as unit/property/metamorphic tests
that run in CI; the empirical numbers (6.B) as a separate, dataset-gated bench that publishes to
the dashboard (never baked into markdown — AGENTS.md *No hard-coded performance numbers*).

**Variance reduction & the firm-verdict gate (sq-4891y).** The per-cell mean ± std over seeds is
*not* the figure to gate on: it is dominated by **common-mode** seed noise (which train/test split
and init a seed draws moves every cell together), so on a schema-bearing slice a cell's std can be
≈ its mean even when a prior has a real effect. Each seed draws all four ablation cells from the
*same* split+init, so the **paired** contrast — closure-on − closure-off computed *within a seed*
and aggregated — cancels that common-mode term (the textbook paired-difference variance reduction,
`Var(b−a) = Var(a)+Var(b)−2·Cov(a,b)`). The harness (`crates/sparq-vectors/src/eval.rs`:
`run_ablation_multiseed_full` → `PairedContrast` / `ClosureVerdict`) reports the paired delta, its
paired std, the standard error (`∝ 1/√n` — more seeds shrink it), and a **`firm`** flag:
`|delta| ≥ t·std_error` at a *small-sample Student-t* threshold (`firm_z_for`, honest about the
handful of seeds a work-box runs) **AND** a unanimous per-seed sign. A prior is adopted only on a
firm, sign-positive lift. **Measured (NON-CANONICAL, work-box, INDICATIVE — re-measure on a
canonical machine + real schema-bearing KG):** on the synthetic gUFO slice the closure lift is
**real but direction-UNSTABLE** — its *sign flips across generator slices* (asserted by the
`closure_lift_sign_is_unstable_across_generator_seeds` characterisation test), and even where it is
large and unanimous-in-sign the paired standard error at a handful of seeds does not clear the
firm-verdict bar. So variance reduction **did not firm up** the synthetic gUFO closure claim: the
instrument now *exists and is honest*, and the verdict on synthetic data is "not firmly adoptable —
needs a real schema-bearing KG to settle". The firm-verdict gate is the mechanism that would let a
real-dataset run on a canonical machine settle it.

---

## 7. sparq integration — opt-in crate over sparq-vectors + sparq-shacl + sparq-reason

**Invariants inherited (non-negotiable):** opt-in crate, trivially removable, **zero
perf/memory/binary impact on the default exact engine**; approximate signals **never serve BGP
answers**; everything keyed by the dict u32 id-space — no side join tables
([`genai-design.md`](genai-design.md) §0).

The work is **additive over existing surfaces**, not a rewrite:

- **sparq-vectors** already gives the `.spqv` fixed-stride store, `StreamingWriter`, exact +
  DiskANN + opt-in HNSW search, `IdMask` filtered-ANN, `fuse_*`/`hybrid_search` late fusion,
  `import_npy`/`import_numeric_dump` out-of-process import, and `verbalize`. The structured-object
  work **adds**: the schema header + per-block metric tags (`store.rs`), the typed literal
  encoders (a new `encode.rs`, pure functions keyed by datatype id), and the block-weighted fusion
  path (extending `fuse.rs`).
- **sparq-shacl** already evaluates `sh:datatype`, `sh:in`, `sh:not`, `min/maxCount` — the prior
  extractor reads these from `model.rs` (no SHACL changes; a new reader in the opt-in crate).
- **sparq-reason** already materialises the `subClassOf` closure, `FunctionalProperty`, and
  `disjointWith`/`cax-dw` — the closure-before-vectorise step (§5.A) **calls** it; no reasoner
  changes.
- **sparq-introspect** already mines characteristic sets, observed+declared domain/range, and
  token-budgeted schema cards — the grounding selector + verbaliser **consume** these.

Suggested home: extend `sparq-vectors` with cargo features (one per prior family, default-off, so
each ships behind the on/off ablation §6.B demands), or a thin new opt-in crate depending on
`sparq-vectors` + `sparq-shacl` + `sparq-reason` if the dependency direction warrants. **No core
crate is modified.**

### 7.1 Phased plan (prioritised — cheapest/lowest-risk first)

| Phase | What | Risk / confidence |
|---|---|---|
| **0** | **Closure-before-vectorise + type-constrained negative sampling.** Materialise the sparq-reason closure, then encode; restrict KGE corruptions to domain/range-consistent entities. No `.spqv` change, no serving change; gated by an on/off Hits ablation. | **Lowest risk, strongest grounding.** The review's "strongest idea". Established KGE win; closure step already property-tested. |
| **1** | **Typed literal encoders** in sparq-vectors: datatype-router + **order-preserving numeric** (§3.1, provable) + boolean-sign + date/`Timeline`. Schema header + per-block metric tags. | Low. Formal properties provable; encoders are pure functions. |
| **2** | **Enum + datatype + cardinality prior extraction** from sparq-shacl (`sh:in`/`sh:datatype`/`min,maxCount`) → codebook + pooling blocks. **QUDT unit-normalisation** before the magnitude encoder. | Low–medium. Enum/boolean exactness provable; unit table bundled. |
| **3** | **Taxonomy block** (Euclidean default; hyperbolic/box **only past the measured-distortion gate**) + **disjointness repulsion/mask**. **gUFO rigid/role split is optional/last** (rare annotations). | Medium. Distortion gate is mandatory before adopting non-Euclidean. |
| **4** | **Flexible minimal-complete grounding selector + verbaliser** (subgraph / vector / NL / typed-value), profile-relative completeness, ABSTAT-style minimality. | Medium. Subgraph path reuses the exact engine; modality dispatch is new. |
| **5** | **Neuro-symbolic propose(neural)-then-verify(deductive) pipeline** (sparq-reason ↔ sparq-vectors), generalising the Diverge invariant to SHACL+closure verification. | Medium. Verify-shrinks-to-sound provable; recall lift benchmarked-only. |
| **6** | **Eval harness + formal-properties write-up** — the 6.A property/metamorphic tests in CI, the 6.B dataset-gated ablation bench on the dashboard. | Ongoing; gates every prior's adoption. |

---

## 8. Literature appendix (real citations)

**Score-function algebra → relation-pattern expressiveness.**
- TransE — Bordes et al., *Translating Embeddings for Modeling Multi-relational Data*, NeurIPS
  2013. Captures inversion/composition; **cannot** model symmetry or N-to-N.
- TransH / TransR — Wang et al. (AAAI 2014) / Lin et al. (AAAI 2015). Relation-specific
  hyperplane/space for multiplicity.
- DistMult — Yang et al., ICLR 2015. Diagonal bilinear; **symmetric-only by construction**.
- ComplEx — Trouillon et al., ICML 2016. Complex embeddings; symmetry **and** antisymmetry **and**
  inversion at 2d floats.
- SimplE — Kazemi & Poole, NeurIPS 2018. Head/tail-role vectors fixing CP independence.
- RESCAL — Nickel et al., ICML 2011. Full bilinear; maximally expressive, d² params/relation.
- **RotatE** — Sun et al., ICLR 2019. Relation = unit-modulus rotation in C^d; **the only classic
  model capturing symmetry + antisymmetry + inversion + composition simultaneously**. *This is the
  load-bearing expressiveness result the §3.3 model choice rests on.*
- QuatE — Zhang et al., NeurIPS 2019. Quaternion rotation; richer relational interaction.

**Literal-aware KGEs (the existing, inconsistent prior art — §2 scoping note).**
- Survey — Gesese, Biswas, Alam, Sack, *A Survey on Knowledge Graph Embeddings with Literals: Which
  model links better literal-ly?*, Semantic Web Journal 2021 (arXiv:1910.12507). **Honest finding:
  incorporating literals helps link prediction inconsistently; gains are dataset- and
  base-model-dependent; simple gating (LiteralE) is often as good as complex fusion.**
- LiteralE — Kristiadi et al., ISWC 2019. Learned gating merges numeric attributes; **helps FB15k
  but not FB15k-237, can hurt ConvE** — the canonical "gains are inconsistent" evidence.
- MTKGNN (multi-task numeric-regression head), KBLRN (product-of-experts), ReaLitE, TransEA,
  DKRL/ConMask/KG-BERT (text), IKRL (image).

**Type / ontology-aware KGEs (the structural-prior family).**
- **Type-constrained negative sampling** — Krompass, Baier, Tresp, *Type-Constrained
  Representation Learning in Knowledge Graphs*, ISWC 2015. *The near-free, lowest-risk Phase-0
  win.*
- TransT, TaRP (type + type-hierarchy priors), TypeComplex, OntoZSL/ontology-grounded methods,
  EmbedS (ontology-aware at scale).

**Hierarchy geometry (taxonomy block).**
- Nickel & Kiela, *Poincaré Embeddings for Learning Hierarchical Representations*, NeurIPS 2017.
  **Low-distortion tree embeddings in low dimension** — *for tree/DAG transitive closures* (the
  §3.3 caveat).
- Chami et al., *Low-Dimensional Hyperbolic Knowledge Graph Embeddings* (RotH/RefH/AttH), ACL 2020.
- MuRP — Balažević et al., NeurIPS 2019. ConE — Bai et al., NeurIPS 2021.
- BoxE — Abboud et al., NeurIPS 2020; box/region embeddings (Vilnis et al.) — subsumption as
  containment.

**Walk-based (neighbourhood, not relation algebra).**
- RDF2Vec — Ristoski & Paulheim, ISWC 2016. Random walks + word2vec; strong for
  classification/clustering, tunable between homophily and structural-equivalence notions of
  "similar".
- struc2vec — Ribeiro et al., KDD 2017. **Structural-role** identity (the §2 degree/multiplicity
  sub-vector).

**Schema / grounding machinery.**
- ABSTAT — Spahiu et al., *ABSTAT: Ontology-driven Linked Data Summaries with Pattern Minimization*,
  ESWC 2016. Minimal type patterns (the §4 minimality criterion).
- GNCE — graph-neural cardinality estimation (the §5.C planner-only hook;
  `research/data-structures.md` §S7).
- gUFO — the lightweight UFO implementation; rigid (`gufo:Kind`) vs anti-rigid
  (`gufo:Role`/`Phase`) — the §2 rigidity split. **Rarely present in wild datasets.**
- GraphRAG / KG-RAG — the literature is **mixed**: wins on multi-hop/hard questions, ties/loses on
  easy single-hop, domain-dependent (the §5 honesty line).

---

## 9. Limitations (frank)

1. **Embedding quality is empirical and may not improve.** The whole literal/type-aware KGE line
   reports **inconsistent** gains. Every prior here ships behind an on/off ablation precisely
   because we **cannot** assert in advance that it helps a given dataset. **No benchmark numbers
   exist; none are claimed.**
2. **The order-preserving claim is narrow.** It holds **only** for the restricted
   quantile-normalise + strictly-monotone encoder (§3.1). A Fourier/sinusoidal code is periodic
   and **non-monotone under cosine** — the metamorphic test will fail it. The provable property is
   *order-preservation of that specific encoder*, **not** "the embedding orders numbers" in
   general.
3. **"Completeness" is profile-relative, never answer-completeness.** It is completeness with
   respect to the materialised RDFS/OWL-RL/N3 closure and the *declared* SHACL shapes only —
   silent outside that profile and outside whatever axioms a dataset actually declares (§4, §6.A).
4. **Hyperbolic/box geometry is gated on measurement.** The low-distortion guarantee is for clean
   tree/DAG transitive closures; real `rdfs:subClassOf` is noisy and multiply-inheriting. Adopt
   non-Euclidean **only past the measured-distortion gate** (§3.3, Phase 3) — never on a density
   heuristic.
5. **gUFO rigidity is largely unvalidatable in the wild.** Most datasets carry no gUFO
   annotations, so the rigid/role split's cold-start benefit is hard to measure on standard
   benchmarks. It is the **last/optional** prior.
6. **GraphRAG is not a free win.** Structure-aware grounding is framed as a **measured comparison**
   against plain-vector RAG and subgraph-only — expected to win on multi-hop, possibly tie/lose on
   single-hop.
7. **Approximate-never-exact is absolute.** None of this touches the exact BGP path; structure-aware
   vectors only *propose* candidates the exact engine re-validates, and the planner hook is
   answer-safe by construction. (No ZK/MPC claim is made or relied upon anywhere in this design.)
8. **The strongest, safest slice is Phase 0** (closure-before-vectorise + type-constrained
   negatives): well-grounded, no format/serving change, property-tested closure, honestly gated by
   an on/off Hits ablation. The speculative tail (gUFO; non-Euclidean geometry) is deferred behind
   measurement.
