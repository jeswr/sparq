# Predicate transfer / bitmap semi-join — measured characterization (M1)

`research/optimization-techniques.md` flagged **semi-join reduction / predicate transfer**
(Yannakakis family; "Not Yannakakis" CIDR'26 exact bitmap semi-join on dense u32 ids) as
the #1 open lever — "almost custom-built for sparq", promise 5, with the caveat that the
*uniform* synthetic shows no win and it needs a selective/skewed multi-join workload to
validate. This note builds that workload and measures it.

## The workload (bench/selective/gen2.py + queries2/two_sided.rq)

A **two-sided selective** query — a rare predicate at BOTH ends of a 2-hop path through a
dense `follows` middle:

```sparql
?a ex:premA ?x .  ?a ex:follows ?b .  ?b ex:follows ?c .  ?c ex:premC ?y
```

premA/premC each tag 0.1% of nodes. Bind join (the shipped index-nested-loop join) seeds
from ONE selective end and propagates, so it must expand `a -> b -> c` through `follows`
BEFORE the far-end premC can prune — the intermediate at `c` grows as |seed|·fanout²
even though the result is tiny. Predicate transfer would first build premC's value-domain
(a dense-id bitmap) and prune the `b -> c` expansion up front.

## Measured scaling (this M1, count mode; intermediate = peak − load-only-peak)

| fanout | edges | result rows | intermediate (est) | count time | extra mem vs load |
|---:|---:|---:|---:|---:|---:|
| 4   | 2.0M  | 8    | ~8k   | **0.89 ms** | ~0 MB    |
| 50  | 5.0M  | 274  | ~250k | **7.46 ms** | ~14 MB   |
| 200 | 10.0M | 1985 | ~2M   | **48.7 ms** | **208 MB** |

(load-only peak measured by pointing `bench` at an empty query dir.)

## Findings

1. **The lever is REAL but DENSITY-CONDITIONAL.** Cost grows ~quadratically with fanout
   (the 2-hop dense path). On sparse graphs (fanout 4) bind join already keeps the
   intermediate at ~8k rows / sub-ms — predicate transfer saves nothing. On dense graphs
   (fanout 200, social-network scale) it's a genuine 208 MB / 49 ms cost that predicate
   transfer would cut to ~thousands of rows / a few ms — a **10×+ win**.
2. **Why sparq's payoff is SMALLER than other engines'.** The dramatic predicate-transfer
   wins reported elsewhere come from eliminating *large* intermediates. sparq's
   intermediates are already compact — `SmallVec<[Id;4]>` rows are ~16–32 B, so even a
   250k-row intermediate is only ~14 MB. Combined with bind join already propagating ONE
   side's selectivity, the prunable surface is just the second side's over-expansion.
   So the lever matters at high fanout/scale, not at the modest sizes where it dominates
   in row-store engines.
3. **This is a TWO-SIDED gap specifically.** The shipped bind join fully solves
   single-selective-point joins (35×, see README.md). The open case is selectivity at
   *both* ends of a dense path, where one-directional propagation over-expands the middle.

## Recommended implementation (when prioritized) — bidirectional, not full Yannakakis

The prior measured note (project memory) rejected *naive* Yannakakis because it SCANS large
relations; bind join doesn't. The right shape here keeps that property: a **bidirectional
bind join / meet-in-the-middle** seeded from BOTH selective ends —

- from premA: `a -> b` via the index → ~|premA|·fanout `b` values;
- from premC: `c -> b` via the OPS/POS index (objects = premC's c's) → ~|premC|·indeg `b`s;
- **intersect the two `b` sets** (dense-id bitmap membership, O(1) branch-free — the
  "Not Yannakakis" primitive) → only `b`s on a valid two-sided path; then assemble.

This never scans the full `follows` relation (both expansions are index lookups from the
selective seeds) and prunes the middle to the intersection. Gate it on the bind-join
heuristic seeing TWO selective patterns whose join variables meet at a shared middle var.

**Validation bar (project discipline):** any such planner change must pass the differential
fuzzer vs Oxigraph (100k+ cases, count + ORDER-BY-sequence) with the new path FORCED on
every eligible join, plus no regression on the non-selective 10M/100M synthetic. The
bitmap intersection is exact (dense ids, zero false positives), so correctness risk is in
the orchestration/routing, not the primitive.

## Status

Benchmark + measured characterization committed. The optimization itself is a planner
change (bidirectional bind join) — deferred to a focused, fuzz-gated session rather than
shipped blind, since the payoff is density-conditional and the routing needs care to avoid
regressing the common single-sided / sparse case.
