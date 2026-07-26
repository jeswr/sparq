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

## 2. Measured scoreboard (runner output, 2026-07-26, pins above)

| operation | pass | fail | skip | total | rate (strict, skips count against) | rate over RUN tests |
|---|---|---|---|---|---|---|
| expand  | 379 | 0 | 6  | 385 | 98.4% | 379/379 = 100% (negatives RUN) |
| compact | 244 | 0 | 2  | 246 | 99.1% | 244/244 = 100% (negatives RUN) |
| flatten | 53  | 0 | 5  | 58  | 91.3% | 53/53 = 100% |
| frame   | 92  | 0 | 0  | 92  | 100.0% | 92/92 (negatives RUN) |
| fromRdf | 52  | 0 | 1  | 53  | 98.1% | 52/52 (negatives RUN) |
| toRdf   | — | — | — | — | NOT IMPLEMENTED natively | see §4 |

> **Update — bead `sq-gzsky` (2026-07-26).** The 2026-07-17 snapshot read
> expand 276/0/109 (71.6%) and compact 228/0/18 (92.6%); the whole difference was
> the DEFERRED NegativeEvaluationTest lanes (§6, then sq-oy1f.31), not wrong
> answers. Those lanes now RUN in both this runner and the `sparq-conformance`
> ratchet, driven exactly like `frame` (sq-oy1f.29): a negative passes **iff** the
> operation errors with EXACTLY the manifest's `expectErrorCode`, so a
> raised-but-WRONG code FAILS. 95 of expand's 103 and 15 of compact's 16 RUNNABLE
> negatives passed unchanged; SEVEN spec-faithful fixes in `sparq-jsonld` flipped
> the remaining nine (`@value`+`@type`+`@direction`; datatype-IRI validation incl. blank
> nodes; `@included` null coercion; the term round-trip check's spurious keyword
> exemption; 1.0-mode `@container`-must-be-a-string; and IRI Compaction's missing
> `IRI confused with prefix` guard, §7.1 step 5 tail, which also exposed a missing
> "IRI-expand the index key first" leg in property-valued `@index` containers).
> Floors raised in lock-step: `floors::expand` 276 → **379**, `floors::compact`
> 228 → **244**. Still **0 outright fails**.

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

- **expand: 6 skips = the 1.0-ONLY negatives** (`#t0115`, `#t0116`, `#ter02`,
  `#ter03`, `#ter24`, `#ter32`). All carry `option.specVersion: json-ld-1.0`,
  which the suite's own `vocab.jsonld` defines as "the JSON-LD version to which
  the test applies". Raising for them would be WRONG, not merely unimplemented —
  every one is behaviour 1.1 deliberately CHANGED: `list of lists` and `recursive
  context inclusion` are codes 1.1 RETIRED (absent from the 1.1 error registry
  that the closed `JsonLdErrorCode` enum models — 1.1 allows lists of lists and
  reports a cyclic context as `context overflow`), and a relative `@vocab` is
  explicitly PERMITTED in 1.1. The exact set is pinned by the ratchet lane's
  `expand_1_0_negative_skips_are_pinned`, so a suite-pin bump that adds one fails
  loudly. NARROW: the `option.processingMode: json-ld-1.0` cases of the 1.1 suite
  (e.g. `#tes01`) a 1.1 processor MUST honour are RUN, and pass.
- **compact: 2 skips = the one 1.0-only NEGATIVE (`#te001`, `compaction to list
  of lists` — another retired 1.1 code) + the one `specVersion: json-ld-1.0`
  positive (`#t0038`)**. sq-uzdw7 decision:
  `#t0038`'s 1.0-era expectation (compact-IRI creation through an expanded
  term definition) directly contradicts what `#tp001` pins for 1.0 processing
  mode under the 1.1 REC — both cannot pass, jsonld.js/pyld make the same
  trade, and the suite's `vocab.jsonld` defines `specVersion` as "the JSON-LD
  version to which the test applies" — so the lane skips it, NARROWLY PINNED
  to the exact manifest id (never a blanket `specVersion` match): the runner's
  `t0038_skip_is_narrowly_scoped` test asserts exactly `#t0038` is skipped,
  that the `processingMode: json-ld-1.0` cases of the 1.1 suite still run (and
  pass), and that a future 1.0-only positive at a suite-pin bump runs rather
  than being silently skipped.
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
| expand  | 98.4% | jsonld-cpp / JSON-LD.ex 100.0, Titanium 98.1, jsonld.js 97.3 | **PARITY-to-AHEAD at sparq's pin** (sq-gzsky ran the negative lane, 71.6 → 98.4; residual = the 6 1.0-only negatives). Denominator 385 vs report 376 — loosely comparable only. |
| compact | 99.1% | JSON-LD.ex 99.6, Titanium 98.0, jsonld.js 97.5 | **PARITY** (sq-gzsky 92.6 → 99.1; .22's 75.6 was doubly stale); residual = `#te001` + `#t0038`, both 1.0-only |
| flatten | 91.3% | listed peers 100.0 | **BEHIND** (small; 5 skips) |
| frame   | 100.0% | jsonld.js 97.8, Titanium 96.7 | **AHEAD at sparq's pin** (.22's 66.3 is stale — floor rose 61→92; denominator 92 vs report 91, loosely comparable) |
| fromRdf | 98.1% | Titanium / Sophia 98.1 (of 52) | **PARITY** (.22 cited the then-floor 51; current measured 52/53) |
| toRdf (native) | not implemented | jsonld-cpp 99.8, JSON-LD.ex 99.6 | **BEHIND** natively; engine-path toRdf is a separate 413/467 = 88.4% ratchet (§4) |

The epic-level verdict is now driven by ONE remaining item: **(b) native toRdf
does not exist yet** (§4). Driver (a) — the deferred negative-test lanes — is
resolved for `expand` and `compact` (bead sq-gzsky, 2026-07-26); `flatten` keeps
one deferred negative in its 5-skip bucket. The .22 snapshot materially
understated the estate on compact and frame, and is now stale on expand too. (The former driver (c), compact `#t0038`, is resolved: an
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

- **sq-oy1f.31** — negative-test lanes (error-code completeness). **DONE for
  expand and compact** (bead sq-gzsky, 2026-07-26: +103 / +16 pass, floors
  276 → 379 and 228 → 244). The remaining slice is `flatten`'s ONE deferred
  negative, still inside its 5-skip bucket.
- **sq-oy1f.30** — native toRdf (Deserialization to RDF); once landed, the
  runner grows a real toRdf row.
- **sq-uzdw7** — RESOLVED (2026-07-17): compact `#t0038` ("Index map
  round-tripping", json-ld-1.0-only positive), formerly the single outright
  failing test at the pin, is reclassified as an honest skip in BOTH the
  ratchet lane and this runner, NARROWLY PINNED to that exact manifest id
  (scope enforced by the ratchet lane's `t0038_skip_is_narrowly_scoped` test —
  a blanket `specVersion: json-ld-1.0` skip was rejected in PR review so no
  future 1.0-only positive is silently absorbed). "Fixing" it was rejected as
  unreachable: its 1.0-era prefixing expectation contradicts `#tp001` (a
  1.1-suite test run in 1.0 processing mode), so a REC-conformant processor
  cannot pass both. Floors unchanged (compact stays 228; fail 1→0,
  skip 17→18).
