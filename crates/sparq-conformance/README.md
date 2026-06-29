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
The scoreboard also surfaces TWO **`sparq extension`** rows, HONESTLY labelled NOT
standards claims and tallied separately: the `sparq-text` BM25 differential oracle
(sq-ripcg; no normative full-text-over-RDF / BM25 suite exists) and the `sparq-rsp`
RSP expressivity / SRBench correctness oracle (sq-mcb3q; no normative
RDF-Stream-Processing / RSP conformance suite exists — RSP-QL is a W3C-community
spec, SRBench a benchmark). Each runner lives crate-local (no dep edge here), its
floor mirrored + guarded textually.

Crate-local `cargo test` lanes also sit behind **opt-in features** (OFF by default, so the lean `cargo test` never links their heavy deps). **`jsonld-suite`** ratchets the W3C `json-ld-api` (toRdf/fromRdf/**compact**/**expand**/**flatten**, sq-oy1f) + `json-ld-framing` (**frame**, sq-oy1f.19) suites against the NORMATIVE expected docs (MEASURED floors, divergences reported not inflated). **`service-loopback`** (sq-ushvx) is the SERVICE-federation keystone — `service_loopback::LoopbackEndpoint` stands up a REAL `sparq_server::serve` on an ephemeral `127.0.0.1:0` port and drives a federated SERVICE query through the engine's REAL `ureq` transport end-to-end; its egress allowlist is scoped to the bound loopback host (NOT a global disable; host- not port-keyed, see rustdoc). **`service`** (sq-ddpgx) builds on it to ratchet the W3C `sparql11/service` EVALUATION suite: each `qt:serviceData` block is served by a loopback endpoint, endpoint IRIs are rewritten to the bound URLs, and the federated query runs end-to-end vs the `.srx` oracle (MEASURED floor; `SILENT`-swallow vs non-`SILENT`-propagate tested against a closed port; a variable `SERVICE ?ep` + a nested non-`SILENT` `SERVICE` are documented Skips). **`http-protocol`** (sq-jaj38) reuses the same loopback server but drives RAW HTTP at the bound port to ratchet the W3C SPARQL 1.1 **Protocol** itself — GET/POST query+update, the `QUERY` method (#1304), `default`/`named-graph-uri` overrides, SRJ/SRX/CSV/TSV negotiation, 200/400/405/415 (MEASURED PASS floor; the 406-less Accept fallback + ASK-in-CSV are documented divergences, NOT summed in). The ODRL ratchet (sq-tmsd6) lives in `sparq-policy`.

> **Internal dev-only harness — not published** (`publish = false`). Test data is
> fetched by `scripts/fetch-conformance.sh`, `fetch-jsonld-tests.sh`,
> `fetch-jsonld-framing-tests.sh`, `fetch-odrl-suite.sh`. Contributing:
> [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
