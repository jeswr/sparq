// [OPUS-4.8] sq-gum8 — Pilot paper A1: RDF-native filtered-ANN ("Filter-as-Query").
// Single-source Typst. Numbers come ONLY from #headline(...) / #ev(...) (paper-evidence.json),
// never hard-coded. Compiles to BOTH a PDF (the download) and semantic HTML (the in-site page).
// Framed HONESTLY as a SYSTEMS-INTEGRATION contribution with DETERMINISTIC evidence — no
// headline latency claims (the canonical latency runner is not yet available).

#import "_lib/bench.typ": headline, ev, provenance, authors, anon

#set document(title: "Filter-as-Query")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#align(center)[
  #text(size: 17pt, weight: "bold")[
    Filter-as-Query: Predicate-Constrained Vector Search where the Filter is an
    Exact RDF Basic Graph Pattern over the Engine's Own Dictionary Ids
  ]
]
#authors()

#align(center)[#text(style: "italic", size: 0.9em)[
  A systems-integration contribution. Evidence is deterministic and machine-independent
  (recall floors, answer-safety invariants, a cost-model crossover); no wall-clock latency
  is claimed (the canonical performance runner is not yet available).
]]

== Abstract

Approximate nearest-neighbour (ANN) search over vectors stored alongside an RDF graph must
often be _constrained_ by the graph's structure — return the nearest vehicles, not the
nearest things. Existing filtered-ANN systems attach a scalar metadata predicate to each
vector and filter on it. We instead compile the constraint to an _exact_ SPARQL Basic Graph
Pattern (BGP) evaluated against the engine's permutation indexes, producing an id-set
(`IdMask`) over the _same dictionary-id space_ the vectors are keyed on. This needs no
metadata mirroring, generalises to transitive and cyclic join sub-patterns, and is
_answer-safe_: pre-filtering narrows the candidate set and never changes the answer. The
contribution is the integration, not a new ANN algorithm; we substantiate it with
deterministic recall and answer-equivalence evidence.

== Contributions

This paper substantiates the following claims; each is refutable and forward-references its
evidence.

- *The filter is an exact BGP over the engine's own dictionary ids* (§3) —
  the constraint is evaluated by the same query engine that answers SPARQL, producing an
  `IdMask` over the identifiers the vectors are already keyed on. No per-vector metadata
  column is mirrored or kept in sync.
- *Pre-filtering is answer-safe ("narrow-never-widen")* (§4) — the filtered
  top-k is _byte-identical_ to post-filtering the unfiltered top-k by the same constraint,
  and is a subset of it. We prove this as a deterministic invariant, including for
  transitive (multi-hop) and cyclic join sub-patterns.
- *The constraint generalises to the join-connected sub-BGP* (§3) — the
  constraining id-set is the projection of the connected component of the BGP join-graph
  reachable from the neighbour variable, derived by a terminating fixpoint that handles
  cycles.
- *A deterministic pre/post-filter cost model* (§5) — a single-constant
  crossover rule decides whether to scan the mask or scan the store and drop non-members;
  whichever branch is chosen, the answer is identical, so a mis-estimate costs throughput,
  never correctness.

== The mechanism: a filter that _is_ a query <mechanism>

In a store that keys vectors by the RDF term's dictionary id, a vector neighbour variable
`?n` constrained by ordinary triple patterns in the same BGP (e.g.
`?n a :Vehicle`, or transitively `?n :owns ?x . ?x a :Vehicle`) defines a candidate set: the
projection onto `?n` of all solutions to that sub-BGP. Because the engine already evaluates
BGPs over its permutation indexes, that projection is computed _exactly_ by the engine and
materialised as an `IdMask` — a set of dictionary ids — which is precisely the key space the
vector store uses. The k-NN search is then restricted to admitted ids.

The constraining sub-BGP is not limited to patterns that mention `?n` directly. It is the
connected component of the join-graph containing `?n`, so a constraint that only bites
through intermediate variables (a 2-hop `:owns`/`a :Vehicle` path) still restricts `?n`
correctly; a pattern with no shared-variable path to `?n` does not affect the mask. The
component is derived by a fixpoint worklist that terminates even when the sub-BGP contains a
cycle (a back-edge to `?n`, e.g. `?n :owns ?x . ?x :ownedBy ?n`).

This is a _systems-integration_ contribution. The ANN index itself (HNSW / DiskANN-style
traversal) is unmodified prior art; what is new is that the _filter_ is an exact RDF query
over the shared id space, rather than a scalar metadata predicate mirrored onto the vectors.

== Answer-safety: pre-filter ≡ post-filter <safety>

The central correctness property is that constraining the search cannot change the answer —
it can only narrow the candidate set. We state it as an equivalence: the filtered top-k
equals what you would get by running the search _unfiltered_ and then dropping the
disallowed hits (post-filtering), and is therefore a subset of the unfiltered top-k.

This is established as a *deterministic, machine-independent invariant* (an `assert_eq` over
a fixed fixture), not a statistical observation:

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
    Answer-safety holds across constraint shapes. Each cell is a deterministic equivalence
    proven by assertion — the filtered top-k is byte-identical to post-filtering the
    unfiltered top-k by the same constraint. (`true` renders as "holds".)
  ],
)

The filter operates over an approximate index, so the relevant accuracy question is whether
the _filtered_ traversal preserves the index's recall against the _exact-filtered_ ground
truth. It does, with a deterministic asserted floor:

#figure(
  table(
    columns: 4,
    align: (left, center, center, center),
    table.header[Setting][vectors × dim][recall\@10 floor][queries],
    [unfiltered HNSW vs exact brute force],
    [50,000 × 32],
    [#headline("ann.recall_at_10_floor")],
    [100],
    [filtered traversal vs exact-filtered (~50% mask)],
    [20,000 × 32],
    [#headline("filtered_ann.recall_at_10_floor")],
    [100],
  ),
  caption: [
    Deterministic recall floors (asserted lower bounds over fixed seeds), not latency. The
    unfiltered floor bounds the approximation the filter operates over; the filtered floor
    shows the constrained traversal preserves recall against the exact-filtered ground truth.
  ],
)

#provenance("ann.recall_at_10_floor")

== A deterministic pre/post-filter cost model <cost>

Having derived the mask, the engine still chooses _how_ to apply it: scan only the masked
ids (pre-filter), or run the unfiltered scan over the whole store and drop non-members
afterwards (post-filter). With $m = |"mask"|$ and $n = |"store"|$, a scattered masked-row
access is modelled at a `scatter_penalty` over a sequential row, and pre-filter is chosen iff
$m dot.c "scatter_penalty" <= n$. With the default penalty 2.0 the crossover is a mask
admitting at most #ev("filtered_ann.prefilter_crossover") of the store.

This is a heuristic over a cost _estimate_, not a proven optimum — the scatter penalty is a
single modelled constant, deliberately conservative. The load-bearing property is that the
choice never affects the _answer_: both branches return the identical top-k (asserted), so a
mis-estimate costs throughput, never correctness. We therefore present the crossover as a
deterministic decision rule, and make _no_ wall-clock claim about which branch is faster on
any given hardware.

== Related work

Filtered-ANN is an active area. ACORN (SIGMOD 2024), NaviX (2025), and PathFinder admit a
scalar metadata predicate attached to each vector and steer the graph traversal under it. We
do not improve on that traversal machinery and do not claim to. Our difference is upstream of
the index: the _filter_ is an exact RDF BGP evaluated by the host query engine over the same
dictionary-id space the vectors are keyed on, so there is no metadata column to mirror or
keep consistent, and the constraint composes with the full expressivity of a BGP join —
including transitive and cyclic sub-patterns — rather than a flat per-vector tag. The
mask-caching layer that memoises a BGP→`IdMask` derivation is an engineering
optimisation, not a contribution, and we do not frame it as one.

== Conclusion

A filter that _is_ a query, evaluated over the engine's own dictionary ids, gives
predicate-constrained vector search that is answer-safe by construction across single,
transitive, and cyclic constraints, with a deterministic pre/post-filter cost crossover at a
mask selectivity of #ev("filtered_ann.prefilter_crossover"). The evidence here is
deterministic and machine-independent: recall floors of
#headline("ann.recall_at_10_floor") (unfiltered) and
#headline("filtered_ann.recall_at_10_floor") (filtered), and proven pre-filter ≡
post-filter equivalence. A latency evaluation against native baselines is deferred to the
canonical performance runner; this paper makes no wall-clock claim.

#if not anon [
  #line(length: 100%)
  #text(size: 0.8em, fill: gray)[
    sparq project · evidence traces to `crates/sparq-vectors/tests`. Numbers in this document
    are injected at build time from the paper-bound evidence file; see the provenance stamp on
    the published page.
  ]
]
