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
  Solid WAC + ACP).

Why it exists: it is the regression oracle that turns "is sparq spec-conformant?"
into a number CI can ratchet upward and never let slip.

> **Internal dev-only harness — not published** to crates.io
> (`publish = false`). Test data is fetched by `scripts/fetch-conformance.sh`.

Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
