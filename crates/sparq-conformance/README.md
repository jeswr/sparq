<!-- [OPUS-4.8] sq-4kr5: internal-stub README for a publish=false crate. -->
# sparq-conformance

The **W3C conformance harness** for [sparq](../../README.md): it runs the official
test suites against the engine and reports a per-suite pass/fail/skip scoreboard,
gated in CI by a pass-count ratchet. Three binaries share manifest-walking /
result-comparison machinery: `sparq-conformance` (W3C SPARQL query/update/syntax),
`sparq-inference-conformance` (RDF Semantics, OWL 2 RL, N3, entailment regimes via
`sparq-reason`), and `sparq-conformance-scoreboard` (consolidated index of every
ratchet — SPARQL, inference, W3C SHACL, OGC GeoSPARQL, Solid WAC + ACP, **W3C
JSON-LD 1.1 toRdf + fromRdf + compact**, **SolidLab ODRL Test Suite**).

A crate-local `cargo test` ratchet behind the **opt-in `jsonld-suite`** feature
drives the `w3c/json-ld-api` suite (toRdf + fromRdf + compact); honest divergences
are reported, never inflated. The **compact** lane (sq-3uos5) parses each
`jld:CompactTest` input to RDF, runs Compaction against the case `@context`, and
requires lossless self-reparse (`reparse(compact(D, ctx)) ≡ D`). The ODRL Test
Suite ratchet (sq-tmsd6) lives in `sparq-policy`; only its FLOOR is mirrored here.

> **Internal dev-only harness — not published** (`publish = false`). Test data is
> fetched by `scripts/fetch-conformance.sh`, `fetch-jsonld-tests.sh`,
> `fetch-odrl-suite.sh`. Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
