# Wikidata low-resource ingest — stage 1 (r8g.large, 16 GB)

**Status: COMPLETE.** All runs finished 2026-06-12T21:50:10Z (remote driver `DONE2`
marker); full logs collected to the supervising machine (`/tmp/sparq-wdbench/results/r/`,
archive `results-final.tgz`). Instance i-0b3e0be20affc86cf terminated 2026-06-12T22:47Z,
verified `terminated`; no other purpose=sparq-hw-validation instances exist.

Goal: measure sparq's external-memory index build of real Wikidata `latest-truthy` slices
from COMPRESSED files (gz, zst) on a deliberately low-resource machine, as the
"low-resource" anchor of the ingest curve and as published-numbers-only context against
RDFox. RDFox is RAM-only by design (no out-of-core build); sparq's claim under test is
that ingest works when the indexes exceed RAM. The full 8–9.4 B-triple truthy run is NOT
part of this stage (not yet authorized; stop at 1 B).

## Machine

| item | value |
|---|---|
| instance | AWS r8g.large on-demand, eu-west-2 (i-0b3e0be20affc86cf, launched 2026-06-12T18:53:01Z) |
| CPU | 2 vCPU Graviton4 (Neoverse-V2, 2 physical cores, no SMT), 36 MiB L3 |
| RAM | 16 GB (15 GiB usable) + 64 GiB swapfile (so over-RAM runs degrade measurably instead of OOM-killing; per-run swap delta recorded) |
| disk | 350 GB gp3, provisioned 4000 IOPS / 250 MB/s. NOTE: r8g.large *sustained* EBS bandwidth is 78.75 MB/s (burst 1250 MB/s ≤30 min) — the honest IO bound for long builds |
| OS / toolchain | Ubuntu 24.04 arm64 (ami-03018a249a89a9ec1), rustc via rustup minimal, sparq @ `4aac23af` (public main), `cargo build --release -p sparq-cli` (default features: mmap + mimalloc) |

## Protocol (exact commands)

Data: live `https://dumps.wikimedia.org/wikidatawiki/entities/latest-truthy.nt.gz`,
first 16 GiB fetched by 2 ranged-HTTP workers (1 GiB ranges). Slices are the first
N lines, one streaming pass each, recompressed BOTH ways during the pass:

```text
rapidgzip -d -c -P 2 truthy-head.gz | head -n $N \
  | tee >(pigz -p 2 -6 -c > $L.nt.gz) >(wc -l > $L.count) \
  | zstd -3 -T2 -q -f -o $L.nt.zst
```

Each build (fresh idx dir, caches dropped, single run — medians are a luxury at this
budget; noted honestly):

```text
SPARQ_SHARDED_DICT=1 SPARQ_BUILD_TIMING=1 /usr/bin/time -v \
  sparq-cli build $L.nt.{gz,zst} ntriples ~/idx 32     # 32M-triple chunks
```

Validation per build (mmap open + COUNTs, RSS via /usr/bin/time -v):

```text
sparq-cli query-mmap ~/idx 'SELECT (COUNT(*) AS ?c) WHERE { ?s ?p ?o }'
sparq-cli query-mmap ~/idx '... { ?s <http://www.wikidata.org/prop/direct/P31> ?o }'
sparq-cli query-mmap ~/idx '... { ?s <http://www.w3.org/2000/01/rdf-schema#label> ?o }'
```

In-memory contrast at 100 M (the "RDFox-style all-in-RAM" data point), cgroup-capped so
an over-RAM attempt OOMs instead of silently swapping:

```text
sudo systemd-run --scope --wait -p MemoryMax=15G -p MemorySwapMax=0 \
  /usr/bin/time -v sparq-cli query 100M.nt.zst ntriples 'SELECT (COUNT(*) AS ?c) ...'
```

1 B is run once, in whichever compressed format was faster at 300 M.

## Results

### Slices (first N lines of live latest-truthy; 16 GiB head downloaded in 2,087 s ≈ 8.2 MB/s)

| slice | lines | .nt.gz (pigz -6) | .nt.zst (zstd -3) | slice-pass wall |
|---|--:|--:|--:|--:|
| 100M | 100,000,000 | 1,085,057,568 B (1.09 GB) | 810,409,179 B (0.81 GB) | 118 s |
| 300M | 300,000,000 | 2,813,353,808 B (2.81 GB) | 2,024,932,855 B (2.02 GB) | 353 s |
| 1B | 1,000,000,000 | 9,230,856,893 B (9.23 GB) | 6,991,319,462 B (6.99 GB) | 1,179 s |

### External-memory build curve (`build … 32`, single runs — medians were out of budget)

| run | wall | input triples/s | peak RSS | swap used | index on disk | B/triple |
|---|--:|--:|--:|--:|--:|--:|
| 100M · gz | 103.98 s | 0.962 M/s | 6.87 GiB | 0 | 9,513,512,133 B (9.51 GB) | 95.2 |
| 100M · zst | 98.81 s | 1.012 M/s | 6.60 GiB | 0 | 9,513,512,133 B | 95.2 |
| 300M · gz | 554.51 s | 0.541 M/s | 13.34 GiB | ~2 MiB | 27,248,936,340 B (27.25 GB) | 90.9 |
| 300M · zst | 516.94 s | 0.580 M/s | 13.52 GiB | ~28 MiB end-delta; ~3.6 GiB transient peak† | 27,248,936,340 B | 90.9 |
| 1B · zst | 4,691.2 s (1:18:11) | 0.213 M/s | 14.87 GiB (RAM-clamped) | **~20.4 GiB peak**; 33 MiB end-delta | 89,546,194,884 B (89.55 GB) | 89.6 |

† "swap used" was originally recorded as the before/after `free` delta, which only
captures end-state. A 30 s `free -m` sampler ran alongside all builds and shows transient
swap peaks the deltas miss: ~6.9 GiB during 300M·gz and ~3.6 GiB during 300M·zst (both
during the sibling-sort phase), vs the ~2 / ~28 MiB end-deltas in the rows above.

Dedup is real-Wikidata: 100M input → 99,918,830 distinct; 300M → 299,758,126 distinct;
1B → 999,268,108 distinct. zst beats gz by ~5–7% wall; format chosen for 1B: **zst**.
(First 1B attempt failed in 0.01 s — wrong slice filename in the driver script, not an
engine failure; rerun with the correct path succeeded.)

1B phase breakdown (`SPARQ_BUILD_TIMING`): parse+intern+spill 1,035.5 s (parallel) →
merge_remap 1,516.8 s + triple-remap 1,352.6 s (serial; 1,768.9 s wall to end of remap) →
k-way SPO merge done at 1,870.8 s → sibling sorts ∥ dict-save done at 4,026.4 s → total
4,691.2 s. CPU utilisation 30% (vs ~100%+ at smaller scales); 7.73 M major page faults;
472.8 M filesystem input blocks — the build is swap/IO-bound, as predicted.

**The headline finding is NEGATIVE for the "bounded-RSS" expectation:** peak RSS is NOT
~constant across scales. The external-memory build bounds the *triple-sorting* memory
(chunked runs + k-way merge), but the term **dictionary remains RAM-resident**
(`extsort.rs`: "only one chunk of triples + the dictionary live in RAM at once"), so RSS
grows with distinct terms: 6.6–6.9 GiB @100M → 13.3–13.5 GiB @300M (93% of the box).
Linear extrapolation puts 1B at ~36–40 GiB — far over 16 GB.

**Measured 1B outcome (swap-mediated, as predicted):** RSS clamps at the physical-RAM
ceiling (14.87 GiB peak) and the rest spills to swap — sampler peak 20.4 GiB swap at
T+55 min, peak total footprint (mem used + swap used) **35.0 GiB**, squarely inside the
36–40 GiB extrapolation. The build *completes correctly* (exit 0, all validation queries
pass) but throughput degrades super-linearly: 1.012 → 0.580 → 0.213 M triples/s at
100M → 300M → 1B (4.8× slower per-triple than 100M), with CPU utilisation collapsing to
30%. Conclusion stands: out-of-core triple sorting works, but the RAM-resident dictionary
is the scaling bottleneck; past ~2× RAM it trades wall-time for feasibility via swap.

### Validation (query-mmap per build; all passed)

Open from cold is O(1)-ish and tiny: 0.011–0.352 s, 28–122 MB RSS — **including at 1B**
(opened 999,268,108 triples in 0.352 s cold / 0.055 s warm, ~120 MB RSS). `COUNT(*)`
returns in 17–66 ms @100M, 16–31 ms @300M, 37 ms @1B; P31 and rdfs:label COUNTs in
0.1–18 ms (@1B: 78.9 ms and 40.6 ms). The opened triple totals (99,918,830 /
299,758,126 / 999,268,108) match the deduplicated build outputs exactly.
(Note: `query-mmap` reports solution counts + opened-triple totals; per-predicate COUNT
*values* are not printed by the CLI, so cross-format consistency + opened totals are the
validation evidence.)

### In-memory contrast @100M

Cgroup-capped (`systemd-run --scope -p MemoryMax=15G -p MemorySwapMax=0`) fully
in-memory load of 100M.nt.zst **succeeded**: 99,918,830 triples loaded in **61.737 s
(1.62 M/s)**, peak RSS **10.14 GiB**; engine self-report: store ~10.39 GB (104 B/triple),
dict ~2.32 GB (33,799,927 terms, 69 B/term); `COUNT(*)` answered in 1.4 ms. So in-RAM
load is ~1.6× faster than the external-memory build at 100M (61.7 s vs 98.8 s) at the
cost of holding everything resident — consistent with the RDFox-style "RAM-only is
faster while it fits" trade-off. (A first attempt exited rc=1 due to a `systemd-run`
flag conflict — `--wait` with `--scope` — not memory pressure; rerun without `--wait`
succeeded.)

## RDFox context (published numbers ONLY — not measured, different hardware)

No RDFox was installed or run: RDFox has no free production license (its no-cost option
is an evaluation-only license, <https://www.oxfordsemantic.tech/rdfox-evaluation-license>);
a same-hardware head-to-head awaits a license. Numbers below are published by Oxford
Semantic Technologies or third parties, on hardware vastly larger than this box —
**none of them are comparable to the table above in absolute terms**.

- **Memory per triple (RDFox docs, v7.6 "Features and Requirements"):** "Fact storage
  costs typically vary between 45 and 85 bytes per fact", plus "an additional 10–100% …
  for operating memory costs". <https://docs.oxfordsemantic.tech/features-and-requirements.html>
  (Older 5.3 docs: "between 60 and 110 bytes of RAM per triple",
  <https://docs.oxfordsemantic.tech/5.3/features-and-requirements.html>; the academic
  RDFox paper, ISWC 2015, reports ~36.9 B/triple in its most compact configuration,
  <https://www.cs.ox.ac.uk/boris.motik/pubs/npmhwb15RDFox-scalable.pdf>.)
  Consequence for THIS box: at 45–85 B/fact, 16 GB of RAM bounds an in-RAM RDFox store
  to roughly **0.19–0.36 B triples** before operating overhead — 100 M fits, 300 M is
  marginal, 1 B does not fit. RDFox documents no out-of-core build path.
- **OST Wikidata load (their blog, updated Jan 2026):** "the initial load took us only
  2 hours and 50 minutes for the entire 15 billion triples" (2021), and "As of RDFox
  v7.5, the initial load of Wikidata can now be completed in 24 minutes and 8 seconds"
  (~10.4 M triples/s derived — derivation ours). **Hardware unstated** in both cases;
  given their own B/fact figure, a 15 B-triple store needs ≳0.7–1.3 TB RAM, i.e. a
  machine ~2 orders of magnitude larger than this one.
  <https://www.oxfordsemantic.tech/blog/enhancing-wikidata-performance-with-rdfox-how-to-dissect-the-worlds-leading-rdf-database-faster>
- **Third-party (ESWC 2023, SINTEF):** RDFox 5.4 imported Wikidata `latest-all.nt.gz`
  (16.3 B triples, 112 GB gzip) in **6 h 25 m** (~706 K triples/s from gzip, derived)
  on an **x1.32xlarge (128 vCPU, 1,952 GB RAM)**; fastest of all engines tested; gzip
  ingested directly. Peak RAM not reported.
  <https://2023.eswc-conferences.org/wp-content/uploads/2023/05/paper_Lam_2023_Evaluation.pdf>
- **W3C LargeTripleStores wiki (self-reported):** "RDFox also loaded 19.47B triples
  (WatDiv) in 11041s on 64 threads, using 1.5TB of RAM" (~1.76 M triples/s derived).
  <https://www.w3.org/wiki/LargeTripleStores>

Honest comparison caveats: (1) hardware differs by ~64× in cores and ~120× in RAM vs
the ESWC setup — absolute triples/s are not comparable; (2) RDFox numbers are for
full Wikidata/latest-all, ours are truthy prefixes (denser, fewer literal languages
early in the dump); (3) RDFox builds an in-RAM store with reasoning available, sparq
builds on-disk mmap indexes — different artifacts; (4) sparq numbers are single runs.
The defensible stage-1 claim is therefore about *feasibility per GB of RAM*, not speed:
what RDFox's published figures say cannot fit in 16 GB (≳0.36 B triples), measured
below for sparq on 16 GB.

## Wider context — published Wikidata-scale loads by other stores

Same caveat as above: none of these ran on a 2-core/16 GB box; provenance flagged per row.

| Store | Dataset | Triples | Wall | ≈Triples/s | Hardware | Provenance |
|---|---|---|---|---|---|---|
| QLever | Wikidata full (2024 dump) | ~19 B | 4.4 h | ~1.2 M/s | Ryzen 9 5900X (16c), 128 GB, 7.3 TB NVMe | developer wiki (ad-freiburg/qlever "Using QLever for Wikidata"); independently reproduced ~4.5 h (ESWC 2023) |
| Virtuoso OS | Wikidata full | 9.5 B | ~43 h | ~61 k/s | community machine | third-party (OpenLink forum); ESWC 2023 "<1 day" |
| Jena TDB2 xloader | Wikidata **truthy** (2021-12) | 6.6 B | ~40 h | ~46 k/s | dev machine | developer-reported (jena dev list, WMF T299460) |
| Blazegraph (WDQS) | Wikidata full | 19.8 B | 5.2 days | ~44 k/s | 6 CPU, 64 GB, NVMe | third-party (Wikimedia tech blog 2025-04-08) |
| Oxigraph | Wikidata full | ~13–17 B | ">1 week" | <20 k/s | n/a | author blog (Pellissier-Tanon, 2023-01-15) |

Same-hardware cross-store reference (DBLP 390 M triples, Ryzen 9 7950X/128 GB/NVMe, QLever
team's measurements): QLever 231 s (~1.7 M/s), Virtuoso 561 s, Oxigraph 640 s, Stardog 724 s,
GraphDB 1,066 s, Jena 2,392 s, Blazegraph 6,326 s.
<https://github.com/ad-freiburg/qlever/wiki/QLever-performance-evaluation-and-comparison-to-other-SPARQL-engines>

## Cost ledger

| item | quantity | rate | cost |
|---|---|---|---|
| r8g.large on-demand | 3.90 h (launch 2026-06-12T18:53:01Z → terminate 22:47Z) | $0.13838/h | $0.540 |
| gp3 350 GB | 3.90 h | $0.0928/GB-mo (÷730 h) | $0.174 |
| gp3 IOPS (4,000; 1,000 over baseline) | 3.90 h | $0.0058/IOPS-mo | $0.031 |
| gp3 throughput (250 MB/s; 125 over baseline) | 3.90 h | $0.0464/MBps-mo | $0.031 |
| **Total** | | | **≈ $0.78** |

Data transfer: 16 GiB ingress (free) + a few MB egress (negligible). Well under the
$3 stage target.

Rates (AWS price list bulk API, retrieved 2026-06-12): r8g.large on-demand eu-west-2
**$0.13838/h**; gp3 **$0.0928/GB-month**; gp3 provisioned IOPS above 3,000 baseline
$0.0058/IOPS-month (this volume: 4,000 IOPS), throughput above 125 MB/s $0.0464/MBps-month
(this volume: 250 MB/s).

## Stage 2 (full truthy, ~8–9.4 B) — what it needs

Based on the measured 1B run (all extrapolations linear from 89.6 B/triple on-disk and
the 35.0 GiB @1B total memory footprint; flagged as estimates):

- **Disk:** index ~717–842 GB at 8–9.4 B triples, plus ~60–70 GB compressed input and
  extsort temp runs — budget ≥1.2 TB gp3. Feasible, cheap.
- **Memory footprint:** ~280–330 GiB dict-driven footprint extrapolated — that is ~18–21×
  this box's RAM and >4× the 64 GiB swapfile used here. On a 16 GB box this is **not a
  "bigger swapfile" problem**: at 1B (≈2.2× RAM) throughput already fell 4.8× with CPU at
  30%; at ~20× RAM the build would be effectively all-swap.
- **Verdict: dictionary spill-to-disk work is a prerequisite** for full-truthy on
  low-resource hardware. Without it, stage 2 honestly requires a ≥384 GB-RAM machine
  (different claim, different cost class). The triple-sorting side needs no changes —
  it stayed bounded throughout.
- **Wall-time floor (if dict were spilled):** at the 300M rate (0.58 M/s) 9.4 B is ~4.5 h;
  at the swap-degraded 1B rate (0.213 M/s) ~12.3 h — the dead-man timer and EBS burst
  windows must be sized accordingly.
