<!-- [SONNET-4.6] Authored by Claude Sonnet 4.6 (SPARQ researcher). Bead sq-7d3dj.32.
Canonical EC2 measurement record: three-rung bytes-per-triple for WatDiv at 1M/10M/50M
triples, aarch64, dedicated quiet instance i-0798990569f269435, 2026-07-07.
Flag for Fable re-review when available. -->

# Memory footprint per triple — canonical measurement 2026-07

**Status:** canonical measurement record (design-for-review). **Date:** 2026-07-07.
**Bead:** sq-7d3dj.32 (parent `sq-7d3dj`). **Feeds:** `research/perf-dominance-gap-2026-07.md`
D7 row (memory footprint axis).

---

## 0. Purpose and scope

This record captures the **canonical bytes-per-triple envelope** for sparq's in-RAM and
on-disk storage modes across three WatDiv scales (1M / 10M / 50M triples). It provides the
measured evidence for the D7 row in the performance-dominance gap table, replaces the
CI-scale proxy (88–92 B/triple) that opened bead sq-7d3dj.32, and drives the improvement
bead program (sq-7d3dj.32.1–.32.6).

Two axes are tracked here, matching the maintainer's framing:

1. **In-memory bytes/triple** — what an operator pays in RAM for a loaded sparq store
   (raw 6-perm default and block-compressed mode).
2. **External-memory build peak** — peak RSS during the bounded-RAM build path (the
   "Wikidata on a 16 GB box" story), which the brief asserts and this data confirms is
   below the in-RAM heap floor at 50M scale.

---

## 1. Protocol

| Field | Value |
|---|---|
| **Instrument** | `bytes-per-triple` v1 (script `scripts/bench/bytes-per-triple.sh`) |
| **Instance** | `i-0798990569f269435` — dedicated quiet aarch64 EC2 box, self-terminated cleanly 08:26 UTC 2026-07-07 |
| **ISA / OS** | aarch64 |
| **Corpus** | WatDiv seed1 (SF=10 / SF=100 / SF=500 → actual triples: 1 085 794 / 10 889 330 / 54 557 829) |
| **Sha** | `5ce8b4f9de0b025660f114b059c03efb477f29df` |
| **Date** | 2026-07-07T08:08:05Z |
| **Shot count** | Single-shot per rung (not min-of-N — the layout metrics are deterministic and noise-immune; timing fields are not used for claims) |
| **Provenance tier** | **Canonical** (dedicated quiet instance, self-terminated; result envelope recovered from EC2 console output at `/tmp/d7-membpt-50m.json`) |

**Cross-ISA note.** The SPARQL-matrix gap table (D1–D4, `research/perf-dominance-gap-2026-07.md`)
was measured on a `c6i.4xlarge` (x86_64, Xeon Platinum 8375C). The layout metrics here
(heap_bpt, store_bpt, dict_bpt) are **deterministic and architecture-independent**: `Id` is
`u32` (4 bytes) on both ISAs, `Vec<[Id;3]>` has the same 24-byte header, and the B/triple
counts trace to data size, not ISA-specific alignment. OS-visible RSS/HWM may vary by
≤ a few percent across ISAs (page size is 4 KiB on both).

---

## 2. Three-rung result table

All values are **bytes per triple** (B/triple) for the actual triple count at each rung.

### 2.1 In-RAM raw mode (6-permutation default)

| Rung | Triples | heap\_bpt | store\_bpt | dict\_bpt | caches\_bpt | dict\_bpterm |
|---|---|---|---|---|---|---|
| 1M (SF=10) | 1 085 794 | 93.41 | 84.14 | 8.45 | 0.81 | 82.98 |
| 10M (SF=100) | 10 889 330 | 88.56 | 79.26 | 8.53 | 0.77 | 88.64 |
| 50M (SF=500) | 54 557 829 | **84.41** | **75.38** | **8.28** | **0.75** | **88.07** |

- `heap_bpt` = total self-accounted heap (store + dict + caches).
- `store_bpt` = permutation-index Vecs only.
- `dict_bpt` = dictionary contribution to heap.
- `caches_bpt` = planner-statistics + overlay caches.
- `dict_bpterm` = bytes per **distinct term** (not per triple; included for diagnosis).

**Trend.** The raw store falls toward the 72 B/triple theoretical floor for six
`Vec<[u32;3]>` permutations (6 × 3 × 4 B): 84.14 (1M) → 79.26 (10M) → 75.38 (50M).
The ~3.4 B/triple gap at 50M is Vec capacity slack in the SPO accumulator (covered by
sq-7d3dj.32.1). Dict cost (8.28 B/triple at 50M) is stable across scales for WatDiv's
vocabulary density.

### 2.2 In-RAM compressed mode (block-compressed perms + blob dict)

| Rung | Triples | comp\_heap\_bpt | comp\_store\_bpt | comp\_dict\_bpt |
|---|---|---|---|---|
| 1M (SF=10) | 1 085 794 | 42.50 | 36.75 | 5.75 |
| 10M (SF=100) | 10 889 330 | 54.73 | 48.75 | 5.98 |
| 50M (SF=500) | 54 557 829 | **54.54** | **48.75** | **5.79** |

**Trend (notable).** Compressed store cost rises sharply between 1M and 10M (36.75 → 48.75)
then **stabilizes** (48.75 at both 10M and 50M). This non-monotone behavior — where the
compressed store costs more per triple at scale than at 1M — is an open question addressed
by sq-7d3dj.32.2 (root-cause the block-size / encoding relationship to triple-id density at
scale). The dict contribution (5.75–5.98 B/triple) is stable.

### 2.3 OS-visible RSS and build high-water mark

| Rung | Triples | rss\_bpt | hwm\_bpt | gnu\_hwm\_bytes |
|---|---|---|---|---|
| 1M (SF=10) | 1 085 794 | 537.09 | 537.09 | 582 934 528 |
| 10M (SF=100) | 10 889 330 | 176.96 | 176.96 | 1 926 266 880 |
| 50M (SF=500) | 54 557 829 | **100.79** | **147.05** | **8 022 597 632** |

- `rss_bpt` = VmRSS / triple\_count post-load (OS-resident pages).
- `hwm_bpt` = VmHWM / triple\_count (peak OS RSS, includes build scratch).

**Key observation (50M).** Post-load RSS (100.79 B/triple) is ~1.20× the self-accounted
heap (84.41) — within the ~1.3× target of sq-7d3dj.32.5. The HWM (147.05 B/triple) reflects
peak RAM during the sort/build phase; the HWM-to-heap ratio of ~1.74× at 50M shows the
in-RAM build uses significant scratch (covered by sq-7d3dj.32.5: free radix scratch +
mimalloc purge). At 1M the RSS/HWM equals 537.09 B/triple — a large fixed-cost overhead on
a small corpus; at scale this amortizes to 100.79 at 50M.

### 2.4 On-disk footprint

| Rung | Triples | disk\_raw\_bpt | disk\_comp\_bpt |
|---|---|---|---|
| 1M (SF=10) | 1 085 794 | 80.59 | 38.14 |
| 10M (SF=100) | 10 889 330 | 80.32 | 40.23 |
| 50M (SF=500) | 54 557 829 | **80.22** | **44.45** |

The raw on-disk cost (~80 B/triple, scale-invariant) reflects the six-permutation binary
layout; the compressed on-disk cost grows slightly with scale (38 → 44 B/triple), mirroring
the compressed in-RAM behavior. For comparison, RDFox's published on-disk claim is
40–60 B/triple (`[ost-docs-feat]` in `research/rdfox-claims-inventory.md`).

### 2.5 External-memory build peak

| Rung | Triples | extbuild\_hwm\_bpt | extbuild\_hwm\_abs |
|---|---|---|---|
| 1M (SF=10) | 1 085 794 | 561.57 | ~610 MB |
| 10M (SF=100) | 10 889 330 | 155.71 | ~1.70 GB |
| 50M (SF=500) | 54 557 829 | **73.37** | **~4.00 GB** |

`extbuild_hwm_bpt` = peak RSS during the external-memory build process (building indexes
incrementally on disk rather than in full in-RAM). At 50M triples the external build needed
only **73.4 B/triple** of peak RAM — **below** the in-RAM heap floor of 84.4 B/triple for the
same corpus. This confirms sparq's external-memory path is more RAM-efficient than the
in-RAM load at scale (the crossover is between 10M and 50M).

At 1M triples the external build peaks at 561.6 B/triple (~610 MB), reflecting the
fixed-overhead nature of the build process (sorting scratch, index bootstrap structures)
that amortizes only at larger scale. The external-memory path is designed for large datasets
where the whole store exceeds available RAM, not as an optimization for small corpora.

---

## 3. Gap analysis vs competitor published / measured figures

### 3.1 RDFox (published claims, from `research/rdfox-claims-inventory.md`)

| RDFox claim | Source | sparq (50M, raw) | sparq (50M, compressed) | Verdict |
|---|---|---|---|---|
| 34.7 B/triple (best in-memory, 1.5 G triples) | `[iswc2015-slides]` (2015), mid-range server | 84.4 B/triple | 54.5 B/triple | **BEHIND** (raw ~2.4×; compressed ~1.6×) |
| 36.9 B/triple (in-memory) | `[iswc2015-paper]` (2015), `[reported]` | 84.4 B/triple | 54.5 B/triple | **BEHIND** (raw ~2.3×; compressed ~1.5×) |
| 45–85 B/fact (current product) | `[ost-docs-feat]` (2026-07-07) | 84.4 B/triple | 54.5 B/triple | raw above upper bound; compressed within band |
| 40–60 B/triple (on-disk) | `[ost-docs-feat]` (2026-07-07) | 80.2 B/triple (raw) / 44.5 B/triple (compressed) | — | raw above band; compressed within band |

**Honest verdict on D7.** The raw 6-perm in-RAM mode (84.4 B/triple at 50M) is
**BEHIND** RDFox's best-published in-memory figure by ~2.3× and above the upper end of the
current product range (45–85 B/fact). The block-compressed mode (54.5 B/triple at 50M) sits
**within** the current RDFox product band, representing comparable per-triple cost for this
corpus at this scale.

**Comparability caveats** (apply before any cross-engine claim):

1. **Scale mismatch.** RDFox's best-published figures are at 1.5 B triples (SPARC T5-8,
   4 TB). sparq's 50M is a 30× smaller corpus. The raw store trend
   (93.4 → 88.6 → 84.4 B/triple) is downward and has not converged at 50M; whether it
   reaches or approaches the 34–85 B/triple range at 1 B+ triples is **unmeasured**.
2. **Architecture.** RDFox's 34.7 B/triple was measured on "mid-range servers" (hardware
   unspecified in that entry); 36.9 B/triple on the SPARC T5-8 (128 cores / 1 024 threads /
   4 TB). Cross-ISA comparisons are indicative only.
3. **Index count.** RDFox uses three permutation indexes (`Ispo`, `Isp`, `Iop`) vs sparq's
   six by default. The 3-perm compact-index profile (sq-7d3dj.32.3) would give sparq a
   36 B/triple raw store floor — directly in RDFox's published best-case band.
4. **Dictionary design.** RDFox's B/fact figures include its hash-indexed triple table and
   dictionary; the exact decomposition is not published for direct comparison.

### 3.2 Oxigraph (open-source Rust peer)

**NOT-MEASURED.** The competitor matrix at the time of the gap table (D7 row, `research/perf-dominance-gap-2026-07.md`) contains no in-RAM B/triple figure for
Oxigraph; the matrix benchmarked query/load latency and correctness only. An Oxigraph
in-RAM memory measurement at comparable scale is a future instrument bead (would be added
to sq-7d3dj.32 scope or as a new child bead if needed).

### 3.3 External-memory build axis vs RDFox

RDFox is an **in-memory, centralised** store with no external-memory build path; it requires
the dataset to fit in RAM. The `extbuild_hwm_bpt` axis is therefore **not comparable** to
any published RDFox figure — it measures a capability RDFox does not offer. The honest
framing: sparq's external-memory build at 50M peaks at 73.4 B/triple (~4 GB), enabling a
50M-triple store to be built on a machine with as little as 6–8 GB RAM (build peak + OS
headroom). At 1 B triples the extrapolated external build peak would be approximately
73 B/triple × 1 G ≈ 73 GB — still bounded but requires a large-RAM machine; the bounded-RAM
design is validated as scaling sub-linearly in B/triple.

---

## 4. Root-cause breakdown (in-RAM raw mode at 50M)

| Component | B/triple at 50M | Theoretical minimum | Gap | Driver / bead |
|---|---|---|---|---|
| Store (6 perms) | 75.38 | 72.00 (6 × 3 × 4 B) | ~3.4 B | SPO Vec capacity slack (sq-7d3dj.32.1) |
| Dictionary | 8.28 | — | — | Blob compaction opportunity (sq-7d3dj.32.4) |
| Caches | 0.75 | — | — | pred\_stats accounting gap (sq-7d3dj.32.6) |
| **Total heap** | **84.41** | **~72 (store floor)** | | |

The single largest controllable lever is the index-count reduction via the compact-index
(3-perm) profile (sq-7d3dj.32.3): halving to three permutations gives a store floor of
36 B/triple, bringing the raw store into RDFox's best-published band. The compressed
mode (54.5 B/triple) is the second lever, already within the RDFox product range.

---

## 5. Fix bead inventory (all pre-existing, no new beads created)

All gaps identified by this measurement are already captured by the sq-7d3dj.32.x program
created when sq-7d3dj.32 was dispatched. No new beads are needed for the patterns seen
here:

| Bead | What | Status |
|---|---|---|
| sq-7d3dj.32.1 | Eliminate SPO Vec capacity slack (~3.4 B/triple at 50M) | IN\_PROGRESS |
| sq-7d3dj.32.2 | Root-cause compressed-store cost growth (36.75 → 48.75 B/triple, 1M→10M) + promote as a first-class native profile with query-latency delta | OPEN |
| sq-7d3dj.32.3 | Benchmark the 3-perm compact-index profile (36 B/triple store floor) vs 6-perm default | OPEN |
| sq-7d3dj.32.4 | Evaluate blob compaction after bulk load (~35% dict cut, 83→53 B/term) | OPEN |
| sq-7d3dj.32.5 | Root-cause and reduce post-load RSS/HWM gap (HWM 147 B/triple vs heap 84 at 50M): radix scratch free + mimalloc purge | OPEN |
| sq-7d3dj.32.6 | Fix `TripleStore::heap_bytes` omitting `pred_stats` map (honesty/accounting) | OPEN |

The data from this record confirms the quantitative inputs to each bead description above:
store slack ~3.4 B/triple (feeds .32.1), compressed growth pattern (feeds .32.2),
3-perm floor argument (feeds .32.3), dict B/term at scale (feeds .32.4),
HWM/RSS ratio at each rung (feeds .32.5).

---

## 6. Updated D7 verdict for the gap table

The D7 row in `research/perf-dominance-gap-2026-07.md` was previously
`NOT-MEASURED at scale / face-value PARITY-or-BEHIND`. The canonical measurement now
gives:

- **Raw in-RAM (50M WatDiv):** heap 84.4 B/triple, store 75.4, dict 8.3, caches 0.75.
- **Compressed in-RAM (50M WatDiv):** heap 54.5 B/triple.
- **External-memory build peak (50M WatDiv):** 73.4 B/triple (~4.0 GB absolute).

Updated verdict: **BEHIND RDFox best in-class** (raw ~2.3×); compressed mode **within
RDFox's current product band** (45–85 B/fact). Oxigraph **NOT-MEASURED** at scale.

Improvement trajectory: sq-7d3dj.32.1 (Vec slack) + .32.3 (3-perm) bring the raw store
toward 36–40 B/triple; .32.2 (compressed promotion) positions the compressed mode as the
preferred low-memory operating point within the RDFox band.

---

## 7. Open questions for the maintainer

1. **Scale target.** Should the memory benchmark extend to 1 B+ triples on a larger
   dedicated instance to confirm whether the downward trend in raw B/triple continues
   toward the RDFox best-in-class range? (Would require a high-RAM EC2 instance for an
   in-RAM 1 B load.)
2. **Compressed mode as default.** Is there appetite to make compressed mode the default
   for read-oriented CLI workflows (after .32.2 measures query-latency cost)? This would
   bring the headline B/triple figure into the RDFox product band without code changes.
3. **Oxigraph memory baseline.** Should the Oxigraph in-RAM B/triple measurement be added
   to the canonical competitor sweep so D7 has an OSS measured peer?

---

## Sources

All quantitative sparq figures in this record trace to the canonical envelope at
`/tmp/d7-membpt-50m.json` (recovered from EC2 console output of instance
`i-0798990569f269435`, self-terminated 08:26 UTC 2026-07-07). RDFox figures are third-party
published claims, attributed and cited via `research/rdfox-claims-inventory.md` (PR #1719).
No figures are fabricated or from work-box runs.
