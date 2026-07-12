# Inference benchmarks

## N3 competitor columns — `eye-comparison.sh` (EYE / cwm / jen3)

`bench/inference/eye-comparison.sh` compares sparq's N3 forward closure against
**EYE** (the pinned reference — required; the run fails early without it) and two
optional columns added by sq-hmd7l.11: **cwm** (W3C SWAP, Python — the honest
*slow* column) and **jen3** (`java -jar jen3.jar`, a Java/Apache-Jena fork with N3
support — *not* an npm library). An absent optional tool prints `absent` in its
column and the run stays green (graceful skip). Knobs: `CWM=`, `JEN3_JAR=`,
`JAVA=`, plus heavy-cell opt-ins `CWM_HEAVY=1` and `EYE_DT100K=1` /
`JEN3_DT100K=1` / `CWM_DT100K=1`.

**Correctness gate before timing** (mirrors `bench/deep-taxonomy/run.sh`): each
workload's sparq closure count is asserted against its deterministic structural
expected size, and each present competitor's closure is cross-checked against the
same count *before* its cell is timed — counting is done by sparq's N3 parser
over a rule-free document (asserted; re-derivation over rules could mask an
under-deriving competitor). A mismatch fails the run loudly. See
`research/gap-n3-2026-07.md` for the method + fidelity caveats.

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
