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
The scoreboard also surfaces FIVE **`sparq extension`** rows, HONESTLY labelled NOT
standards claims and tallied separately: the `sparq-text` BM25 oracle (sq-ripcg), the
`sparq-rsp` RSP/SRBench oracle (sq-mcb3q), the **RIF-Core** monotone-Horn expressivity
ratchet (sq-rh4gu), the **OWL 2 QL** DL-Lite_R certain-answer oracle (sq-qo1a9), and the
**OWL 2 EL** classification ratchet (sq-pbz04.2.4 — `sparq-reason-el`'s CR1–CR6 classifier,
NOT full OWL 2 EL conformance: CR7–CR9 concrete domains + ABox inconsistency deferred).
Each floor is MEASURED, divergences reported not inflated, mirrored + guarded textually.

Crate-local `cargo test` lanes also sit behind **opt-in features** (OFF by default, so the lean `cargo test` never links their heavy deps). **`jsonld-suite`** ratchets the W3C `json-ld-api` (toRdf/fromRdf/**compact**/**expand**/**flatten**, sq-oy1f) + `json-ld-framing` (**frame**, sq-oy1f.19) suites against the NORMATIVE expected docs (MEASURED floors, divergences reported not inflated). **`service-loopback`** (sq-ushvx) is the SERVICE-federation keystone — `service_loopback::LoopbackEndpoint` stands up a REAL `sparq_server::serve` on an ephemeral `127.0.0.1:0` port and drives a federated SERVICE query through the engine's REAL `ureq` transport end-to-end; its egress allowlist is scoped to the bound loopback host (NOT a global disable; host- not port-keyed, see rustdoc). **`service`** (sq-ddpgx) builds on it to ratchet the W3C `sparql11/service` EVALUATION suite: each `qt:serviceData` block is served by a loopback endpoint, endpoint IRIs are rewritten to the bound URLs, and the federated query runs end-to-end vs the `.srx` oracle (MEASURED floor; `SILENT`-swallow vs non-`SILENT`-propagate tested against a closed port; a variable `SERVICE ?ep` is the one documented Skip (nested non-`SILENT` `SERVICE` now handled via per-endpoint egress config, sq-my8wd.1)). **`http-protocol`** (sq-jaj38) reuses the same loopback server but drives RAW HTTP at the bound port to ratchet the W3C SPARQL 1.1 **Protocol** itself — GET/POST query+update, the `QUERY` method (#1304), `default`/`named-graph-uri` overrides, SRJ/SRX/CSV/TSV negotiation, 200/400/405/415 (MEASURED PASS floor; the 406-less Accept fallback + ASK-in-CSV are documented divergences, NOT summed in). **`federation-descriptors`** (sq-1uuxz) reuses the same loopback server — with the server's `federation-descriptors` flag ON — to ratchet the SPARQL 1.1 **Service Description** (the `GET /sparql` no-query `sd:Service` advertises exactly the formats/languages/versions/features the server genuinely implements — no over-advertising, each result format cross-checked against a real request) + the **Graph Store Protocol** (a GET/PUT/POST/DELETE round-trip on a named graph — indirect `?graph=` + direct `/graphs/<path>` — and the default graph `?default`, verifying store state after each op; 200/201/204/400/404/405/415; the absent-graph 200-empty read is a documented divergence, NOT summed in). The inference-side lanes **`d-entail`** (sq-e5atd), **`rif-core`** (sq-rh4gu), **`ql-experimental`** (sq-qo1a9) and **`el-suite`** (sq-pbz04.2.4 — the OWL 2 EL classifier via `sparq-reason-el/classify_graph`) are likewise opt-in crate-test ratchets (see the `scoreboard` rustdoc). The ODRL ratchet (sq-tmsd6) lives in `sparq-policy`.

> **Internal dev-only harness — not published** (`publish = false`). Test data is
> fetched by `scripts/fetch-conformance.sh`, `fetch-jsonld-tests.sh`,
> `fetch-jsonld-framing-tests.sh`, `fetch-odrl-suite.sh`. Contributing:
> [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
