<!-- [OPUS-4.8] sq-4kr5: internal-stub README for a publish=false crate. -->
# sparq-conformance

The **W3C conformance harness** for [sparq](../../README.md): it runs the official
test suites against the engine and reports a per-suite pass/fail/skip scoreboard,
gated in CI by a pass-count ratchet. Three binaries share manifest-walking /
result-comparison machinery: `sparq-conformance` (W3C SPARQL query/update/syntax),
`sparq-inference-conformance` (RDF Semantics, OWL 2 RL, N3, entailment regimes via
`sparq-reason`), and `sparq-conformance-scoreboard` (consolidated index of every
ratchet — SPARQL, inference, W3C SHACL, OGC GeoSPARQL, Solid WAC + ACP, **W3C
JSON-LD 1.1 toRdf + fromRdf + compact + frame**, **SolidLab ODRL Test Suite**).

A crate-local `cargo test` ratchet behind the **opt-in `jsonld-suite`** feature
drives the `w3c/json-ld-api` suite (toRdf + fromRdf + **compact**, lossless
self-reparse `reparse(compact(D,ctx)) ≡ D`; floor raised 163→186 by sq-oy1f.16) and
the SEPARATE `w3c/json-ld-framing` suite (**frame**, sq-oy1f.19): each `jld:FrameTest`
EXPANDED input is framed via the native Framing Algorithm and compared by
RDF-equivalence to the suite's NORMATIVE expected output (framing is a SELECT+RESHAPE,
so the oracle anchors on `expected`, not the input). Honest divergences are reported,
never inflated. The ODRL ratchet (sq-tmsd6) lives in `sparq-policy`; only its FLOOR is
mirrored here.

> **Internal dev-only harness — not published** (`publish = false`). Test data is
> fetched by `scripts/fetch-conformance.sh`, `fetch-jsonld-tests.sh`,
> `fetch-jsonld-framing-tests.sh`, `fetch-odrl-suite.sh`. Contributing:
> [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
