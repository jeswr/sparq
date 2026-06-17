<!-- [OPUS-4.8] sq-lrp9 — HDT load-and-decode suite. Design: research/capability-benchmark-program.md §3.7. -->
# HDT load-and-decode suite

The HDT analogue of the LUBM / SHACL / FTS / RSP template: a self-asserting
per-commit runner over a **fixed `.hdt` archive**, built around a
**load-and-decode correctness gate** with **advisory** load timing on the side.

It exercises [`sparq-hdt`](../../crates/sparq-hdt): loading an HDT
(Header-Dictionary-Triples) archive — the de-facto binary RDF archive format
(FourSectionDictionary with Plain Front Coding + BitmapTriples, SPO order, as
written by hdt-cpp / hdt-java) — straight into a sparq `Graph` and resolving
triple patterns over the decoded graph's own index.

## The like-for-like axis is LOAD-AND-DECODE only (honest)

**Critical caveat (design §3.7):** `sparq-hdt` **decodes HDT into sparq's own
`Dict`/`Graph`** — it treats HDT as an *ingest format*, then queries its own
permutation indexes. The reference engines **hdt-cpp / hdt-java query the
compressed BitmapTriples in place** — they treat HDT as a *queryable store*. So
**load-and-decode-to-native is the ONLY like-for-like comparison**; a "query over
HDT" head-to-head is **NOT** like-for-like (different data structure, different
work) and is an explicit **NON-goal**. This suite gates and compares **load +
decode** only. (On the honesty ladder in §4.2, HDT load-decode-vs-hdt-cpp is a
*fair* comparison; query-over-HDT is not.)

## The gate: load-and-decode counts (deterministic)

The runner loads the **vendored real-world `snikmeta.hdt`**
([`crates/sparq-hdt/tests/fixtures/snikmeta.hdt`](../../crates/sparq-hdt/tests/fixtures/snikmeta.hdt)
— a hdt-cpp/java-shaped archive, **NOT** produced by sparq's writer; the `hdt`
crate's own tests assert its 328 triples) and asserts every `count`-unit metric
against [`expected.tsv`](./expected.tsv). `run.sh` exits non-zero on any drift —
exactly like LUBM's count diff. Five metrics, all asserted:

| metric | what | meaning |
|---|---|---|
| `snikmeta_triples` | stored triples of the decoded graph | the HDT `expected-rows.tsv` (328) |
| `snikmeta_terms` | distinct dictionary terms interned into sparq's `Dict` | the id-translation produced exactly this many terms |
| `snikmeta_distinct_predicates` | distinct predicates via a full SPO scan | triple-pattern resolution over the decoded graph's index |
| `snikmeta_rdf_type_triples` | a single bound-predicate triple-pattern resolution | count of `rdf:type` triples |
| `snikmeta_direct_eq_upstream` | `1` iff the direct decoder == the upstream-backed path on the SAME bytes | the id-translation **oracle** (the exhaustive form is in `tests/roundtrip.rs`) |

The first two (`_triples` / `_terms`) are the **load-and-decode-to-native** gate;
`_distinct_predicates` / `_rdf_type_triples` add **triple-pattern resolution on
the same archive** (design §3.7(a)); `_direct_eq_upstream` pins the
`.hdt` → sparq `Dict` id-translation against the wrapped `hdt` crate's own
`Hdt::read` + per-id `id_to_string` path. These complement the differential +
rejection oracles already in
[`crates/sparq-hdt/tests/roundtrip.rs`](../../crates/sparq-hdt/tests/roundtrip.rs)
(triple-for-triple HDT-vs-N-Triples equality, multi-codec decode, truncation /
byte-flip rejection); the bench pins the *shape* of a fixed load so a regression
alerts on the dashboard.

> **Write-path note.** Optionally the **bytes-on-disk of a `save`-produced
> archive** could become a gate once the in-memory PFC+BitmapTriples encoder
> (bead `sq-ashy`) is the production write path; until then write still goes
> through a temp-NT round-trip, so **encode-perf parity is an explicit non-goal**
> and no write metric is gated here.

## Advisory timing (trend-only, NOT a gate)

The runner also emits two advisory rows on a deterministic **synthetic** archive
(default ~250k triples; override with `$HDT_BENCH_N`):

- `hdt_load_s` — wall-clock to load+decode the `.hdt` to a native `Graph`
  (seconds, **machine-sensitive**, trend-only).
- `hdt_vs_ntgz_load_s` — the **ratio** of gzipped-N-Triples load time to HDT
  load time on the *same* triples. A unitless ratio **survives box contention**
  (per [`bench/CATALOG.md`](../CATALOG.md)), so it is the more trustworthy
  advisory signal — but it is **NEVER** asserted; the deterministic counts above
  are the real gate.

The fuller perf sketches — the ~1M-triple HDT-vs-`.nt.gz` headline and the
direct-vs-upstream-decoder A/B — live in
[`crates/sparq-hdt/examples/bench_load.rs`](../../crates/sparq-hdt/examples/bench_load.rs)
and `examples/bench_direct_vs_upstream.rs` (the `hdt-load-bench` / `hdt-stage-split`
registry entries).

## Competitor: hdt-cpp (decode-only)

The natural HDT peers are the **rdfhdt reference implementations** — **hdt-cpp**
and **hdt-java** (core LGPL, tools Apache; the `rdfhdt/hdt-cpp` Docker image
exists; tools `rdf2hdt` / `hdt2rdf` / `hdtSearch`). The **fair** benchmark is
**load time + memory + triple-pattern resolution on the same `.hdt`**, matching
the mapped (`mapHDT` / mmap) vs loaded (`loadHDT` / in-RAM) regimes — and **only**
that axis (query-over-HDT is not like-for-like, see above). The competitor is
registered in [`bench/competitors.json`](../competitors.json) (`hdt-cpp`) as a
**docker / `report-cli`** adapter, gather-only (Docker is not on the dev box;
zero recurring CI cost), behind the `hdt` cargo feature. Numbers ship **empty**
in git and are populated only by a real measured gather run.

## Files

| file | role |
|---|---|
| `crates/sparq-hdt/examples/bench_oracle.rs` | the TSV-emitting runner (the crate is isolated — not a `sparq-cli` dependency, so the runner is a crate example, like FTS's `bench_text` / RSP's `rsp_oracle`) |
| `expected.tsv` | the deterministic load-and-decode gate (snikmeta triples / terms / predicates + the direct==upstream id-translation oracle) |
| `run.sh` | self-asserting entry point CI calls: run the example, assert every `count`-unit metric vs `expected.tsv`, forward the 3-column `<metric>\t<value>\t<unit>` hook contract |

## Run it

```sh
cargo build --release -p sparq-hdt --example bench_oracle
bench/hdt/run.sh                       # asserts + prints the metric TSV; exit 1 on any drift
HDT_BENCH_N=1000000 bench/hdt/run.sh   # bigger synthetic archive for the advisory timing
```

A divergence in `expected.tsv` means the HDT decode / id-translation /
triple-pattern resolution changed — regenerate only after confirming the change
is intended (and update the differential oracle in `tests/roundtrip.rs` if so).
