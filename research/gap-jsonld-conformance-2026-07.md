# Gap record: W3C JSON-LD 1.1 self-conformance scoreboard (2026-07)

> 🤖 SPARQ agent — written by **Claude Fable 5** [FABLE-5]
>
> Bead **sq-w42s3** (epic sq-hmd7l, comparative-benchmarking-EVERYTHING).
> Companion to `research/gap-conformance-cross-engine-2026-07.md` (sq-hmd7l.22),
> whose JSON-LD row this record refreshes with a durable, re-runnable
> sparq-side runner.

## 1. What this record is

sq-hmd7l.22 found JSON-LD 1.1 **BEHIND** the peers' published EARL results, but
sparq's own row in that table was assembled from the CI floor consts by hand.
This record pins the durable **self-scoreboard runner** that produces the
sparq-side row from an actual run, and reconciles the per-operation verdicts
against the current estate (the floors have RISEN since .22 was written —
compact 186→228, frame 61→92 — so several .22 cells are stale-low).

**Runner:** `crates/sparq-jsonld/examples/jsonld_conformance.rs`

```sh
scripts/fetch-jsonld-tests.sh && scripts/fetch-jsonld-framing-tests.sh
cargo run -p sparq-jsonld --release --example jsonld_conformance             # table + fail listing
cargo run -p sparq-jsonld --release --example jsonld_conformance -- --json   # machine-readable row
cargo run -p sparq-jsonld --release --example jsonld_conformance -- --smoke  # pinned acceptance subset
```

It drives the **native `sparq-jsonld` document-level pipeline** (the crate's
public `expand` / `compact::compact` / `flatten` / `frame::frame` /
`from_rdf::from_rdf` entries) over the pinned W3C suites
(`w3c/json-ld-api` `8654ac22b6cf`, `w3c/json-ld-framing` `3bf782b`), and
emits per-operation `pass / fail / skip / total`, a TRUNCATED (never
rounded-up) pass-rate, and the complete failing-test listing. Full runs assert
the ratchet floors (`pass >= FLOOR`); `--smoke` asserts EXACT pinned counts on
the first-30-per-manifest subset (the bead's acceptance gate).

**Invariant (sq-w42s3):** every rate below comes ONLY from this runner's
output at the pins above; every failing test is listed; nothing is rounded up;
the BEHIND verdicts are preserved honestly. Conformance pass-rates are
correctness counts, not performance figures — no latency/throughput numbers
appear in this record.

## 2. Measured scoreboard (runner output, 2026-07-17, pins above)

| operation | pass | fail | skip | total | rate (strict, skips count against) | rate over RUN tests |
|---|---|---|---|---|---|---|
| expand  | 276 | 0 | 109 | 385 | 71.6% | 276/276 = 100% |
| compact | 228 | 0 | 18  | 246 | 92.6% | 228/228 = 100% |
| flatten | 53  | 0 | 5   | 58  | 91.3% | 53/53 = 100% |
| frame   | 92  | 0 | 0   | 92  | 100.0% | 92/92 (negatives RUN) |
| fromRdf | 52  | 0 | 1   | 53  | 98.1% | 52/52 (negatives RUN) |
| toRdf   | — | — | — | — | NOT IMPLEMENTED natively | see §4 |

There are **0 outright fails** at the pin: every sub-100% strict rate is a
**skip bucket**, itemised below. (The 2026-07-12 snapshot had 1 fail — compact
`#t0038`, a JSON-LD-**1.0**-only positive the compact lane RAN rather than
skipped; bead sq-uzdw7 resolved it as an honest 1.0 skip — see the compact
skip-bucket below and `floors::compact`.)
The document-level oracle is the pinned sq-kk1mq comparator (key order
insignificant; array order significant only inside `@list`; exact integral /
f64-fallback numerics) — the same oracle the `sparq-conformance` ratchet lane
asserts, ported onto the crate's own `Json` AST so this runner stays
dependency-free.

Skip-bucket composition (from the pinned manifests, no estimates):

- **expand: 109 skips = exactly the 109 NegativeEvaluationTests.** The
  expander raises spec error codes but error-code COMPLETENESS is unverified,
  so the negative lane is deferred (bead sq-oy1f.31) and honestly skipped —
  never counted as passes. This bucket is the entire expand gap.
- **compact: 18 skips = 17 NegativeEvaluationTests** (same deferral) **+ the
  one `specVersion: json-ld-1.0` positive (`#t0038`)**. sq-uzdw7 decision:
  `#t0038`'s 1.0-era expectation (compact-IRI creation through an expanded
  term definition) directly contradicts what `#tp001` pins for 1.0 processing
  mode under the 1.1 REC — both cannot pass, jsonld.js/pyld make the same
  trade — so the lane adopts the fromRdf convention (1.0-only tests are run by
  1.0 processors) and skips on `specVersion` only; the
  `processingMode: json-ld-1.0` cases of the 1.1 suite still run (and pass).
- **flatten: 5 skips** = 1 negative + the JSON-LD-1.0-only positives + the
  post-flatten-compaction (`context`-member) compositions.
- **fromRdf: 1 skip** = the one `specVersion: json-ld-1.0` case (a 1.1
  processor intentionally does not reproduce the 1.0 partial-list algorithm);
  its 2 negatives are RUN and pass.

## 3. Cross-engine reconciliation (peer cells from sq-hmd7l.22, unchanged provenance)

Peer numbers are the official JSON-LD 1.1 EARL report subjects quoted in
`research/gap-conformance-cross-engine-2026-07.md` §3.5 (retrieved 2026-07-10;
report denominators are SMALLER — a real suite-version mismatch, so rates are
loosely comparable only).

| operation | sparq (strict) | best published peers | verdict |
|---|---|---|---|
| expand  | 71.6% | jsonld-cpp / JSON-LD.ex 100.0, Titanium 98.1, jsonld.js 97.3 | **BEHIND** (driver: deferred negative lane, sq-oy1f.31) |
| compact | 92.6% | JSON-LD.ex 99.6, Titanium 98.0, jsonld.js 97.5 | **BEHIND**, narrowed from .22's stale 75.6 (floor has risen 186→228); residual = 17 negatives + the `#t0038` 1.0-only skip |
| flatten | 91.3% | listed peers 100.0 | **BEHIND** (small; 5 skips) |
| frame   | 100.0% | jsonld.js 97.8, Titanium 96.7 | **AHEAD at sparq's pin** (.22's 66.3 is stale — floor rose 61→92; denominator 92 vs report 91, loosely comparable) |
| fromRdf | 98.1% | Titanium / Sophia 98.1 (of 52) | **PARITY** (.22 cited the then-floor 51; current measured 52/53) |
| toRdf (native) | not implemented | jsonld-cpp 99.8, JSON-LD.ex 99.6 | **BEHIND** natively; engine-path toRdf is a separate 413/467 = 88.4% ratchet (§4) |

The epic-level verdict stays **BEHIND** — honestly driven by (a) the deferred
negative-test lanes (expand/compact/flatten) and (b) native toRdf not existing
yet — but the .22 snapshot materially understated the current estate on
compact and frame. (The former driver (c), compact `#t0038`, is resolved: an
honest 1.0 skip per sq-uzdw7, still counted against the strict rate.)

## 4. toRdf honesty note

`sparq_jsonld::to_rdf` is still the sq-oy1f.23 scaffold; the native
Deserialization-to-RDF algorithm is bead **sq-oy1f.30**. The ENGINE's toRdf
ingest (via `oxjsonld`) is ratcheted separately in `sparq-conformance`
(`floors::to_rdf`, 413/467 at the pin). The runner reports the native toRdf
row as `implemented: false` rather than borrowing the engine number — the two
are different implementations and conflating them would overstate the native
pipeline.

## 5. Relationship to the ratchet lane (no drift, no double ownership)

The CI ratchet of record remains
`crates/sparq-conformance/tests/jsonld_suite.rs` + the lib-side
`sparq_conformance::floors::<lane>` consts (sq-oy1f.40). This runner cannot
import those consts (sparq-conformance depends on sparq-jsonld, not vice
versa), so it carries documented mirrors asserted with FLOOR (`>=`) semantics
— an upstream ratchet rise never breaks it, a regression below the shared
floor fails loudly. Its `fromRdf` oracle is document-level only (the ratchet
lane adds a scoped oxjsonld round-trip leg), calibrated independently: at the
current pin both measure 52. No overlap with `bench/jsonld` (throughput,
sq-hmd7l.15/.43) or the cross-engine table script (sq-hmd7l.22).

## 6. Gap-closure levers (beads, not TODOs)

- **sq-oy1f.31** — negative-test lanes (error-code completeness) for
  expand/compact/flatten: worth up to +109 / +17 / +1 strict-rate points and
  is the whole remaining expand gap.
- **sq-oy1f.30** — native toRdf (Deserialization to RDF); once landed, the
  runner grows a real toRdf row.
- **sq-uzdw7** — RESOLVED (2026-07-17): compact `#t0038` ("Index map
  round-tripping", json-ld-1.0-only positive), formerly the single outright
  failing test at the pin, is reclassified as an honest 1.0 skip in BOTH the
  ratchet lane and this runner (the fromRdf `specVersion`-only convention).
  "Fixing" it was rejected as unreachable: its 1.0-era prefixing expectation
  contradicts `#tp001` (a 1.1-suite test run in 1.0 processing mode), so a
  REC-conformant processor cannot pass both. Floors unchanged (compact stays
  228; fail 1→0, skip 17→18).
