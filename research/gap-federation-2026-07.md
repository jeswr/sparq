<!-- [FABLE-5] sq-hmd7l.12 — per-axis gap record: comparative SPARQL federation.
First-read only; work-box WALL-TIME readings are NON-canonical by construction and are
never transcribed here. Per-member HTTP request counts ARE transcribed: they are
deterministic engine-behaviour facts (identical on every box for a fixed corpus seed),
not timings. -->

# Gap record — SPARQL federation (2026-07)

**Axis:** federation (epic `sq-hmd7l`, bead `sq-hmd7l.12`).
**Status:** harness DELIVERED + smoke green (sparq + gather-time Comunica); Jena naive
column implemented but NOT-MEASURED (needs a Jena install: `FED_JENA_ARQ`); no canonical
run yet (§4).
**Harness:** `bench/federation/` (`run.sh` + `compare.py` + `comunica_runner.mjs`),
registered as `federation-fedshop`.
**Feeds:** the master table in `research/perf-dominance-gap-2026-07.md` once a canonical
run exists.

## 1. Engines and honest scope

| Engine | Federation surface | Comparability |
|---|---|---|
| sparq | sparq-server `--features service`: SPARQL 1.1 SERVICE eval + on-by-default bound-join pushdown | subject (server regime: HTTP round-trip timed) |
| Comunica (`@comunica/query-sparql@^4`) | explicit SERVICE **and** virtual `sources=[members]` (engine-side source selection) | reference federated-SPARQL engine (library regime: engine-internal exec timed; both regimes recorded in the results JSON) |
| Jena `arq` | explicit SERVICE over an empty default graph | naive baseline (JVM startup included, flagged) |

Scope caveats:

- **Correctness before stopwatch (INVARIANT):** per query the canonical result-set
  MULTISETS (not counts) must agree across every engine that executed, BEFORE any
  timing; an executed-but-disagreeing pair is a hard red. The oracle is hermetically
  regression-tested (`compare.py --self-test`: order-insensitive, multiset-sensitive,
  datatype-sensitive, RDF 1.1 plain≡xsd:string, bnode-label-independent).
- **Corpus is FedShop-SHAPED, not upstream FedShop:** a deterministic in-repo
  vendor+ratingsite shop federation (shared product IRIs = join keys). Upstream
  FedShop's dockerized members are replaced by same-box sparq-server replicas —
  variance-controlled, and the only honest option (LargeRDFBench's public endpoints
  are dead). Per-member corpus sha256 + resolved Comunica version are recorded in
  every results JSON.
- **Request counts are uniform:** every engine is pointed at a counting reverse proxy
  per member; ground-truth member relevance is MEASURED (block probes against the
  members' real ports), never assumed.
- **sparq virtual-regime cell is n/a:** sparq-fedclient (the source-selecting
  federation client) is a library with no server-exposed endpoint; the harness reports
  an honest n/a and a fedclient runner column is a follow-up bead (sq-hmd7l.47).
- **SolidBench** (many small pod-shaped sources) is a follow-up suite candidate.

## 2. First-read findings (work box — request counts transcribed as deterministic facts; wall times NOT)

Full panel, 500-product corpus, seed 42 (5 queries, all oracles AGREED sparq↔Comunica):

1. **Bound-join dominance in requests (deterministic):** on the offer⋈review join (q01)
   sparq answers with **vendor:1 + ratingsite:5** member requests (VALUES-batched
   bound-join) vs Comunica-explicit **vendor:2 + ratingsite:248** (per-binding
   requests) and Comunica-virtual **vendor:7 + ratingsite:38**. The selective
   bind-join probe (q05) reads sparq **1+6** vs Comunica-explicit **2+318**. Request
   count is the axis's structural lever: sparq issues 1–2 orders of magnitude fewer
   member requests on join-heavy shapes.
2. **Wall-time ordering (ordinal only, NON-canonical):** sparq's HTTP round-trip was
   consistently 1–2 orders of magnitude below Comunica's engine-internal exec on every
   query — directionally consistent with the request-count gap, but the two timing
   regimes differ (server vs library) and the box is shared: a canonical quiet-box run
   must confirm before any dominance claim enters the master table.
3. **Source-selection signal is real in the virtual regime:** Comunica-virtual contacts
   both members on the single-member query (q04: precision 0.50 — 4 requests to the
   irrelevant ratingsite member) where the explicit regime is precision 1.0 by query
   text. This validates the precision/recall instrumentation; sparq cannot enter this
   comparison until a fedclient runner exists (sq-hmd7l.47).
4. **OPTIONAL-SERVICE anti-join agrees cross-engine (q03):** `OPTIONAL { SERVICE … } +
   FILTER(!bound)` produces identical multisets on sparq and Comunica — a semantics
   corner worth locking in as a conformance fact independent of timing.
5. **Comunica v5 requires Node ≥ 22** (undici@8); the harness pins `@^4` by default and
   records the resolved version — a canonical gather on a newer box may bump the major
   deliberately.

## 3. Gap verdict (per the dominance mandate)

- **Requests/join-strategy: CLEARLY-AHEAD (deterministic).** No fix bead needed.
- **Wall time: LIKELY-AHEAD, NOT-CANONICAL.** Instrument bead = run the panel on the
  canonical quiet box (§4); no P1 fix bead — no dimension read behind.
- **Virtual-regime source selection: NOT-COMPARABLE yet** (sparq column missing, not
  behind) — follow-up bead sq-hmd7l.47 (sparq-fedclient runner column).

## 4. What a canonical run needs

Quiet dedicated box; `bash bench/federation/run.sh` with `FED_COMUNICA_SPEC` pinned to
an exact version; optionally `FED_JENA_ARQ` for the naive baseline; archive the results
JSON (corpus sha256s + versions are embedded) under the canonical results store, then
promote the verdicts into `research/perf-dominance-gap-2026-07.md`.
