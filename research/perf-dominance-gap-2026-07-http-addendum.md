<!-- [FABLE-5] Authored by Claude Fable 5. Bead sq-7d3dj.34 (parent sq-7d3dj; from the
sq-mk6wx gap table, PR #1727). This is the D9/D10 ADDENDUM to
research/perf-dominance-gap-2026-07.md — kept as a separate file because #1727 was still
in flight when this landed (coordinated follow-up, not a conflict). Same honesty rules:
measured numbers with provenance only; any dimension not CLEARLY-AHEAD carries a P1
profiling-first bead. -->

# Perf-dominance gap table 2026-07 — D9/D10 HTTP/TTFB addendum

**Status:** research record / measured gap analysis (addendum). **Date:** 2026-07-07.
**Bead:** sq-7d3dj.34. **Extends:** `research/perf-dominance-gap-2026-07.md` (PR #1727),
which recorded **D9 (HTTP serving / TTFB) = NOT-MEASURED** (sparq ran CLI-mode in the
canonical matrix while fuseki/virtuoso/qlever ran HTTP) and **D10 (fuseki) = FAILED**
(1800 s load timeout in both suites).

**Provenance (canonical).** Dedicated quiet **c6i.4xlarge** (16 vCPU / 32 GiB, x86_64,
Xeon Platinum 8375C) instances in eu-west-2, single-tenant, one engine active at a time,
same corpus + query files per suite, gathered 2026-07-07: SP2Bench 250 000 triples on
`i-0cad0f973c11788dc` (git `13bfa892`), WatDiv SF=1 on `i-01577af5e2c80956f` (git
`246b1bfe` — same harness, one gather-orchestration fix in between; both instances
terminated after pull, evidence in the PR). Two back-to-back gathers per suite,
**min-of-3 per connection regime per gather** (reported numbers = best-of the 2 gathers =
min-of-6); solution counts cross-checked engine-vs-engine and vs
`bench/<suite>/expected-rows.tsv` before any timing is trusted. Raw envelopes:
`bench/canonical-competitor-results/2026-07-07-http/`.

## 1. What was fixed and measured

### D10 root cause — the fuseki "load timeout" was a harness bug, not a Fuseki result

Root-caused ([FABLE-5], reproduced from the image bits): the docker backend of
`scripts/fuseki-same-box.sh` ran `docker run docker.io/stain/jena-fuseki tdb2.tdbloader …`,
and that image

1. **ships no `tdb2.tdbloader` at all** — it bundles Fuseki 5.1.0 plus only the TDB1
   `tdbloader`/`tdbloader2` scripts (not on `PATH`); and
2. its `/docker-entrypoint.sh` runs `exec "$@" &` (backgrounds the command) and then polls
   `http://localhost:3030` in an **unbounded** `until curl` loop — waiting for a server
   that a one-shot loader command never starts. The container therefore **hangs forever**,
   and the outer harness timeout (1800 s) is what actually fired — deterministically, in
   both suites (`fuseki_wall_s = 1800.7/1800.8`).

Per the mandate this was recorded as a **re-run action, not a sparq win** — correctly, as
the numbers below confirm Fuseki is a competitive engine once actually measured (it is the
**best correct competitor** on SP2Bench q12b).

**Fix (committed).** `scripts/fuseki-same-box.sh` now defaults to Fuseki's **intended bulk
path** — the official Apache Jena 6.1.0 tarballs (sha512-pinned auto-fetch), **offline
`tdb2.tdbloader`** into a TDB2 store, then `fuseki-server --tdb2 --loc` — and the docker
fallback now preflights that the image actually contains a TDB2 loader and bypasses the
broken entrypoint with `--entrypoint`. Canonical confirmation: **fuseki `status: ok` in
all four gathers**; the 250k SP2Bench corpus that "timed out at 1800 s" tdb2-loads in
seconds (fuseki whole-recipe wall incl. all queries + its own two honest 300 s per-query
timeouts: 615.6 s on sp2b, **7.4 s on watdiv**).

### D9 instrument — the canonical HTTP/TTFB panel

New committed harness (`scripts/bench/canonical-competitor-bench.sh` +
`canonical-http-gather-instance.sh`): **all five engines in the same HTTP regime** —

- **sparq** as `sparq-server` (SPARQL 1.1 protocol, `/sparql`) — no longer CLI-mode;
- **oxigraph** as `oxigraph serve-read-only` (prebuilt sha256-pinned CLI);
- **fuseki** via the fixed offline-tdb2 path above (Apache Jena 6.1.0);
- **virtuoso** and **qlever** exactly as in the CLI matrix (docker recipes, digests pinned);

with the shared `http_sparql_adapter.py --profile` measuring, per query, **full-request
latency AND TTFB** (time to status-line+headers) in **BOTH connection regimes**:
**keep-alive** (one persistent HTTP/1.1 connection; min-of-N discards the connect-bearing
first iteration) **and fresh-connect** (new TCP connection per iteration). Measuring both
resolves the keep-alive-vs-fresh fairness question the gap table left open with data:

> **Measured connect overhead (loopback), keep-alive→fresh delta on the empty-result
> floor row (sp2b q12c, best-of):** sparq +28 µs (183→211), oxigraph +162 µs (163→325),
> virtuoso +43 µs (448→491), fuseki +82 µs (3471→3553). The regime choice shifts sub-ms rows by
> tens-of-µs and changes **no verdict**; all tables below use the keep-alive full-request
> best (the steady-state server measure), with the fresh twin carried in the envelopes +
> dashboard (`values_fresh*`).

## 2. WatDiv SF=1 over HTTP — sparq AHEAD on 16/16, but floor-bound (not OOM)

Best-of-2-gathers keep-alive full-request µs; "best correct" = fastest competitor whose
solution count matched the expected rows.

| query | rows | sparq | best correct competitor | ratio |
|---|---|---|---|---|
| C3 | 8763 | 1940 | qlever 29891 | **15.4×** |
| F2 | 1 | 244 | oxigraph 638 | 2.6× |
| F3 | 3 | 232 | virtuoso 744 | 3.2× |
| F5 | 34 | 390 | virtuoso 1821 | 4.7× |
| L1 | 4 | 221 | virtuoso 672 | 3.0× |
| L2 | 3 | 224 | virtuoso 600 | 2.7× |
| L3 | 41 | 261 | oxigraph 700 | 2.7× |
| L4 | 2 | 212 | oxigraph 402 | 1.9× |
| L5 | 1 | 220 | oxigraph 294 | **1.3×** |
| S1 | 6 | 417 | virtuoso 983 | 2.4× |
| S2 | 4 | 269 | virtuoso 629 | 2.3× |
| S3 | 6 | 221 | virtuoso 716 | 3.2× |
| S4 | 4 | 232 | virtuoso 644 | 2.8× |
| S5 | 8 | 256 | virtuoso 770 | 3.0× |
| S6 | 1 | 227 | oxigraph 369 | 1.6× |
| S7 | 2 | 240 | oxigraph 310 | **1.3×** |

**Honest reading.** sparq wins **every** WatDiv row over HTTP — but by **1.3–15×, not the
orders of magnitude of the CLI matrix (205–23 453×)**. The reason is structural, not an
engine regression: at SF=1 every WatDiv query is sub-100 µs of engine compute (CLI matrix
D1), so the HTTP panel is measuring each stack's **per-request transport floor**. sparq's
floor is the lowest (~190–230 µs keep-alive) vs oxigraph ~290–700 µs, virtuoso ~600–1800 µs,
qlever ~2.5–30 ms — a real serving-stack win, but the mandate's "order(s) of magnitude on
every axis" is **not met on the floor-bound rows** (L5/S7 = 1.3×). The engine-compute
dominance is only visible where compute exceeds the floor (C3: 15.4×).

## 3. SP2Bench 250k over HTTP — ahead 6/14, BEHIND 8/14 (matched-mode confirmation of D3)

Same combine rules; qlever is count-DISQUALIFIED on q08/q12b (returned 0 rows vs expected
358/1 — same wrong-answer DIFF as the CLI matrix), oxigraph ERROR'd q07/q08, fuseki hit
its honest 300 s cap on q04/q07.

| query | rows | sparq ka | sparq TTFB | best correct competitor | ratio | verdict |
|---|---|---|---|---|---|---|
| q01 | 1 | 192 | 187 | virtuoso 536 | 2.8× | ahead |
| q02 | 6067 | 21318 | 18579 | qlever 237107 | 11.1× | ahead |
| q03a | 15823 | 17985 | 17061 | qlever 33913 | 1.9× | ahead |
| q03b | 114 | 12698 | 12667 | virtuoso 1235 | **0.10×** | **BEHIND** |
| q03c | 0 | 12585 | 12578 | virtuoso 462 | **0.04×** | **BEHIND** |
| q04 | 541911 | 428428 | 372029 | qlever 2115683 | 4.9× | ahead |
| q05b | 6933 | 22284 | 22152 | qlever 24309 | 1.1× | ahead (margin below floor variance — fragile) |
| q07 | 48 | 24220 | 24213 | qlever 7984 | **0.33×** | **BEHIND** |
| q08 | 358 | 159142 | 159113 | virtuoso 12710 | **0.08×** | **BEHIND** |
| q09 | 4 | 22897 | 22890 | qlever 10878 | **0.48×** | **BEHIND** |
| q10 | 452 | 309 | 289 | oxigraph 2264 | 7.3× | ahead |
| q11 | 10 | 26793 | 26783 | qlever 4695 | **0.18×** | **BEHIND** |
| q12b | 1 | 156866 | 156850 | **fuseki 4966** | **0.03×** | **BEHIND** |
| q12c | 0 | 183 | 178 | oxigraph 163 | **0.89×** | **BEHIND (floor parity)** |

**Reading.**

- The seven engine-compute losses (q03b/c, q07, q08, q09, q11, q12b) are **exactly the
  D3 complex-shape class**, now confirmed with sparq in the SAME HTTP mode — the
  CLI-vs-HTTP asymmetry caveat is closed and the deficits stand (they are engine plan
  cost, not transport). These remain covered by the existing **P1 sq-7d3dj.30**
  (profiling-first). The fixed Fuseki adds a new data point: it is the best correct
  competitor on q12b (ASK), 32× faster than sparq there.
- **q12c is a NEW floor finding**: on an empty-result query, `oxigraph serve-read-only`'s
  keep-alive request floor (163 µs) is **below sparq-server's (183–191 µs)** — a ~20–30 µs
  per-request stack gap that shows on every floor-bound row (also WatDiv L5/S7 at 1.3×).
  → **NEW P1 sq-7d3dj.34.1** (profile the sparq-server request path; target the lowest floor
  on every row).

## 4. TTFB — the streaming gap (new, measured)

TTFB ≈ full-request for every engine on small results (the response fits one write). On
the **large-SELECT** row (q04, 541 911 rows, SPARQL-JSON) the engines diverge sharply:

| engine | q04 full (µs) | q04 TTFB (µs) | first-byte strategy (observed) |
|---|---|---|---|
| sparq | 428428 | 372029 | first byte only after ~87% of total — mostly-materialize-then-stream |
| oxigraph | 18114230 | **7825** | streams immediately; slowest total |
| qlever | 2115683 | 89005 | streams early |
| virtuoso | 8075623 | 7940246 | no streaming (TTFB ≈ full) |
| fuseki | ERROR (300 s cap) | — | — |

**Honest verdict:** sparq has the **fastest q04 full-request by 4.9×** — the number an
end-to-end consumer sees — but **oxigraph's first byte arrives 48× earlier** (7.8 ms vs
372 ms) and qlever's 4× earlier. For latency-sensitive streaming consumers (paginated UIs,
early-abort clients) TTFB-to-first-solution is a real axis where sparq is **BEHIND** the
streaming engines. → **NEW P1 sq-7d3dj.34.2** (profile where the first row of a large SELECT
is emitted in sparq-server; push-based emission before full materialization).

## 5. Time-to-serving (load axis, HTTP mode)

sparq-server goes from process spawn to **answering HTTP queries over the 250k corpus in
0.52 s** (WatDiv: 0.51 s) — vs oxigraph offline bulk-load 1.3 s (then serve), qlever
index+serve recipe 24.3 s, virtuoso 74.8 s, fuseki offline tdb2 load + serve within its
615.6 s recipe wall (dominated by two honest 300 s query timeouts, not the load). On
time-to-serving sparq is **CLEARLY-AHEAD** of every server that requires an offline
index/load step.

## 6. Updated D9/D10 verdict rows (for the #1727 gap table)

| # | dimension | sparq measured | best competitor measured | verdict | action |
|---|---|---|---|---|---|
| D9a | HTTP full-request latency — WatDiv | ahead 16/16 (1.3–15.4×) | oxigraph/virtuoso floors 294–1821 µs | **AHEAD-BUT-NOT-OOM** (transport-floor-bound) | **NEW P1 sq-7d3dj.34.1** (request-floor profile; beat oxigraph's 163 µs floor) |
| D9b | HTTP full-request latency — SP2Bench | ahead 6/14 | virtuoso/qlever/fuseki on the D3 class | **BEHIND 8/14** | engine-compute class = existing **sq-7d3dj.30**; floor row = **sq-7d3dj.34.1** |
| D9c | TTFB on large SELECT (q04) | 372 ms first byte (4.9× fastest full) | oxigraph 7.8 ms / qlever 89 ms first byte | **BEHIND on first-byte streaming** | **NEW P1 sq-7d3dj.34.2** (stream-before-materialize) |
| D9d | keep-alive vs fresh fairness | both measured (Δ +20 µs sparq … +162 µs oxigraph) | — | **RESOLVED** (regime changes no verdict) | envelope + dashboard carry both |
| D9e | time-to-serving (spawn→first answer) | 0.52 s @250k | qlever 24.3 s / virtuoso 74.8 s recipes | **CLEARLY-AHEAD** | hold |
| D10 | fuseki | — | fuseki `ok` in 4/4 gathers, best-correct on q12b | **FIXED** (was a harness bug: no TDB2 loader + unbounded entrypoint loop in the docker image) | committed fuseki-same-box fix; folded into future gathers |

**No spin:** the WatDiv HTTP win is real but floor-bound and labelled so; the SP2Bench
complex-shape deficit **survives** the matched-mode re-measurement and stays BEHIND; the
TTFB streaming axis is a genuine new BEHIND with its own P1 bead. Fuseki's q04/q07 cells
are honest per-query-cap ERRORs (its own compute exceeded 300 s), not missing data; a
future gather can raise `QTO` to fill them.
