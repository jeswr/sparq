<!-- [FABLE-5] sq-01xlp decision record (follow-up from
research/gap-shacl-wasm-2026-07.md, sq-i858h / epic sq-hmd7l). ALL timings here
are NON-CANONICAL first reads from the shared work box, and NATIVE (x86_64) —
wasm-representative by dependency graph, not a wasm gather. The wasm-runtime
column rides bench/shacl-wasm/ FEATURES=stateful on the canonical wave. -->

# sparq-shacl-wasm: pre-parsed/stateful validation — measurement + decision (2026-07-17)

**Status:** decided + implemented. **Bead:** sq-01xlp (gh-2481).
**Decision:** **(b)** — an opt-in pre-parsed `ParsedGraph` handle on
`sparq-shacl-wasm` (non-default cargo feature `stateful`), NOT (a) blessing the
lean bundle's `Store.validate`.

## The question

The showcase `Validator` is stateless one-shot: every call re-parses both
documents. `research/gap-shacl-wasm-2026-07.md` showed the one-shot beating the
peer's parse-free steady-state at micro scale, but left open whether scale-tier
repeat validation (a large fixed data graph re-checked as shapes are edited, or
one shapes graph over many documents) needs a parse-free path. Options as filed:
(a) document the lean `sparq-wasm` bundle's `Store.validate` as THE stateful
path, or (b) add a parsed-graph handle here, opt-in.

## Why (a) was never available — a correction

The gap record cited the lean bundle's `Store.validate` (its `shacl` feature) as
the pre-parsed shape. Reading `crates/sparq-wasm/src/shacl.rs`: **it is also a
stateless one-shot** — it parses both string arguments per call and (per its own
docs) "does not consult the receiver's stored triples". Neither wasm surface had
a pre-parsed validate. So (a) reduces to documenting something that does not
exist; the real choice was (b) or nothing. A store-consulting validate on the
lean bundle is a separate question (follow-up filed from gh-2481).

## Measurement (drives the "or nothing" call)

No wasm toolchain on this box, so the split was measured **natively with the
exact wasm dependency graph** — `sparq-core`/`sparq-shacl` with
`default-features = false` (no rayon, single-threaded parse; the same code
`wasm32-unknown-unknown` compiles). Corpus: the vendored hand-countable
micro-ABox (`bench/shacl-wasm/data/abox.ttl`) replicated under per-replica IRI
suffixes (violations scale linearly — counts at every scale matched the
`expected.tsv` constants ×scale, so the runs were non-vacuous). Best-of-3,
x86_64 shared work box, 2026-07-17. **ADVISORY / directional only — native, not
wasm; not a quiet box. Do not cite these numbers as wasm performance.**

Data-parse share of the one-shot total (parse data + parse shapes + validate),
per committed workload:

| corpus | triples | data-parse share of one-shot |
|---|---:|---:|
| micro (×1) | 86 | 32–57% |
| ×100 | 8 600 | 72–88% |
| ×1000 (scale-tier proxy) | 86 000 | **67–88%** |

At the scale-tier proxy the one-shot spends ~57 ms parsing the data document
against ~8–28 ms validating (workload-dependent); shapes-parse is a constant
~0.1 ms. I.e. repeat validation through the stateless surface pays a
**~3–8× per-call penalty** over a parse-free path, and the penalty grows with
corpus size. At micro scale the one-shot is fine (the gap record's finding
stands). Conclusion: the stateful path is worth having, opt-in.

## The decision, shape, and guardrails

- **`stateful` cargo feature** (non-default, no new deps) on `sparq-shacl-wasm`
  adds `ParsedGraph`: `parse(text, format)` once, then `validate` /
  `validateTurtle` / `validateText` / `conforms` against another handle —
  the full `Validator` report surface at validate-only cost. Handles hold wasm
  linear memory (`free()` from JS when done).
- **Default surface unchanged**: the showcase artifact and its deterministic
  bundle-bytes record (gap record §bundle-bytes) are untouched; CI
  builds/clippys/wasm-pack-tests every feature state (default, `shacl-af`,
  `stateful`).
- **Equivalence obligation**: native + wasm tests assert the pre-parsed report
  is byte-identical to the one-shot's over the same documents, and repeat
  validation is stable.
- **Measurement seam**: `bench/shacl-wasm/run.sh FEATURES=stateful` builds the
  feature artifact and the harness records the symmetric advisory column
  `sparq_validate_only_us` (counts cross-checked against the one-shot every
  iteration; column absent — never 0 — on a default build). The wasm-runtime
  confirmation of the native split above rides the scale-tier + quiet-box
  canonical wave (sq-hmd7l.39/40) through this seam.
