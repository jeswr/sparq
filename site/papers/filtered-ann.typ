// [FABLE-5] sq-gum8.3 — REWRITE of pilot paper A1 (filtered-ann) executing the venue-audit
// verdict (research/papers-venue-audit.md, bead sq-gum8.2: verdict REWRITE).
//
// Single-source Typst. Numbers come ONLY from #headline(...) / #ev(...) (paper-evidence.json),
// never hard-coded. Compiles to BOTH a PDF (the download) and semantic HTML (the in-site page).
//
// What this rewrite changes versus the original draft (audit gaps → actions):
//   1. Related work rebuilt around the 2023–2026 filtered-ANN wave (ACORN / NaviX /
//      Filtered-DiskANN / PathFinder / EMA / adaptive termination / the unified benchmark),
//      with a feature-by-feature delta table instead of one paragraph.
//   2. Answer-safety DEMOTED from headline theorem to an enforced engineering invariant
//      (it is near-tautological for an exact mask); the genuine delta — the shared
//      dictionary-id space and connected-component pushdown — is promoted to the headline.
//   3. An honest Evaluation section that says precisely what the deterministic evidence
//      does and does NOT establish, plus a pre-registered design for the deferred
//      performance evaluation (baselines, datasets, metrics, falsification criteria).
//   4. A first-class Limitations section.
//
// DRAFT TODOs (tracked for the sq-gum8.3 review loop; also flagged visibly in the text):
//   - Verify bibliographic metadata of the four 2025–26 arXiv preprints (see refs.yml note).
//   - TODO(code): confirm traversal behaviour when the mask is so selective that fewer than
//     k admissible candidates fall in the unfiltered top-k (under-fill boundary condition).
//   - TODO(code): document the concrete SPARQL surface syntax of the kNN operator and how
//     the neighbour bindings recompose with the remainder of the query.
//   - TODO(code): mask validity under concurrent updates (snapshot semantics).
//   - The performance evaluation itself is BLOCKED on the canonical bare-metal runner.

#import "_lib/bench.typ": headline, ev, provenance, authors, anon

#set document(title: "Filter-as-Query")
#set text(size: 11pt)
#set par(justify: true)
// Sections are level-2 headings (the site's HTML convention: h2 under the page h1), so a
// plain "1." pattern would render "0.1", "0.2" — drop the never-used level-1 component.
#set heading(numbering: (..n) => {
  let ns = n.pos()
  numbering("1.", ..if ns.len() > 1 { ns.slice(1) } else { ns })
})

#align(center)[
  #text(size: 17pt, weight: "bold")[
    Filter-as-Query: Filtered Approximate Nearest-Neighbour Search over SPARQL,
    where the Filter is an Exact Basic Graph Pattern over the Engine's Own
    Dictionary Ids
  ]
]
#authors()

#align(center)[#text(style: "italic", size: 0.9em)[
  Draft under internal review (rewrite of pilot A1). A systems-integration contribution.
  All evidence in this document is deterministic and machine-independent (asserted
  equivalence invariants, recall floors, a cost-model constant); no wall-clock latency or
  throughput is claimed — the performance evaluation is specified in §6.5 and deferred to
  the canonical runner.
]]

#heading(level: 2, numbering: none, outlined: false)[Abstract]

Knowledge graphs increasingly carry dense vector embeddings next to their symbolic triples,
and queries increasingly mix the two: _return the nearest neighbours of this embedding, but
only those entities that satisfy this graph pattern_. Filtered approximate nearest-neighbour
(ANN) systems support such constraints by mirroring a scalar attribute onto every vector and
evaluating a flat predicate over the mirrored copy. We describe an RDF-native alternative
implemented inside a dictionary-encoded SPARQL engine: the constraint is the join-connected
sub-pattern of the query's own Basic Graph Pattern (BGP), evaluated _exactly_ by the host
engine over its permutation indexes, and materialised as an id-set (`IdMask`) over the _same
dictionary-id space_ the vectors are keyed on. This design needs no metadata mirroring and no
consistency protocol between an attribute store and the vector index; the filter language is
the full BGP join — including multi-hop (transitive) and cyclic sub-patterns that per-vector
attribute filters cannot express; and pre-filtering is _answer-safe by construction_,
enforced end-to-end as machine-checked equivalence invariants rather than claimed. We are
explicit about what this paper is and is not: the ANN traversal is unmodified prior art, the
answer-safety property is mathematically simple, and no performance numbers are reported —
the contribution is the integration architecture, its enforced correctness envelope, and an
honest, pre-registered evaluation design against the current filtered-ANN state of the art.

== Introduction <intro>

Two representations of the same entities now routinely coexist: a knowledge graph stores what
is _asserted_ about an entity, and a learned embedding stores what is _similar_ to it
@transe. Query workloads follow: retrieval-augmented pipelines and hybrid search
applications ask for the nearest neighbours of a query vector _subject to_ symbolic
constraints — the nearest `:Vehicle`s, the nearest products still in stock, the nearest
papers by authors at a given institution.

The database community has converged on _filtered ANN_ as the abstraction for this workload:
an approximate nearest-neighbour index whose traversal is constrained to vectors satisfying
a predicate @filtered-diskann @acorn @navix. In every deployed design we are aware of, the
predicate is evaluated over per-vector _metadata_ — a scalar tag, label set, or attribute
column stored with (or mirrored onto) the vector index. That architecture has three
consequences that matter in an RDF setting:

+ *Mirroring and drift.* The attributes the filter can see must be copied out of the
  primary store into the vector index's metadata columns and kept in sync under updates —
  a second copy of the data with its own consistency obligations.
+ *A flat filter language.* A per-vector tag can express `colour = red` but not a join —
  _vehicles whose owner is based in Berlin_ — let alone a multi-hop or cyclic graph
  constraint. The filter language is strictly weaker than the query language of the host
  system.
+ *An unspecified correctness contract.* Whether the filtered result equals what
  post-filtering the unfiltered result would have produced is typically a property of the
  traversal heuristics, observed empirically rather than enforced.

This paper describes _Filter-as-Query_, the filtered-ANN integration of the sparq RDF
engine, which dissolves all three at the architecture level rather than the algorithm level.
The engine dictionary-encodes RDF terms to integer ids and keys its vectors by those same
ids. A vector-neighbour variable in a SPARQL query is constrained by the ordinary triple
patterns it joins with; the engine evaluates that sub-BGP — _exactly_, over its existing
permutation indexes @rdf3x — and projects the admissible ids into an `IdMask` that the ANN
traversal consumes directly. The filter _is_ a query: there is no mirrored metadata, the
filter language is the BGP join language itself (including transitive and cyclic
sub-patterns; @method), and the
narrow-never-widen property is enforced as a deterministic, machine-checked invariant across
every constraint shape and both physical execution strategies.

=== Contributions and non-contributions <contributions>

We claim the following, each forward-referencing its evidence:

- *C1 — Filter compilation into the engine's own id space* (@method). The constraint on a
  neighbour variable `?n` is the connected component of the BGP join-graph containing `?n`,
  extracted by a terminating fixpoint (@alg-cc) that handles multi-hop and cyclic
  sub-patterns, evaluated exactly by the host engine, and projected as an `IdMask` over the
  dictionary ids the vectors are already keyed on. No metadata is mirrored; ids are never
  re-encoded at the boundary.
- *C2 — An enforced answer-safety envelope* (@safety). Pre-filtering narrows the candidate
  set and never changes the answer: the filtered top-$k$ is byte-identical to
  post-filtering the unfiltered result by the same constraint, asserted as deterministic
  equivalence invariants over single-pattern, transitive, and cyclic constraint shapes. We
  are explicit that this property is mathematically near-immediate for an _exact_ mask
  (@safety-demotion); the contribution is that it is _enforced end-to-end_ — including
  across the physical pre/post decision of C3 — not that it is deep.
- *C3 — A deterministic pre/post-filter decision rule with a confined failure mode*
  (@cost). A one-constant cost model chooses between scanning the mask and scanning the
  store; both branches provably return the identical answer, so a mis-estimate costs
  throughput, never correctness.
- *C4 — An honest evaluation protocol* (@evaluation). Every number in this paper is
  injected at build time from an evidence ledger whose records are labelled by environment;
  headline tables can only cite deterministic, machine-independent records (the build fails
  otherwise). Because the canonical performance runner is not yet available, we report _no_
  latency or throughput and instead pre-register the evaluation design — baselines,
  datasets, metrics, and the results that would falsify the approach.

Equally important is what we do _not_ claim. The ANN index and its traversal (an HNSW-style
graph @hnsw) are unmodified prior art; we contribute no new traversal, pruning, or
termination strategy, and systems whose contribution is a better constrained traversal
@acorn @navix @filtered-diskann are complementary rather than competitors: our mask could
feed any of them. The mask-caching layer that memoises BGP→`IdMask` derivations is an
engineering optimisation, not a contribution. And this draft contains no performance
evaluation — by our own venue analysis that is the single largest gap between this document
and a database-venue submission, and we say so plainly in @limitations rather than let two
correctness tables masquerade as one.

== Related work <related>

*Graph-based ANN indexes.* HNSW @hnsw and DiskANN @diskann are the dominant graph-traversal
ANN families; Faiss @faiss is the standard library baseline. sparq's vector index is an
HNSW-style graph; nothing in this paper modifies it.

*Filtered ANN.* Filtered-DiskANN @filtered-diskann constrains a DiskANN graph by per-vector
_labels_, building label-aware graph edges; filters are conjunctions over a bounded label
vocabulary attached to each vector. ACORN @acorn is _predicate-agnostic_: it accepts an
arbitrary per-query predicate set (in the limit, a bitset over vector ids) and searches a
denser HNSW variant under it, explicitly decoupling the filter's semantics from the index.
NaviX @navix brings predicate-agnostic filtered search _inside_ a graph DBMS (Kùzu),
selecting adaptively among filtered-traversal strategies. SeRF @serf specialises to range
filters. A 2025–2026 wave extends the space — PathFinder @pathfinder, EMA @ema, adaptive
end-to-end termination policies @e2e-termination — and a unified filtered-ANN benchmark
@filtered-ann-benchmark now exists to compare them. #text(style: "italic")[Draft note: the
four preprints are cited by arXiv identifier from our internal venue audit; their
bibliographic metadata is pending verification and their technique descriptions here are
correspondingly coarse.]

Relative to this line, our delta is deliberately _upstream_ of the index, and it is honest
to state it narrowly. ACORN already accepts an arbitrary admissible-id set, so an `IdMask`
interface is not novel. What is new here is _who computes the mask, over what id space, and
under what enforced contract_: (i) the mask is derived by the host SPARQL engine as an exact
BGP evaluation — the filter language is the engine's join language, including transitive and
cyclic sub-patterns no per-vector attribute scheme expresses; (ii) the mask lives in the
engine's dictionary-id space, which _is_ the vector key space, so there is no mirrored
metadata to maintain and no id translation at the boundary; and (iii) the
narrow-never-widen contract is machine-checked across constraint shapes and physical plans,
not observed. @tab-delta summarises the comparison qualitatively; no performance ordering is
implied.

#figure(
  table(
    columns: 5,
    align: (left, left, left, left, left),
    table.header[System][Filter language][Filter evaluated by][Vector key space][Answer contract],
    [Filtered-DiskANN @filtered-diskann],
    [label conjunctions],
    [index (label-aware edges)],
    [index ids + mirrored labels],
    [empirical recall],
    [ACORN @acorn],
    [arbitrary per-query predicate / bitset],
    [caller supplies; index traverses under it],
    [index ids + mirrored attributes],
    [empirical recall],
    [NaviX @navix],
    [DBMS predicates],
    [graph DBMS, adaptively],
    [DBMS-internal ids],
    [empirical recall],
    [VBASE / Milvus-class @vbase @milvus @analyticdb-v],
    [relational predicates],
    [relational executor over attribute columns],
    [index ids + mirrored columns],
    [empirical / engine-specific],
    [*Filter-as-Query* (this work)],
    [full BGP join incl. transitive + cyclic sub-patterns],
    [host SPARQL engine, exactly, over permutation indexes],
    [the engine's dictionary ids (shared; nothing mirrored)],
    [enforced pre≡post equivalence (deterministic asserts)],
  ),
  caption: [
    Qualitative delta against the nearest prior art. The traversal machinery of the first
    three rows is a contribution we do _not_ compete with — our mask could feed any
    predicate-agnostic index. The delta is confined to the filter's provenance (an exact
    query), the shared id space (no mirroring), and the enforced answer contract.
  ],
) <tab-delta>

*Hybrid vector + structured engines.* AnalyticDB-V @analyticdb-v, Milvus @milvus, and VBASE
@vbase integrate vector search with relational predicates, including pre-/post-filter
strategy switching (AnalyticDB-V's plans; VBASE's relaxed-monotonicity unified iterator).
These systems evaluate filters over attribute columns co-located with (or mirrored into) the
vector subsystem. Our cost-model crossover (@cost) is a deliberately minimal instance of
their strategy-selection idea; the difference, again, is that the admissible set is an exact
query answer in a shared id space, and that strategy mis-selection is proven
answer-invariant.

*Vector search in RDF and graph stores.* Property-graph and RDF stores increasingly bundle
vector indexes (NaviX in Kùzu is the research frontier @navix; several production stores
ship vector plugins). To our knowledge no prior RDF engine compiles the SPARQL BGP itself
into the ANN filter over a shared dictionary-id space; dictionary encoding with permutation
indexes is, however, entirely standard RDF-engine architecture @rdf3x — our point is
precisely that the standard architecture already contains the right key space for filtered
ANN, if the integration is done inside the engine rather than beside it.

== Preliminaries <prelim>

Let $I$, $B$, $L$ be the pairwise-disjoint sets of IRIs, blank nodes, and literals; an RDF
graph $G$ is a finite set of triples $(s, p, o)$. A _dictionary-encoded_ engine assigns each
term a unique integer id via a bijective dictionary $"dict": I union B union L -> NN$ and
stores $G$ as integer triples in several sort orders (permutation indexes) @rdf3x. A basic
graph pattern (BGP) $P$ is a set of triple patterns over terms and variables; its semantics
$⟦P⟧_G$ is the set of solution mappings from the variables of $P$ to terms, per Pérez et
al. @sparql-semantics and the SPARQL 1.1 recommendation @sparql11. For a variable $x$,
$pi_x (⟦P⟧_G)$ denotes the projection of the solutions onto $x$.

The engine additionally stores a vector table $V: "dom"("V") -> RR^d$ _keyed by dictionary
id_, with $"dom"("V") subset.eq "range"("dict")$: an embedding is attached to an RDF term by
id, not by a separate vector-store key. A $k$-NN query is $(q, k)$ with $q in RR^d$; the
exact answer over a candidate set $C subset.eq "dom"("V")$ is the $k$ ids in $C$ minimising
the distance to $q$ (deterministic id-order tie-break). An ANN index answers approximately;
its quality is measured by recall\@$k$ against the exact answer. A _filtered_ $k$-NN query
additionally supplies an admissible set $M subset.eq "dom"("V")$ (here: an `IdMask`), and
must answer over $C = "dom"("V") inter M$.

== Filter-as-Query <method>

=== The constraint is the join-connected sub-BGP <cc>

Consider a query whose BGP contains a vector-neighbour variable `?n` (declared by the
engine's kNN operator; the surface syntax is out of scope here). Ordinary triple patterns
mentioning `?n` — directly (`?n a :Vehicle`) or through intermediate variables
(`?n :ownedBy ?p . ?p :basedIn :Berlin`) — constrain which ids `?n` may bind to. The
constraining sub-pattern is _not_ just the patterns that mention `?n`: it is the connected
component of the BGP's join-graph (patterns as nodes, edges between patterns sharing a
variable) that contains `?n`. A pattern with no shared-variable path to `?n` cannot
constrain it and is excluded — its bindings join in later, unaffected by the vector search.

#figure(
  kind: "algorithm",
  supplement: [Algorithm],
  caption: [
    Connected-component constraint extraction. The worklist fixpoint terminates because $C$
    grows monotonically and is bounded by $P$; cyclic sub-BGPs (a back-edge to `?n`) add no
    complication because membership, not path enumeration, drives the loop.
  ],
  [
    #set align(left)
    #block(width: 100%, inset: 8pt, stroke: 0.5pt + gray)[
      #raw(
        "extract(P: BGP, ?n: neighbour variable) -> C(P, ?n):\n" +
        "  V <- { ?n }          // reached variables\n" +
        "  C <- {}              // constraining sub-BGP\n" +
        "  repeat until no change:\n" +
        "    for each t in P \\ C:\n" +
        "      if vars(t) ∩ V ≠ {}:\n" +
        "        C <- C ∪ {t};  V <- V ∪ vars(t)\n" +
        "  return C",
        block: true,
      )
    ]
  ],
) <alg-cc>

The engine then evaluates $C(P, `?n`)$ — exactly, with its ordinary BGP machinery over the
permutation indexes — and materialises the mask
$M = "dict-ids"(pi_(`?n`) (⟦C(P, `?n`)⟧_G))$. Because $⟦dot⟧_G$ is the engine's own exact
semantics, $M$ contains _precisely_ the ids consistent with the constraint: no
approximation enters through the filter, only through the ANN traversal it feeds.

_Worked example._ In the BGP `{ ?n :ownedBy ?p . ?p :basedIn :Berlin . ?d :caption ?t }`,
the component of `?n` is the first two patterns; $M$ is the set of ids of things owned by
Berlin-based owners; the caption pattern is disconnected and does not narrow the search. A
cyclic component such as `{ ?n :owns ?x . ?x :ownedBy ?n }` is handled identically —
@alg-cc terminates on membership, and the exact evaluation of the cyclic join is the
engine's job, not the index's.

=== Why the shared id space matters <idspace>

The mask's elements are dictionary ids, and the vector table is keyed by dictionary ids.
Three practical properties follow. _No mirroring:_ there is no per-vector metadata column
to extract, denormalise, or keep consistent — the constraint reads the primary triple
indexes. _No boundary translation:_ the mask flows into the traversal as-is; there is no
join between a vector-store id space and an engine id space at query time. _Full filter
expressivity for free:_ any future improvement to the engine's BGP evaluation (new join
algorithms, better cardinality estimation) is automatically an improvement to the filter,
because the filter _is_ a query.

The honest converse: deriving $M$ costs an exact BGP evaluation, which for an expensive
constraint (a large transitive component over a big graph) may dominate the vector search
itself. We treat mask-derivation cost as a first-class open measurement in the deferred
evaluation (@evaluation), not as a solved problem; a memoising mask cache exists in the
implementation but is an optimisation, not a contribution.

=== Answer-safety as an enforced invariant <safety>

The correctness contract is _narrow-never-widen_: constraining the search must only remove
candidates, never change what is returned for the candidates that remain.

_Proposition (exact path)._ For the exact evaluation path, the filtered top-$k$ over
$"dom"("V") inter M$ equals the result of ranking all of $"dom"("V")$, deleting non-members
of $M$, and truncating to $k$; in particular it is a subset of any unfiltered ranking
prefix that contains at least $k$ members of $M$. _Proof sketch:_ deleting non-members
commutes with ranking by distance under a deterministic total order (distance, then id);
both sides denote the $k$ minimal admissible elements. #h(0.3em) $qed$

=== This property is shallow — and that is the point <safety-demotion>

We flag plainly what an earlier draft of this paper presented as its headline theorem: for
an _exact_ mask, the proposition above is near-tautological. The reason answer-safety
deserves space at all is not mathematical depth but _enforcement surface_. In deployed
filtered-ANN systems the analogous property spans several moving parts — the mirrored
metadata's freshness, the traversal's pruning heuristics, the pre/post strategy switch —
and typically holds empirically rather than by contract. Here it is pinned as a set of
deterministic, machine-checked equivalences (byte-identical results, fixed fixtures, fixed
seeds) across every constraint shape the compiler accepts — single-pattern, transitive, and
cyclic — _and across both physical plans_ of the decision rule below, so a regression in
any layer fails the build rather than skewing an experiment. The evidence table is in
@evaluation. One boundary condition remains open and is stated in @limitations: fixtures
assert equivalence on broad masks, and the behaviour when fewer than $k$ admissible
candidates fall within the unfiltered traversal's reach is a documented TODO against the
implementation, part of the deferred evaluation's selectivity sweep.

=== A deterministic pre/post-filter decision rule <cost>

Given $M$, the engine chooses _how_ to apply it: visit only masked ids (pre-filter), or run
the unfiltered scan and drop non-members (post-filter). With $m = |M|$, $n = |"dom"("V")|$,
and a single modelled constant `scatter_penalty` pricing a scattered masked-row access
against a sequential row, pre-filter is chosen iff $m dot.c "scatter_penalty" <= n$. With
the default penalty the crossover sits at a mask admitting at most
#ev("filtered_ann.prefilter_crossover") of the store.

This is a heuristic over an estimate, and we make no claim that the constant is optimal on
any hardware — that is exactly the kind of claim this paper refuses to make without the
canonical runner. The property we _do_ claim is architectural: both branches return the
identical top-$k$ (asserted, same fixtures as above), so the decision rule's failure mode
is confined to throughput. A mis-estimated crossover can make a query slower; it cannot
make it wrong. This separation — correctness pinned by invariant, performance left to
honest measurement — is the design stance of the whole integration.

== Evaluation <evaluation>

=== Methodology and evidence discipline <methodology>

Every number in this paper is injected at build time from a versioned evidence ledger; the
build fails if a headline table cites any record not labelled _canonical_ — deterministic,
machine-independent, asserted in CI (fixed seeds, fixed fixtures, `assert!` floors). Non-
canonical (work-box, "indicative") measurements are barred from headline use by the same
gate. This discipline is why the present section contains correctness evidence only: _no
canonical performance environment exists yet_, and we prefer an honest gap to an
unreproducible number. The provenance stamp on the published page records the evidence
commit.

=== What is established: answer-safety across constraint shapes <eval-safety>

#figure(
  table(
    columns: 3,
    align: (left, center, left),
    table.header[Constraint shape][pre-filter ≡ post-filter][evidence],
    [single-pattern (`?n a :Vehicle`)],
    [#if headline("filtered_ann.prefilter_equals_postfilter") [holds] else [—]],
    [`filtered_bgp.rs`],
    [transitive (2-hop join)],
    [#if headline("filtered_ann.transitive_equals_postfilter") [holds] else [—]],
    [`filtered_bgp_transitive.rs`],
    [cyclic sub-BGP],
    [#if headline("filtered_ann.cyclic_equals_postfilter") [holds] else [—]],
    [`filtered_bgp_cyclic.rs`],
  ),
  caption: [
    Enforced answer-safety across constraint shapes. Each cell is a deterministic
    equivalence proven by assertion over a fixed fixture — the filtered top-$k$ is
    byte-identical to post-filtering the unfiltered result by the same constraint, and a
    subset of it ("true" renders as "holds"). Per @safety-demotion, read this as an
    _enforcement_ result (the invariant is CI-pinned end-to-end), not a deep one.
  ],
) <tab-safety>

=== What is established: the approximation budget <eval-recall>

The mask is exact; the only approximation is the ANN traversal it feeds. The relevant
sanity question is whether the _filtered_ traversal preserves the index's recall against
the _exact-filtered_ ground truth. On deterministic fixtures it does, with asserted floors:

#figure(
  table(
    columns: 4,
    align: (left, center, center, center),
    table.header[Setting][vectors × dim][recall\@10 floor][queries],
    [unfiltered HNSW vs exact brute force],
    [50,000 × 32],
    [#headline("ann.recall_at_10_floor")],
    [100],
    [filtered traversal vs exact-filtered (≈50% mask)],
    [20,000 × 32],
    [#headline("filtered_ann.recall_at_10_floor")],
    [100],
  ),
  caption: [
    Deterministic recall floors (asserted lower bounds over fixed seeds). These are
    correctness sanity checks on an unmodified HNSW-style index under masking — they bound
    the approximation the integration inherits. They are _not_ a recall-versus-latency
    evaluation and we do not present them as one: a single broad selectivity, synthetic
    vectors, and low dimensionality (see @limitations).
  ],
) <tab-recall>

#provenance("ann.recall_at_10_floor")

=== What is established: the decision constant <eval-cost>

The pre/post crossover of @cost is pinned as a canonical constant — a mask admitting at
most #headline("filtered_ann.prefilter_crossover") of the store selects the pre-filter
branch under the default scatter penalty — together with the branch-equivalence assertions
of @tab-safety. No claim is made about where the crossover _should_ sit on real hardware;
that is a measurement, and it is deferred.

=== What is not established, and the pre-registered design to establish it <eval-plan>

This draft reports no latency, no throughput, no recall-versus-latency Pareto, no
selectivity sweep, and no comparison measurement against any external system. For a
filtered-ANN paper that gap is disqualifying at a database venue, and we neither hide it
nor pad around it. To make the deferred evaluation falsifiable rather than aspirational, we
pre-register its design:

- *Baselines.* ACORN @acorn (predicate-agnostic, accepts our mask directly — the critical
  comparison, since it isolates the value of _in-engine_ mask derivation), NaviX @navix
  (the in-DBMS state of the art), post-filtered unmodified HNSW, and exact filtered scan.
  Where the unified filtered-ANN benchmark @filtered-ann-benchmark applies, its harness and
  datasets take precedence over bespoke ones for comparability.
- *Workloads.* (i) Standard filtered-ANN benchmark suites for the flat-predicate regime —
  the regime where we expect _no_ advantage and must show we are not worse; (ii) a
  KG-native workload — entity embeddings over a public knowledge graph with BGP constraints
  of graded join depth (1–3 hops, with and without cycles) and graded selectivity — the
  regime flat-attribute systems cannot express natively, which exists to measure whether
  the expressivity delta carries real workloads rather than to flatter the system.
- *Metrics.* Recall\@10 versus queries-per-second Pareto frontiers per selectivity band;
  mask-derivation time reported _separately_ and end-to-end (the integration's honest
  overhead, amortised and unamortised — the mask cache disabled and enabled); index memory
  including any metadata mirroring the baselines require and we avoid.
- *Selectivity sweep.* Mask fractions from highly selective through the cost-model
  crossover to near-unfiltered, explicitly probing the under-fill boundary condition of
  @safety-demotion.
- *Environment.* The canonical bare-metal runner, with every number environment-labelled
  under the discipline of @methodology; nothing measured on shared development hardware
  will be reported as evidence.
- *Falsification criteria.* The integration thesis _fails_ if (a) mask derivation dominates
  end-to-end latency at realistic selectivities on the KG-native workload even amortised,
  or (b) the shared-id-space design shows no measurable cost advantage (memory or
  freshness/update overhead) over mirrored-metadata baselines at equal recall, or (c)
  filtered-traversal recall collapses at selective masks with no viable fallback crossover.
  Publishing the evaluation commits to reporting these outcomes if they occur.

== Limitations <limitations>

- *No performance evidence.* The largest limitation is stated throughout: this paper
  contains no wall-clock measurement of any kind. The evaluation design of @eval-plan is
  specified but unexecuted, blocked on the canonical runner. Until it runs, the claim
  inventory is architectural and correctness-only.
- *Answer-safety is shallow.* Per @safety-demotion, the headline invariant is
  near-immediate for an exact mask; its value is enforcement breadth, not depth. Readers
  seeking a theoretical contribution will not find one here.
- *Recall floors are sanity checks.* @tab-recall covers one broad selectivity, synthetic
  low-dimensional vectors, and modest scale — floors chosen to be deterministic in CI, not
  to characterise the index. They bound nothing about behaviour at high selectivity or at
  realistic embedding dimensionality.
- *Under-fill boundary condition (open).* When the mask is selective enough that fewer
  than $k$ admissible candidates are reachable by the unfiltered traversal, the equivalence
  fixtures do not yet pin the behaviour; confirming and documenting the traversal's
  contract there is an open implementation TODO carried into the selectivity sweep.
- *Mask-derivation cost is unmeasured.* An expensive constraint component may cost more
  than the search it narrows; the crossover between "derive the mask" and "post-filter
  without one" is precisely the kind of question only the deferred evaluation can answer.
- *Updates and concurrency.* Mask validity under concurrent writes (snapshot semantics of
  the derived `IdMask` versus the store it was derived from) is not treated in this paper
  and is an open documentation TODO against the implementation.
- *Single node.* The integration is single-node; interaction with federation is out of
  scope.
- *Related-work caveat.* The feature table (@tab-delta) is qualitative; the four 2025–26
  preprints are cited by arXiv identifier pending bibliographic verification, and our
  descriptions of them are coarse. No claim in this paper depends on their details.

== Conclusion <conclusion>

Filter-as-Query is a small architectural thesis executed inside a real RDF engine: if the
engine dictionary-encodes its terms and keys its vectors by the same ids, then the right
filtered-ANN filter is not a mirrored attribute but _the query itself_ — the join-connected
sub-BGP, evaluated exactly by the machinery that already exists, projected into the id
space the index already speaks. The filter language becomes the BGP join language
(transitive and cyclic constraints included), metadata mirroring disappears, and the
narrow-never-widen contract can be enforced end-to-end as build-failing invariants rather
than observed empirically — with approximation confined to the unmodified ANN traversal and
strategy mis-selection confined, provably, to throughput. What remains — and what we have
pre-registered rather than performed — is the honest performance case against ACORN-class
and in-DBMS filtered search. The architecture stands or falls on that measurement, and this
paper is written so that either outcome is reportable.

#heading(level: 2, numbering: none)[References]
#bibliography("filtered-ann.refs.yml", style: "ieee", title: none)

#if not anon [
  #line(length: 100%)
  #text(size: 0.8em, fill: gray)[
    sparq project · draft under internal review (rewrite executing the sq-gum8.2 venue-audit
    verdict). Evidence traces to `crates/sparq-vectors/tests`. Numbers in this document are
    injected at build time from the paper-bound evidence file; see the provenance stamp on
    the published page.
  ]
]
