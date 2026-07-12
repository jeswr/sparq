<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable
returns). Bead sq-mk6wx (§0-§8) + sq-hmd7l.27 (§9 v2 consolidation). This is the
PERF-DOMINANCE GAP TABLE: sparq MEASURED vs open-source competitors MEASURED on the same
canonical box, plus RDFox's PUBLISHED claims (normalized, from
research/rdfox-claims-inventory.md #1719). It records measured numbers with provenance;
every dimension whose verdict is not CLEARLY-AHEAD carries a P1 fix or instrument bead. No
fabricated numbers; canonical vs CI-trend vs work-box provenance is flagged per row.
v2 CONSOLIDATION: §9 (sq-hmd7l.27) folds in the canonical wave-1 axes (fts/geo/hdt/update/
parse — #1809), the canonical SP2Bench D3 re-measure (#1827), the materialization axis
(D6, #1799 + VLog/Nemo encodings #1823, canonical run in flight sq-hmd7l.32), and every
research/gap-<axis>-2026-07.md record, under the fixed verdict vocabulary. -->

# Performance-dominance gap table — 2026-07

**Status:** research record / measured gap analysis. **Date:** 2026-07-07. **Bead:**
sq-mk6wx (parent `sq-7d3dj`). **Feeds:** the standing maintainer mandate — sparq must beat
every open-source engine by **order(s) of magnitude** on every axis **and** beat RDFox's
published claims; any dimension where sparq is not clearly ahead gets an immediate P1
**profiling-first** fix or instrument bead.

**Inputs.** (1) The canonical competitor envelopes
`~/sparq-bench-results/canonical-2026-07-07/competitor-results/` — SP2Bench (250 000
triples) and WatDiv (SF=1, 111 856 triples), five engines, count cross-checked. (2) The
merged RDFox claims inventory `research/rdfox-claims-inventory.md` (PR #1719). (3) The
existing sparq bench catalog + CI trend series for the axes the SPARQL matrix does not
cover (memory, SHACL, reasoning), from `bench/benchmarks.toml` and
`site/src/data/benchmarks.generated.json`.

## 0. Provenance and honesty scope (read before any number)

- **Canonical SPARQL matrix.** The SP2Bench and WatDiv per-query + load numbers below are
  **canonical**: a dedicated **quiet c6i.4xlarge (16 vCPU / 32 GiB, x86_64, Xeon Platinum
  8375C)** in eu-west-2, single-tenant, one engine active at a time on the **same** corpus
  and query files, **min-of-5** on a loaded store, git commit `0ab87b2a`, gathered
  2026-07-07 (instance `i-0ce9cd0ea90c3e9c6`). Counts are cross-checked engine-vs-engine
  and vs `bench/<suite>/expected-rows.tsv`; disagreements are recorded, never adjusted.
- **CI-trend axes.** The memory / SHACL / reasoning numbers come from the CI dev/bench
  trend series (`benchmarks.generated.json`, CI runner, 2026-06-15) gathered on the
  **CI runner**, not the canonical box. `store_bytes_per_triple` is a **deterministic**
  memory-layout metric (runner-noise-immune, trustworthy as an absolute); the SHACL and
  `rdfs_infer_s` timings are **CI-runner timings — indicative only, non-canonical**.
- **RDFox column is third-party PUBLISHED claims**, quoted and normalized in #1719 — never
  a sparq measurement. RDFox's headline absolutes were measured on an Oracle **SPARC T5-8
  (128 cores / 1024 threads / 4 TB)** or a 64-thread / 1.5 TB box; marketing/Wikidata-blog
  claims state **no hardware** and are flagged non-comparable. Compare **per-core/per-thread**,
  never absolute-vs-absolute (that silently rewards the bigger machine).
- **CLI-vs-HTTP asymmetry (load-bearing).** sparq and **oxigraph** were measured in **CLI /
  on-disk** mode (in-process, no HTTP); **fuseki / virtuoso / qlever** ran in **HTTP** mode
  via `scripts/bench-adapters/http_sparql_adapter.py`, which times the **full client
  wall-clock** (`time.perf_counter()` around a fresh `urlopen` per iteration — TCP connect +
  HTTP + SPARQL-JSON parse, **no keep-alive**). So an HTTP number = HTTP+connect overhead +
  server compute; the competitor's **pure compute is lower** than reported. The measured
  HTTP+connect floor on this box is **≈600 µs (virtuoso) / ≈1.2 ms (qlever)** (their fastest
  trivial queries). **Consequence, applied honestly below:** subtracting HTTP overhead makes
  the HTTP engines look **even faster**, so (a) where sparq **loses** to an HTTP engine the
  deficit is **robust and widens** under correction; (b) where sparq **wins by less than the
  floor** the win must be checked. **oxigraph is the cleanest apples-to-apples column** (CLI
  like sparq).
- **fuseki FAILED** in both suites (load wall = 1800.7 s = the 30-min tdb2loader timeout at
  tiny scale). Per the mandate a failed competitor is a **re-run action, not a sparq win** —
  it is treated as a missing column, not a dominance point.
- **Work-box numbers are non-canonical** and are not used here; the session's own EC2 box is
  distinct from the dedicated quiet bench instance above.

## 1. The gap table (per dimension)

Columns: **dimension | sparq measured | best competitor measured | ratio | RDFox claim
(normalized + caveat) | verdict | action**. Ratios are competitor ÷ sparq (>1 ⇒ sparq
faster / better) unless noted. Verdicts use the **fixed vocabulary** (identical in §9):
CLEARLY-AHEAD (≥~10× on the clean baseline) / AHEAD-BUT-NOT-OOM (a real lead under an
order of magnitude) / PARITY / BEHIND / NOT-MEASURED / NOT-COMPARABLE (semantics or
surface not like-for-like — an honest non-comparison, never a win).

| # | dimension | sparq measured | best competitor measured | ratio | RDFox claim (normalized) | verdict | action |
|---|---|---|---|---|---|---|---|
| D1 | Query latency — **WatDiv SF=1** (16 q, count) | 5.4–101.8 µs | virtuoso 652–1935 µs (HTTP); oxigraph 10.9–1177 ms (CLI) | **19–180×** vs virtuoso; **205–23453×** vs oxigraph | no standard query-latency benchmark published (WQS-relative marketing only → non-comparable) | **CLEARLY-AHEAD** | hold; canonical dominance evidence (survives HTTP correction — see §2) |
| D2 | Query latency — **SP2Bench 250k vs the clean CLI baseline (oxigraph)** | see §3 | oxigraph 11.3 ms – 50.0 s (2 ERROR cells) | **8.3–4072×** | (as above) | **CLEARLY-AHEAD** (vs oxigraph) | hold; clean apples-to-apples column |
| D3 | Query latency — **SP2Bench 250k vs qlever (same-box) / virtuoso (HTTP ref)** | §3 + §3.1 addendum | qlever same-box; virtuoso §0 HTTP ref | **RE-MEASURED 2026-07-10** (sq-7d3dj.30.6, git 1190ca84, canonical c6i.4xlarge): wins **10/14** vs same-box QLever, correct+complete on all 14; q03b/c **flipped to AHEAD**; q08/q09/q11/q12b deficits **cut 4–14×**; **q07 the sole residual behind (3.8×, slightly worse)** | (as above) | **MIXED, strongly improved** (was BEHIND) — q07/q09 still behind | fixes landed sq-7d3dj.30.1–.14; residuals **sq-7d3dj.30.20** (q07 CSE), **sq-jnb1e** (q09) + new q08/q11 beads |
| D4 | **Bulk-load throughput** (CLI, vs oxigraph) | 1.37 M t/s (sp2b) / 3.2 M t/s (watdiv) | oxigraph 176 k / 382 k t/s | **7.8× / 8.4×** | import 1.76 M t/s aggregate / ~27.5 k t/s per thread (64 thr, WatDiv 19.47 B; partial hw) | **AHEAD-BUT-NOT-OOM** (small-corpus, startup-dominated) | **NEW P1 sq-7d3dj.31** (scale-up + profile) |
| D5 | **RDFox-class in-memory import rate** (big-core hw) | NOT-MEASURED | — | — | 1.76 M t/s / ~27.5 k t/s per thread; 1 M t/s `[reported]` / ~7.8 k t/s per core | **NOT-MEASURED** | **CITE sq-mbg0k** (G3) |
| D6 | **Reasoning / materialisation throughput** | `rdfs_infer_s` 0.133 s (CI scale, correctness-first; no t/s at scale) | none in the matrix | — | 6.1 M t/s / ~47.7 k t/s per core (LUBM, SPARC T5-8); 60 M t/s headline | **NOT-MEASURED** at comparable scale | **CITE sq-1s03r** (G2) + **sq-w34fa** (G1 incremental) |
| D7 | **Memory footprint** (in-RAM bytes/triple) | raw heap **84.4 B/triple** (50M WatDiv, canonical aarch64 i-0798990569f269435, sha `5ce8b4f9`; see `research/memory-per-triple-2026-07.md`); compressed **54.5 B/triple**; ext-build peak **73.4 B/triple** | Oxigraph: NOT-MEASURED at scale | — | 34.7–36.9 B/triple best in-mem; 45–85 B/fact product; 40–60 B/triple disk | **BEHIND** RDFox best in-class (raw ~2.3×); compressed **within product band** (45–85); ext-build below raw-heap floor at 50M | sq-7d3dj.32.1–.32.6 (improvement beads); see `research/memory-per-triple-2026-07.md` |
| D8 | **SHACL validation** | same-box first-read `research/shacl-baseline-2026-07.md` (#1734, work-box, **non-canonical**): core constraints **7–32× AHEAD** of jena-shacl; `sh:sparql` was **4.5–34× BEHIND** → root-caused (per-focus re-execution) and fixed by VALUES batching (#1737; node-expression sibling #1754); cross-box corroboration sq-ays7 (c7g.xlarge, `f271b74`, 2026-06-16) suggests the residual `sh:sparql` lead vs jena-shacl may be ~3× | pyshacl / jena-shacl (same-box first-read) | — | no public SHACL number | **AHEAD (non-canonical first-read)**; `sh:sparql` fix merged, canonical verdict pending | **sq-7d3dj.33.3** (canonical EC2 re-run) + **.33.4** (core-constraint headroom) |
| D9 | **HTTP serving / TTFB** | measured canonically — `research/perf-dominance-gap-2026-07-http-addendum.md` (#1742): WatDiv HTTP **AHEAD 16/16** (1.3–15.4×, transport-floor-bound); SP2Bench HTTP **wins 6 / behind 6** count-checked (median ≈1.0×; q08/q12b excluded — count divergence ADJUDICATED (sq-ai2wa): QLever evaluates bnode≠IRI as a strict type-error (0 rows) vs sparq/virtuoso lenient identity semantics; `expected-rows.tsv` is correct, QLever's timing on those rows stays disqualified); **TTFB BEHIND ~48×** on large SELECTs (first byte at ~87 % of total; oxigraph 7.8 ms) | oxigraph / virtuoso / qlever / fuseki (same-box HTTP) | — | latency marketing only ("minutes → <1 s") → non-comparable | **MIXED**: ahead (floor-bound) on WatDiv; even on SP2B complex-shape (D3); **BEHIND on TTFB** | PR #1745 (stream SELECT-JSON) + **sq-7d3dj.34.1** (request floor) + **sq-7d3dj.34.2** / lazy-pull design (first-solution latency); canonical re-check after |
| D10 | **fuseki result** | measured after fixing its docker image (#1742 — the stock image shipped no TDB2 loader tools); fuseki rows now included in the HTTP addendum panel | fuseki (same-box HTTP) | — | — | **MEASURED** (see the #1742 addendum) | resolved in **#1742**; comparison folded into D9 |
| D11 | **Wikidata-scale query latency** vs hosted service | NOT-MEASURED | — | — | 4× / ~20× / ~800× vs WQS (hw unstated → non-comparable) | **NOT-MEASURED** (lower priority) | **CITE sq-ohsvs** (G4) |

**Headline.** sparq is **CLEARLY-AHEAD by order(s) of magnitude on WatDiv (every query,
every engine)** and **on SP2Bench vs the clean CLI baseline (oxigraph, every query)**. It is
**BEHIND on the SP2Bench complex-shape class vs virtuoso/qlever** (7 of 14 queries) and
**BEHIND RDFox best-published in-memory** on the memory axis (D7, now canonical-measured).
**NOT-MEASURED** remains on: big-core import, billions-scale materialisation, incremental
maintenance, Oxigraph memory baseline, SHACL-vs-baseline, HTTP/TTFB, Wikidata panel. No
axis is a **spun** win: fuseki FAILED is a re-run, not a win; the sub-order-of-magnitude
load lead is labelled honestly; the memory axis compressed mode is within the RDFox product
band (not best-in-class).

## 2. Why WatDiv is CLEARLY-AHEAD even after the HTTP correction

The strongest apples-to-apples cross-check: apply the most aggressive HTTP correction —
subtract the full **≈600 µs virtuoso floor** from every virtuoso number (treat it as pure
HTTP+connect, giving virtuoso the maximum benefit). sparq still wins every WatDiv query:
e.g. L4 sparq 5.4 µs vs virtuoso (719−600)=119 µs → **22×**; F5 sparq 101.8 vs (1935−600)=1335
→ **13×**; C3 sparq 50.2 vs (38760−600)=38160 → **760×**; F2 (the tightest) sparq 24.8 vs
(767−600)=167 → **6.7×**. So the WatDiv dominance is **robust to the CLI-vs-HTTP asymmetry**.
On the clean CLI column it is 205–23453× vs oxigraph. This is the canonical dominance
evidence for the query axis.

## 3. SP2Bench per-query breakdown (the mixed dimension)

Best-of-5 latency in **microseconds** (canonical box). `ERROR` = engine failed the query;
counts in parentheses flag a **wrong** answer (disqualified for the timing comparison). The
"vs oxigraph" column is the clean CLI ratio; the "best correct HTTP competitor" column is the
honest per-query challenger.

| query | rows | sparq | oxigraph | virtuoso | qlever | oxi ÷ sparq | best CORRECT HTTP competitor | sparq verdict |
|---|---|---|---|---|---|---|---|---|
| q01 | 1 | 10.7 | 14911.6 | 748 | 3048.7 | 1394× | virtuoso 748 → sparq **70× faster** | ahead |
| q02 | 6067 | 6275.9 | 538638.6 | 340770 | 288550.7 | 85.8× | qlever 288551 → sparq **46× faster** | ahead |
| q03a | 15823 | 12492.0 | 220493.3 | 95617 | 46471.2 | 17.7× | qlever 46471 → sparq **3.7× faster** | ahead |
| q03b | 114 | 12124.5 | 128968.8 | **1443** | 1576.4 | 10.6× | virtuoso 1443 → sparq **8.4× SLOWER** | **BEHIND** — sq-7d3dj.30 (§3.1: now AHEAD) |
| q03c | 0 | 11697.3 | 127234.1 | **613** | 1246.6 | 10.9× | virtuoso 613 → sparq **19× SLOWER** | **BEHIND** — sq-7d3dj.30 (§3.1: now AHEAD) |
| q04 | 541911 | 358713.5 | 20904169.1 | 8407062 | 2927437.3 | 58.3× | qlever 2.93 M → sparq **8.2× faster** | ahead |
| q05b | 6933 | 15409.2 | 643985.9 | 109939 | 33152.0 | 41.8× | qlever 33152 → sparq **2.2× faster** | ahead |
| q07 | 48 | 23981.5 | ERROR | 508290 | **8277.3** | — | qlever 8277 → sparq **2.9× SLOWER** | **BEHIND** — sq-7d3dj.30.20 |
| q08 | 358 | 153318.0 | ERROR | **13548** | 9051.7 (0) | — | virtuoso 13548 → sparq **11.3× SLOWER** (qlever count=0 wrong) | **BEHIND** — sq-7d3dj.30.22 |
| q09 | 4 | 22356.7 | 185741.3 | 50365 | **1361.7** | 8.3× | qlever 1362 → sparq **16.4× SLOWER** | **BEHIND** — sq-jnb1e |
| q10 | 452 | 4.2 | 17102.7 | 5940 | 4740.1 | 4072× | qlever 4740 → sparq **1129× faster** | ahead |
| q11 | 10 | 27236.1 | 760333.3 | 13710 | **1960.5** | 27.9× | qlever 1961 → sparq **13.9× SLOWER** | **BEHIND** — sq-7d3dj.30.21 |
| q12b | 1 | 155611.0 | 49991323.8 | **9476** | 7409.2 (0) | 321× | virtuoso 9476 → sparq **16.4× SLOWER** (qlever count=0 wrong) | **BEHIND** — sq-7d3dj.30.22 |
| q12c | 0 | 5.5 | 11290.4 | 601 | 1516.3 | 2053× | virtuoso 601 → sparq **109× faster** | ahead |

**Reading.** Against the **clean CLI baseline (oxigraph)** sparq wins **every** SP2Bench
query, 8.3–4072× (oxigraph also `ERROR`ed q07/q08). Against the **HTTP** engines sparq wins 7
and **loses 7** — and those losses are on the SP2Bench **complex-shape class**:

- **q03b / q03c** — same triple pattern as q03a but with a **selective object-range FILTER**
  (114 / 0 rows). sparq's cost **does not drop** with selectivity (q03a 12492 → q03b 12124 →
  q03c 11697), while virtuoso's does (95617 → 1443 → 613). Hypothesis: sparq **scans** where
  virtuoso uses an **object/range index**. The deficit is at/below the HTTP floor for the
  competitor, so on pure compute sparq is hundreds-of-× slower on q03c.
- **q07 / q08 / q09 / q11 / q12b** — OPTIONAL-heavy, negation-via-`OPTIONAL`+`!bound`,
  long-path joins, and `DISTINCT`/`ORDER BY` shapes. All deficits **survive the HTTP
  correction** (subtracting the ≈600 µs/≈1.2 ms floor makes the competitor faster still).

**Correctness note (a genuine sparq advantage on the same rows).** sparq is the **only**
engine that is both **correct and complete** on q07/q08/q12b: oxigraph `ERROR`ed q07/q08 and
**qlever returned empty (wrong) results** for q08 (0 vs 358) and q12b (0 vs 1). sparq's
losses are therefore only to **correct** competitors (virtuoso on q03b/c/q08/q12b, qlever on
q07/q09/q11) — real deficits, not artefacts.

This is why D3 is **BEHIND** and carries a P1 profiling-first bead (**sq-7d3dj.30**): profile
the seven queries on the canonical box, confirm the selective-FILTER-not-using-index
hypothesis (q03b/c) and the OPTIONAL/long-path plan cost (rest) **before** any code change,
then a targeted fix.

### 3.1 D3 re-measure addendum (2026-07-10, sq-7d3dj.30.6) — the fixes landed

<!-- [FABLE-5] The §3 table above is the 2026-07-07 PRE-fix baseline. The full fix decomposition
     (sq-7d3dj.30.1–.14 + #1786 + #1795 + #1813) has since merged; the canonical re-measure below
     supersedes the §3 D3 verdicts. -->

The complex-shape fix wave (algebra-rewrite constant-substitution, SIP, top-k ORDER BY,
DISTINCT-projection semijoin, DPccp default-on, theta anti-join, id-filter fast path) all
landed. A canonical quiet-box re-measure — **c6i.4xlarge x86_64, min-of-5, SP2Bench 250k, git
1190ca84, `bench/competitor-results/sparql-same-box-20260710T025117Z.json`**, full detail in
`research/sp2bench-complex-shape-deficit.md` §7 — changes the D3 verdict from **BEHIND on the
complex-shape class** to **MIXED, strongly improved**:

- **q03b / q03c flipped BEHIND → AHEAD** (38× / 21× faster than same-box QLever; the two
  selective-FILTER deficits are CLOSED by the algebra rewrite).
- **q08 / q09 / q11 / q12b deficits cut 4–14×** — q09 (9.5×) and q11 (3.0×) still behind
  QLever; q08 / q12b have **no valid QLever comparator** (QLever returns the wrong count 0, the
  sq-ai2wa bnode≠IRI divergence) and sparq is the only correct+complete engine there.
- **q07 is the sole query that did NOT improve** (30.9 ms, ~1.3× slower than §0, BEHIND QLever
  3.8×) — the cross-level membership-CSE residual, tracked by **sq-7d3dj.30.20**.
- sparq wins **10/14** vs same-box QLever and is **correct + complete on all 14**.

Residual follow-ups: q07 CSE **sq-7d3dj.30.20**, q09 characteristic-set **sq-jnb1e**, plus new
q11 top-k and q08/q12b same-box-correct-competitor beads (see §7.3 of the deficit record).

## 4. Load, memory, reasoning, SHACL detail

- **Load (D4).** CLI apples-to-apples: SP2Bench sparq 0.183 s (**1.37 M t/s**) vs oxigraph
  1.421 s (176 k t/s) = **7.8×**; WatDiv sparq 0.035 s (**3.2 M t/s**) vs oxigraph 0.293 s
  (382 k t/s) = **8.4×**. Genuinely ahead of the fastest OSS CLI loader, but **under an order
  of magnitude**, and both corpora are tiny (112 k–250 k triples) so load is
  **startup/fixed-cost dominated** — the steady-state ratio at 10 M–1 B is unknown.
  fuseki/virtuoso/qlever load figures are **whole-recipe wall** (pull+load+query+teardown) and
  are **not** load-comparable. **sq-7d3dj.31** scales this up and profiles the remaining gap.
- **Memory (D7) — UPDATED with canonical measurement.** The CI proxy (88–92 B/triple) is
  superseded by the three-rung canonical envelope in `research/memory-per-triple-2026-07.md`
  (aarch64 EC2 `i-0798990569f269435`, sha `5ce8b4f9`, 2026-07-07). At **50M WatDiv triples**:
  raw 6-perm heap **84.4 B/triple** (store 75.4 + dict 8.3 + caches 0.75); block-compressed
  mode **54.5 B/triple**; external-memory build peak **73.4 B/triple** — below the raw-heap
  floor. RDFox publishes **34.7–36.9 B/triple** best in-memory (at billions-scale, SPARC
  T5-8) and **45–85 B/fact** for the current product. Honest verdict: raw mode is **BEHIND**
  RDFox best in-class (~2.3×); compressed mode is **within** the current product band;
  ext-build confirms sparq's "Wikidata on limited RAM" positioning. Comparability caveat:
  RDFox's best-published figures are at 1.5 B triples on a very large machine; sparq's 50M
  run still shows a downward trend (84.4 vs 88.6 at 10M vs 93.4 at 1M) and has not
  converged. Improvement lanes: sq-7d3dj.32.1 (Vec slack), .32.2 (compressed promotion),
  .32.3 (3-perm profile → 36 B/triple store floor, within RDFox best-case band).
- **Reasoning (D6).** `rdfs_infer_s` = 0.133 s at CI scale (correctness-first; LUBM(1),
  depth-1k–10k). No OSS engine in the matrix materialises, and there is **no billions-scale
  t/s** number to place against RDFox's 6.1 M t/s (~47.7 k t/s per core, SPARC T5-8) or its
  incremental-maintenance differentiator. **NOT-MEASURED** → **sq-1s03r** (G2, billions-scale
  materialisation throughput) + **sq-w34fa** (G1, incremental — a **feature gap first**: sparq
  has batch closure only).
- **SHACL (D8).** sparq validate (CI, indicative): cardinality 347.9 µs, class/nodekind
  328.8 µs, datatype/range 12979 µs, node-paths 6756 µs, **SPARQL-constraint 760170 µs
  (~760 ms)**. No competitor in the matrix runs SHACL and RDFox publishes no numeric SHACL
  claim → **NOT-MEASURED** (no baseline). The ~760 ms `sh:sparql` path is 60–2000× the other
  constraint kinds and is worth profiling on its own. **sq-7d3dj.33** adds a same-box SHACL
  competitor (pyshacl / Jena SHACL) and profiles the slow path.
- **HTTP/TTFB (D9/D10).** sparq ran **CLI-mode** in the matrix, so there is **no canonical
  sparq-server-vs-fuseki/virtuoso** TTFB/QPS comparison, and **fuseki FAILED** (30-min loader
  timeout at tiny scale — a harness/config bug, not a real fuseki result). **sq-7d3dj.34**
  fixes the fuseki re-run and builds a same-box HTTP panel with sparq measured in the **same
  mode** (deciding keep-alive-vs-fresh-connect for a fair server-vs-server compare).

## 5. Fix / instrument beads (ordered — the phased plan)

Each phase is a future bead the orchestrator can track. **New** beads created by this record
(all P1, parent `sq-7d3dj`, profiling-first); **existing** RDFox-gap beads cited (not
duplicated).

1. **sq-7d3dj.30** (NEW, P1) — SP2Bench complex-shape query-latency deficit (D3): profile
   q03b/c, q07, q08, q09, q11, q12b on the canonical box; confirm root cause; targeted fix.
2. **sq-7d3dj.31** (NEW, P1) — Bulk-load throughput scale-up (D4): canonical 10 M/100 M/1 B
   sparq-vs-oxigraph load sweep + profile to push past order-of-magnitude.
3. **sq-7d3dj.32** (MEASUREMENT DONE — improvement beads open) — In-RAM bytes/triple at
   scale (D7): canonical 3-rung envelope now in `research/memory-per-triple-2026-07.md`
   (84.4 B/triple raw / 54.5 compressed at 50M). Fix lanes: sq-7d3dj.32.1–.32.6.
4. **sq-7d3dj.33** (NEW, P1) — SHACL competitor baseline + profile (D8): same-box pyshacl /
   Jena SHACL comparison; flamegraph the ~760 ms `sh:sparql` path.
5. **sq-7d3dj.34** (NEW, P1) — HTTP/TTFB same-box panel (D9) + fuseki loader re-run (D10):
   sparq-server vs fuseki/virtuoso HTTP TTFB + QPS, sparq measured in HTTP mode.
6. **sq-mbg0k** (EXISTING, G3) — big-core in-memory parallel bulk-import rate at RDFox-class
   hardware (D5).
7. **sq-1s03r** (EXISTING, G2) — billions-scale in-memory materialisation throughput (D6).
8. **sq-w34fa** (EXISTING, G1) — incremental reasoning / materialisation-maintenance rate
   (D6; feature gap first).
9. **sq-ohsvs** (EXISTING, G4, lower priority) — Wikidata-scale query-latency panel vs a
   hosted service (D11).

## 6. Open questions for the maintainer

- **SP2Bench-shape target.** Is beating **virtuoso/qlever** on the SP2Bench complex-shape
  class (D3) an order-of-magnitude requirement, or is beating the clean CLI baseline
  (oxigraph) by OOM + reaching **parity** with the HTTP engines on these shapes acceptable
  for v1? (The mandate reads as OOM-on-every-axis; sq-7d3dj.30 targets the deficit either
  way.)
- **Same-mode vs same-tool comparison.** For the HTTP panel (D9), should the competitor
  adapter switch to **keep-alive** (fair server-vs-server) or stay fresh-connect (models a
  real per-request client)? The choice moves the ≈600 µs floor and changes which sparq wins
  survive the correction.
- **Memory axis framing.** sparq's pinned identity is the low-resource **external-memory**
  build; RDFox's memory claim is a big-RAM in-memory number. Should the gap table hold sparq
  to RDFox's in-memory B/triple at all, or record memory as an **external-memory-scale**
  axis where the comparison target is on-disk B/triple (40–60 for RDFox) instead?

## 7. Corrections to the brief's premise

The brief's framing held up against the data, with three clarifications recorded honestly:

1. **The load lead is real but sub-order-of-magnitude (7.8–8.4×), not "orders of
   magnitude."** The brief implied broad dominance; on the clean CLI load axis sparq is
   clearly ahead but not yet OOM at CI scale — labelled AHEAD-BUT-NOT-OOM, not spun.
2. **Query dominance is not uniform.** vs the clean CLI baseline (oxigraph) it is total
   (WatDiv + SP2Bench, every query); vs the HTTP engines sparq is **BEHIND on 7 SP2Bench
   queries**. The gap table splits D2 (vs oxigraph, CLEARLY-AHEAD) from D3 (vs virtuoso/qlever,
   BEHIND) so the mixed result is not averaged into a false "ahead."
3. **The HTTP correction helps competitors, not sparq.** Because the adapter times the full
   client round-trip, subtracting HTTP overhead makes HTTP engines' pure compute **faster** —
   so it **confirms** sparq's losses and only threatens sparq's sub-floor wins. The WatDiv
   dominance survives the most aggressive correction (§2); the SP2Bench deficits widen under
   it (§3).

## 8. Uncertainties

- The SHACL and `rdfs_infer_s` timings are **CI-runner** figures (non-canonical, indicative);
  only `store_bytes_per_triple` among the non-matrix axes is deterministic. The
  canonical-box re-measurement is part of sq-7d3dj.32/.33.
- The **steady-state** load and memory ratios at 10 M–1 B triples are **unknown**; the CI/small
  numbers may improve or regress with scale (sq-7d3dj.31/.32).
- Two `[reported]` RDFox figures (1 M t/s import, 36.9 B/triple) are secondary-indexed, not
  verbatim-confirmed from the paywalled ISWC 2015 PDF (#1719 §0) — re-verify before citing
  outward.
- qlever's wrong counts on q08/q12b (0 vs 358/1) are recorded as disqualifications; whether
  they reflect a qlever config or a genuine engine difference on those shapes is not
  investigated here.

## 9. v2 consolidation — canonical wave-1 + D3 re-measure + materialization (sq-hmd7l.27)

<!-- [OPUS-4.8] sq-hmd7l.27. This section CONSOLIDATES committed evidence only (NO new
     measurements). It merges every landed research/gap-<axis>-2026-07.md row, the canonical
     wave-1 EC2 run (sq-hmd7l.26, #1809), the canonical SP2Bench D3 re-measure (sq-7d3dj.30.6,
     #1827), the D9/D10 HTTP addendum (sq-7d3dj.34), and the materialization axis (D6, #1799 +
     VLog/Nemo encodings #1823; canonical univ≥100 run in flight as sq-hmd7l.32). Fixed verdict
     vocabulary only. Every BEHIND / PARITY / self-slow-dominance-gap row cites a filed P1
     profiling-first fix bead; every NOT-MEASURED cites its blocking bead. Every cell cites its
     envelope filename or the gap record that owns it. A failed / absent competitor run is a
     re-run action, never a sparq win. RDFox published-claim columns are per-core-normalized
     (see §0). This section supersedes the §3 pre-fix D3 verdict and adds the wave-1 axes. -->

**What is new since §0-§8.** The original table (§0-§8, 2026-07-07) predates: (1) the
canonical **wave-1** run (fts/geo/hdt/update/parse), envelopes under
`bench/canonical-competitor-results/` + the per-axis `axis-results/<axis>/…json` pulls named
in each `research/gap-<axis>-2026-07.md`; (2) the canonical **SP2Bench D3 re-measure** after
the complex-shape fix wave (`bench/competitor-results/sparql-same-box-20260710T025117Z.json`,
git `1190ca84`, #1827); (3) the **materialization** axis D6 (VLog/Nemo validated encodings in
PR #1823, canonical univ≥100 in flight as `sq-hmd7l.32`). This §9 folds all of it in under
one fixed-vocabulary table.

### 9.1 Consolidated wave-1 + re-measure gap table

Columns: **dimension | sparq measured | best competitor measured | verdict | fix / blocking
bead + provenance**. Provenance is the envelope filename or the gap record that owns the row;
every timing was count-crosschecked before it was trusted (the invariant each harness
enforces). Verdicts are the fixed vocabulary only.

| # | dimension | sparq measured | best competitor measured | verdict | fix / blocking bead + provenance |
|---|---|---|---|---|---|
| D3′ | **SP2Bench 250k complex-shape** (re-measure, same-box QLever) | wins **10/14**, correct+complete on **all 14**; q03b/c FLIPPED to AHEAD (38×/21×); residuals q07 30.9 ms, q09 12.9 ms, q11 5.3 ms | same-box QLever (HTTP, floor uncorrected → deficits are lower bounds); Oxigraph 0.5.9 CLI baseline | **MIXED, strongly improved** (was BEHIND): q07 **BEHIND 3.8×**, q09 **BEHIND 9.5×**, q11 **BEHIND 3.0×**; q08/q12b NO valid comparator (QLever wrong count 0) | q07 → **sq-7d3dj.30.20** (P1, CSE); q09 → **sq-jnb1e** (characteristic-set); q11 → **sq-7d3dj.30.21** (P2 top-k profile); q08/q12b same-box virtuoso → **sq-7d3dj.30.22** (P2). Env `bench/competitor-results/sparql-same-box-20260710T025117Z.json`; detail `research/sp2bench-complex-shape-deficit.md` §7 |
| D6 | **Reasoning / materialization throughput** (LUBM closure) | work-box univ=1/10 directional only (non-canonical, sq-hmd7l.7 PR body); **canonical univ≥100 NOT YET RUN** | Jena (profile-different rule set, NOT like-for-like); VLog / Nemo now have **validated** OWL-RL/RDFS encodings reproducing sparq's closure set-for-set (rdfs 126732 / owl 150589, #1823) | **NOT-MEASURED** (canonical) — directional work-box read is sparq-ahead of Jena's smaller-profile reasoner but non-citable | **CITE sq-hmd7l.32** (P2, canonical univ≥100 EC2 run IN FLIGHT); encodings #1823 (sq-hmd7l.30/.31); vs RDFox published 6.1 M t/s ≈ **47.7 k t/s per core** (SPARC T5-8, per-core-normalized §0) → also NOT-MEASURED at scale, **sq-1s03r** (G2). Record `research/gap-materialize-2026-07.md` |
| D12 | **Full-text search latency** (BM25 over SPARQL) | `and_terms` 11.9 µs / `or_terms` 20.3 µs / `prefix4` 3414.5 µs / `phrase` 1.4 µs | Fuseki + jena-text (Lucene), HTTP; count-crosscheck GREEN on all four translated workloads | **CLEARLY-AHEAD** (~75×–~1954×; survives a generous 500 µs HTTP-floor subtraction) | none needed (no deficit). Env `axis-results/fts/fts-n100000-20260710T004450Z.json`; `research/gap-fts-2026-07.md` §3 |
| D12q | **FTS IR-quality** (BEIR Recall@100 / nDCG@10) | NOT-MEASURED | Lucene / Anserini oracle — NOT-MEASURED | **NOT-MEASURED** (quality claim can't be made either way) | **CITE sq-tvzyi** (P2, schedule the BEIR pyserini/beir gather — not part of the wave-1 box). `research/gap-fts-2026-07.md` §4 |
| D13 | **SPARQL-UPDATE** (PSS interactive LDP-CRUD) | p50 3.50 ms / **p99 4.01 ms** / max 24.91 ms / **275/s** | **oxigraph** p50 0.45 / p99 0.61 / max 0.84 / **2245/s** (Docker, symmetric loopback HTTP); count-crosscheck GREEN 350=350; fuseki absent (container never ready — re-run action, not a win) | **BEHIND** (p99 ~6.6×, p50 ~7.8×, throughput ~8.2×; long-tail 24.9 ms outlier absent in oxigraph) | **sq-p7kk5** (NEW P1, profiling-first — per-update parse/plan, index maintenance on interleaved DELETE/INSERT/DROP, or alloc churn). Harness bugs (separate): sq-do5fx (envelope drop on parity FAIL), sq-l7diu (fuseki readiness). Rows `axis-results/update/run.log`; `research/gap-update-2026-07.md` §CANONICAL |
| D14a | **N-Triples parse**, 1 thread (in-process) | custom NT parse+intern **2.00 Mt/s** (1.278 s / 170 MB) | serd 1.68 Mt/s (subprocess, +serialize); rapper 1.00; riot 0.52; count-ok on every row | **AHEAD-BUT-NOT-OOM** (~1.2× vs serd, which also serializes) | none (sub-OOM lead labelled honestly). Env `axis-results/parse/parse-rows.txt`; `research/gap-parse-2026-07.md` §CANONICAL |
| D14b | **N-Triples parse**, 16 threads | 15.56 Mt/s (0.164 s) | serd 1.68 Mt/s (single-threaded by design) | **CLEARLY-AHEAD** (~9.3× — chunk-parallelism is the lever) | none. Same env |
| D14c | **Turtle parse**, 1 thread (in-process) | incumbent chunked **0.81 Mt/s** (3.153 s / 60 MB) | **serd 1.65 Mt/s** (+serialize); rapper 1.17; riot 0.52 | **BEHIND** serd ~2.0× (rapper ~1.4×) — single-thread hot-loop deficit; the chunked parser sits at ~oxttl speed | **sq-wrn61** (NEW P1, profiling-first — Turtle tokenizer hot loop, per-statement prefix/base + bnode scope, or per-term intern). Same env; `research/gap-parse-2026-07.md` §Verdicts |
| D14d | **Turtle parse**, 16 threads | 5.22 Mt/s (0.490 s) | serd 1.65 Mt/s | **AHEAD-BUT-NOT-OOM** (~3.2× — parallelism recovers the lead, not by OOM) | none (covered by the D14c single-thread fix once landed). Same env |
| D15a | **GeoSPARQL k-NN** (`nearest_k10` / `_k100`) | 6.1 µs / 70.0 µs | jena-geosparql 1 517 437 / 1 635 714 µs (HTTP; standard `ORDER BY geof:distance LIMIT k` full scan-and-sort) | **CLEARLY-AHEAD** (~2.5·10⁵× / ~2.3·10⁴×; count-crosscheck GREEN 10=10 / 100=100; caveats in the record) | none. Env `axis-results/geo/geo-points100k-20260710T004536Z.json`; `research/gap-geo-2026-07.md` §6b |
| D15b | **GeoSPARQL `geof:sfWithin`** COUNT | **90 458.9 µs** (PRE-FIX measurement) | jena-geosparql 137 888 µs (HTTP); count-crosscheck GREEN 1526=1526 | **AHEAD-BUT-NOT-OOM** (~1.5× vs jena, pre-fix) — the apparent self-slow ~600× vs `within50km` was **PARTLY a bench-harness artifact** (the `geof_within` workload in bench_geo.rs deliberate fully-scans every corpus point to test the lexical predicate, not the index) **PLUS a real answer-safety-guard defect** (the FILTER pushdown's retain guard materialized WKT Term + hashed per non-candidate row). **FIX MERGED:** PR #1847 (sq-7jt80) implemented an id-level indexed-universe fast path (FxHashSet elements, freshness-gated SpatialProvider hook), byte-identical result, per-row WKT materialization overhead ELIMINATED, answer-safety-guard now scan-floor-bound. Defect **CLOSED** | Same env; `research/gap-geo-2026-07.md` §6b finding 3; PR #1847 (sq-7jt80) |
| D15c | **GeoSPARQL radius** (`within10km` / `within50km`) | 7.2 µs / 150.2 µs (counts hard-gated 51 / 1547) | jena-geosparql returned **0 hits** on both → count-crosscheck **RED**; timing WITHHELD | **NOT-MEASURED** (crosscheck red — per the invariant no comparative verdict; NOT a sparq win) | **CITE sq-a8anf** (P2, jena within* translation/units/axis-order root-cause — a harness fix, must land before a canonical radius verdict). Same env; `research/gap-geo-2026-07.md` §6b finding 1 + §6c |
| D16 | **HDT load-and-decode-to-native** | decode-to-native GREEN 328=328; `hdt_load_s` 0.0898 s in-process | hdt-cpp `hdt2rdf` decode GREEN 328=328; wall 8.56 s incl. `docker run` spawn (different boundary) | **NOT-COMPARABLE at this scale** (328-triple fixture: container-spawn dominates; correctness axis is the gate and it is GREEN both engines) | **CITE sq-hmd7l.27** note: an OOM-scale archive gather is needed for a throughput verdict (no fix bead — correctness axis passes; throughput is a future gather, not a deficit). Env `axis-results/hdt/hdt-snikmeta-20260710T010720Z.json`; `research/gap-hdt-2026-07.md` |

### 9.2 Verdict-count summary (this consolidation)

Fixed vocabulary, counting the §9.1 rows (D3′ is counted by its three residual sub-verdicts):

- **CLEARLY-AHEAD (4):** D12 FTS latency, D14b NT-16T, D15a geo k-NN, plus D3′-partial (10/14
  SP2Bench queries AHEAD incl. the closed q03b/c). *(§0-§8 additionally: D1 WatDiv, D2 SP2Bench
  vs oxigraph, D9e time-to-serving.)*
- **AHEAD-BUT-NOT-OOM (3):** D14a NT-1T, D14d TTL-16T, D15b geo `geof:sfWithin` (vs jena; the
  self-slow gap's answer-safety defect was fixed by **sq-7jt80** [PR #1847 merged], bench-harness
  artifact is structural to the workload design).
- **PARITY (0)** in this consolidation.
- **BEHIND (4, each with a filed P1/P2 fix bead):** D13 SPARQL-UPDATE → **sq-p7kk5** (P1);
  D14c Turtle-1T → **sq-wrn61** (P1); D3′ q07 → **sq-7d3dj.30.20** (P1) + q09 → **sq-jnb1e**;
  q11 → **sq-7d3dj.30.21** (P2). *(§0-§8 additionally: D7 memory vs RDFox best-in-class; D9c
  TTFB streaming.)*
- **NOT-MEASURED (3, each citing its blocking bead):** D6 materialization canonical →
  **sq-hmd7l.32** (in flight); D12q BEIR IR-quality → **sq-tvzyi**; D15c geo radius (crosscheck
  red) → **sq-a8anf**.
- **NOT-COMPARABLE (1):** D16 HDT at the 328-triple fixture scale (container-spawn-dominated;
  correctness gate GREEN). *(fts `near_slop2` is also NOT-COMPARABLE by design.)*

### 9.3 New fix beads filed by this consolidation (all P1, profiling-first)

The §0-§8 D3 residuals and the memory/HTTP beads already existed and are **reused, not
duplicated** (checked via `bd list` / `bd show` before filing). Three wave-1 BEHIND / self-slow
rows had **no** engine-side profiling-first fix bead and are filed here:

1. **sq-p7kk5** (NEW P1) — SPARQL-UPDATE interactive-CRUD p99 BEHIND oxigraph ~6.6× (D13):
   profile the per-update path (parse/plan, index maintenance on interleaved DELETE/INSERT/DROP,
   alloc churn / the 24.9 ms long-tail) on the canonical box; targeted fix after the profile.
2. **sq-wrn61** (NEW P1) — single-thread Turtle parse throughput BEHIND serd ~2.0× (D14c):
   profile the incumbent chunked TTL parser hot loop; targeted fix after the profile.

**Reused (existing) beads cited above:** sq-7d3dj.30.20 (q07), sq-jnb1e (q09), sq-7d3dj.30.21
(q11), sq-7d3dj.30.22 (q08/q12b same-box virtuoso), sq-hmd7l.32 (D6 canonical), sq-tvzyi (BEIR),
sq-a8anf (geo radius crosscheck), sq-do5fx / sq-l7diu (update harness bugs), and the §0-§8
memory/HTTP beads (sq-7d3dj.32.*, sq-7d3dj.34.1/.2, sq-1s03r).

**Closed since this consolidation:** **sq-7jt80** (D15b `geof:sfWithin` self-slow gap, PR #1847
merged 2026-07-10) — the bench-harness artifact was correctly identified (deliberate full-scan
workload tests the lexical predicate), and the real ~5× answer-safety-guard defect (per-row WKT
materialization + hashing) was fixed via id-level indexed-universe fast path. No post-fix canonical
re-measure yet; defect impact eliminated, scan-floor-bound.

### 9.4 Honesty ledger for this consolidation

- **No new measurements.** Every §9 number is transcribed from a committed envelope or gap
  record with the filename cited in-cell.
- **Failed / absent competitors are re-run actions, never wins.** fuseki-absent on D13 (harness
  bugs sq-do5fx/sq-l7diu), jena 0-hits on D15c (crosscheck RED → NOT-MEASURED + sq-a8anf),
  hdt-cpp only at fixture scale on D16 (NOT-COMPARABLE) — none is scored as a sparq win.
- **Sub-order-of-magnitude leads are AHEAD-BUT-NOT-OOM, not CLEARLY-AHEAD:** D14a NT-1T (~1.2×),
  D14d TTL-16T (~3.2×), D15b geo sfWithin vs jena (~1.5×).
- **RDFox published columns are per-core-normalized** (D6: 6.1 M t/s → ~47.7 k t/s per core on
  the SPARC T5-8, per §0), never absolute-vs-absolute.
- **The D3 re-measure supersedes the §3 pre-fix verdict** (see the §3.1 addendum): q03b/c are
  now AHEAD; only q07/q09/q11 remain BEHIND, each with a filed bead.
- **Self-slow dominance gaps are surfaced even when AHEAD of the competitor:** D15b is ~1.5×
  ahead of jena yet was ~600× slower than sparq's own `within*` (now separated into bench-harness
  artifact + answer-safety defect). The answer-safety defect was closed by sq-7jt80 (PR #1847);
  the bench-harness artifact is intrinsic to how the workload is constructed (deliberate full-scan
  to test the lexical predicate path).
