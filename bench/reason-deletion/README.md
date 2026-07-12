# reason-deletion — deletion-workload correctness + timing for incremental reasoning

> 🤖 SPARQ agent — written by **Claude Fable 5** (bead `sq-31fza`; parent `sq-6tykl.4`;
> comparative axis registered under `sq-hmd7l` in `bench/benchmarks.toml`).

The CORRECTNESS + MEASUREMENT half of the FBF/DRed-grade retraction program: randomized
insert/**delete** batches at several deletion ratios over a LUBM-class materialized
closure, with a **differential-correctness gate** — after the randomized sequence the
incrementally-maintained closure must be set-equal to a from-scratch re-materialization
(and ABox deltas must never have triggered a full rebuild). This is what separates real
incremental deletion (RDFox FBF/DRed/B-F; GraphDB smooth-delete) from a demo reasoner.
Any `incremental.rs` OPTIMIZATION found wanting here goes to a follow-up sparq-reason
bead — this suite is bench-dir-only by design (the crate was in-flight).

## How it works

```
bash bench/reason-deletion/run.sh                       # per-commit tiers (small medium)
REASONDEL_TIERS="small medium large" bash bench/reason-deletion/run.sh   # + nightly tier
```

1. `gen.sh <units>` → a deterministic LUBM-class synthetic ABox (`gen_reason_deletion.py`;
   1 unit = 1 athlete + 1 result = 8 triples; shared teams/games/events scale
   sub-linearly, like LUBM's departments-per-university).
2. The **existing, unmodified** driver
   `crates/sparq-reason/examples/incremental_olympics_bench.rs` runs on that corpus. The
   driver itself performs the randomized (fixed-seed xorshift) insert/delete batches over
   three workloads — RDFS (`MaterializedGraph`), OWL 2 RL mono + fixpoint
   (`MaterializedOwlGraph`) — and carries the load-bearing built-in asserts:
   incremental == from-scratch closure after the sequence, zero full rebuilds on ABox
   deltas, plus the N3/WAC section's own asserts. A violated assert panics → non-zero
   exit → `run.sh` is RED.
3. `parse_and_assert.py` additionally pins every tier's parsed ABox count and all three
   closure sizes to `expected.tsv` (deterministic; catches silent under/over-derivation
   and generator drift), emits `<metric>\t<value>\t<unit>` TSV timing lines, and writes
   the standard JSON results envelope (default under `/tmp/reason-deletion/`, git-ignored
   territory; `REASONDEL_JSON_OUT` overrides). The envelope's host block marks work-box
   timings NON-canonical (`quiet_box: false`) — timings are trend-only, never perf-gated
   and never baked into docs.

## Deletion ratios

The driver's batch sizes are fixed (1 / 100 / 10,000 triples), so the ratio axis comes
from varying the base size per tier: a 10,000-triple delete batch is ~41% of the small
tier's ABox (24,160 triples), ~8.3% of medium (120,000), ~1.7% of large (600,000). The
envelope records `delete_ratio = delta / abox_triples` per cell. Ratios here are dataset
geometry (fixture constants), not measured performance.

## Honesty notes

- **Why not literal LUBM data:** the driver synthesizes its TBox over the olympics
  vocabulary (`foaf:Person`/`dbo:team`/`oly:athlete`/…), and this bead forbids touching
  sparq-reason source. Feeding it univ-bench IRIs would make every rule a no-op and the
  deletion workload vacuous. The generator is LUBM-**class** (deterministic,
  scale-parameterized instance generator over a fixed schema with class hierarchies and
  domain/range/subPropertyOf/inverseOf-typed properties) bound to the vocabulary the
  driver's TBox actually ranges over, so the closure is materially larger than the ABox
  and deletion has real re-derivation work.
- **Pinned tier sizes:** the driver's delete-sampler excludes the six TBox *predicates*
  but not the `[locatedIn, rdf:type, owl:TransitiveProperty]` axiom-typing triple; if
  drawn, deleting it legitimately takes the documented rebuild path and fails the
  no-rebuild assert. Tier unit counts are pinned to values where the fixed-seed draw
  sequence never hits it (fully deterministic — see `expected.tsv`). The driver-side fix
  (exclude axiom-typing triples from ABox sampling) is a sparq-reason change, tracked as
  follow-up bead `sq-x58ow`.
- **Timing scope:** delta timings are single-shot (the driver measures one batch per
  size) and machine-dependent — comparative claims belong in envelope-carrying gathers
  on a quiet box, per `bench/CATALOG.md`. The RDFox/GraphDB competitor comparison is the
  parent bead's follow-up, not part of this suite.
