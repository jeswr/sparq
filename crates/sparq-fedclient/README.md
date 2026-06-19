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

Capability discovery prefers a published Service Description and falls back to a FedX-style
`ASK { ?s ?p ?o }` reachability probe; the probe's SPARQL-Results-JSON answer is parsed
**strictly** — the `boolean` value must be an exact lowercase JSON `true`/`false` literal at a
value boundary, so a junk-suffixed token (`trueish` / `falsex`) is rejected, not silently read
as a boolean (sq-2gfe).

Correctness suite under `tests/` (gated on the `fedclient` feature; the default build compiles it to nothing) — run on the REAL path, with local `sparq-engine` evaluation as the canonical answer: result-equivalence (planner / streaming / multi-source UNION / adaptive vs. static), fail-closed wire & error paths (SRJ decode, brTPF binary codec, interpreter), discovery + source-adapter error paths (incl. the SSRF egress guard), and the one-way `sparq-core`/`sparq-engine` dependency boundary.

## License

[MIT](../../LICENSE).
