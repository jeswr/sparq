<!-- [FABLE-5] sq-hmd7l.15 — per-axis gap record: JSON-LD processing. First-read
throughput below is from a shared WORK BOX and is NON-canonical by construction;
the canonical wave re-run (quiet EC2 box) supersedes §3 when it lands. Conformance
numbers ARE deterministic (suite-pinned) and are canonical for the cited pins. -->

# Gap record — JSON-LD (2026-07)

**Axis:** JSON-LD 1.1 processing (epic `sq-hmd7l`, bead `sq-hmd7l.15`).
**Subject:** `sparq-jsonld` (native document-level expand/flatten) + the engine's
toRdf/fromRdf/compaction paths. **Peers:** jsonld.js (BSD-3-Clause, JS reference
implementation), titanium-json-ld (Apache-2.0, Java) — both in `bench/competitors.json`.
**Harness:** `bench/jsonld/` (this bead's deliverable). **Invariant:** no throughput
row without output-equality agreement (see `bench/jsonld/README.md`).

## 1. Conformance — pass-rate table (pinned published sources, never estimated)

Machine-readable source + full provenance/caveats: `bench/jsonld/conformance-peers.json`.

| Lane | sparq (measured 2026-07-11, pin `8654ac2`) | jsonld.js (W3C EARL 2025-04-10) | Titanium (README `ab0ccd4`, 2026-06-06) |
|---|---|---|---|
| expand | **276/385** (0 fail, 109 skip¹) | 97.3% of 376 | 376/376 |
| flatten | **53/58** (0 fail, 5 skip) | 100% of 55 | 55/55 |
| toRdf | **413/467** (32 fail, 22 skip) | 95.2% of 456 | 454/456 |
| fromRdf | **52/53** (0 fail, 1 skip) | 94.2% of 52 | 52/52 |
| compact | 186/246² (35 fail, 25 skip) | 97.5% of 244 | 244/244 |
| frame | 61/92² (28 fail, 3 skip) | 97.8% of 91 | 90/91 |

¹ sparq's expand lane SKIPS all NegativeEvaluationTests (error-code completeness
unverified — an honest skip, never a counted pass); peers run them. Most of the 109.
² **Different oracle — not 1:1 comparable.** sparq's compact/frame counts assert
round-trip RDF losslessness, not the suite's structural expected-document assertion
the peers report; sparq has no native document-level Compaction/Framing algorithm yet
(the writer compacts RDF). Denominators also differ per source (older suite snapshots
for both peers) — compare within a row's own denominator only.

**Conformance gaps (honest):** (a) toRdf 32 fails vs Titanium's 454/456 — the largest
same-oracle deficit; (b) expand negatives unverified (109 skips vs peers ~100%);
(c) no native document-level compact/frame (blocks a same-oracle comparison at all).

## 2. Throughput protocol

`bench/jsonld/run.sh --gather` (peers gather-only, never committed deps): for every
(fixture × op), each engine's OUTPUT passes the equality gate — expand via the
conformance deep-equality comparator, flatten/toRdf via canonical-dataset equality,
compact via round-trip losslessness — **before** any timing row is emitted; failures
are recorded exclusions, never silent drops. Fixtures: WebDataCommons-shaped
schema.org docs + a ~100-term context-heavy doc (`bench/jsonld/fixtures/`, synthetic —
real WDC snippets are not license-clean to vendor).

## 3. First read — 2026-07-11, WORK BOX (NON-canonical; ratios only, µs omitted)

All 36 (engine × fixture × op) rows passed the equality gate. Directional read:

| Fixture | op | sparq vs jsonld.js | sparq vs Titanium |
|---|---|---|---|
| wdc-product | expand / flatten / toRdf | ~2.4–4.9× faster | ~12–15× faster |
| people-graph | expand / flatten / toRdf | ~4.3–4.6× faster | ~15–25× faster |
| context-heavy | **expand** | **~5× SLOWER** | ~1.2× faster |
| context-heavy | **flatten** | **~3× SLOWER** | **~0.7× (slower)** |
| context-heavy | toRdf | ~2× faster | ~8× faster |
| all | compact³ | ~1–3× faster | ~3–12× faster |

³ compact compares DIFFERENT pipelines at the same task (sparq: toRdf + RDF-writer
compaction; peers: document-level Compaction Algorithm) — the envelope carries this
caveat on every row.

**GAP (per the performance-dominance mandate):** the context-heavy fixture inverts
the result — sparq's native `expand`/`flatten` fall behind jsonld.js (and flatten
behind Titanium) when the inline `@context` carries ~100 term definitions. Working
hypothesis (UNVERIFIED — profiling first): per-document Context Processing cost
(term-definition creation / `ActiveContext` cloning) dominates and is not amortised,
where jsonld.js caches processed contexts. Fix bead: `sq-hmd7l.42` (P2,
profiling-first). Everywhere else sparq leads on this first read, but the mandate's
bar is order(s)-of-magnitude on EVERY axis — canonical quiet-box re-run required
before any claim.

## 4. Not measured yet (deferred, beaded)

- Canonical quiet-box wave for this suite (ride the epic's canonical-run bead).
- Corpus-scale runs (real WebDataCommons samples, downloaded at gather time,
  git-ignored) + MB/s on large multi-node documents: `sq-hmd7l.43`.
- Remote-context workloads (sparq's loader is deny-by-default; peers cache remote
  contexts — needs a fixture-served loader on both sides to be fair).
