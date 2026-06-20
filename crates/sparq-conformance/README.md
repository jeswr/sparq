<!-- [OPUS-4.8] sq-4kr5: internal-stub README for a publish=false crate. -->
# sparq-conformance

The **W3C conformance harness** for [sparq](../../README.md): it runs the official
test suites against the engine and reports a per-suite pass/fail/skip scoreboard,
gated in CI by a pass-count ratchet. Three binaries share manifest-walking /
result-comparison machinery:

- `sparq-conformance` — the W3C SPARQL suites (query/update evaluation + syntax).
- `sparq-inference-conformance` — the reasoning suites (RDF Semantics, OWL 2 RL,
  N3, SPARQL entailment regimes) run against `sparq-reason`.
- `sparq-conformance-scoreboard` — the consolidated index of every ratchet
  (SPARQL, inference, W3C SHACL, OGC GeoSPARQL, Solid WAC + ACP, **W3C JSON-LD 1.1
  toRdf + fromRdf + compact**, **SolidLab ODRL Test Suite**).

A crate-local `cargo test` ratchet behind the **opt-in `jsonld-suite`** feature
drives the `w3c/json-ld-api` suite (toRdf + fromRdf + compact); honest divergences +
not-yet-implemented buckets are reported, never inflated. The **compact** lane
(sq-3uos5) parses each `jld:CompactTest` input to RDF, runs the native Compaction
Algorithm against the case `@context`, and requires the compacted document to
re-parse back to the SAME RDF (`reparse(compact(D, ctx)) ≡ D` — lossless
compaction, the same oxjsonld self-reparse oracle toRdf/fromRdf use). The **SolidLab
ODRL Test Suite** ratchet (sq-tmsd6) is crate-local in `sparq-policy` — this
dev-only crate must not depend on it, so only its FLOOR is mirrored here (59/68; see
the SKILL).

> **Internal dev-only harness — not published** (`publish = false`). Test data is
> fetched by `scripts/fetch-conformance.sh` / `scripts/fetch-jsonld-tests.sh` /
> `scripts/fetch-odrl-suite.sh`.

Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
