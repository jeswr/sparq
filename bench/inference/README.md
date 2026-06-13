# Inference benchmarks

## DeepTaxonomy (DT) — the rule-heavy stress test
`gen_deeptaxonomy.py N` → 1 instance fact + an N-deep subClassOf chain + a transitivity rule.
The instance must be re-typed up the whole chain; NAIVE forward chaining re-derives the entire
closure every round (O(N²)), while semi-naive (delta-driven) + fact indexing + delta-first
join ordering is LINEAR.

This harness measures the N3 reasoner's semi-naive+index materialization vs the naive
fixpoint across the DT-N transitive-closure workload (N = 1k / 10k / 100k levels). Run it
for the numbers (the harness prints both the before/after times and the speedup).

Reference (RR-2023 paper, i7-1165G7 laptop) for DT-1000: EYE-fw 0.1 s, VLog 1.6 s, Nemo 1.7 s,
cwm 180 s — sparq is faster than EYE-fw and substantially faster than VLog/Nemo on DT-1000
(different hardware; indicative). These are CITED external figures; see
research/inference-sota.md for the full landscape.

## RDFS/OWL materialization (semi-naive + index)
The RDFS materializer (`materialize_rdfs`) also moved from naive fixpoint to semi-naive +
incremental `RdfsIndex` (per-rule delta-driven joins, both transitivity directions). The
harness measures the realistic pattern (instances × a moderate class hierarchy, where the
semi-naive win is large) and a pure subClassOf chain (bounded by its quadratic closure
size); run it for the figures.

(The pure chain is bounded by its quadratic closure size; the instance-heavy case is the
representative RDFS/OWL-RL workload where semi-naive's delta-driven joins pay off.)
