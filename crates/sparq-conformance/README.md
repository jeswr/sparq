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
  toRdf + fromRdf**).

A crate-local `cargo test` ratchet behind the **opt-in `jsonld-suite`** feature
(OFF by default) drives the official `w3c/json-ld-api` suite (toRdf + fromRdf) with
rising pass-floors; honest divergences + the not-yet-implemented Compaction/Framing
buckets are reported, never inflated. See the data-formats SKILL to run it.

> **Internal dev-only harness — not published** (`publish = false`). Test data is
> fetched by `scripts/fetch-conformance.sh` / `scripts/fetch-jsonld-tests.sh`.

Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
