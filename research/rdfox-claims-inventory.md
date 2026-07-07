# RDFox published-claims inventory

<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable
returns). Bead sq-hv0w3. This is an INVENTORY of third-party (RDFox / Oxford Semantic
Technologies) published performance claims, recorded VERBATIM with citations. It makes NO
comparison to sparq and asserts NO sparq number — the measured gap analysis is sq-mk6wx's
job on measured data. -->

**Status:** research record / claims inventory (data-gathering only). **Date:** 2026-07-07.
**Feeds:** the sparq performance-dominance gap table (standing maintainer mandate: sparq must
beat every open-source engine by order(s) of magnitude AND beat RDFox's published claims on
every axis). **Companion:** `research/competitor-benchmark-landscape.md` (which frames RDFox
as a *kernel reference*, not a SPARQL-surface peer), `bench/benchmarks.toml` (the sparq
instruments named below).

## 0. Honesty scope and caveats

- Every quantitative claim below is **RDFox's own or a third party's**, quoted verbatim and
  cited (URL + date). No sparq figure appears; **no** "sparq beats X" editorialising appears
  (that is sq-mk6wx). This satisfies the AGENTS.md "no fabricated numbers" rule the same way
  `competitor-benchmark-landscape.md` does — third-party numbers attributed to a source.
- **Provenance tiers.** `[primary-verbatim]` = quoted from text I extracted directly from the
  source at the accessed date. `[reported]` = a figure attributed to the ISWC 2015 paper by
  secondary indexing whose primary PDF was paywalled (Springer) or TLS-broken
  (`iswc2015.semanticweb.org` cert) at fetch time — recorded but **not** verbatim-confirmed
  here; re-verify against the paper before citing it outward.
- **Hardware is load-bearing.** RDFox's headline academic numbers were measured on an Oracle
  **SPARC T5-8 (128 physical cores, 1024 hardware threads, 4 TB RAM)** — an extreme box.
  Cross-ISA per-core normalisation (SPARC V9 8-way-SMT cores vs x86) is only *indicative*; a
  raw absolute-vs-absolute comparison silently rewards the bigger machine. Marketing-page and
  Wikidata-blog claims state **no hardware at all** and are flagged non-comparable.

## 1. Sources

| tag | source | date | URL |
|---|---|---|---|
| `[iswc2015-paper]` | Nenov, Piro, Motik, Horrocks, Wu, Banerjee — *RDFox: A Highly-Scalable RDF Store*, ISWC 2015 | 2015 | <https://dblp.uni-trier.de/rec/conf/semweb/NenovPMHWB15.html> / <https://link.springer.com/chapter/10.1007/978-3-319-25010-6_1> / <https://ora.ox.ac.uk/objects/uuid:2a08b023-77be-431a-a08c-89b47381586a> |
| `[iswc2015-slides]` | Nenov et al. — authors' ISWC 2015 presentation slides (Bethlehem, PA) | 2015 (accessed 2026-07-07) | <https://pdfs.semanticscholar.org/88f2/ac0f0a15ad2340985cfdba14c589e53c51fc.pdf> |
| `[aaai2014]` | Motik, Nenov, Piro, Horrocks, Olteanu — *Parallel Materialisation of Datalog Programs in Centralised, Main-Memory RDF Systems*, AAAI 2014, pp. 129–137 | 2014 (accessed 2026-07-07) | <https://www.cs.ox.ac.uk/people/boris.motik/pubs/mnpho14parallel-materialisation-RDFox.pdf> / <https://ora.ox.ac.uk/objects/uuid:5800d74f-b9f6-4b12-9908-a45be70b29d2> |
| `[w3c-lts]` | W3C Wiki — *LargeTripleStores* (RDFox entry, submitted by the RDFox team) | accessed 2026-07-07 | <https://www.w3.org/wiki/LargeTripleStores> |
| `[ost-rdfox]` | Oxford Semantic Technologies — RDFox product page | accessed 2026-07-07 | <https://www.oxfordsemantic.tech/rdfox> |
| `[ost-docs-feat]` | RDFox documentation — *Features and Requirements* | accessed 2026-07-07 | <https://docs.oxfordsemantic.tech/features-and-requirements.html> |
| `[ost-wikidata]` | Mulford — *Enhancing Wikidata Performance with RDFox* (OST blog) | published 2021-08-27, updated 2026-01-19 | <https://www.oxfordsemantic.tech/blog/enhancing-wikidata-performance-with-rdfox-how-to-dissect-the-worlds-leading-rdf-database-faster> |

### Reference hardware (named once, cited per claim below)

- **SPARC T5-8** `[iswc2015-slides]`: verbatim "Oracle SPARC T5-8", "4 TB of RAM", "8 SPARC V9
  processors at 3.6 GHz", "128 physical threads and 1024 virtual threads" (i.e. 128 cores /
  1024 hardware threads).
- **Dell 2× Xeon E5-2650** `[aaai2014]`: verbatim "a Dell computer with 128 GB of RAM … two
  Xeon E5-2650 processors with 16 physical cores, extended to 32 virtual cores via
  hyperthreading"; "RDFox was allowed to use at most 100 GB of RAM". (Also the box for the
  `[w3c-lts]` LUBM-5k point.)
- **"64 threads / 1.5 TB" box** `[w3c-lts]`: CPU model unstated (partial hardware).
- **Wikidata blog / product page**: hardware **unstated**.

## 2. Claims inventory

Columns: **dimension | verbatim claim | hardware | source (date) | normalised | comparability caveat**.
Pipe-free verbatim quotes; `t/s` = triples/second as written by the source.

### 2.1 Bulk import / parsing rate

| verbatim claim | hardware | source (date) | normalised | caveat |
|---|---|---|---|---|
| "RDFox also loaded 19.47B triples (WatDiv benchmark) in 11041s on 64 threads, using 1.5TB of RAM" | 64 threads, 1.5 TB RAM (CPU unstated) | `[w3c-lts]` (2026-07-07) | 19.47e9 / 11041 s ≈ **1.76 M t/s** aggregate ≈ **27.5 k t/s per thread** | partial hardware (no CPU model / ISA); "threads" not "cores"; WatDiv load, not parse-only |
| "importation rates of up to 1 million triples per second" | SPARC T5-8 (128 cores, 4 TB) | `[iswc2015-paper]` (2015) — `[reported]` | 1e6 / 128 cores ≈ **7.8 k t/s per core** | `[reported]`, not verbatim-confirmed; SPARC cores ≠ x86; parse+intern+index bundled |
| "the initial load took us only 2 hours and 50 minutes for the entire 15 billion triples" | **unstated** | `[ost-wikidata]` (2021-08-27) | 15e9 / 10 200 s ≈ **1.47 M t/s** aggregate; per-core **N/A** | hardware unstated → **non-comparable** |
| "the initial load of Wikidata can now be completed in 24 minutes and 8 seconds" (RDFox v7.5) | **unstated** | `[ost-wikidata]` (updated 2026-01-19) | 15e9 / 1 448 s ≈ **10.36 M t/s** aggregate; per-core **N/A** | hardware unstated → **non-comparable**; version v7.5 |
| "went from a 3 day backload of our graph database to it taking 20 minutes" (customer quote) | **unstated** | `[ost-rdfox]` (2026-07-07) | — | dataset + hardware unstated → **non-comparable** |

### 2.2 Materialisation / reasoning rate (datalog / OWL 2 RL)

| verbatim claim | hardware | source (date) | normalised | caveat |
|---|---|---|---|---|
| "Max Rate 6.1M t/s" (LUBM, materialising "6.7G —> 9.3G" triples) — parallelisation-scalability table | SPARC T5-8 (128 cores / 1024 threads, 4 TB) | `[iswc2015-slides]` (2015) | 6.1e6 / 128 cores ≈ **47.7 k t/s per core** (≈ 6.0 k t/s per HW thread) | SPARC 8-way-SMT core ≠ x86 core; LUBM datalog closure |
| "Max Rate 4.2M t/s" (Claros, "19M —> 539M"); "Max Rate 4.0M t/s" (DBpedia, "113M —> 1.5G") | SPARC T5-8 | `[iswc2015-slides]` (2015) | 4.2e6 / 128 ≈ **32.8 k**; 4.0e6 / 128 ≈ **31.3 k t/s per core** | as above; real-world (Claros/DBpedia) rule sets |
| "reasoning speeds of up to 60M t/s" (conclusion slide) | SPARC T5-8 | `[iswc2015-slides]` (2015) | 60e6 / 128 ≈ **469 k t/s per core** | headline figure; the deck's per-workload tables show a materialisation Max Rate of 6.1 M t/s on LUBM — treat the exact import-vs-reason split per the ISWC 2015 paper |
| "speedups of up to 213 times using 1024 threads"; DBpedia "87x" at 1024 threads | SPARC T5-8 | `[iswc2015-slides]` (2015) | speedup vs 1 core; per-core efficiency 213/1024 ≈ **0.21** | speedup, not a rate; SMT threads inflate the ceiling |
| "materialisation can be up to 13.9 times faster than with just one core … rising up to 19.3 with 32 virtual cores obtained by hyperthreading" | Dell 2× Xeon E5-2650 (16 cores / 32 threads, 128 GB) | `[aaai2014]` (2014) | 13.9 / 16 ≈ **0.87** per-core efficiency | speedup vs 1 core, not a t/s rate |
| "on a computer two Xeon E5-2650 processors with 16 physical cores it materialised LUBM 5k in only 42s" | Dell 2× Xeon E5-2650 (16 cores) | `[w3c-lts]` (2026-07-07) | wall-clock; rate depends on LUBM(5000) closure size | absolute wall-clock, not per-core |
| "RDFox can also process 2-3 million inferences of new facts per second" | **unstated** | `[ost-rdfox]` (2026-07-07) | per-core **N/A** | hardware unstated → **non-comparable**; "inferences", not raw t/s |

### 2.3 Query performance

| verbatim claim | hardware | source (date) | normalised | caveat |
|---|---|---|---|---|
| "RDFox is four times faster than the Wikidata Query Service" (simple query) | **unstated** | `[ost-wikidata]` (2021-08-27) | ratio only | vs a **public shared service**, not same-hardware → **non-comparable** |
| complex query "increases to a factor of almost 20"; layered query "quicker … by an order of magnitude, and nearly 800 times when querying over the subgraph" | **unstated** | `[ost-wikidata]` (2021-08-27) | ratio only | as above → **non-comparable** |
| "loading and query times that can often be 10-1000x faster or more"; "A request time of several minutes … is often solved in less than one second" | **unstated** | `[ost-rdfox]` (2026-07-07) | ratio only | marketing; no benchmark named → **non-comparable** |
| Benchmarks RDFox *cites* for query/materialisation: **LUBM**, **WatDiv**, **UOBM**, **Claros**, **DBpedia** | (see §2.1–2.2) | `[iswc2015-slides]` / `[aaai2014]` / `[w3c-lts]` | — | RDFox publishes materialisation numbers on these; it names **no standard query-latency benchmark** on its product pages |

### 2.4 Memory footprint (bytes / triple)

| verbatim claim | hardware / mode | source (date) | normalised | caveat |
|---|---|---|---|---|
| "stores 1.5 G triples in 52 GB of RAM" (mid-range servers, previous work) | mid-range, in-memory | `[iswc2015-slides]` (2015) | 52e9 / 1.5e9 = **34.7 bytes/triple** | in-memory incl. dictionary; workload-dependent |
| "memory usage as low as 36.9 bytes per triple" | SPARC T5-8, in-memory | `[iswc2015-paper]` (2015) — `[reported]` | **36.9 bytes/triple** | `[reported]`, not verbatim-confirmed |
| "can store between 1 and 1.5 billion triples in 50 GB" | in-memory | `[w3c-lts]` (2026-07-07) | 50e9 / (1.0–1.5e9) = **33.3–50 bytes/triple** | range; incl. indexes/dictionary |
| "we need at most 80 bytes per triple; this drops to 46 bytes for p = 4 (but then we can store at most 2^32 triples)" (analytical worst case of the index scheme) | in-memory, analytical | `[aaai2014]` (2014) | **46–80 bytes/triple** (worst case) | analytical upper bound of the 3-index layout, not a measurement |
| "Fact storage costs typically vary between 45 and 85 bytes per fact" (+ "10-100% of this for operating memory costs") | current product, in-memory | `[ost-docs-feat]` (2026-07-07) | **45–85 bytes/fact** base, +10–100% operating | excludes query working memory; workload-dependent |
| "40-60 bytes of disk space per triple" | current product, on-disk | `[ost-docs-feat]` (2026-07-07) | **40–60 bytes/triple** (disk) | on-disk persistence, a different axis than in-memory |

### 2.5 SHACL / validation and update (incremental) claims

| dimension | verbatim / status | hardware | source (date) | caveat |
|---|---|---|---|---|
| SHACL validation throughput | **No public numeric claim found.** RDFox exposes SHACL via the `rdfox:SHACL` built-in table but publishes no validation-rate figure. | — | `[ost-docs-feat]` / release notes (2026-07-07) | search returned no RDFox SHACL benchmark number |
| Incremental reasoning (materialisation maintenance) | "efficient incremental reasoning for small and medium sized updates (with and without native equality reasoning)"; algorithm = FBF / DRed (AAAI 2015, IJCAI 2015) | SPARC T5-8 / Xeon (per paper) | `[iswc2015-slides]` (2015) | qualitative — **no single headline add/delete-rate (facts/sec)** is published; the FBF/IJCAI papers give per-dataset update *times*, not a rate |

## 3. Architecture context (why the numbers look like they do)

RDFox is an **in-memory, centralised, parallel datalog engine**. The headline rates come from
three design choices, each of which explains a column above: (a) a **RAM-resident hash-indexed
triple table** with three permutation indexes (`Ispo`, `Isp`, `Iop`) plus 'mostly' lock-free
insertion — this is the source of the 34–85 bytes/triple figures and of parallel-import
scaling `[aaai2014]`; (b) a **workload-distributing parallel materialisation** algorithm that
hands rule instantiations to cores dynamically — the 6.1 M t/s / 13.9×–213× speedup numbers
`[aaai2014]`/`[iswc2015-slides]`; (c) **incremental maintenance (FBF/DRed)** so small updates
don't re-materialise from scratch. The extreme absolutes (60 M t/s, 9.3–21 G triples) are a
**SPARC T5-8 (128 cores / 1024 threads / 4 TB)** artefact — the algorithm's per-core
efficiency (§2.2) is the transferable quantity; the absolute is a big-machine result.

## 4. Mapping each dimension to the sparq instrument that measures the comparable number

Instrument ids are rows in `bench/benchmarks.toml` (human guide `bench/CATALOG.md`). This
section maps *what to measure against each RDFox claim*; it records **no sparq value**.

| RDFox dimension (§) | comparable sparq instrument(s) — `bench/benchmarks.toml` id | notes |
|---|---|---|
| Bulk import / parse rate (§2.1) | `cli-ingest` (M t/s: parse \| intern \| full), `parse-baseline` (MB/s decode+parse), `dict-baseline` (dict-build t/s), `cli-scaling` (parallel scaling sweep), `wikidata-8b` (external build wall) | present, but tuned/reported for the **low-resource external-memory** build (the "Wikidata on a 16 GB box" claim) — a *different axis* from RDFox's 4 TB / 128-core in-memory parallel import (see gap G3) |
| Materialisation / reasoning rate (§2.2) | `lubm` (OWL-RL closure time + expected-rows), `deep-taxonomy`, `owl-sameas`, `reason-el-classify`, `reason-ql-rewrite`, `inference-eye-comparison` (wall closure vs EYE), `inference-owl-bench` | emit `closure_s` + `closure_triples` (a t/s rate is derivable) but are **CI-scale + correctness-first** (LUBM(1); depths 1k–10k) — no billions-scale reasoning-*throughput* number (see gap G2) |
| Query performance (§2.3) | `lubm`, `watdiv`, `bsbm`, `sp2b`, `dbpsb`, `operators`, `qlever-olympics`, `qlever-synthetic-10m/100m`, `sparq-bench-compare` (vs Oxigraph) | strong coverage on standard suites; RDFox itself publishes only WQS-relative marketing here, so the honest sparq target on this axis is **QLever/Oxigraph**, not an RDFox number |
| Memory footprint bytes/triple (§2.4) | `cli-probe-compress` (B/triple raw vs delta+LEB128 vs gzip), `cli-compare-compress` (in-RAM footprint B/triple), `dict-baseline` (dict bytes/term), `wikidata-8b` (on-disk index bytes), `sparq-bench-compare` / `cli-bench-mmap` (peak RSS) | covered — sparq has both in-RAM and on-disk B/triple instruments |
| SHACL / validation (§2.5) | `shacl` (per-workload validate time; W3C core 98/98 ratchet) | sparq instrument exists; RDFox publishes no numeric SHACL claim to target |
| Concurrent query throughput (loosely, §2.3 "minutes → <1s") | `serve-throughput`, `memtier` | maps loosely to RDFox's latency marketing; no RDFox QPS number published |

## 5. Dimensions where sparq has NO instrument yet

Listed as **proposed beads** — recorded here only; **not created** (per bead sq-hv0w3
instructions). Each is the direct analogue of an RDFox headline with no sparq counterpart.

- **G1 — Incremental reasoning / materialisation-maintenance rate.** RDFox's differentiator
  (FBF/DRed incremental maintenance, §2.5) has **no sparq counterpart at all**: sparq reasoning
  is batch closure (`sparq-cli reason`), with no add/delete-a-fact → re-derive-the-delta path
  and no benchmark for it. This is a **feature gap first, instrument gap second**. Proposed
  bead: add an incremental-maintenance capability to `sparq-reason` + an
  add-rate/delete-rate (facts/sec) bench suite.
- **G2 — Billions-scale in-memory materialisation *throughput* (t/s) on a big-core box.** The
  direct analogue of RDFox's 6.1 M t/s / LUBM / SPARC T5-8. sparq's reasoning suites are
  CI-scale and correctness-gated; there is no 1 B+-output OWL-RL materialisation-*rate* number
  on a comparable large-memory instance. Proposed bead: an EC2 large-scale OWL-RL
  materialisation-rate suite (LUBM(50k)-class) emitting `triples/sec` + per-core.
- **G3 — Big-core in-memory parallel bulk-import *rate* (t/s) at RDFox-class hardware.**
  `cli-ingest` emits M t/s, but sparq's pinned ingest story is the low-resource external build;
  there is no pinned **in-memory** parallel-import-rate number on a 100+ core / TB-RAM instance
  for an apples-to-apples read against RDFox's 1 M t/s import / 1.76 M t/s WatDiv load. Proposed
  bead: a high-core-count in-memory ingest-rate reference run **or** an explicit gap-table
  caveat that sparq normalises per-core while RDFox's absolute is a 128-core/4 TB result.
- **G4 (lower priority) — Wikidata-scale query-latency-vs-hosted-service panel.** RDFox's
  "4× / 20× / 800× vs WQS" (§2.3). sparq has `wikidata-8b` build/validation-COUNT but no
  query-latency comparison against a hosted Wikidata endpoint. Proposed bead: a Wikidata-scale
  query-latency panel (noting the same hardware-unstated non-comparability RDFox's own claim
  carries).

## 6. One-line handoff to the gap analysis (sq-mk6wx)

The transferable RDFox targets, per axis, are: **import** ≈ 1.76 M t/s measured / ~27.5 k t/s
per thread (WatDiv, `[w3c-lts]`) with 1 M t/s `[reported]`; **reasoning** 6.1 M t/s / ~47.7 k
t/s per core (LUBM, SPARC T5-8, `[iswc2015-slides]`) with a 60 M t/s headline; **memory**
34.7–36.9 bytes/triple in-memory (45–85 bytes/fact current product); **query** only
WQS-relative marketing (no standard-benchmark number). Absolute headlines (60 M t/s, 21 G
triples) are SPARC T5-8 artefacts — compare per-core, and flag the hardware gap. sq-mk6wx owns
the measured comparison; this record owns only the claims.
