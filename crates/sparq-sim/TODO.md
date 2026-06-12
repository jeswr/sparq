# sparq-sim — gaps and follow-ups

Constraint honoured in v1 and v2: **no existing crate was modified**. Items below are
either resolved here or deferred with rationale.

## v1 gaps — status

1. ~~**Neighbor-sparse entities get no `most_similar` candidates**~~ **DONE** —
   `SimConfig::profile_fallback` (default on): when `(predicate, neighbor)` candidate
   generation yields fewer than `k` results, the remaining slots fill with
   `Predicates`-mode candidates (entities sharing a `(direction, predicate)` role),
   ranked BELOW every exactly-scored result and scored by the predicate-profile
   Jaccard. Predicate blocks are scanned most-selective-first and generation stops at
   `max(4k, 64)` fallback candidates, so hub blocks are touched only when starvation
   is real. `profile_fallback: false` restores the strict v1 behaviour.

   Re-measured on the real olympics eval (1.78M triples,
   `cargo run -p sparq-sim --example olympics_eval --release`), before → after:

   | class | precision@10 before | after |
   |---|---|---|
   | dbo:Sport | 0.000 (0/0 — NO candidates) | **1.000 (400/400)** |
   | dbo:City | 0.500 (2/4) | **0.995 (398/400)** |
   | foaf:Person / dbo:SportsTeam / dbo:SportsEvent / dbo:Olympics | 1.000 | 1.000 |
   | **overall** | 0.999 (1489/**1491** scored) | 0.999 (2398/**2400** scored) |

   Candidate coverage went from 1 491/2 400 returned results to full 2 400/2 400
   (every seed now returns k = 10). Latency cost: `most_similar(k=10)` mean
   1.48 → 1.75 ms, p50 0.29 → 0.77 ms, p95 8.88 → 10.37 ms (240 calls, M1).

2. **Hub cap is global** (`max_pair_frequency`); a per-element budget proportional to
   IDF would spend the scan budget better. **MEASURED-AND-REJECTED**: with the
   profile fallback in place the eval is saturated — precision@10 is 1.000 on five
   of six classes and 0.995 on the sixth, with full candidate coverage (2398/2400)
   at mean 1.75 ms — so a per-element budget has nothing left to win on this
   dataset: it cannot raise precision (ceiling) and the scan cost it would shave is
   already bounded by the global cap + the fallback's selectivity ordering and
   `max(4k, 64)` stop. Revisit only with a denser benchmark graph where candidate
   generation provably misses sub-cap evidence.

3. ~~**No graph-level entity enumeration in `sparq-core`'s public API**~~ **DONE
   (engine-seams wave)**: `Graph::iter_ids` (S-sorted: distinct subjects fall out
   of run boundaries) / `Graph::iter_ids_sorted(2)` (O-sorted: distinct objects),
   plus `Dict::iter()` for vocabulary-level enumeration — all borrowing, zero
   alloc per row. Swapping the `rdf:type`-block workaround for these is this
   crate's owner's call.

## Deferred (per research/genai-design.md phasing)

- v1.1: FST lexical tier over the sorted dictionary; T-box-aware signatures (closure
  via `sparq-reason`, opt-in).
- Phase 4 escape hatch: MinHash/LSH signature sketches if candidate generation hits
  its scaling wall on graphs denser than olympics.
- ~~Hybrid score fusion (lexical + structural + vector RRF) once `sparq-vectors`
  exists~~ — landed on the `sparq-vectors` side (`fuse_rrf` / `fuse_rrf_weighted` /
  `fuse_scores`), keeping the crates independent.
