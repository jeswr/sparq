# Dogfooding sparq as a Project Knowledge Graph (PKG)

**Status:** design record (no implementation shipped beyond the cited crates). [OPUS-4.8]
**Scope:** use sparq's own RDF + SPARQL + reasoning + SHACL + vector + provenance
stack to store, query, and govern sparq's own project knowledge — research
findings, sources, concepts, techniques, and the `bd` task backlog — and to
serve that knowledge to LLM agents working on the repo.

> **STATUS / OUTCOME (2026-06-21).** [OPUS-4.8] Phase 1 of this plan was BUILT
> (`crates/sparq-kb`, the PKG ontology + SHACL shapes, the ingestion PoC, the
> `query-pkg` skill, and the `bench/pkg-dogfood/` A/B harness — all on `origin/main`)
> and the §5 token A/B was **RUN**. The measured verdict supersedes the pre-measurement
> *expectations* below: it is **positive, not "modest"**. The sanctioned results record
> is **[`bench/pkg-dogfood/RESULTS.md`](../bench/pkg-dogfood/RESULTS.md)** (`bench/` is
> the only home for the measured numbers — they are not repeated here). In a real N=30
> cache-discounted 3-arm transcript A/B, having Opus run `pkg-query` itself roughly
> halves its effective input tokens versus reading the source docs, and a cheap
> Haiku NL-tool answers the same PKG-answerable questions far more cheaply at equal or
> better quality (it also **safely abstains off-PKG**). So `recommend_adopt = true` for
> the PKG-answerable question class on THIS corpus. The DESIGN content below is retained
> as the historical plan; statements that pre-date the run are corrected inline with a
> pointer to `RESULTS.md`. The load-bearing caveats survive: the win is scoped to the
> PKG-answerable class, every number is NON-CANONICAL (work-box, list-price), and the
> result is one directional N=30 run, not a significance study — see `RESULTS.md`.

This is a design-for-review. Every empirical claim is tagged `[established]` /
`[claimed]` / `[measured]` per the project's empirical-honesty rule, and every
performance number is reported at runtime only (never frozen into this doc, per
`check-no-perf-numbers.py`).

> **Written against `origin/main`.** [OPUS-4.8] This record cites code, ontologies,
> bead verdicts, and provenance artifacts that live on `origin/main` and are
> **not** present on a behind/stale local `main`. In particular: the entire §2
> ontology-reuse foundation (the vendored `zkp-sparql` vocab, `sig-impl:Assertion`,
> the `secx:` assurance axis, `crates/sparq-trust/ontologies/zkp-sparql/PROVENANCE.md`,
> and `research/security-properties-ontology-design.md`), the §3 structure-aware
> vector surfaces (`crates/sparq-vectors/src/{grounding,encode,structure}.rs`,
> `structure`/`kge` features), and the §3.2 KGE-ablation verdict (bead sq-0wo9e.9)
> all exist only on `origin/main`. A reader on a behind checkout should `git fetch
> origin main` (and check out / diff against `origin/main`) before reading any cited
> path as fabricated. Where a fact is verifiable only from a bead verdict rather
> than a committed doc, that is flagged inline.

---

## 0. The goal, verbatim, and the core reframe

The maintainer's goals, as stated:

> Dogfood sparq as a project knowledge graph: store discoveries alongside their
> provenance and the confidence of their sources; record sources with an
> explored-status so follow-up can be targeted at the un-explored ones; capture
> concepts, claims, and algorithms/techniques (with relations like *supersedes*,
> *alternative-to*, *could-be-merged-with*, *implemented-by*); represent the `bd`
> task model (tasks + `dependsOn`/`blockedBy` + status + complex structural
> dependencies) in RDF; and link knowledge to tasks (e.g. *a novel or
> mergeable algorithm in `research/` implies an open bead should exist*).
> Serve this to LLM agents so they can (1) query minimal facts instead of
> loading whole docs into the context window, (2) keep durable long-term memory
> with provenance and confidence rather than re-reading sources or relying on
> lossy auto-memory, and (3) act under guardrails — sourced, confidence-tagged,
> filler-free entries plus bounded query answers — to reduce hallucination and
> rambling.

### The core reframe: prose to queryable facts

sparq's formalizeable knowledge corpus is **~62,000 lines / 5.7 MB across 190+
files**: `AGENTS.md` (charter, gate rules), 30+ `SKILL.md` files (~8,000 lines),
40 crate `README.md`s (~3,560 lines), 120 `research/*.md` design records
(~45,000 lines), `.beads/issues.jsonl` (1,277 structured records, 1.6 MB), and
`.claude/` agent config. An agent currently *reads* this prose. The reframe is to
*query* it: turn the load-bearing facts into RDF triples behind a SPARQL/SHACL/
reasoning surface so an agent pays tokens proportional to the **answer**, not the
corpus.

The honest order of value (read-frequency x formalizability):

| Target | Read freq | Formalizability | State |
|---|---|---|---|
| `.beads/issues.jsonl` | very high (session start) | very high (already JSON) | DONE as JSON; needs RDF projection |
| `AGENTS.md` gate matrix | very high (every PR) | very high (declarative) | prose |
| Skills frontmatter / maturity tags | high | very high (YAML + inline tags) | YAML + grep-needed inline tags |
| Crate feature graph (`Cargo.toml` + README) | high | high | semi-structured |
| Research design decisions + citations | medium | medium-high | prose + `[id]` refs |
| Historical decision rationale | low | low (~10k lines explanatory prose) | prose |

The reframe is **not** "formalize everything." It is: formalize the
high-frequency, high-formalizability head; leave the long tail as prose; and
prove the win on the head before extending.

---

## 1. How a PKG addresses the three LLM challenges — honestly

### 1.1 Context window — query minimal facts vs load whole docs

**Mechanism (genuinely won):** a SPARQL `SELECT` returns exactly the binding rows
that satisfy a pattern, so token cost is `O(rows x cols)` of the answer plus a
fixed schema-card prefix, versus `O(corpus)` for load-everything. The saving
scales with `corpus_size / answer_size` and is large precisely in the multi-hop /
needle-in-haystack regime where graph retrieval wins (GraphRAG-Bench
`[established]`, external; arXiv:2311.07914 `[established]`, external — neither is
a sparq-measured benchmark, so these are the weakest-grounded support for the
core context-window-win mechanism and are stated as external prior art only).
Two levers:

- **`sparq-introspect`** renders a token-budgeted schema card
  (`to_text_summary(budget_chars)`) so the agent learns the dataset's effective
  schema in <=N chars instead of scanning sample data; the card persists to a
  `.introspect` sidecar (mine once, summarise forever).
- **`sparq-nlq`** turns a question into SPARQL whose result is computed
  **deterministically by the engine** — the answer is not LLM-generated.

**The honest boundary — when it turns negative.** For *small* docs the mechanism
*hurts* net. Querying carries fixed overheads: (i) the schema-card prefix (a few
hundred to a few thousand tokens), (ii) the NL→SPARQL generate→validate→repair
round-trip(s), each a full LLM call, and (iii) for an MCP-served PKG, a per-turn
tool-definition tax in the prompt prefix. If the whole document fits comfortably
in context, loading it directly is fewer tokens **and** fewer round-trips. The
crossover is when `corpus >> context budget` **or** the same corpus is queried
many times (amortising the one-time `.introspect` build). This matches the
codebase's own honest finding (`research/agent-efficiency-tooling.md`): a
code-index/memory layer "carries a per-turn tool-definition tax that can erase
its own savings on small tasks."

**The biggest overclaim trap — nominal vs effective tokens.** `AGENTS.md` /
skills sit in the *cached prefix* served at ~0.1x on warm turns; a PKG's tool
defs + per-query results are net-new input at 1.0x (or invalidate the prefix on
connect). A naive "docs are big, queries are small" comparison is invalid. The
A/B (§5) must use **cache-discounted effective tokens**.

### 1.2 Long-term memory — durable provenance+confidence vs re-reading vs lossy auto-memory

**Mechanism (the strongest genuine differentiator):** an agent writes structured
findings back as triples (SPARQL UPDATE) and `sparq-prov` captures lineage
automatically — each derived/asserted fact gets `prov:wasGeneratedBy` an activity
that `prov:used` named source(s), with timestamps, and the IRIs are
**content-addressed**, so overlapping derivations stitch into **one** provenance
DAG (same fact = same entity node). This makes *"don't re-read sources already
explored"* a **query, not a heuristic**:

- *Already explored source S?* → `ASK { GRAPH ?g { ?act prov:used <S> } }`.
- *Still to explore* → `candidate-sources MINUS sources-with-provenance`.

This is a precise, auditable membership test that lossy natural-language
auto-memory (`MEMORY.md`-style) cannot give — auto-memory is a summary that
silently drops detail and has no closed-world "have I seen X?" predicate.

**Honest limit — confidence-weighted querying is not first-class.** The building
blocks exist: RDF 1.2 triple terms + `rdf:reifies` annotate a statement with a
confidence literal; named graphs partition by source; `sparq-nlq` computes a
per-answer confidence signal (repair-iters + link-scores + result-size). What
ships today: **store and `FILTER` on a confidence literal with plain SPARQL.**
What does **not** ship: principled confidence/trust **propagation through joins**
(Hartig's tSPARQL / annotated-RDF semirings, ESWC 2009 `[established]` prior art,
flagged "ambiguous — ask user" in `research/feature-research-hartig.md`). And
`sparq-trust` is an explicitly-labelled **research prototype** (no privacy,
operator-asserted keys, unaudited ZK) — not a security guarantee.

### 1.3 Guardrails — sourced, confidence-tagged, filler-free entries + bounded answers

**Mechanism (partly won):**

- **Bounded answers** — every `sparq-nlq.ask()` executes under a `QueryBudget`
  (default wall + row + byte caps); the answer is the engine's exact result set,
  so *within the store* the agent cannot fabricate a fact the data does not
  contain. NL→SPARQL turns the LLM into a query front-end; the answer is
  computed, not generated (the hallucination-free-when-SPARQL-is-correct property,
  arXiv:2311.07914 `[established]`).
- **SHACL as a write-gate** — `sparq-shacl` (W3C 98/98 core `sht:Validate`)
  validates candidate triples *before* commit, rejecting malformed/contradictory
  entries at write time rather than poisoning future reads.
  `sparq-introspect.to_shacl()` can bootstrap a shapes *floor* from the data.
- **Sourcing** — `sparq-prov` means every committed fact carries its source, so
  answers can be returned **with citations**.

**Honest limit — two distinct hallucination surfaces.** Never claim
"hallucination-free." Bounded-exact-answer eliminates **in-store answer
fabrication** but does **not** stop (a) the LLM hallucinating the **SPARQL
itself** (wrong predicate → wrong-but-valid answer), nor (b) **rambling in the NL
verbalisation** (the engine bounds rows, not the prose around them). sparq's
mitigations are real but uneven: the dictionary constraint (`check_dictionary`,
opt-in) catches dangling IRIs with did-you-mean repair; **true
grammar-constrained decoding** (forcing only grammatical SPARQL + existing-IRI
tokens) is documented **not implemented** because the Anthropic Messages API does
not expose logit/grammar access — feasible only on a local backend. And SHACL
gates **structure, not truth**: a hallucinated-but-well-formed triple committed
to the PKG becomes durable poison future queries trust. `empty-result != wrong`
is preserved (the "I don't know" abstention path).

### 1.4 The honesty anchor — the project already measured the counter-case

`research/agent-efficiency-tooling.md` surveyed exactly this idea —
structured/persistent graph memory for coding agents — and for a real
multi-agent workflow the verdict was **"status quo wins"**: the knowledge-graph
memory MCP server was rated "No — overkill"; mem0 showed a modest **single-digit
to low-tens-of-percent** token saving in the *one* independent controlled
benchmark; the dominant lever was **prompt-cache hygiene + brief discipline**,
not a memory product (an AGENTS.md flat-file study measured a modest output-token
and runtime reduction `[independent]`; the exact figures live in
`research/agent-efficiency-tooling.md`, reported at source and not frozen here).
**Pre-measurement conclusion (now superseded for this case): a queryable PKG can
help, but the honest expected magnitude for agent memory is modest and contingent
on first measuring a real re-derivation problem.** A flat structured file often
matches it at lower overhead. This is why §5 exists and why adoption is gated on a
verdict, not a gut read.

> **MEASURED UPDATE (2026-06-21).** [OPUS-4.8] The §5 A/B has since been RUN, and for
> the PKG-answerable question class the win is **larger than this "modest" prior**: see
> [`bench/pkg-dogfood/RESULTS.md`](../bench/pkg-dogfood/RESULTS.md). The general
> counter-case above still stands — graph-memory MCP products without a controlled
> benchmark remain unproven, and a flat structured file is the right baseline — but for
> *this* scoped use (answer-sized SPARQL facts over the head-slice corpus) the verdict
> came back `recommend_adopt = true`, not "status-quo wins". The earlier `agent-efficiency-tooling.md`
> verdict was about agent-memory *products* on a different workload, not this PKG-query
> mechanism.

### 1.5 What is genuinely novel vs me-too

The NL→SPARQL embed-schema → retrieve → generate-query → execute → verbalise loop
is the **same** shape as GraphRAG / Stardog Voicebox / Ontotext GraphDB — sparq
is not first here. Genuinely differentiated for a *project* KG:

1. **fully-local / WASM** — store + retrieval + reasoning client-side, data never
   leaves device (standout for a personal/project KG);
2. **one dict u32 id-space** — embeddings are a `Vec<f32>` keyed by id; no bolt-on
   vector DB, no join tables;
3. **reasoning colocated with retrieval** — embed/query the OWL/N3 **closure** so
   entailed-but-not-asserted facts surface;
4. **per-rule provenance** — `prov_from_proof` names the rule + premises for *each*
   inferred fact (finer than GraphRAG's "trust the summary"; finer than a
   nanopublication's assertion/provenance/pubinfo split — sparq has no nanopub
   *packaging* today but PROV-O + named graphs + RDF 1.2 are a strictly finer
   substrate).

---

## 2. The PKG ontology — reuse-first

> **Origin/main dependency (§2):** every external vocab cited below — the vendored
> `zkp-sparql` ontologies (`sig-impl`, `sec-prop`/`secx:`), their `PROVENANCE.md`,
> and the precedent design record `research/security-properties-ontology-design.md`
> — exists on `origin/main` and **not** on a behind local `main`. Reuse claims are
> against the `origin/main` tree. [OPUS-4.8]

**Design principle (copied from the maintainer's own `sec-prop` discipline,
`research/security-properties-ontology-design.md`):** the PKG needs almost **no**
net-new classes. PROV-O carries who/what/when/derived-from; SKOS carries
concepts/topics; DCAT + FaBiO/FRBR carry the source catalog; schema.org fills
gaps; and the vendored `zkp-sparql` ontologies already define the exact
*Claim + Evidence + Confidence + Source* pattern as `sig-impl:Assertion` (a
reified per-(thing, property) verdict with `sig-impl:verdict ∈ {yes,no,partial}`
+ `sig-impl:justification` + `prov:wasDerivedFrom`). The honest move is to
**generalise `sig-impl:Assertion` into `pkg:Finding`, not fork it.**

The seven conventions to follow (from `sec-prop`): (1) terms are `skos:Concept`
subclasses with labels; (2) every assertion **must** carry
`prov:wasDerivedFrom`; (3) one **orthogonal** assurance/confidence axis, stated
once; (4) SHACL shapes enforce required fields; (5) British-English prose;
(6) an irreducibility argument per term; (7) ordered enums materialised as
`atLeast`/`strongerThan` facts so ODRL `gteq`/`lteq` and N3 reasoning work.

### 2.1 Namespaces (reuse-first; mint only under `pkg:`)

```turtle
@prefix pkg:    <https://sparq.dev/ns/pkg#> .      # net-new sparq terms ONLY
@prefix prov:   <http://www.w3.org/ns/prov#> .
@prefix skos:   <http://www.w3.org/2004/02/skos/core#> .
@prefix dcat:   <http://www.w3.org/ns/dcat#> .
@prefix dcterms:<http://purl.org/dc/terms/> .
@prefix fabio:  <http://purl.org/spar/fabio/> .
@prefix cito:   <http://purl.org/spar/cito/> .
@prefix frbr:   <http://purl.org/vocab/frbr/core#> .
@prefix bibo:   <http://purl.org/ontology/bibo/> .
@prefix schema: <http://schema.org/> .             # http:// form, per crates/sparq-trust
@prefix np:     <http://www.nanopub.org/nschema#> .
@prefix sigimpl:<https://w3id.org/zkp-sparql/sig-impl#> .  # maintainer's reified-assertion pattern
@prefix secx:   <https://w3id.org/zkp-sparql/sec-prop#> .  # maintainer's assurance axis
```

### 2.2 Reuse-provenance table (which external vocab each PKG term reuses)

| PKG term | reuses |
|---|---|
| `pkg:Source` | `fabio:Expression` + `dcat:CatalogRecord` + `dcterms:*` + `bibo:doi` + `frbr:realizationOf` |
| `pkg:exploredStatus` / `pkg:followUpPriority` | **NET-NEW** (skos scheme; align `schema:ActionStatusType`) |
| topics | `skos:Concept` + `dcterms:subject` (no mint) |
| `pkg:Finding` | `rdfs:subClassOf sigimpl:Assertion`; `skos:Concept` |
| `pkg:verdict` | reuse `sigimpl:yes/no/partial` (allow `pkg:`-specific enum if semantics diverge) |
| `pkg:confidence` | **NET-NEW** numeric 0..1; align `schema:Rating` |
| `pkg:assurance` | reuse `secx:` Proven/Claimed/Conjectured (orthogonal axis) |
| supporting/refuting | `cito:supports` / `cito:citesAsEvidence` / `cito:disagreesWith` |
| provenance | `prov:wasDerivedFrom` / `wasGeneratedBy` / `wasAttributedTo` / `generatedAtTime` |
| nanopub wrapper | `np:hasAssertion` / `hasProvenance` / `hasPublicationInfo` |
| supersedes | `dcterms:replaces` / `dcterms:isReplacedBy` |
| alternative-to | `skos:related` |
| implemented-by | `pkg:implementedBy` → `schema:SoftwareSourceCode` + `prov:wasGeneratedBy` (PR) |
| could-be-merged-with | **NET-NEW** `pkg:couldBeMergedWith` (`owl:SymmetricProperty`) |
| `pkg:Task` | `rdfs:subClassOf schema:Action` |
| status | skos scheme; align `schema:ActionStatusType` |
| `pkg:dependsOn`/`pkg:blockedBy` | **NET-NEW** (`owl:inverseOf` pair) |
| parent-child | `dcterms:isPartOf` |
| discovered-from | `pkg:discoveredFrom` `rdfs:subPropertyOf prov:wasDerivedFrom` |
| surface | `skos:Concept` (mirrors `bd` `area:*` labels) |

**Only four terms are genuinely net-new:** `pkg:exploredStatus`,
`pkg:followUpPriority`, `pkg:confidence`, `pkg:couldBeMergedWith` — plus the
`pkg:dependsOn`/`pkg:blockedBy` inverse pair.

**The single task-dependency inverse pair, fixed once.** [OPUS-4.8] The PKG uses
exactly **`pkg:dependsOn`** (a task waits on its dependency) and its OWL inverse
**`pkg:blockedBy`** — no third predicate. `bd`'s `blocks` edge is the
*inverse-of-`pkg:dependsOn`* direction, and is modelled **as `pkg:blockedBy`**, not
a separate `pkg:blocks`. Every later use in this record — the §2.3 OWL
declaration, the §4.1 frontier query, and the §4.4 SHACL stale-edge shape — names
**only** these two predicates so a constraint cannot silently target an undefined
property.

### 2.3 Turtle sketch

**Source / Document** (DCAT + FaBiO/FRBR + DC; mint only explored-status):

```turtle
pkg:Source a owl:Class ; rdfs:subClassOf fabio:Expression , dcat:CatalogRecord .

<src:neumann-moerkotte-2011> a pkg:Source , fabio:ConferencePaper ;
  dcterms:title "Characteristic Sets for cardinality estimation"@en ;
  dcterms:creator <person:neumann> , <person:moerkotte> ;
  dcterms:issued "2011"^^xsd:gYear ; bibo:doi "10.1109/ICDE.2011.5767868" ;
  pkg:confidence 0.95 ;                        # source reliability 0..1
  pkg:exploredStatus pkg:Explored ;            # NEW: Unexplored|Exploring|Explored|DeadEnd
  pkg:followUpPriority 1 ;                      # NEW: targeted-follow-up ordering
  dcterms:subject <topic:cardinality-estimation> .

# explored-status values as a SKOS scheme so they are queryable + extensible:
pkg:ExplorationStatus a skos:ConceptScheme .
pkg:Unexplored a skos:Concept ; skos:inScheme pkg:ExplorationStatus ;
  skos:closeMatch schema:PotentialActionStatus .   # ALIGN to schema.org
pkg:Exploring  a skos:Concept ; skos:inScheme pkg:ExplorationStatus ;
  skos:closeMatch schema:ActiveActionStatus .
pkg:Explored   a skos:Concept ; skos:inScheme pkg:ExplorationStatus ;
  skos:closeMatch schema:CompletedActionStatus .
pkg:DeadEnd    a skos:Concept ; skos:inScheme pkg:ExplorationStatus .
```

**Concept / Topic** (pure SKOS — no mint):

```turtle
pkg:Topics a skos:ConceptScheme ; dcterms:title "sparq project topics"@en .
<topic:cardinality-estimation> a skos:Concept ; skos:inScheme pkg:Topics ;
  skos:prefLabel "Cardinality estimation"@en-GB ;
  skos:broader <topic:query-planning> ; skos:related <topic:characteristic-sets> .
# every Finding/Source/Technique/Task tags itself with dcterms:subject -> a skos:Concept
```

**Finding / Claim** (generalise `sig-impl:Assertion`; nanopub wrapper; PROV-O + CiTO):

```turtle
pkg:Finding a owl:Class ; rdfs:subClassOf sigimpl:Assertion , skos:Concept .

<find:cs-dual-use> a pkg:Finding ;
  rdfs:label "Characteristic sets serve both planner cardinality and LLM grounding"@en-GB ;
  pkg:about <tech:characteristic-sets> ;
  pkg:verdict sigimpl:yes ;                       # reuse yes|no|partial
  sigimpl:justification "One SPO scan yields {predicate set -> count, multiplicity, observed domain/range}."@en-GB ;
  pkg:confidence 0.9 ;                            # NEW numeric 0..1
  pkg:assurance secx:Claimed ;                    # reuse Proven>Claimed>Conjectured
  cito:citesAsEvidence <src:neumann-moerkotte-2011> ;
  cito:supports <src:genai-ontology-introspection> ;
  prov:wasDerivedFrom <src:genai-ontology-introspection> ;  # MANDATORY (SHACL minCount 1)
  prov:wasGeneratedBy <activity:session-2026-06-21> ;
  prov:wasAttributedTo <agent:sparq-opus48> ;
  prov:generatedAtTime "2026-06-21T00:00:00Z"^^xsd:dateTime ;
  dcterms:subject <topic:cardinality-estimation> .

# packaged as a nanopublication (three named graphs):
<np:cs-dual-use> a np:Nanopublication ;
  np:hasAssertion <graph:assertion> ;
  np:hasProvenance <graph:prov> ;
  np:hasPublicationInfo <graph:pubinfo> .
```

> **Refuting sources are kept, not dropped.** If a later finding flips the
> verdict, retain `cito:disagreesWith <src:...>` and let `pkg:confidence` carry
> the weight — never silently delete the refuting edge.

**Algorithm / Technique** (reuse `dcterms:replaces` / `skos:related` /
`schema:SoftwareSourceCode`; mint only `couldBeMergedWith`):

```turtle
pkg:Technique a owl:Class ; rdfs:subClassOf skos:Concept .

<tech:characteristic-sets> a pkg:Technique ;
  skos:prefLabel "Characteristic-set cardinality estimation"@en-GB ;
  dcterms:replaces <tech:predicate-only-histogram> ;   # SUPERSEDES (inverse dcterms:isReplacedBy)
  skos:related <tech:gnce-cardinality> ;               # ALTERNATIVE-TO
  pkg:couldBeMergedWith <tech:qse-shape-mining> ;       # NEW symmetric merge hint
  pkg:implementedBy <crate:sparq-introspect> ;          # -> schema:SoftwareSourceCode
  prov:wasGeneratedBy <pr:NNN> .                         # implementing PR as a prov:Activity
<crate:sparq-introspect> a schema:SoftwareSourceCode .
```

**Task** (the `bd` model → RDF; schema.org status + DC structure):

```turtle
pkg:Task a owl:Class ; rdfs:subClassOf schema:Action .   # (or prov:Activity)

# the single inverse pair (§2.2): pkg:dependsOn owl:inverseOf pkg:blockedBy
pkg:dependsOn a owl:ObjectProperty ; owl:inverseOf pkg:blockedBy .
pkg:blockedBy a owl:ObjectProperty .

<task:sq-0dksu> a pkg:Task ;
  dcterms:title "..." ; pkg:issueType pkg:Epic ;          # bug|feature|task|epic|chore|decision|milestone|spike
  pkg:status pkg:Open ;                                   # Open|InProgress|Blocked|Deferred|Closed
  pkg:priority 1 ;
  pkg:dependsOn <task:sq-pfae> ;                          # bd 'blocks' edge, modelled as the dependsOn direction
  pkg:blockedBy <task:sq-qhy4> ;                          # owl:inverseOf pkg:dependsOn (entailed both ways)
  dcterms:isPartOf <task:sq-0dksu-parent> ;               # bd parent-child
  pkg:discoveredFrom <find:audit-gap> ;                   # rdfs:subPropertyOf prov:wasDerivedFrom
  skos:related <task:sq-yh427> ;                          # bd 'related'
  pkg:surface <surface:sparq-introspect> ;                # bd area:* label -> skos:Concept
  dcterms:subject <topic:cardinality-estimation> ;
  dcterms:relation <research:genai-ontology-introspection> .  # bd --spec-id

pkg:Open       skos:closeMatch schema:PotentialActionStatus .
pkg:InProgress skos:closeMatch schema:ActiveActionStatus .
pkg:Closed     skos:closeMatch schema:CompletedActionStatus .
pkg:Surfaces a skos:ConceptScheme .
<surface:sparq-introspect> a skos:Concept ; skos:inScheme pkg:Surfaces .
```

### 2.4 SHACL guardrail shapes (sourced + confidence + no-filler)

The load-bearing honesty discipline — `pkg:confidence` and `pkg:assurance` are
**mandatory** on every Finding (no discovery stored without its epistemic basis);
filler/placeholder prose is rejected; derived claims must cite their rule. Gate
ingestion on `report.conforms_violations_only()`.

```turtle
pkg:FindingShape a sh:NodeShape ; sh:targetClass pkg:Finding ;
  sh:property [ sh:path prov:wasDerivedFrom ; sh:minCount 1 ; sh:nodeKind sh:IRI ;
                sh:message "every Finding must cite >=1 source (prov:wasDerivedFrom)" ] ;
  sh:property [ sh:path pkg:confidence ; sh:minCount 1 ; sh:maxCount 1 ;
                sh:datatype xsd:decimal ; sh:minInclusive 0.0 ; sh:maxInclusive 1.0 ] ;
  sh:property [ sh:path pkg:assurance ; sh:minCount 1 ;
                sh:in ( secx:Proven secx:Claimed secx:Conjectured ) ] ;
  sh:property [ sh:path sigimpl:justification ; sh:minCount 1 ; sh:minLength 12 ;
                sh:pattern "^(?!(TODO|TBD|lorem|placeholder|FIXME)).*" ;
                sh:flags "i" ; sh:message "justification must be non-filler, >=12 chars" ] ;
  # cross-field: a derived claim must cite the rule that produced it
  sh:sparql [ a sh:SPARQLConstraint ;
              sh:message "a derived Finding must cite the rule that produced it" ;
              sh:select """SELECT $this WHERE {
                $this pkg:derivedBy ?rule .
                FILTER NOT EXISTS { $this cito:citesAsEvidence ?r } }""" ] .

pkg:SourceShape a sh:NodeShape ; sh:targetClass pkg:Source ;
  sh:property [ sh:path pkg:exploredStatus ; sh:minCount 1 ; sh:maxCount 1 ;
                sh:in ( pkg:Unexplored pkg:Exploring pkg:Explored pkg:DeadEnd ) ] ;
  sh:property [ sh:path dcterms:title ; sh:minCount 1 ; sh:minLength 4 ] .

# supersedes must name BOTH techniques (catch dangling claims)
pkg:SupersedesShape a sh:NodeShape ; sh:targetClass pkg:Technique ;
  sh:sparql [ a sh:SPARQLConstraint ;
              sh:message "dcterms:replaces must point at an existing pkg:Technique" ;
              sh:select """SELECT $this WHERE {
                $this dcterms:replaces ?o .
                FILTER NOT EXISTS { ?o a pkg:Technique } }""" ] .
```

> Bootstrap the first draft of `pkg:FindingShape`/`pkg:SourceShape` from
> `sparq-introspect.to_shacl()` over the existing graph (a data-grounded floor),
> then hand-tighten. Data-mined shapes only enforce what the data already
> satisfies — they are a starting point, not the contract.

### 2.5 Open ontology decisions (flagged for the maintainer)

- **`pkg:verdict` enum**: reusing `sigimpl:yes/no/partial` is a judgement call — a
  research finding's verdict space (holds/refuted/uncertain/superseded) may not
  map 1:1 onto a security-property verdict. Subclass the pattern but allow a
  `pkg:`-specific enum if semantics diverge.
- **`pkg:couldBeMergedWith`** is a forward-looking hint with no established
  precedent → it risks being a dumping ground. Assert it **as a `pkg:Finding`**
  about two Techniques (carrying evidence + confidence), not a bare triple, per
  the repo's "TODOs → beads, not scattered notes" rule.
- **`skos:closeMatch` alignments** to schema.org / CiTO / nanopub were cited from
  established knowledge of the SPAR/W3C-community vocabularies, **not** verified
  against the live ontologies in this design session. Each alignment must be
  checked against the current published ontology before shipping (exactly as
  `sec-prop` did its DPV/security-vocab survey).
- **schema.org form**: the repo's trust crate uses `http://schema.org/`; the PKG
  follows that for consistency (convention to confirm, not a settled fact).

---

## 3. sparq surface mapping — ready vs needs-work

> **Origin/main dependency (§3):** the grounding selector, structure-aware
> encoders, and the `kge` ablation verdict cited below are on `origin/main`
> (`crates/sparq-vectors/src/{grounding,encode,structure}.rs`, the `structure`/`kge`
> features, bead sq-0wo9e.9). A behind checkout lacks them — `git fetch origin
> main` first; do not read them as fabricated. [OPUS-4.8]

| Capability | Crate | Maturity | Role in the PKG |
|---|---|---|---|
| **SPARQL query** | `sparq-engine` | **production-ready** | the load-bearing exact fact-lookup primitive |
| **SHACL** | `sparq-shacl` | **production-ready** (98/98 core) | write-time admission gate (§2.4) |
| **Reasoning** | `sparq-reason` | **production-ready** | transitive deps, regulation→requirement (N3), `sameAs`, proof-tree "why surfaced" |
| **Provenance** | `sparq-prov` | **production-ready** | durable memory + "have I seen S?" membership |
| **Vectors (store/search)** | `sparq-vectors` | **production-ready** storage+search | semantic "find related I didn't name" |
| **Structural similarity** | `sparq-sim` | **production-ready** | training-free, model-free retrieval fallback |
| **Introspection** | `sparq-introspect` | **production-ready** | token-budgeted schema cards; bootstrap shapes |
| **Grounding selector** | `sparq-vectors` (`structure` feat) | **built** (origin/main) | per-request modality dispatch — the token-reducer |
| **NL→SPARQL** | `sparq-nlq` | **mature PoC, accuracy UNMEASURED** | assistive, not a trusted oracle |
| **Structure-aware embeddings** | `sparq-vectors` (`kge` feat) | **built, empirically NEUTRAL** | priors are no-ops on plain triples |
| **CLI/HTTP exposure** of nlq/introspect/grounding | `sparq-server` | **GAP — not wired** | in-process Rust today; raw SPARQL + VoID only over the wire |
| **propose-then-verify (P5)** | — | **OPEN, not built** (sq-0wo9e.6) | the safety wrapper for an LLM writing INTO the KG |

### 3.1 Production-ready (use today)

- **SPARQL** is full 1.1/1.2 (all 8 path operators, aggregates, subqueries, RDF-1.2
  triple terms, prepared queries, EXPLAIN, custom functions, window functions,
  `QueryBudget`). For "what depends on X / who owns Y / status of Z" this is the
  cheapest, exact path.
- **Reasoning** (`materialize()` forward-chains; incremental maintenance keeps the
  closure `==` from-scratch under ABox edits; `inconsistencies()` flags OWL
  clashes; `why()` proof-trees under the `explain` feature). Caveats to check at
  runtime: OWL incremental silently drops to full re-materialise on
  `sameAs`/functional/property-chains/TBox edits (`.mode()`); N3 incremental is a
  narrow monotone fragment (`.fallback_reason()`).
- **The flexible grounding selector** (`grounding.rs`, `structure` feature) is the
  token-reducer the dogfooding use needs: a per-request modality dispatcher —
  `OutputType{Facts→Subgraph, Vector→TypedSubVector, Text→NlString,
  Value→TypedValue}`. The Subgraph is ABSTAT-style minimal (only the predicates
  the node's effective/minimal type carries). **Always call
  `structure::close_for_vectorise(RDFS|OWL-RL)` before grounding** so completeness
  is profile-relative. The token win comes from this minimal+complete projection,
  **independent of whether embeddings help.**

### 3.2 Needs work / honest gaps

- **NL→SPARQL accuracy is UNMEASURED.** The lean ground→generate→validate(spargebra
  parse)→execute(under budget)→repair loop is fully built with read-only
  enforcement, offline record/replay, and an opt-in live backend; the
  exec-accuracy harness (sq-05rv) is built (answer-set F1, oracle-vs-end-to-end x
  grounded-vs-ungrounded). **But every CI number comes from scripted/recorded
  fixtures** — it validates the *harness/mechanism*, not a real model's accuracy.
  The live run is `#[ignore]`'d + feature-gated OFF. Treat nlq as an assistive
  PoC; surface the executed SPARQL + transcript for verification; keep
  `check_dictionary` + entity-linking ON and the repair loop bounded.
- **Structure-aware embeddings are empirically neutral on plain data.** The
  canonical KGE ablation (verdict recorded against bead sq-0wo9e.9 on
  `origin/main`; the measured numbers live in the bead verdict + the supporting
  `research/structure-aware-vectorisation.md` history on `origin/main`, not on a
  behind local `main`) found, for filtered link-prediction with ComplEx over
  multiple seeds, that the structure priors are **no-ops on plain triple sets**
  (closure-on indistinguishable from closure-off; type-constrained negatives
  indistinguishable from uniform) because a plain `.nt` has no RDFS schema to
  close or type-constrain. They fire only on the gUFO/ontology-rich slice (a
  materially higher MRR) but the per-seed variance was large enough that the
  effect was **not statistically firm** (standard deviation on the order of the
  mean), and type-constrained negatives *depress* the gUFO result. **Verdict:**
  closure-before-vectorise **ADOPT** for ontology-rich KGs (harmless no-op
  elsewhere); type-constrained negatives **REJECT** (default off). The
  token-reduction win is from the schema/grounding machinery, **not** from
  measurably-better vectors. (Exact MRR/std figures are reported at the bead /
  bench source, never frozen into this doc per `check-no-perf-numbers.py`.)
- **Wiring gap.** None of nlq / introspect / grounding / vector structure features
  are exposed via CLI or HTTP — today an agent dogfooding the KG embeds the Rust
  crates in-process or drives raw SPARQL over the HTTP endpoint. The
  propose-then-verify pipeline (sq-0wo9e.6) — the loop that lets an LLM propose
  facts the reasoner+SHACL verify-and-shrink-to-sound — is **OPEN**.

---

## 4. The `bd` task tracker — bridge, do not replace

**Recommendation: bridge first, gate any replacement behind a measured eval.**

sparq has every engine primitive the idea needs (verified in-tree): SPARQL with
`FILTER NOT EXISTS`/`MINUS`/`GROUP BY`/property paths/aggregates; a
forward-chaining N3 rule engine (`reason_n3`, EYE/cwm 98.8% parity, proof trees);
SHACL Core + SHACL-SPARQL; RDFS/OWL-RL with incremental maintenance. The `bd`
model (1,277 issues) maps cleanly — typed dependency edges
(blocked-by/blocks/parent-child/discovered-from/related), status, issue_type,
priority, labels, external_ref — a near-mechanical JSONL→triples projection.

**But what `bd`/Dolt gives that a naive replacement would lose:**

1. **git-native, conflict-safe storage** — `.beads/issues.jsonl` is the committed
   source-of-record; Dolt gives branch-aware **three-way merge of issue rows**;
   `AGENTS.md` has hard-won merge discipline ("never edit `.beads/` in a
   worktree") *because* concurrent agents mutate tasks. An RDF file as
   source-of-record re-introduces a merge-conflict problem sparq does not solve
   for task rows. **This is the single biggest reason to bridge, not replace.**
2. **`bd ready` is offline + millisecond** — no engine spin-up; a sparq-backed
   frontier pays graph-load + query setup. The SessionStart hook and tight
   orchestration loops need that latency.
3. **maturity/ecosystem** — `bd dep`, audit trail, the whole `AGENTS.md` workflow
   + SessionStart hook + bead-autoclose CI are built around the CLI.

**The ready-frontier is only HALF a pure SPARQL query.** The dependency frontier
is fully expressible (§4.1, verified-expressible); the load-bearing half in
`scripts/push-frontier.sh` (in-flight-by-unpushed-git, per-crate/code-crate
conflict partition, held-lanes, umbrella-vs-child, CPU cap) depends on **live
git/gh/nproc state that is not in any task store** — so SPARQL replaces the easy
half and would need a pre-materialised "live state" named graph for the rest.

**The clearest pro-sparq case — the knowledge↔task JOIN.** "An algorithm in
`research/` that is novel and not-yet-implemented IMPLIES a bead should exist" is
something `bd` **cannot express at all** because `bd` has no model of the
knowledge. Only a KG holding *both* tasks and domain knowledge can run that rule.
This is the bridge's killer feature and argues for a **showcase mirror**, not a
replacement — the demo value is fully realised by mirroring `bd` into a KG
read-model; replacing the write/merge path buys none of it and incurs all the
conflict-safety risk. **Maturity caveat (read §4.6):** the *value* of this join
is high, but its *readiness* is the lowest of the §4 items — the SPARQL frontier
(§4.1) is expressible today, whereas the N3 rules (§4.2, §4.3) depend on
scoped-negation semantics that must be verified on a live `reason_n3` run, and the
input data (`pkg:novel`/`pkg:mergeableInto`) is an unsolved LLM-extraction problem.

### 4.1 Ready-frontier SPARQL (dependency half — verified-expressible TODAY)

This is the one §4 item that is **soundly expressible on the engine as written**;
the N3 rules below (§4.2, §4.3) are not yet at this level — see §4.6.

```sparql
PREFIX pkg: <https://sparq.dev/ns/pkg#>
SELECT ?t ?prio WHERE {
  ?t a pkg:Task ; pkg:status ?s ; pkg:priority ?prio .
  FILTER(?s IN (pkg:Open, pkg:InProgress))
  # no OPEN blocking dependency (uses ONLY the §2.2 pkg:dependsOn predicate):
  FILTER NOT EXISTS { ?t pkg:dependsOn ?b . ?b pkg:status ?bs . FILTER(?bs != pkg:Closed) }
  # not human-gated:
  FILTER NOT EXISTS { ?t pkg:label ?l . FILTER(STRSTARTS(STR(?l), "needs:")) }
  # not an umbrella (parent of any other task):
  FILTER NOT EXISTS { ?child dcterms:isPartOf ?t }
} ORDER BY ?prio ?t
```

The conflict-partition + CPU-cap + in-flight-by-git layers are **not** in this
query — they require the live-state named graph (Phase 2).

### 4.2 N3 trigger-rule (a ready, conflict-free task is launchable) — VERIFY ON A LIVE RUN

**Maturity: NOT yet verified-expressible like §4.1.** This rule depends on
`log:notIncludes` scoped negation, whose N3 semantics are subtle; treat it as a
**Phase-2 "diff against `push-frontier.sh` on a live `reason_n3` run"** deliverable,
not as turnkey.

```n3
@prefix pkg: <https://sparq.dev/ns/pkg#> .
@prefix log: <http://www.w3.org/2000/10/swap/log#> .
# A ready, unblocked, non-gated task whose surface crate is NOT in flight is launchable:
{ ?t a pkg:Task ; pkg:status pkg:Open ; pkg:crate ?c .
  ?t log:notIncludes { ?t pkg:dependsOn [ pkg:status pkg:OpenBlocker ] } .
  ?c pkg:inFlightCount 0 .
} => { ?t pkg:launchable true } .
```

Run via `reason_n3`; `reason_n3_proof()` yields the proof tree → a real "why did
the scheduler pick this bead" explanation, replacing the procedural `--explain`
flag. *(N3 scoped-negation semantics are subtle — illustrative pending a live run;
see §4.6.)*

### 4.3 Research↔bead link (the killer feature — impossible in `bd`) — VERIFY ON A LIVE RUN

**Maturity: NOT yet verified-expressible like §4.1.** Same `log:notIncludes`
caveat as §4.2 **plus** an unsolved input-data problem; this is a **Phase-3**
deliverable.

```n3
# knowledge side (projected from research/*.md + crate capabilities):
<tech:algoX> a pkg:Technique ; pkg:novel true ; dcterms:relation <research:foo> ;
  pkg:implementedBy pkg:none .
# rule: a novel, un-implemented technique with no covering task IMPLIES a candidate bead:
{ ?a a pkg:Technique ; pkg:novel true ; pkg:implementedBy pkg:none .
  ?a log:notIncludes { [] pkg:about ?a ; a pkg:Task } .
} => { ?a pkg:needsBead true ; pkg:beadKind "feature" } .
# 'mergeable' variant: ?a pkg:mergeableInto ?upstream => ?a pkg:needsUpstreamBead true .
```

**The rule PROPOSES; `bd` RECORDS.** A human/agent turns each `pkg:needsBead`
into `bd create` so the source-of-record invariant holds. Two distinct unsolved
parts, both keeping this off the turnkey list: (i) the `log:notIncludes`
scoped-negation must be verified on a live `reason_n3` run (as §4.2); (ii)
populating `pkg:novel true` / `pkg:mergeableInto` reliably is an LLM-judged
extraction problem — the rule is feasible, **the input data is the genuinely
unsolved part.** Don't present the link as turnkey.

### 4.4 SHACL task-validity (catch stale edges)

This shape queries **only** the §2.2 `pkg:blockedBy` predicate (the OWL inverse of
`pkg:dependsOn`) — it does **not** reference an undefined `pkg:blocks`, so it will
actually fire. [OPUS-4.8]

```turtle
pkg:TaskShape a sh:NodeShape ; sh:targetClass pkg:Task ;
  sh:property [ sh:path pkg:status ; sh:minCount 1 ; sh:maxCount 1 ;
                sh:in ( pkg:Open pkg:InProgress pkg:Blocked pkg:Closed pkg:Deferred ) ] ;
  sh:property [ sh:path pkg:priority ; sh:datatype xsd:integer ;
                sh:minInclusive 0 ; sh:maxInclusive 4 ] ;
  sh:sparql [ a sh:SPARQLConstraint ;
    sh:message "a closed task must not still block an open one (stale edge)" ;
    # ?o pkg:blockedBy $this  <=>  $this pkg:dependsOn ?o  (owl:inverseOf, §2.2)
    sh:select """SELECT $this WHERE {
      $this pkg:status pkg:Closed . ?o pkg:blockedBy $this . ?o pkg:status ?os .
      FILTER(?os != pkg:Closed) }""" ] .
```

This catches the exact bug `autonomous-scheduler-design.md` cites (beads left
`in_progress` after PR merge) as a declarative constraint instead of a CI script.
(If the OWL inverse closure is materialised first, the constraint may equivalently
be written over `$this pkg:dependsOn ?o`; either way it names only the two §2.2
predicates.)

### 4.5 The eval gate (when replacement may even be *considered*)

Replacement is admissible **only** if the bridge demonstrably proves, on real
data, **all four**: (a) conflict-safe concurrent task mutation across worktrees
>= Dolt's row-level three-way merge; (b) a frontier that includes the live
git/gh/nproc state; (c) ready-query latency comparable to `bd ready` (ms,
offline); (d) SessionStart/CLI ergonomics matching `bd`. Until all four pass:
**bridge only.** Per the empirical-honesty norm, "replace bd to dogfood sparq" is
attractive scope creep the measurement discipline should reject absent this
evidence.

### 4.6 Readiness ranking of the §4 items (do not present N3 as turnkey)

[OPUS-4.8] The §4 items are **not** equally ready, and the phased plan reflects
this:

| Item | Mechanism | Readiness | Phase |
|---|---|---|---|
| §4.1 dependency-frontier SPARQL | `FILTER NOT EXISTS` over `pkg:dependsOn` | **verified-expressible today** | Phase 1/2 |
| §4.4 stale-edge SHACL | SHACL-SPARQL over `pkg:blockedBy` | expressible (Core+SPARQL shipped) | Phase 2 |
| §4.2 launchable N3 trigger | `log:notIncludes` scoped negation | **VERIFY on a live `reason_n3` run** | Phase 2 |
| §4.3 research↔bead N3 rule | `log:notIncludes` + LLM-extracted input | **VERIFY live + unsolved input data** | Phase 3 |

The killer research↔bead feature (§4.3) is the **highest-value, lowest-readiness**
item — its prominence in the value argument must not be read as proximity to
turnkey.

---

## 5. Measurement protocol — falsifiable, with kill criteria

**Build on the existing instrument, don't reinvent it.**
`scripts/agent-telemetry/agent_telemetry.py` already parses Claude Code
transcript JSONL and aggregates `input_tokens`, `output_tokens`,
`cache_read_input_tokens`, `cache_creation_input_tokens` (5m vs 1h split),
cache-hit ratio, tool-call counts, duration — stdlib-only, no baked-in prices.
This is the canonical A/B accounting engine: **diff two of its JSON reports**
(read-the-docs arm vs query-the-PKG arm), do not hand-count tokens.

### 5.1 Goal 1a — token reduction (counterbalanced within-task A/B)

For each task `t` in a frozen, stratified set, run:

- **arm A (read-the-docs):** agent answers using only Read/Grep over
  `AGENTS.md`/skills/research/code.
- **arm B (query-the-PKG):** agent answers using `sparq-nlq`/`introspect`/
  `vectors` against the ingested PKG, with PKG tools **deferred-loaded** so they
  sit behind the cache breakpoint.

Pin model completions via the `sparq-nlq` record/replay trait so both arms see
identical model behaviour. Parse each transcript with `agent_telemetry.py` and
judge on **cache-discounted effective input tokens**:

```text
effective_input = 1.0*fresh_input + 0.1*cache_read + 1.25*cache_write
```

Report per-arm distributions, the paired delta per task, **and the components**
(so a "win" that is purely a cache artifact is visible). Counterbalance arm order
and warm/cold cache state to neutralise cache-warmth bias.

**Charge arm B all its costs** (excluding any is the canonical way to fake a
win): per-query KG round-trip tokens, the schema-grounding summary, every
repair-loop retry, the deferred-tool-definition tokens actually pulled in, **plus**
an amortised slice of one-time ingestion = `(ingest_build + embed) / N`. Arm A is
charged its full doc-read input including first-read cache writes.

**Pre-registered significance bar (declare BEFORE running):** a win requires the
paired-median effective-input reduction to exceed **both** (i) a **>=20% relative
reduction** (below ~20% is within the noise of the one independent memory
benchmark, not worth the maintenance) **and** (ii) **p<0.05** via Wilcoxon
signed-rank over **>=30 tasks**, with a bootstrap 95% CI on the median delta whose
lower bound is also `>0`. Report the full distribution — a couple of huge-doc
tasks can move a mean while the median is flat.

### 5.2 Goal 2a — research quality (non-circular, three axes)

Never let an LLM grade its own KG.

1. **Execution/answer accuracy** — score the final answer against held-out gold
   (set equality / answer-F1) over a **pinned corpus snapshot**; reuse the
   `genai-benchmarks` oracle-vs-end-to-end split so retrieval quality is isolable.
2. **Provenance-completeness** — parse the answer into atomic claims; score the
   fraction whose cited source (triple + source-graph IRI / `file:line`) actually
   **resolves and supports** the claim, by a **deterministic resolver**, not a
   judge.
3. **Hallucination rate** — fraction of atomic claims with **no** resolvable
   support.

**Anti-circularity guardrails (explicit):** forbid the PKG being both answer
source and grading oracle; forbid a model judging arms it can identify; forbid
counting unresolvable/non-supporting citations as "present." Require arm-blinded
grading (strip arm labels + tool traces), a held-out gold set the agents never
ingested, and inter-rater agreement (>=2 graders or human+model, report Cohen's
kappa) on a sample.

### 5.3 Break-even

```text
cost_B(N) = C_ingest + N*(C_query + C_maintain_per_use)
cost_A(N) = N*C_docread
N* = C_ingest / (C_docread - C_query - C_maintain_per_use)
```

All in cache-discounted effective tokens. `C_maintain_per_use` = amortised
re-ingestion when docs/code change (measure the real change cadence from `git log`
over a representative window). **If the denominator <= 0, break-even is infinite —
the idea never pays back regardless of any single-query saving.**

### 5.4 Kill criteria (pre-registered, mechanical)

- **KILL 1 (token):** paired-median effective reduction `<20%`, OR Wilcoxon
  `p>=0.05`, OR the bootstrap median-delta CI includes 0, OR the saving is entirely
  the cache-discount component (nominal input did **not** drop — arm A gets
  cache-warmth for free). → do not adopt for token reasons.
- **KILL 2 (break-even):** `N*` infinite, OR `N*` exceeds the realistic number of
  times a question class recurs before docs/code change forces re-ingestion. → net
  loss; kill.
- **KILL 3 (quality):** PKG arm increases hallucination rate, OR drops answer
  accuracy with the lower CI bound below the docs arm, OR has more
  present-but-unresolvable citations. → a token saving bought with worse/ungrounded
  answers fails the bar; kill.

### 5.5 Overclaim traps to bake into the harness

- **nominal vs effective tokens** — comparing raw input and ignoring arm A's ~0.1x
  cached docs is invalid by construction.
- **MCP tool-definition prefix tax** — if tools load into the prefix (not deferred)
  they re-bill every turn across every parallel agent and can erase the saving.
  Measure the config **actually intended for production**; if production can't use
  deferral (Haiku / Vertex / custom gateway), charge the tax.
- **task-set selection bias** — a point-lookup-heavy set trivially favours KG
  queries; stratify (>=4 types: point-lookup, multi-hop, synthesis-across-docs,
  negative/out-of-KG existence) and report per-stratum.
- **circular quality measurement** — the grader must be arm-blinded and must not be
  the retrieval substrate under test; gold held-out from ingestion.

### 5.6 Outputs as a verdict object (decide on the object, not a gut read)

```json
{ "token_win": "bool", "token_delta_median_pct": "float", "token_delta_ci": ["lo", "hi"],
  "quality_delta": { "exec_acc": {}, "provenance_completeness": {}, "hallucination_rate": {} },
  "break_even_N": "int", "break_even_infinite": "bool",
  "honest": "bool", "recommend_adopt": "bool" }
```

`recommend_adopt = true` requires `token_win` **AND** quality non-regression
(accuracy-delta lower CI >= 0, hallucination not increased) **AND** a finite
`break_even_N` within a realistic horizon. Per the repo's arm-on-verdict
discipline, decide on the verdict object — never on one number. Every number is
**non-canonical** (work-box) — it informs the verdict at runtime, never frozen
into committed markdown (`check-no-perf-numbers.py` would flag it). The committed
artifacts are the harness code, fixtures, and the verdict schema.

---

## 6. Phased implementation plan

**Discipline throughout:** opt-in / feature-gated crate, strict-additivity,
zero-impact on the default engine; synthetic fixtures + committed code + a verdict
object as outputs; no hard-coded perf numbers in markdown; `[OPUS-4.8]` markers on
new code/notes; capture discovered work as beads (`bd create`).

### Phase 1 — prove the head (the only phase greenlit by this record)

Goal: prove the token A/B on the 3–5 highest-value docs before any broad
ingestion. Deliverables:

1. **The ontology + SHACL shapes** — `crates/sparq-kb/ontology/pkg.ttl` (the §2.3
   sketch, including the single `pkg:dependsOn owl:inverseOf pkg:blockedBy`
   declaration) + `shapes/pkg.shapes.ttl` (§2.4 + §4.4) + a `PROVENANCE.md`
   documenting the reuse + namespace-stability decision, shipped the way
   `zkp-sparql` ships (yaml-ld/turtle source + SHACL + provenance, feature-gated).
   Mirror the vocab constants in a Rust module (`crates/sparq-kb/src/vocab.rs`)
   following `crates/sparq-trust/src/vocab.rs`. Verify every `skos:closeMatch`
   alignment against the live external ontology first.
2. **A small ingestion PoC** over the highest-value docs:
   `.beads/issues.jsonl` → `pkg:Task` triples (mechanical projection, reuse `bd`'s
   audit trail unchanged) + `AGENTS.md` gate matrix + Skills frontmatter (3–5
   targets total). Validate every entry against the SHACL shapes; reject on
   violation. `publish=false` crate; in-memory graph (oxigraph or `sparq_core::Graph`).
3. **A "query-the-PKG" skill PoC** — a thin recipe/wrapper (in-process Rust or raw
   SPARQL over HTTP) the agent calls: `introspect → ground → ask` against the
   ingested PKG, returning minimal subgraphs (always
   `close_for_vectorise(RDFS|OWL-RL)` first). Surface the executed SPARQL. The only
   query forms relied on at this phase are the §4.1-class SPARQL frontier queries
   (verified-expressible); the §4.2/§4.3 N3 rules are explicitly Phase-2/3.
4. **The token-A/B harness** — `bench/pkg-dogfood/{tasks/,run.sh,stats.py,grade.py,PREREG}`
   per §5: a tracked stratified frozen task set + pinned-corpus sha256 manifest; a
   counterbalanced A/B driver with record/replay completion pinning; an
   effective-token wrapper over `agent_telemetry.py`; a Wilcoxon + bootstrap +
   break-even stats step emitting the verdict object; a blind deterministic
   provenance grader; the pre-registration fixture fixing the thresholds before
   any run.

**Phase 1 exit gate:** run the A/B; record the (non-canonical, runtime-only)
verdict. Proceed to Phase 2 **only if** `recommend_adopt = true`. Otherwise record
the negative result and stop — this is the honest off-ramp.

> **GATE CLEARED (2026-06-21).** [OPUS-4.8] The A/B was run; the verdict is
> `recommend_adopt = true` for the PKG-answerable question class. The measured record
> is [`bench/pkg-dogfood/RESULTS.md`](../bench/pkg-dogfood/RESULTS.md) (numbers live
> there, not here). The off-ramp was not taken. The caveats in `RESULTS.md` bound the
> claim: PKG-answerable questions only, NON-CANONICAL numbers, one directional N=30 run.

### Phase 2 — full ingestion + the live-state graph (gated on Phase 1)

- Extend ingestion to crate feature graph (`Cargo.toml` + README), research design
  decisions + the citation graph (extract `[id]` refs → `pkg:Source`/`cito:`), and
  the conformance-suite floor matrix.
- Build the **live-orchestration-state** named graph (open PRs from `gh`, worktree
  unpushed-commit status, nproc, held-lanes) populated by the cheap shell probes
  `push-frontier.sh` already runs — so the frontier becomes a SPARQL/N3 query over
  (task-graph ∪ live-state).
- **Verify the §4.2 launchable N3 trigger on a live `reason_n3` run** (it is *not*
  verified-expressible like §4.1): port the rule, run it under `reason_n3`,
  confirm the `log:notIncludes` scoped-negation behaves as intended, and diff its
  output **side-by-side against `push-frontier.sh`** on real data (equivalence on
  real data is the strongest evidence for/against eventual replacement). Port the
  four hard structural rules (sq-8rpq, sq-6ip4, sq-751l, sq-p7nw) to N3 the same
  way.

### Phase 3 — research-provenance automation + the knowledge↔task link

- Automate the research-citation index (`research-citations.jsonl` via a CI grep
  script + one-time link-check) and a stale-doc audit (grep
  `not implemented`/`future work` x cross-check `published=true` crates → flag or
  convert to beads). Known high-risk docs: `zkp-query-proofs-plan.md`,
  `mpc-*-design.md`, `autonomous-scheduler-design.md`.
- **Verify and implement the §4.3 research↔bead N3 rule on a live `reason_n3` run**
  — **rule proposes, `bd` records.** This is the lowest-readiness §4 item: it
  requires both (i) the live `reason_n3`/`log:notIncludes` verification (as Phase 2)
  and (ii) the knowledge-side projection (`pkg:novel`/`pkg:mergeableInto`) via the
  LLM-judged extraction step — the genuinely unsolved input. Do not schedule it as
  if §4.1-expressible.
- Optionally build the propose-then-verify (P5, sq-0wo9e.6) pipeline as the safety
  wrapper for an LLM writing INTO the KG; and close the wiring gap by exposing
  `grounding` + `introspect` schema-cards over `sparq-server`.

### Phase 4 — bd bridge → evaluation (the eval gate from §4.5)

- Run the bridge as the operational read-model in anger; measure the four eval-gate
  criteria. Only if all four pass on real workloads is a write-path replacement even
  a candidate. Default expectation: **bridge stays, replacement does not happen** —
  the dogfooding/demo value is captured by the bridge alone.

---

## Honesty summary — what this record does and does not claim

- **Claims:** the reframe (prose→queryable facts) is sound for the
  high-frequency/high-formalizability head; the production-ready surfaces (SPARQL,
  SHACL, reason, prov, introspect, the grounding selector) are real and usable
  today; the ontology is reuse-first with only ~4 net-new terms; the bridge is the
  honest `bd` recommendation; the measurement protocol is falsifiable with
  pre-registered kill criteria.
- **Does NOT claim:** that NL→SPARQL has measured accuracy (UNMEASURED on a real
  model); that structure-aware embeddings beat plain ones (no-ops on plain data);
  that the PKG eliminates hallucination (eliminates *in-store answer fabrication*
  only — query-construction + verbalisation hallucination is *reduced*, not
  removed); that confidence-weighted querying is first-class (store+FILTER works;
  join-propagation does not); that any of this is wired over the network (in-process
  Rust + raw SPARQL/VoID today); or that a queryable PKG is an automatic win for
  agent memory in general (the project measured the counter-case for memory
  *products* on a different workload).
- **Now MEASURED (2026-06-21):** the context-window-win mechanism HAS since been
  sparq-measured for the PKG-answerable question class — the §5 A/B was run and
  `recommend_adopt = true`; the numbers live in
  [`bench/pkg-dogfood/RESULTS.md`](../bench/pkg-dogfood/RESULTS.md). The win is
  **larger than the "modest" prior expectation**, but scoped (PKG-answerable
  questions, NON-CANONICAL numbers, one directional N=30 run). The original
  pre-measurement framing above is retained as the historical plan.
- **Built since (2026-06-21):** Phase 1 LANDED on `origin/main` — the `sparq-kb`
  crate, the PKG ontology/SHACL files, the ingestion PoC, the `query-pkg` skill, and
  the `bench/pkg-dogfood/` A/B harness. The §5 falsification **has been RUN**; its
  pre-registered thresholds now have a result — [`bench/pkg-dogfood/RESULTS.md`](../bench/pkg-dogfood/RESULTS.md).
  Still Phase-2/3 beads, not built: the live-state graph and the research↔bead N3 rule.
- **Read against `origin/main`:** the §2 ontology-reuse foundation, the §3
  structure-aware vector surfaces, and the §3.2 KGE-ablation verdict are on
  `origin/main` only — a behind local `main` lacks them; they are not fabricated.

---

## Citations

- `research/security-properties-ontology-design.md` (origin/main) — the reuse-first
  discipline, `sig-impl:Assertion` reified-claim pattern, orthogonal assurance axis,
  SHACL + British-English conventions.
- `crates/sparq-trust/ontologies/zkp-sparql/` (origin/main) (`vocab/sec-prop.yaml.ld`,
  `vocab/sig-impl.yaml.ld`, `shapes/sig-impl.shapes.ttl`, `shapes/sec-prop.shapes.ttl`,
  `PROVENANCE.md`) — the Claim+Evidence+Source template and how the maintainer ships
  ontologies.
- `crates/sparq-trust/src/vocab.rs` (origin/main) — the Rust vocab-constants-module
  pattern to mirror.
- `skills/{genai-retrieval,prov-lineage,shacl-validation,vector-search,sparql-query,inference}/SKILL.md`.
- `research/{structure-aware-vectorisation,genai-nl-to-sparql,genai-design,genai-benchmarks-and-synthesis,genai-ontology-introspection,feature-research-hartig,agent-efficiency-tooling,autonomous-scheduler-design,task-tracking-best-practices,data-structures,ARCHITECTURE}.md`.
- `crates/sparq-vectors/src/{grounding,structure,encode}.rs` (origin/main;
  `structure`/`kge` features), `crates/sparq-nlq/src/{lib,link,constrain,eval}.rs`,
  `crates/sparq-introspect/src/lib.rs`, `crates/sparq-reason/`, `crates/sparq-shacl/`.
- `scripts/agent-telemetry/agent_telemetry.py` + `README.md` — the A/B measurement
  instrument; `bench/nlq/README.md` — the offline-deterministic harness template.
- `scripts/push-frontier.sh`, `.beads/issues.jsonl`, `.beads/config.yaml`,
  `AGENTS.md` — the `bd` model + frontier logic + merge discipline.
- Beads: sq-0wo9e (structure-aware-vectorisation epic), sq-0wo9e.5 (grounding,
  merged), sq-0wo9e.6 (propose-verify, OPEN), sq-0wo9e.9 (KGE ablation verdict,
  origin/main — measured numbers in the bead verdict + supporting bench history,
  not on a behind local `main`), sq-05rv (exec-accuracy harness), sq-0dksu (sec-prop
  ontology precedent), sq-lc3 (ABSTAT minimalization), sq-t80n4 (deferred QUDT render).
- External `[established]`: Neumann & Moerkotte (ICDE 2011, characteristic sets);
  Hartig tSPARQL (ESWC 2009); arXiv:2311.07914 (KGs reduce hallucination — external,
  not sparq-measured); GraphRAG-Bench (external, not sparq-measured); SPAR
  (FaBiO/CiTO); PROV-O; SKOS; DCAT; nanopublications; SPARQL-LLM (arXiv:2512.14277).
