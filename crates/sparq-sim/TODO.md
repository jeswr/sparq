# sparq-sim — gaps and follow-ups

Constraint honoured in v1: **no existing crate was modified**. Items below are either
deferred features (design doc phases) or places where a small `sparq-core` public-API
addition would help.

## Known v1 gaps

1. **Neighbor-sparse entities get no `most_similar` candidates** (olympics: Sport 0/0,
   City 2/4). In `PredicateNeighbor` mode an entity whose signature is only
   `(In, p, x)` elements from degree-1 neighbors (every event names exactly one sport;
   every athlete one birthplace) generates no candidates: the reverse scan from each
   neighbor finds only the entity itself. Fix: fall back to `Predicates`-mode
   candidate generation (or a characteristic-set index from `sparq-introspect`,
   phase 2) when fewer than `k` candidates are found. Pairwise `similarity()` is
   unaffected.
2. **Hub cap is global** (`max_pair_frequency`); a per-element budget proportional to
   IDF would spend the scan budget better.
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
- Hybrid score fusion (lexical + structural + vector RRF) once `sparq-vectors` exists.
