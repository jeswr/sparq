<!-- [OPUS-4.8] sq-4kr5: internal-stub README for a publish=false crate. -->
# sparq-conformance

The **W3C conformance harness** for [sparq](../../README.md): it runs the official
test suites against the engine and reports a per-suite pass/fail/skip scoreboard,
gated in CI by a pass-count ratchet. Three binaries share manifest-walking /
result-comparison machinery: `sparq-conformance` (W3C SPARQL query/update/syntax),
`sparq-inference-conformance` (RDF Semantics, OWL 2 RL, N3, entailment regimes via
`sparq-reason`), and `sparq-conformance-scoreboard` (consolidated index of every
ratchet — SPARQL, inference, W3C SHACL, OGC GeoSPARQL, Solid WAC + ACP, **W3C
JSON-LD 1.1 toRdf + fromRdf + compact + expand + flatten + frame**, **SolidLab ODRL**).

Crate-local `cargo test` lanes also sit behind **opt-in features** (OFF by default, so the lean `cargo test` never links their heavy deps). **`jsonld-suite`** ratchets the W3C `json-ld-api` (toRdf/fromRdf/**compact**/**expand**/**flatten**, sq-oy1f) + `json-ld-framing` (**frame**, sq-oy1f.19) suites against the NORMATIVE expected docs (MEASURED floors, divergences reported not inflated). **`service-loopback`** (sq-ushvx) is the SERVICE-federation keystone — `service_loopback::LoopbackEndpoint` stands up a REAL `sparq_server::serve` on an ephemeral `127.0.0.1:0` port and drives a federated SERVICE query through the engine's REAL `ureq` transport end-to-end; its egress allowlist is scoped to the bound loopback host (NOT a global disable; host- not port-keyed, see rustdoc), no floor graduated yet. The ODRL ratchet (sq-tmsd6) lives in `sparq-policy`.

> **Internal dev-only harness — not published** (`publish = false`). Test data is
> fetched by `scripts/fetch-conformance.sh`, `fetch-jsonld-tests.sh`,
> `fetch-jsonld-framing-tests.sh`, `fetch-odrl-suite.sh`. Contributing:
> [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
