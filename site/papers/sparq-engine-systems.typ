// [OPUS-4.8] sq-gum8.8 — Paper P3: the engine systems paper (the maintainer's extreme-SOTA
// axis). Positioning: research/paper-selection.md §3.4 (spine) + §3.5 (WASM folded) + §3.6
// (conformance folded) + §5-P3. Venue plan: PVLDB Vol 20 rolling (monthly through 2027-03-01);
// second choice ICDE 2027 R2 (2026-11-11) or EDBT 2027 cycle 3 (2026-10-07, stretch). CIDR 2027
// (2026-08-04) is deliberately NOT targeted — an unbenchmarked systems paper must not be rushed.
//
// STATUS: FIRST DRAFT — the SUBMISSION-GATING evaluation has NOT run. This paper's headline
// claim is a PERFORMANCE + memory claim, and the honesty invariant of the paper factory is that
// no performance headline may appear before a canonical (deterministic, machine-independent, or
// canonical-EC2-host) measurement exists. Accordingly EVERY performance/memory figure in this
// draft is an EXPLICIT at-risk slot rendered by the `at_risk(...)` block below — never a typed
// number and never a `#headline(...)` call against a record that does not yet exist. The four
// gating experiments (canonical EC2 baselines vs NATIVE non-Docker competitors; a Sparqloscope
// run; a qEndpoint memory-honesty comparison; the substrate zero-overhead measurement on the
// canonical host) are tracked by bead sq-vw3ax.12 and are NOT run by this drafting bead; §8 fixes
// their methodology so a later measurement cannot steer the framing.
//
// Single-source Typst. Registry + evidence-record wiring is the DOWNSTREAM bead sq-gum8.9 — this
// file deliberately touches neither site/src/data/papers.ts nor site/src/data/paper-evidence.json.
// Proposed evidence keys for sq-gum8.9 to wire when the gating experiments land (each carries an
// explicit `environment`; a PERFORMANCE key MUST be environment="canonical" or it cannot be cited
// via #headline and must stay an at-risk callout):
//   engine.wasm_bundle_bytes             — deterministic CI ratchet (bench/perf-baseline.json wasm_bundle_bytes); environment="canonical"
//   engine.store_bytes_per_triple        — deterministic layout ratchet (store_bytes_per_triple); environment="canonical"
//   engine.dict_bytes_per_term           — deterministic layout ratchet (dict_bytes_per_term); environment="canonical"
//   engine.conformance_families          — count of spec families with a pinned conformance floor (crates/sparq-conformance scoreboard)
//   engine.conformance_<family>_floor    — per-family pinned pass floor (SPARQL / OWL-RL / SHACL / RSP / EL / QL / geo / …)
//   bench.sp2bench_<engine>_geomean      — canonical-EC2 wall-clock vs a NATIVE competitor; environment="canonical" ONLY
//   bench.sparqloscope_<engine>          — Sparqloscope run on the canonical host; environment="canonical" ONLY
//   bench.qendpoint_committed_bytes      — qEndpoint memory-honesty comparison (committed RSS, stated method); environment="canonical" ONLY
//   substrate.overhead_<kernel>          — substrate zero-overhead delta on the canonical host; environment="canonical" ONLY
// The structural counts written in prose below (six permutations, the family-crate list, the
// deployment-surface crate list) are architecture facts traced inline to their crate, not
// measurements; they migrate to #headline(...)/#ev(...) accessors when sq-gum8.9 wires records.

#import "_lib/bench.typ": headline, ev, provenance, authors, anon, paper_heading_numbering

#set document(title: "One Substrate, Many Standards: An Out-of-Core SPARQL Engine and a Measured Zero-Overhead Evaluation Core Across the W3C/OGC Spec Families")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: paper_heading_numbering)

// A loud, unmissable at-risk block for every element gated on the (unrun) canonical evaluation.
#let at_risk(body) = block(
  width: 100%,
  inset: 8pt,
  radius: 4pt,
  stroke: 1pt + rgb("#b45309"),
  fill: rgb("#fff7ed"),
)[
  #text(weight: "bold", fill: rgb("#92400e"))[AT-RISK — gated on the canonical evaluation (bead `sq-vw3ax.12`); no number appears here until it does.]
  #body
]

#align(center)[
  #text(size: 17pt, weight: "bold")[
    One Substrate, Many Standards: An Out-of-Core SPARQL Engine and a Measured
    Zero-Overhead Evaluation Core Across the W3C/OGC Spec Families
  ]
]
#authors()

#align(center)[#text(style: "italic", size: 0.9em)[
  *DRAFT — in progress.* This paper's headline is a performance and memory claim, and its
  submission-gating evaluation has _not_ run. Every performance and memory figure below is an
  explicitly marked at-risk slot; no benchmark number is claimed anywhere in this document. The
  evaluation methodology (§8) — canonical-host baselines against _native_ (non-container)
  competitors, a Sparqloscope run, and a memory-accounting-honest comparison against qEndpoint —
  is fixed here _before_ the measurement so results cannot steer the framing. The architecture,
  the spec-family breadth, and the correctness evidence (§3–§7) are real and traceable to the
  codebase today; the competitive numbers are not, and are not asserted.
]]

#heading(level: 2, numbering: none, outlined: false)[Abstract]

RDF triple stores have historically forced a three-way trade-off: an engine is fast (QLever,
Virtuoso, RDF-3X), or it is broad in reasoning and standards coverage (RDFox, GraphDB, Jena), or
it fits large graphs on modest hardware (HDT, qEndpoint) — but a single system rarely claims all
three, and no peer-reviewed engine paper combines SPARQL query evaluation, the OWL 2 profiles,
RIF-Core, RDF stream processing, GeoSPARQL, and SHACL over one _measured shared_ evaluation core.
We describe an engine whose organising thesis is that this trade-off is largely an artifact of
duplicated evaluation machinery. The system stores triples out-of-core as exhaustively-permuted,
block-compressed, memory-mapped indexes (six permutations in the RDF-3X lineage @rdf3x @hexastore)
with inline-tagged term identifiers (the QLever/Virtuoso encoding lineage @qlever @virtuoso), so
that both the working set and the dictionary spill to disk and only the touched pages are resident.
Its query evaluator combines binary joins with worst-case-optimal Leapfrog Triejoin @triejoin
@wcoj-ngo and bind joins over a monomorphic, allocation-lean id-level kernel. The paper's central
_systems_ contribution is that this kernel — row layout, the numeric tower, the join families, and
SPARQL's total term order — is extracted into a single leaf crate consumed unchanged by the query
engine _and_ by every standards family (the OWL 2 RL/EL/QL reasoners, RIF-Core, an RSP-QL stream
processor, GeoSPARQL, and SHACL), so breadth is not a bag of independent subsystems but one core
carrying many standards. We frame breadth as a _measured-substrate_ claim, not a feature list:
the extraction is validated as behaviour-neutral by deterministic byte-exact ratchets and a
cross-family differential conformance scoreboard, and the marginal cost of the shared kernel over
the engine's hand-tuned original is a measurement (§8), not an assertion. We are explicit about
what is _not_ yet measured: the competitive performance and memory-frugality numbers that a
reviewer will demand are gated on a canonical-host evaluation against native competitors, a
Sparqloscope run, and a memory-accounting-honest comparison against qEndpoint; this draft fixes
that methodology and states, everywhere a number would go, that the number does not yet exist.

== Introduction

Three properties of an RDF engine tend to trade against one another. _Speed_ — the property that
made QLever @qlever, Virtuoso @virtuoso, and the RDF-3X line @rdf3x competitive — usually buys its
index and cache structure at the cost of memory and of a narrow query-only feature set. _Breadth_
— OWL profile reasoning, rule interchange, stream processing, geospatial querying, shape
validation — is the province of RDFox @rdfox, GraphDB @owlim, and Jena @jena, typically as
separately engineered subsystems. _Frugality_ — serving a graph far larger than RAM on commodity
hardware — is the explicit pitch of HDT @hdt and qEndpoint @qendpoint, usually for a restricted
query surface. Systems that claim two of the three are common; a single system that credibly
claims all three, over _one_ evaluation core, is not something the peer-reviewed record contains:
engine systems papers exist for QLever, RDFox, MillenniumDB @millenniumdb, Tentris @tentris,
Virtuoso, Jena, and OWLIM/GraphDB, but none combines query, the OWL profiles, RIF, RSP, geo, and
SHACL over a demonstrably shared substrate, and neither Oxigraph @oxigraph nor Stardog has a
peer-reviewed engine paper at all.

This is a systems paper about how one engine reaches for all three at once, and — as important —
about the honesty discipline required to claim it. Breadth alone is not a research contribution:
GraphDB, Stardog, and Jena already ship comparable standards coverage commercially, so "we also
support SHACL and geo" reads as product engineering. Memory frugality alone invites an accounting
attack: an mmap-resident store that reports a small _committed_ footprint while relying on the OS
page cache can be dismissed as measuring the wrong thing. Our claim is therefore deliberately
narrow and falsifiable: _the standards breadth costs one evaluation core, not many, and that core
costs no measured marginal overhead over the engine's own hand-tuned evaluation_. The word
_measured_ is load-bearing and its measurement (§8) is gated, so this draft does not yet assert
the number.

We contribute:

+ *An out-of-core storage and evaluation architecture* (§3, §4): six block-compressed,
  memory-mapped permutation indexes in the RDF-3X/Hexastore lineage @rdf3x @hexastore; a dictionary
  that spills to disk with inline-tagged identifiers in the QLever/Virtuoso lineage @qlever
  @virtuoso; and a query evaluator that mixes binary joins, worst-case-optimal Leapfrog Triejoin
  @triejoin @wcoj-ngo, and bind joins over dictionary-encoded ids. The architecture is real and
  traceable to the codebase; its _competitive_ performance is the at-risk part (§8).
+ *A measured zero-overhead shared evaluation substrate* (§5): the id-level row layout, numeric
  tower, join kernels, and SPARQL total-order comparator extracted into a single leaf crate, and
  consumed _unchanged_ by the query engine and by every standards family. This is the paper's
  central systems contribution and the one that makes the breadth claim a claim about engineering
  economy rather than surface area.
+ *Breadth framed as a measured-substrate claim, not a feature list* (§6): the OWL 2 RL/EL/QL
  reasoners, RIF-Core, an RSP-QL stream processor, GeoSPARQL, and SHACL, each shown to route its
  evaluation through the shared kernel, with a deployment surface (§7) that compiles the same core
  to WebAssembly under a deterministic bundle-size ratchet.
+ *A correctness-evidence layer* (§7): a cross-family differential conformance scoreboard that
  pins a machine-checked pass floor per standard, so that the substrate extraction is validated
  as behaviour-neutral rather than merely asserted to be.
+ *A fixed evaluation methodology* (§8) for the four submission-gating experiments, committed in
  this draft before any of them runs, with the memory-accounting method stated explicitly because
  it is the most contestable number in the paper. #text(weight: "bold")[No benchmark number is
  claimed in this draft.]

== Storage: out-of-core, exhaustively permuted, memory-mapped <storage>

*Six permutations (RDF-3X lineage).* A triple $(s, p, o)$ of dictionary-encoded identifiers is
stored in all six column orderings — SPO, SOP, PSO, POS, OSP, OPS — so that any triple pattern
with any subset of bound positions is answered by a contiguous range scan over exactly one
permutation, and any two patterns can be merge-joined on a shared variable without an intermediate
sort. Exhaustive permutation indexing is the design of RDF-3X @rdf3x and Hexastore @hexastore; we
adopt it and do not claim it. Each permutation is a _block-compressed, random-accessible_ sorted
run: the column-0 ids of each block form a directory over which a pattern's bound prefix is
binary-searched, and only the addressed blocks are decoded. The permutations are memory-mapped
files, so the resident set is the set of _touched_ blocks, not the graph.

*Inline-tagged identifiers (QLever/Virtuoso lineage).* The dictionary maps RDF terms to integer
ids, and — following the encoding lineage of QLever @qlever and Virtuoso @virtuoso — a class of
values (small integers and other compactly-representable terms) is _tagged inline_ in the
identifier itself rather than stored in the dictionary, so common numeric and typed-literal
operations read the value straight from the id without a dictionary indirection. Terms that are
not inlined spill to an on-disk record store; the dictionary keeps a memory-mapped index and does
not hold the term bytes in RAM. The consequence is the frugality property: both the triple store
_and_ the dictionary are out-of-core, and a graph substantially larger than physical memory is
queryable with a resident set bounded by the working set of the query, not the size of the data.

*Deterministic layout ratchets.* The layout has two machine-independent, deterministic size
metrics — the store's bytes-per-triple and the dictionary's bytes-per-term — pinned as CI
ratchets, so a regression in the on-disk footprint fails the build. These are _deterministic_
(a fixed function of the input, identical on any machine), which is exactly why they may become
canonical headline evidence when wired (§8); the draft names them without a value.

#at_risk[
  The competitive storage-footprint and query-latency numbers — bytes-per-triple against HDT and
  qEndpoint, cold- and warm-cache scan latency against native QLever and Virtuoso — are gated on
  the canonical-host evaluation (§8). The _deterministic_ layout ratchets above are machine-
  independent and will be cited via the paper-factory canonical accessor once wired
  (`engine.store_bytes_per_triple`, `engine.dict_bytes_per_term`); the _competitive_ comparison
  numbers are canonical-EC2 measurements that do not yet exist.
]

== Query evaluation: mixed binary, worst-case-optimal, and bind joins <eval>

The evaluator plans a basic graph pattern over the six permutations and executes it with three
join strategies chosen by the planner. _Binary joins_ (merge and hash) handle the common
two-relation case; the sorted permutations make a merge join on a shared variable an id-level
linear scan with no re-sort. _Worst-case-optimal_ multiway joins use Leapfrog Triejoin
@triejoin — the algorithm of Veldhuizen (ICDT 2014), whose optimality is characterised by the
AGM bound and the broader worst-case-optimal-join theory @wcoj-ngo — for the cyclic and
many-way star/path patterns where a binary plan would materialise an intermediate result
asymptotically larger than the output. _Bind joins_ push bindings from an outer pattern into an
inner one, the strategy that also carries the engine's federation and reasoner integration. All
three operate over the same id-level row representation (a small-vector of integer ids) and the
same total-order comparator, so no join strategy pays a representation-conversion tax to hand
results to another. This uniformity is not incidental — it is the property §5 turns into the
shared substrate.

The evaluator's counting and streaming behaviour is lazy: aggregate and cardinality queries that
the planner recognises as not needing full materialisation are answered by streaming counts over
the addressed permutation blocks, so a `COUNT`-shaped query need not resident-load the graph.

#at_risk[
  Per-operator throughput, the crossover point at which the worst-case-optimal path beats a binary
  plan on cyclic queries, and end-to-end query latency against native competitors are gated on the
  canonical-host evaluation (§8). No such number appears in this draft.
]

== The shared evaluation substrate <substrate>

The central systems contribution is that the parts of evaluation common to _every_ standards
family — the id-tuple row layout, the numeric value tower over dictionary-encoded literals, the
join kernels, and SPARQL's total order over RDF terms — live in exactly one place and are consumed
unchanged by all of them.

*Why one core is not the default.* The honest starting point (recorded in the project's design
audit, `research/shared-eval-substrate.md`) is that this was _not_ the state of the code: the
query engine and the OWL RL materialiser each had their own join implementation — the engine's
merge/hash/bind/Leapfrog families, and the reasoner's own hash-map adjacency indexes and
union-find. Two independent join implementations is the norm across the field, and it is why
breadth usually means duplicated machinery. The substrate work is an _extraction and unification_,
not a relabelling: the common kernels are lifted into a single leaf crate that depends only on the
core term/dictionary layer, placed _below_ the query engine in the dependency graph so that a
reasoner can reach the kernel without taking a dependency on the whole engine (which would pull the
planner, the SPARQL protocol client, and the serializers into a lean reasoner or a browser bundle).

*Monomorphic and allocation-lean.* The kernel is monomorphic over the concrete id and row types
with no dynamic dispatch on the hot path, so the shared abstraction is a source-level unification,
not a runtime indirection. The extraction is therefore expected to be _behaviour-_ and
_performance-neutral_ by construction — a code-move-and-generalise, not a rewrite — and that
expectation is exactly what the measurement in §8 is designed to confirm or falsify.

*How zero-overhead is validated, not asserted.* Two mechanisms make "zero measured marginal
overhead" a testable claim rather than a slogan. First, the deterministic byte-exact ratchets
(the WebAssembly bundle size, the store and dictionary layout metrics) are unchanged by the
extraction — a behaviour-altering move would move a byte and fail the ratchet. Second, the
cross-family differential conformance floors (§7) are bit-stable across the extraction — a
semantics-altering move would change a conformance result. The remaining question — whether the
generalised kernel costs any _wall-clock_ over the engine's hand-specialised original — is a
micro-benchmark on the canonical host, and it is the substrate half of the §8 gate.

#at_risk[
  The substrate zero-overhead result — the wall-clock delta of the shared kernel versus the
  engine's pre-extraction hand-tuned join/numeric/compare loops, per kernel, on the canonical host
  — is the load-bearing measurement for the "zero-overhead" half of the paper's claim and is gated
  on bead `sq-vw3ax.12`. It is reported nowhere in this draft; §8 fixes how it will be measured
  (proposed keys `substrate.overhead_<kernel>`, environment="canonical").
]

== Breadth as a measured-substrate claim <breadth>

We deliberately do not present breadth as a coverage checklist. The claim is structural: each
standards family below is implemented as a consumer of the shared kernel of §5, so that adding a
standard adds a _front end_ (its syntax, its entailment or validation rules, its result shape) over
a core that already exists, rather than a new evaluation engine.

- *SPARQL 1.1 / 1.2 query* @sparql11-query @sparql12-query — the reference consumer; the kernel is
  the engine's own evaluation core.
- *OWL 2 profiles* @owl2-profiles — RL, EL, and QL reasoners that materialise or rewrite through
  the shared semi-naive join instead of a hand-rolled adjacency index, plus a Direct-semantics and
  a D-entailment path.
- *RIF-Core* @rif-core — rule evaluation over the same join kernel.
- *RDF stream processing (RSP-QL)* @rsp-ql — windowed continuous evaluation reusing the id-level
  join and comparator.
- *GeoSPARQL* @geosparql — spatial filter functions over the shared numeric/term layer.
- *SHACL* @shacl — shape validation, including SHACL-SPARQL, evaluated through the same engine.

The measured-substrate framing is what distinguishes this from the commercial breadth of GraphDB
or Stardog: we do not merely claim the families exist, we claim — and in §7 evidence — that they
share one measured-neutral core, and we expose the extraction's cost as a number rather than
hiding it behind a product surface.

== Correctness and deployment evidence <evidence>

*Cross-family differential conformance scoreboard.* Correctness is the load-bearing evidence for a
_shared_ core: if two families shared a kernel but diverged on the standards, the sharing would be
a bug, not a feature. The engine carries a cross-family conformance scoreboard
(`crates/sparq-conformance`) that pins a machine-checked pass floor per standard — the SPARQL
suites, RDFS/OWL-RL entailment, SHACL core and SHACL-SPARQL, the OWL EL and QL profiles,
D-entailment, GeoSPARQL, RSP, JSON-LD, and the protocol/service-description lanes — and fails the
build if any floor regresses. The scoreboard is a differential gate, not a self-report: a family's
results are checked against the standard's own test manifests, and the floors are the appendix
evidence that the shared substrate has not been shared at the cost of conformance. We are careful
about what a scoreboard is and is not: it is _self-evaluation against fixed suites_, strong for
demonstrating that breadth is not achieved by cutting corners, but it is not community-adoption
evidence and we do not present it as such (§9).

*Deployment surface: one core, compiled to the browser.* The same evaluation core compiles to
WebAssembly and runs client-side — the full SPARQL 1.1 engine including Leapfrog Triejoin, plus
reasoning, RSP, SHACL, and text-search WASM surfaces — under a _deterministic_ bundle-size ratchet
that fails CI if the compiled artifact grows. Following the honest verdict of the project's own
paper-selection audit, we do _not_ claim compiling to WebAssembly as a contribution: Oxigraph,
Comunica @comunica, and others already run SPARQL in the browser, and DuckDB-Wasm @duckdb-wasm set
the bar for "X-in-WASM as a first-tier result" with async workers, a paged browser filesystem, and
JS UDFs — engineering we have not done. The WASM story is therefore one _deployment-surface_
section and one _deterministic_ figure (the bundle-size ratchet), evidence that the shared core is
portable and size-bounded, not a headline.

#at_risk[
  The per-family conformance floor counts and the deterministic bundle-size figure are exact,
  machine-independent facts and will be cited via the canonical accessor once wired
  (`engine.conformance_families`, `engine.conformance_<family>_floor`, `engine.wasm_bundle_bytes`).
  They are named without a value here only because the evidence-record wiring is the downstream
  bead `sq-gum8.9`; they are _not_ competitive performance numbers and are not gated on §8.
]

== Evaluation methodology (fixed before the measurement) <methodology>

The competitive evaluation is committed here, before any competitive result exists, so that
results cannot steer the method. It is gated on bead `sq-vw3ax.12` (canonical-host competitor
baselines) and is _not_ run by this drafting bead. Four experiments, each with its honesty
constraint stated:

+ *Canonical-host baselines against native competitors.* SP2Bench @sp2bench, WatDiv @watdiv,
  LUBM @lubm, and BSBM @bsbm run on a single fixed, quiet canonical host (a pinned EC2 instance
  type), against _natively built_ QLever, Virtuoso, Jena, Oxigraph, and others — not container
  images, whose scheduling and I/O overhead would confound a systems comparison. Every reported
  latency or throughput is an `environment="canonical"` record; a work-box or developer-laptop
  number can never be a headline (the paper-factory `#headline` accessor refuses a non-canonical
  record by construction). The engine's own results are reported with the same discipline, and
  losses are reported, not omitted.
+ *A Sparqloscope run.* The Bast-group Sparqloscope benchmark @sparqloscope — designed for a fair,
  comprehensive SPARQL comparison — is run on the canonical host. A reviewer from that group will
  expect it, and running it on their terms rather than a hand-picked query mix is the honest
  choice.
+ *A memory-accounting-honest comparison against qEndpoint.* qEndpoint @qendpoint published the
  closest prior "large graph on commodity hardware" claim, and memory frugality is the paper's most
  contestable number because an mmap-resident store can appear frugal by relying on the OS page
  cache. We therefore _fix the accounting method in advance_ and report both quantities side by
  side: peak _committed_ resident memory (RSS excluding reclaimable page cache) and the _page-cache
  working set_ under a stated cache budget, for the engine and for qEndpoint, on identical
  hardware and data. The claim will be phrased in terms of committed bytes under a bounded cache,
  the accounting a hostile reviewer would demand — never a page-cache-flattered figure.
+ *The substrate zero-overhead measurement.* The micro-benchmark of §5 — the shared kernel versus
  the engine's pre-extraction hand-tuned loops, per kernel, on the canonical host — quantifies the
  marginal cost of the shared abstraction. A non-negligible overhead would _weaken the central
  claim_, and the honest disposition, decided now, is to report it as measured and re-scope the
  "zero-overhead" language to the measured bound rather than the aspiration.

Reporting rules, fixed now: every performance and memory figure is a canonical-host record cited
through the paper-factory evidence accessor; the deterministic ratchets (bundle size, layout
bytes, conformance floors) are reported as the machine-independent facts they are; and any
experiment that does not favour the engine is reported alongside those that do.

== Limitations and honest status <limitations>

*This is a draft without the competitive evaluation.* The paper's headline is a performance and
memory claim, and none of the four gating experiments (§8) has run. This draft therefore claims
_no_ competitive number; the architecture, the substrate extraction, the standards breadth, and
the conformance evidence (§3–§7) are real and traceable to the codebase, but the numbers that
would make the extreme-SOTA claim are not, and are not asserted. If the canonical evaluation shows
the engine is not competitive on speed, or that the memory frugality does not survive
committed-bytes accounting, the honest disposition — decided in advance — is to re-scope the paper
to the contribution that survives measurement (the shared-substrate engineering and the
conformance breadth) rather than to soften the benchmark.

*Breadth is not adoption.* The conformance scoreboard (§7) is self-evaluation against fixed
standard test suites. It is strong evidence that the shared substrate does not sacrifice
correctness, and it is _not_ evidence that anyone outside the project uses the engine. We do not
present the scoreboard as community-reuse evidence; external adoption, were it to exist, would be a
separate and stronger claim.

*The substrate claim rests on a measurement not yet taken.* "Zero measured marginal overhead" is
falsifiable and, in this draft, unfalsified only because §5's micro-benchmark is part of the §8
gate. The deterministic ratchets bound the _behavioural_ neutrality of the extraction today; the
_wall-clock_ neutrality is the pending measurement, and the claim is scoped to it.

*Fragment and scope.* The out-of-core claim is bounded by the working-set behaviour of the query,
not a promise of RAM-independence for adversarial full-scan workloads; the breadth claim covers
the families enumerated in §6 at their pinned conformance floors, not the entire surface of each
standard. Blank-node handling, federated-query performance, and update-workload behaviour are out
of scope for this draft's evaluation.

== Related work <related>

*Fast SPARQL engines.* QLever @qlever, Virtuoso @virtuoso, RDF-3X @rdf3x, Hexastore @hexastore,
Tentris @tentris, and MillenniumDB @millenniumdb established the storage and join techniques we
build on — exhaustive permutation indexing (RDF-3X, Hexastore), inline id encoding
(QLever, Virtuoso), and tensor/graph-native execution (Tentris, MillenniumDB). We adopt the six-
permutation layout and the inline-id encoding explicitly and attribute them; our delta is not a
new join algorithm but the extraction of the evaluation core into a substrate shared across
standards families.

*Worst-case-optimal joins.* Leapfrog Triejoin is due to Veldhuizen @triejoin; the worst-case-
optimality theory and the AGM bound are due to Ngo, Porat, Ré, and Rudra and collaborators
@wcoj-ngo. We use Leapfrog Triejoin as one of three planner-selected strategies and claim no
contribution to the algorithm itself.

*Broad reasoning/standards engines.* RDFox @rdfox, GraphDB/OWLIM @owlim, and Jena @jena ship broad
reasoning and standards coverage, generally as separately engineered subsystems, and without a
peer-reviewed claim of a measured shared evaluation core across the families. Our contribution is
precisely that shared-core claim and its measurement, not the breadth per se.

*Memory-frugal RDF.* HDT @hdt and qEndpoint @qendpoint are the closest prior art for serving large
graphs on commodity hardware; qEndpoint's "Wikidata on commodity hardware" @qendpoint is the
nearest published memory-frugality claim, and §8 targets a memory-accounting-honest comparison
against it precisely because that number will be attacked.

*In-browser engines.* Oxigraph @oxigraph and Comunica @comunica already run SPARQL in the browser;
DuckDB-Wasm @duckdb-wasm is the bar for a first-tier WASM systems result. We therefore fold the
WASM story into a deployment-surface section (§7) rather than claiming it, and report only the
deterministic bundle-size ratchet.

*Delta.* Against all of the above, the paper's delta is: (i) a single out-of-core engine that
carries SPARQL query, the OWL profiles, RIF, RSP, GeoSPARQL, and SHACL over _one_ evaluation core;
(ii) that core extracted into a shared substrate whose behavioural neutrality is validated by
deterministic ratchets and a differential conformance scoreboard, and whose wall-clock neutrality
is a stated pending measurement; and (iii) an honesty discipline — canonical-only performance
records, an explicitly-stated memory-accounting method, losses reported — that makes the
extreme-SOTA claim falsifiable rather than promotional.

== Conclusion

The speed/breadth/frugality trade-off that shapes the RDF-engine landscape is, we argue, largely
an artifact of duplicated evaluation machinery: engines are narrow because breadth has meant
building a second evaluator, and frugal engines are slow because frugality has meant a restricted
core. We described an out-of-core engine — six memory-mapped permutation indexes in the RDF-3X
lineage, inline-tagged ids in the QLever/Virtuoso lineage, a mixed binary/worst-case-optimal/bind
join evaluator — whose evaluation core is extracted into a single substrate shared unchanged across
SPARQL query, the OWL profiles, RIF, RSP, GeoSPARQL, and SHACL. We framed breadth as a
measured-substrate claim, validated the extraction's behavioural neutrality with deterministic
ratchets and a cross-family differential conformance scoreboard, and — crucially — refused to
assert the competitive performance and memory numbers that the extreme-SOTA claim ultimately rests
on, because the canonical-host evaluation that would earn them has not run. That evaluation, whose
methodology (including the contestable memory-accounting method) is fixed in this draft, is the
next step; whatever it shows will be reported against the rules committed here.

#heading(level: 2, numbering: none)[References]

#bibliography("sparq-engine-systems.refs.yml", style: "ieee", title: none)

#if not anon [
  #line(length: 100%)
  #text(size: 0.8em, fill: gray)[
    sparq project · DRAFT, in progress — the submission-gating canonical evaluation (bead
    `sq-vw3ax.12`) has not run; no competitive performance or memory number is claimed. Architecture
    evidence traces to the codebase: the six memory-mapped permutation indexes in
    `crates/sparq-core` (`compress.rs`), the inline-tagged / dict-spill id encoding
    (`dict.rs`, `dictspill.rs`), the mixed join families in `crates/sparq-engine`, the shared
    kernel in `crates/sparq-substrate` (`rows.rs`, `numeric.rs`, `join.rs`, `compare.rs`), the
    standards families in `crates/sparq-reason{,-el,-ql,-dl}`, `-rsp`, `-geo`, `-shacl`, and the
    cross-family scoreboard in `crates/sparq-conformance`. Design record:
    `research/shared-eval-substrate.md`. Positioning per `research/paper-selection.md` §3.4/§3.5/§3.6
    + §5-P3. Performance and memory numbers will flow through the paper-factory canonical evidence
    accessor (bead `sq-gum8.9`) once the §8 experiments run; none appears here.
  ]
]
