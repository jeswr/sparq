<!-- [OPUS-4.8] sq-4kr5: internal-stub README for a publish=false crate. -->
# sparq-conformance

The **W3C conformance harness** for [sparq](../../README.md): it runs the
official test suites against the engine and reports a per-suite
pass/fail/skip scoreboard, gated in CI by a pass-count ratchet. Three binaries
sit on shared manifest-walking / result-comparison machinery:

- `sparq-conformance` — the W3C SPARQL suites (query/update evaluation + syntax).
- `sparq-inference-conformance` — the reasoning suites (RDF Semantics, OWL 2 RL,
  N3, SPARQL entailment regimes) run against `sparq-reason`.
- `sparq-conformance-scoreboard` — the consolidated index of every conformance
  ratchet across the workspace (SPARQL, inference, W3C SHACL, OGC GeoSPARQL,
  Solid WAC + ACP, **W3C JSON-LD 1.1 toRdf + fromRdf**).

A crate-local `cargo test` ratchet also lives here behind the **opt-in
`jsonld-suite`** feature (OFF by default — forwards to `sparq-core/jsonld` +
`sparq-engine/serialize-rdf`; when OFF it compiles to a self-skip so the default
build links no JSON-LD code): `tests/jsonld_suite.rs` drives the official
`w3c/json-ld-api` suite — **toRdf** through the real oxjsonld parse path,
**fromRdf** through the native writer (re-parse round-trip) — with pinned
pass-count floors. It is **not** 100% conformant (honest divergences + the
not-yet-implemented Compaction/Framing buckets are reported, never inflated; the
floors only RISE). Run with `scripts/fetch-jsonld-tests.sh` then
`cargo test -p sparq-conformance --features jsonld-suite --test jsonld_suite`.

Why it exists: it is the regression oracle that turns "is sparq spec-conformant?"
into a number CI can ratchet upward and never let slip.

> **Internal dev-only harness — not published** to crates.io
> (`publish = false`). Test data is fetched by `scripts/fetch-conformance.sh`
> (SPARQL) / `scripts/fetch-jsonld-tests.sh` (JSON-LD).

Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
