<!-- [FABLE-5] sq-01xlp decision record (follow-up from
research/gap-shacl-wasm-2026-07.md, sq-i858h / epic sq-hmd7l). The measurement
behind this record was a NON-CANONICAL first read on the shared work box, and
NATIVE (x86_64) — wasm-representative by dependency graph, not a wasm gather.
Per repo policy (no hard-coded perf numbers in markdown; work-box timings are
non-canonical) only the qualitative shape is recorded here; quantitative
evidence is regenerated via bench/shacl-wasm/run.sh (git-ignored envelope).
The wasm-runtime column rides bench/shacl-wasm/ FEATURES=stateful on the
canonical wave. -->

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
wasm; not a quiet box; NON-canonical.**

The qualitative shape (concrete percentages/timings deliberately not committed
— repo policy forbids hard-coded perf numbers in markdown, and work-box
timings are non-canonical): at micro scale, validation is a comparable share
of the one-shot total (parse data + parse shapes + validate), so the one-shot
is fine and the gap record's finding stands. As the corpus is replicated
toward the scale-tier proxy, parsing the data document comes to dominate the
one-shot total while shapes-parse stays negligible — i.e. repeat validation
through the stateless surface pays a multiple of the parse-free per-call cost,
and the penalty grows with corpus size. Conclusion: the stateful path is worth
having, opt-in. For the quantitative split, regenerate the envelope with
`bench/shacl-wasm/run.sh` (output under the git-ignored
`bench/shacl-wasm/results/`); the canonical quantitative record is the pending
quiet-box wasm gather (sq-hmd7l.39/40) via the measurement seam below.

## The decision, shape, and guardrails

- **`stateful` cargo feature** (non-default, no new deps) on `sparq-shacl-wasm`
  adds `ParsedGraph`: `parse(text, format)` once, then `validate` /
  `validateTurtle` / `validateText` / `conforms` against another handle —
  the full `Validator` report surface at validate-only cost. Handles hold wasm
  linear memory (`free()` from JS when done).
- **Default surface unchanged**: the showcase artifact and its deterministic
  bundle-bytes record (gap record §bundle-bytes) are untouched; CI
  builds/clippys/wasm-pack-tests every feature state (default, `shacl-af`,
  `stateful`, and the combined `shacl-af`+`stateful` via `--all-features`).
- **Equivalence obligation**: native + wasm tests assert the pre-parsed report
  is byte-identical to the one-shot's over the same documents, and repeat
  validation is stable.
- **Measurement seam**: `bench/shacl-wasm/run.sh FEATURES=stateful` builds the
  feature artifact and the harness records the symmetric advisory column
  `sparq_validate_only_us` (counts cross-checked against the one-shot every
  iteration; column absent — never 0 — on a default build). The wasm-runtime
  confirmation of the native split above rides the scale-tier + quiet-box
  canonical wave (sq-hmd7l.39/40) through this seam.
