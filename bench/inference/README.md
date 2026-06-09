# Inference benchmarks

## DeepTaxonomy (DT) — the rule-heavy stress test
`gen_deeptaxonomy.py N` → 1 instance fact + an N-deep subClassOf chain + a transitivity rule.
The instance must be re-typed up the whole chain; NAIVE forward chaining re-derives the entire
closure every round (O(N²)), while semi-naive (delta-driven) + fact indexing + delta-first
join ordering is LINEAR.

Measured (sparq, fanless M1), N3 reasoner:

| N (levels) | naive (before) | semi-naive+index (now) |
|---|--:|--:|
| 1 000   | 0.46 s | **0.008 s** |
| 10 000  | 52 s   | **0.084 s** (620×) |
| 100 000 | (hours)| **0.96 s** |

Reference (RR-2023 paper, i7-1165G7 laptop) for DT-1000: EYE-fw 0.1 s, VLog 1.6 s, Nemo 1.7 s,
cwm 180 s. sparq at 0.008 s is ~12× faster than EYE-fw and ~200× faster than VLog/Nemo on DT-1000
(different hardware; indicative). See research/inference-sota.md for the full landscape.

## RDFS/OWL materialization (semi-naive + index)
The RDFS materializer (`materialize_rdfs`) also moved from naive fixpoint to semi-naive +
incremental `RdfsIndex` (per-rule delta-driven joins, both transitivity directions). On the
realistic pattern (instances × a moderate class hierarchy) the win is large:

| RDFS workload | naive | semi-naive |
|---|--:|--:|
| 100 000 instances × depth-20 hierarchy (→ 2.1 M closure) | (slow) | **0.69 s** (~3 M triples/s) |
| pure subClassOf chain N=1000 (→ 500 k O(N²) closure) | 7.6 s | 2.36 s |

(The pure chain is bounded by its quadratic closure size; the instance-heavy case is the
representative RDFS/OWL-RL workload where semi-naive's delta-driven joins pay off.)
