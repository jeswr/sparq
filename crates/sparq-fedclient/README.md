<!-- [OPUS-4.8] sq-inzv: internal-stub README for a publish=false crate; full design lives in research/federation-client-design.md. -->
# sparq-fedclient

A streaming federation **client** over heterogeneous remote RDF sources (full
SPARQL endpoints, bindings-restricted brTPF servers, plain TPF servers, and the
local sparq engine) — the query *consumer* half of federation. It discovers each
source's capability, reuses the `sparq-fedplan` cost-based planner, pushes the most
precise sub-query each source can answer (FedX exclusive groups; an exact
common-variable FILTER-pushdown safety check), and streams results through
non-blocking operators behind a default-deny SSRF egress guard. The full design
(architecture, phased build plan, honest risks, and the load-bearing dependency
boundary that keeps it out of `sparq-core`/`sparq-engine` and the wasm artifact)
lives in [`research/federation-client-design.md`](../../research/federation-client-design.md).

> **Internal crate — not published** to crates.io (`publish = false`). Opt-in and
> OFF by default (`fedclient` / `fedclient-adaptive` cargo features), native-only
> HTTP transport (never enters the wasm bundle). Early/research: no performance
> claim is asserted here — any "better than Comunica" prediction in the design
> record must be validated head-to-head before being stated as fact.

Design: [`research/federation-client-design.md`](../../research/federation-client-design.md).
Planner it reuses: [`skills/federated-planning/SKILL.md`](../../skills/federated-planning/SKILL.md).
Contributing: [`AGENTS.md`](../../AGENTS.md).

## Correctness tests

The client's correctness rests on **result-equivalence** — a federated answer equals local
`sparq-engine` evaluation over the merged data — plus **answer-safety / fail-closed** error
handling on every untrusted-input boundary (a remote endpoint's body, a fragment server's
triples, an endpoint IRI's resolved IP). All tests are gated on the `fedclient` feature; the
default build compiles them to nothing. They run on the REAL path (the canonical answer IS the
engine), not against a mock that bypasses the logic:

* **Result-equivalence (the load-bearing invariant)** — `planner_result_equals_local_eval`
  (materialised single-source = local eval), `streaming_result_equals_phase3` (streamed =
  materialised, for any source-arrival interleaving), `multi_source_union_result_equals_local`
  (per-leaf UNION fan-out = local eval over the merged graph), and
  `adaptive_result_equals_static` (a re-plan changes the plan, never the answer).
* **Wire / error paths** — `srj_parse_correctness` (the SPARQL-Results-JSON decode: every
  term kind plus the malformed / partial / adversarial-body branches all fail closed),
  `wire_pushdown_extra` (the brTPF binding-block binary codec rejects truncated / wrong-magic /
  wrong-version buffers without a panic — the crate is `forbid(unsafe_code)`), and
  `interpreter_error_paths` (a transport failure, a malformed body, and a plan/resolver
  mismatch each surface the right `InterpError`, fail-closed).
* **Discovery + source adapters** — `discovery_error_paths` (the SD / VoID / ASK-probe
  orchestration: malformed-SD propagation, VoID-without-SD, best-effort VoID, the recall-safe
  unknown-version capability) and `source_adapter_error_paths` (the SSRF egress guard refuses a
  host that resolves only to private/internal addresses; a transport / fragment-server error is
  surfaced as `FedError::Transport`; a runaway paginator is fail-stopped by the page cap).
* **The dependency boundary** — `boundary` proves `sparq-core` / `sparq-engine` never gain an
  edge to this crate (the dependency arrow points one way *into* the engine).

## License

[MIT](../../LICENSE).
