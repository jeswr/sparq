// [FABLE-5] sq-gum8.3-fokm — Paper B5 (DRAFT): formal-ontology choice for LLM-agent
// knowledge management over a project knowledge graph (schema.org vs gUFO vs DOLCE vs no-FO).
// Single-source Typst. Numbers come ONLY from #headline(...) / #ev(...) (paper-evidence.json),
// never hard-coded. Compiles to BOTH a PDF (the download) and semantic HTML (the in-site page).
//
// HONESTY FRAMING (load-bearing, do not soften):
//   - Every accuracy figure in this paper is environment=indicative (an LLM-graded outcome of a
//     SINGLE counterbalanced run on the dev work-box). None may pass headline(); all appear via
//     ev(...) inside explicitly-labelled INDICATIVE material. The only canonical records are
//     structural corpus counts (task count, condition count).
//   - Status is DRAFT: a pilot. The venue-audit record (research/papers-venue-audit.md, missing
//     topic #1) is explicit that N=16 single-run heuristic grading is too thin for a top-tier
//     submission and that a larger pre-registered multi-run study is required first. The paper
//     says so on its face (see the Draft-status box and §7).
//   - Authored from the prose digest under the Fable provenance protocol: code-level details not
//     present in the digest are marked as DRAFT OBLIGATIONS rather than invented. Grep marker:
//     "DRAFT OBLIGATION".

#import "_lib/bench.typ": headline, ev, provenance, authors, anon

#set document(title: "Formal Ontologies for LLM-Agent Knowledge Management: schema.org vs gUFO vs DOLCE")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#align(center)[
  #text(size: 17pt, weight: "bold")[
    Formal Ontologies for LLM-Agent Knowledge Management:\
    schema.org vs gUFO vs DOLCE
  ]
]
#authors()

#align(center)[#text(style: "italic", size: 0.9em)[
  DRAFT — a controlled *pilot* study. Every accuracy figure in this paper is an _indicative_
  measurement: the outcome of one counterbalanced run of a non-deterministic LLM agent, graded
  heuristically, on a development machine. No accuracy figure is canonical evidence, none is
  co-tabulated with canonical evidence, and no statistical significance is claimed. The pilot's
  effect direction motivates — but does not yet license — a venue submission; the pre-registered
  multi-run study this draft commits to (@limitations) is the gate.
]]

== Abstract

An LLM agent that manages durable project knowledge — decisions, provenance, task structure,
design records — increasingly does so over a _project knowledge graph_ (PKG) that the agent
itself queries. Knowledge engineering doctrine says such a graph should be typed under a formal
upper ontology, but which one, and whether typing helps an LLM consumer _at all_, has not been
measured. We compare four typing conditions over the same PKG instance data — no formal ontology,
schema.org-as-top, gUFO, and DOLCE — on a corpus of #headline("fo_km.task_count") repository
knowledge-management questions answered by an LLM agent through SPARQL, in a single
counterbalanced pilot run with heuristic grading. The indicative result is counterintuitive:
only schema.org improved over the untyped baseline (#ev("fo_km.accuracy_schema_org") vs
#ev("fo_km.accuracy_no_fo") answer accuracy), DOLCE matched the baseline
(#ev("fo_km.accuracy_dolce")), and gUFO — the condition with the richest formal
axiomatization — fell _below_ it (#ev("fo_km.accuracy_gufo_lo")–#ev("fo_km.accuracy_gufo_hi")).
The ranking tracks the vocabulary's frequency in LLM training data, not the ontology's formal
richness, suggesting that for LLM-agent consumers, _fluency beats formality_ when choosing a
top-level ontology — and that an ill-chosen formal layer is worse than none.

== Introduction

Agentic LLM systems accumulate knowledge they must later retrieve precisely: which design was
chosen and why, what a measurement showed, which tasks block which. A natural substrate is a
_project knowledge graph_ — an RDF graph ingested from the project's durable knowledge surfaces
(agent instructions, skill documents, the dependency-aware task tracker) that the agent queries
with SPARQL instead of re-reading whole documents. This is the classical knowledge-management
stack with a new consumer: the reader of the ontology is no longer a human or a description-logic
reasoner but a large language model that must _translate its own natural-language framing of a
question into the graph's vocabulary_.

Knowledge-engineering doctrine holds that a knowledge graph gains from a formal upper ontology:
a top level such as DOLCE or UFO disambiguates categories (endurant vs perdurant, kind vs role),
supports consistency checking, and makes modelling decisions explicit. But every argument in
that tradition assumes a consumer that benefits from formal discipline. An LLM consumer is
different in exactly the dimension the tradition ignores: its competence over a vocabulary is a
function of how often that vocabulary occurs in its training distribution. schema.org terms are
embedded in billions of web pages; DOLCE and UFO terms live in a specialist literature. Whether
formal richness or distributional fluency dominates is an empirical question, and to our
knowledge nobody had measured it.

This paper reports a controlled pilot that measures it. We hold the PKG instance data, the agent,
the question corpus, and the grading fixed, and vary ONLY the typing overlay: (1) the untyped
incumbent graph, (2) schema.org-as-top, (3) gUFO (the lightweight OWL distillation of UFO), and
(4) DOLCE (via its DUL OWL rendering). The agent answers each of #headline("fo_km.task_count")
knowledge-management questions by querying the graph; answers are graded against gold facts.

The pilot's indicative finding is the paper's motivation and its headline-in-waiting: the only
condition that beat the untyped baseline was the formally _weakest_ one, and the formally
_richest_ overlay actively hurt. If the finding survives the scale-up study (@limitations), the practical
guidance inverts a default: when the consumer of your knowledge graph is an LLM agent, choose the
top-level vocabulary by training-data fluency first and formal richness second — and measure
before you type at all, because typing is not free.

*Contributions.* This draft substantiates the following; each claim is refutable and
forward-references its evidence. No accuracy claim is canonical, and none is a performance
(latency/throughput) claim.

- *A controlled four-condition study design for ontology choice under an LLM consumer* (@method) —
  same instance graph, same agent, same #headline("fo_km.task_count")-question corpus, same
  grader; only the committed typing overlay (#headline("fo_km.condition_count") conditions)
  varies. The corpus, overlays, task builder, validator, and grader are committed and re-runnable
  (`bench/fo-km/`), making the design a reusable resource independent of the pilot's numbers.
- *An indicative, counterintuitive pilot result* (@results) — in one counterbalanced run, schema.org
  was the only overlay to improve over no-FO (#ev("fo_km.accuracy_schema_org") vs
  #ev("fo_km.accuracy_no_fo")); DOLCE tied the baseline and gUFO fell below it. Explicitly
  indicative: single run, heuristic grading, non-deterministic agent.
- *A fluency-over-formality reading with a falsifiable mechanism* (@mechanism) — the accuracy ranking
  matches the vocabularies' web-scale training-data frequency and inverts their formal-richness
  ranking; we state the mechanism as a hypothesis the scale-up study can falsify, not as an
  established cause.
- *An honest evidence protocol for LLM-graded results* (@grading, @limitations) — every accuracy figure is
  routed through an evidence file that labels it `indicative` and a build gate that structurally
  prevents it from ever being cited as canonical headline evidence; the pre-submission
  obligations (pre-registration, multi-run, human adjudication) are enumerated on the face of
  the paper.

== Related work

*Upper ontologies.* DOLCE (Masolo et al. 2003; Borgo et al. 2022) and UFO (Guizzardi 2005;
Guizzardi et al. 2022) are the two foundational ontologies with the deepest methodological
literature; gUFO (Almeida et al. 2019–) is UFO's lightweight OWL 2 DL rendering intended exactly
for typing knowledge graphs, and DOLCE's DUL rendering plays the same role. BFO (Arp, Smith &
Spear 2015) and SUMO (Niles & Pease 2001) serve adjacent niches. The selection
literature in this tradition compares ontologies by formal criteria — categorial coverage,
axiomatic rigour, modelling guidance — always assuming a human modeller or a DL reasoner as the
consumer. We are not aware of prior work that selects an upper ontology by measured downstream
task accuracy of an LLM consumer; this is the gap the pilot addresses.

*schema.org.* schema.org (Guha, Brickley & Macbeth 2016) was designed with the opposite
priorities: a shallow, wide,
deliberately informal vocabulary optimised for mass adoption by web authors and consumption by
search engines. Its formal weaknesses (thin axiomatization, permissive domains/ranges) are
documented and real. Precisely because of its adoption, however, its terms are among the
highest-frequency structured-data vocabulary in any web-scale training corpus — the property our
mechanism hypothesis (@mechanism) turns on.

*LLMs and knowledge graphs.* The LLM+KG literature (Pan et al. 2024) spans KG-augmented
prompting (Baek et al. 2023), iterative graph reasoning (Sun et al. 2024), and graph-indexed
retrieval (Edge et al. 2024). Closest to us, Sequeda, Allemang & Jacob (2023) and Allemang &
Sequeda (2024) show that putting an ontology (and a SPARQL-over-ontology layer) between an LLM
and enterprise data substantially improves question-answering accuracy over schema-less access.
Their comparison is _ontology vs no ontology_ for one fixed ontology; ours holds "there is an
ontology" fixed and varies _which_ ontology, finding the choice can swing the outcome from a
gain to a regression. Work on LLMs as ontology engineers (Babaei Giglou et al. 2023; Caufield
et al. 2024) measures the converse direction (LLMs producing ontology artifacts) and does not
compare upper ontologies as consumers' vocabularies. Agent-memory systems (Packer et al. 2023;
Park et al. 2023) and personal knowledge graphs (Balog & Kenter 2019) motivate the PKG substrate
but do not type it formally.

// DRAFT OBLIGATION (sq-gum8.3-fokm): verify every citation key below against the actual
// bibliography before submission; the reference list is drafted from the author's knowledge of
// the literature and must be checked for venue/year/author accuracy.

*Positioning.* The nearest prior art establishes that structure helps LLM question answering
(Sequeda et al. 2023; Allemang & Sequeda 2024). Our delta is the controlled comparison _across_ upper-ontology
choices with an untyped control, and the negative half of the finding — that a formally rich
overlay can land _below_ the untyped baseline — which we have not found reported elsewhere.

== Study design <method>

=== Substrate: a project knowledge graph with a real workload

The substrate is the PKG of a working RDF-engine repository: an RDF graph ingested from the
project's durable knowledge surfaces — the agent-instructions document, the per-surface skill
documents, and the dependency-aware task tracker. The graph is in production use by the
project's own agents, which answer provenance and status questions over it via SPARQL rather
than re-reading documents. The workload is therefore not synthetic: the question corpus (@corpus)
is drawn from the question shapes those agents actually pose ("where was X decided", "what is
the status and provenance of Y", "what depends on task Z").

=== Conditions: four committed typing overlays over one instance graph

The experimental variable is a _typing overlay_: a Turtle document asserting class membership
(and, where the ontology provides them, object-property specialisations) for the PKG's instance
data under one top-level vocabulary. #headline("fo_km.condition_count") overlays are committed
under `bench/fo-km/overlays/`:

- *no-FO* (`no-fo.ttl`) — the incumbent untyped control: the PKG as ingested, with only its
  native ad-hoc vocabulary.
- *schema.org* (`schema-org.ttl`) — instances typed under schema.org classes
  (e.g. `schema:SoftwareSourceCode`, `schema:CreativeWork`, `schema:Action`) with schema.org
  properties layered over the native predicates.
- *gUFO* (`gufo.ttl`) — instances typed under gUFO's OWL rendering of UFO categories
  (kinds, phases, roles, relators, situations).
- *DOLCE* (`dolce-dul.ttl`) — instances typed under the DOLCE+DnS Ultralite (DUL) rendering
  (endurant/perdurant/quality/abstract partitions and DUL's description-and-situation pattern).

The instance data is identical across conditions; only the overlay differs. Overlay authoring
followed the target ontology's own modelling guidance.

// DRAFT OBLIGATION (sq-gum8.3-fokm): the digest does not carry the overlay-construction
// protocol details (who authored, cross-review, ontology versions/IRIs pinned). Recover them
// from bench/fo-km/README.md + overlays before submission; the overlay-quality-asymmetry
// threat in §6 must cite the actual protocol.

=== Task corpus and run protocol <corpus>

The corpus is #headline("fo_km.task_count") knowledge-management questions
(`bench/fo-km/tasks.jsonl`, one task per line, schema-validated by
`bench/fo-km/validate_tasks.py`), each pairing a natural-language question about the repository's
knowledge estate with gold facts an answer must contain. The agent answers each question by
introspecting the graph's vocabulary, composing SPARQL against the overlaid PKG, and synthesising
an answer from the bindings. Condition assignment was counterbalanced across the corpus so that
no condition saw a systematically easier question slice, and the pilot consists of a _single_
counterbalanced run — a deliberate scope choice for a pilot, and the study's primary limitation
(@limitations).

// DRAFT OBLIGATION (sq-gum8.3-fokm): pin the exact agent model identifier, run date, and
// prompting scaffold from bench/fo-km/RESULTS.md — the digest records the design and the
// outcomes but not the model id. The fluency mechanism (§5) is model-relative, so the
// scale-up study must report it per model.

=== Grading and the evidence protocol <grading>

Answers are graded by a heuristic grader (`bench/fo-km/analyze.py`) that checks an answer for
the task's gold facts; accuracy is the graded-correct fraction of the corpus. Heuristic grading
is cheap and reproducible but is a known weak point (it can reward keyword overlap and miss
paraphrase), so we treat the resulting figures as _indicative only_ and route every one through
the project's evidence file, where each record carries `environment: "indicative"`, the run
provenance, and the grader source. The build machinery structurally refuses to render an
indicative record as headline evidence — the same gate that keeps non-canonical timing out of
the project's other papers keeps single-run LLM-graded accuracy out of any headline here. The
only canonical records this paper cites are the two structural corpus counts
(#headline("fo_km.task_count") tasks, #headline("fo_km.condition_count") conditions).

#provenance("fo_km.task_count")

== Pilot results (indicative) <results>

#figure(
  table(
    columns: 3,
    align: (left, right, left),
    table.header[Condition][Answer accuracy][Relation to untyped baseline],
    [schema.org-as-top], [#ev("fo_km.accuracy_schema_org")], [above],
    [no-FO (untyped control)], [#ev("fo_km.accuracy_no_fo")], [—],
    [DOLCE (DUL overlay)], [#ev("fo_km.accuracy_dolce")], [ties],
    [gUFO], [#ev("fo_km.accuracy_gufo_lo")–#ev("fo_km.accuracy_gufo_hi")], [below],
  ),
  caption: [
    INDICATIVE pilot measurements — not canonical evidence. Graded-correct fraction of the
    #headline("fo_km.task_count")-task corpus in ONE counterbalanced run of a non-deterministic
    LLM agent with heuristic grading on a development machine (`environment: indicative` in the
    project's evidence file; see @grading). No significance test is possible or claimed at this
    scale. The gUFO row shows the two figures the results record reports for that condition.
  ],
)

Three readings, each stated at pilot strength:

*Only the fluent vocabulary helped.* schema.org's #ev("fo_km.accuracy_schema_org") against the
baseline's #ev("fo_km.accuracy_no_fo") is an absolute gap of about a fifth of the corpus — on
#headline("fo_km.task_count") tasks, roughly three additional questions answered correctly. At
this N a gap of that size is directionally interesting and nothing more; we flag it as the
effect the scale-up study must confirm or kill.

*Formal richness did not convert into accuracy.* DOLCE — with a categorial apparatus far richer
than schema.org's — tied the untyped control (#ev("fo_km.accuracy_dolce") vs
#ev("fo_km.accuracy_no_fo")). Whatever value its distinctions add for a human modeller, the LLM
consumer extracted none of it on this workload.

*The richest overlay was worse than nothing.* gUFO's two graded figures
(#ev("fo_km.accuracy_gufo_lo") and #ev("fo_km.accuracy_gufo_hi")) both fall below the untyped
baseline. Typing the graph is not a monotone improvement: an overlay whose vocabulary the agent
handles poorly _displaces_ the native terms the agent handled adequately, and the net effect can
be negative.

== Why fluency, not formal richness — a falsifiable hypothesis <mechanism>

The accuracy ranking (schema.org #sym.gt no-FO #sym.approx DOLCE #sym.gt gUFO) inverts the
formal-richness ranking of the same vocabularies and matches their plausible frequency ranking
in web-scale training corpora: schema.org is embedded in billions of pages as JSON-LD and
microdata; DOLCE/DUL and gUFO occur in a specialist ontology-engineering literature; gUFO — the
most recent and most technical vocabulary of the three — is plausibly the rarest.

The mechanism we hypothesise is _query-side vocabulary translation_. To answer a KM question the
agent must map its natural-language framing onto graph terms. A high-fluency vocabulary acts as
a set of familiar anchors: `schema:Action`, `schema:about`, `schema:isPartOf` mean to the model
roughly what they mean in the graph, so the composed SPARQL binds the right classes on the first
attempt. A low-fluency vocabulary (gUFO kinds/phases/relators; DOLCE endurants/perdurants) inserts
an unfamiliar indirection layer between the question and the data: the agent must first infer
what the ontology means by its terms — a step where a rare technical vocabulary invites exactly
the near-miss (a plausible-but-wrong class pick) that grades as a wrong answer.

We state this as a hypothesis, not a demonstrated cause: the pilot measured outcomes, not
mechanism. It is falsifiable in at least three ways the scale-up study can operationalise:
(a) a per-error analysis attributing failures to vocabulary-translation misses vs retrieval or
synthesis misses; (b) a fluency probe (does the model define/instantiate each vocabulary term
correctly in isolation?) whose per-term scores should predict per-task outcomes if the mechanism
is real; (c) an ablation that renames gUFO/DOLCE classes to descriptive plain-English aliases
while keeping the class _structure_ — if fluency is the driver, aliasing should recover most of
the gap; if formal structure is the driver, it should not.

The hypothesis is also explicitly _model-relative and time-indexed_: fluency is a property of a
training distribution, not of an ontology. A future model trained on a corpus where gUFO is
well-represented could invert the ranking — which is a further reason the finding must be
reported per model and date, never as a timeless property of the ontologies.

== Limitations and threats to validity <limitations>

This section is the paper's honesty boundary; the draft-status box below restates the
pre-submission obligations operationally.

- *Pilot scale.* #headline("fo_km.task_count") tasks, one counterbalanced run. The headline gap
  is about three tasks; single-run variance of a non-deterministic agent could account for a material
  share of it. No confidence intervals or significance tests are possible, and none are claimed.
- *Heuristic grading.* The grader checks gold-fact presence heuristically; it can both
  over-credit keyword overlap and under-credit paraphrase. Grading error is not guaranteed to be
  condition-independent (conditions change the agent's phrasing), which is a bias channel, not
  just noise. The scale-up study needs human-adjudicated (or at minimum adversarially-validated)
  grading.
- *One model family, one date.* The fluency mechanism is model-relative (@mechanism); the pilot ran one
  agent configuration. Generalisation across model families is untested.
- *One domain.* The PKG is a software-project knowledge graph; KM over other domains (scientific,
  enterprise) may reward ontological precision differently.
- *Overlay-quality asymmetry.* The overlays were hand-authored; despite following each ontology's
  own modelling guidance, the authors' own fluency asymmetry (schema.org is easier to apply
  well) could contaminate the comparison. The aliasing ablation (ablation (c) in @mechanism) and independent overlay
  review are the countermeasures.
- *Deferred second metric.* The pilot's planned graph-representational metric (a
  knowledge-graph-embedding evaluation over the same overlays) was deferred for compute and is
  not reported; the accuracy metric stands alone.
- *Non-canonical environment.* All measurements executed on a development work-box; per the
  project's standing evidence policy such numbers are indicative and are labelled so throughout.

#block(
  inset: 8pt,
  stroke: 0.5pt + gray,
  radius: 4pt,
)[
  *Draft status — obligations before any venue submission* (tracked as bead `sq-iw378`):
  (1) re-verify the DOLCE figure and the two gUFO figures against `bench/fo-km/RESULTS.md`
  (this draft was authored from the audited prose digest of that record, under a protocol that
  forbade re-opening repository files; two records carry explicit verify-before-submission
  notes); (2) pin the agent model id, run date, prompting scaffold, and overlay-construction
  protocol from the bench records; (3) run the pre-registered multi-run scale-up with
  human-adjudicated grading, CIs, and the mechanism probes (@mechanism); (4) run the deferred KGE metric;
  (5) verify all bibliography entries.
]

== Conclusion

We measured a question knowledge-engineering doctrine had answered only by argument: whether a
formal upper ontology helps when the consumer of the knowledge graph is an LLM agent, and which
one. In a four-condition counterbalanced pilot over a production project knowledge graph
(#headline("fo_km.task_count") tasks, #headline("fo_km.condition_count") committed overlay
conditions), the indicative answer inverted the doctrine's ordering: schema.org-as-top — the
formally weakest vocabulary — was the only condition to beat the untyped baseline
(#ev("fo_km.accuracy_schema_org") vs #ev("fo_km.accuracy_no_fo")), DOLCE tied it, and gUFO
landed below it. The ranking tracks training-data fluency, not formal richness, and we give a
falsifiable mechanism (query-side vocabulary translation) plus the ablations that would confirm
or kill it. The pilot is honest about being a pilot: every accuracy figure is single-run,
heuristically graded, environment-labelled indicative, and structurally barred from ever being
cited as canonical evidence; the pre-registered scale-up is the gate to any venue. If the effect
survives, the practical rule for anyone typing a knowledge graph for an LLM agent is simple and
slightly uncomfortable: choose the vocabulary the model already speaks — and measure, because an
ontology the model does not speak is worse than none.

#line(length: 100%)
#text(size: 0.9em)[
  *References* — draft bibliography; verify before submission.

  #set par(justify: false)
  / Masolo et al. 2003: _WonderWeb Deliverable D18: Ontology Library_ (DOLCE). Masolo, Borgo,
    Gangemi, Guarino, Oltramari.
  / Borgo et al. 2022: "DOLCE: A Descriptive Ontology for Linguistic and Cognitive Engineering."
    Borgo, Ferrario, Gangemi, Guarino, Masolo, Porello, Sanfilippo, Vieu. _Applied Ontology_ 17(1).
  / Guizzardi 2005: _Ontological Foundations for Structural Conceptual Models._ PhD thesis,
    University of Twente.
  / Guizzardi et al. 2022: "UFO: Unified Foundational Ontology." Guizzardi, Botti Benevides,
    Fonseca, Porello, Almeida, Prince Sales. _Applied Ontology_ 17(1).
  / Almeida et al. 2019–: _gUFO: A Lightweight Implementation of the Unified Foundational
    Ontology._ Almeida, Guizzardi, Falbo, Prince Sales. OWL vocabulary.
  / Arp, Smith & Spear 2015: _Building Ontologies with Basic Formal Ontology._ MIT Press.
  / Niles & Pease 2001: "Towards a Standard Upper Ontology." _FOIS_.
  / Guha, Brickley & Macbeth 2016: "Schema.org: Evolution of Structured Data on the Web."
    _CACM_ 59(2).
  / Pan et al. 2024: "Unifying Large Language Models and Knowledge Graphs: A Roadmap."
    Pan, Luo, Wang, Chen, Wang, Wu. _IEEE TKDE_.
  / Baek et al. 2023: "Knowledge-Augmented Language Model Prompting for Zero-Shot Knowledge
    Graph Question Answering." Baek, Aji, Saffari.
  / Sun et al. 2024: "Think-on-Graph: Deep and Responsible Reasoning of Large Language Model on
    Knowledge Graph." _ICLR_.
  / Edge et al. 2024: "From Local to Global: A Graph RAG Approach to Query-Focused
    Summarization."
  / Sequeda et al. 2023: "A Benchmark to Understand the Role of Knowledge Graphs on Large
    Language Model's Accuracy for Question Answering on Enterprise SQL Databases." Sequeda,
    Allemang, Jacob.
  / Allemang & Sequeda 2024: "Increasing the LLM Accuracy for Question Answering: Ontologies to
    the Rescue!"
  / Babaei Giglou et al. 2023: "LLMs4OL: Large Language Models for Ontology Learning." Babaei
    Giglou, D'Souza, Auer. _ISWC_.
  / Caufield et al. 2024: "Structured Prompt Interrogation and Recursive Extraction of Semantics
    (SPIRES)." _Bioinformatics_.
  / Packer et al. 2023: "MemGPT: Towards LLMs as Operating Systems."
  / Park et al. 2023: "Generative Agents: Interactive Simulacra of Human Behavior." _UIST_.
  / Balog & Kenter 2019: "Personal Knowledge Graphs: A Research Agenda." _ICTIR_.
]

#if not anon [
  #line(length: 100%)
  #text(size: 0.8em, fill: gray)[
    sparq project · the study corpus, overlays, builder, validator, grader, and results record
    live under `bench/fo-km/` (`tasks.jsonl`, `overlays/`, `build_tasks.py`,
    `validate_tasks.py`, `analyze.py`, `RESULTS.md`). Numbers in this document are injected at
    build time from the paper-bound evidence file (accuracy records are
    `environment: indicative`; only the structural corpus counts are canonical); see the
    provenance stamp on the published page. Draft authored under the Fable provenance protocol
    from the audited prose digest (`research/papers-venue-audit.md`).
  ]
]
