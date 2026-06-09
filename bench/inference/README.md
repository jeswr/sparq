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
