<!-- [OPUS-4.8] sq-im8u — single-source include wiring. The two research-scaffold
maturity caveats are {{#include}}d verbatim from their canonical skills/*/SKILL.md
`scaffold-caveat` anchors (build-time content injection), so they cannot drift and
their not-yet-audited / not-yet-hardened hedges stay intact at the source (privacy-claims
gate; sq-toze.35 / sq-qhy4). Do not weaken the included hedges or restate the guarantees
inline here.

The capability link-TABLE below and the SPARQL-Update note are deliberately NOT included
from the README's Features list: the README links per-surface guides with REPO-RELATIVE
paths (`skills/<x>/SKILL.md`) — required there by the lychee internal-links gate — and
mdBook rewrites a relative link relative to THIS page's mount, so an included copy would
resolve to a non-existent `getting-started/skills/...` and 404 under GitHub Pages. The
table therefore uses mount-portable ABSOLUTE GitHub URLs; it is a navigation matrix
(mount-point-specific link targets), not duplicated prose. Single-sourcing the table
itself is tracked as a follow-up (would require a README-side portable-link form the
lychee gate does not currently permit).
[OPUS-4.8] sq-tfpq / issue #813 — Update row links SPARQL 1.2 Update alongside 1.1 and a
short note documents the 1.2 triple-term delta (no new ops); honest scoping kept (engine
write tests, not a formal conformance run). -->

# Capabilities at a glance

The engine core is always built; every capability below is an **opt-in** crate or feature that the
core does not depend on, so the core stays lean. Each links the standard it implements and its
usage guide.

| Capability | Standard | Guide |
| --- | --- | --- |
| SPARQL query | [SPARQL 1.1](https://www.w3.org/TR/sparql11-query/) / [1.2](https://www.w3.org/TR/sparql12-query/) | [sparql-query](https://github.com/jeswr/sparq/blob/main/skills/sparql-query/SKILL.md) |
| SPARQL Update | [SPARQL 1.1 Update](https://www.w3.org/TR/sparql11-update/) / [1.2](https://www.w3.org/TR/sparql12-update/) | [sparql-query](https://github.com/jeswr/sparq/blob/main/skills/sparql-query/SKILL.md) |
| RDF parsing & ingest | Turtle / N-Triples / N-Quads / TriG (+ `.gz` / `.bz2` / `.zst`) | [data-formats](https://github.com/jeswr/sparq/blob/main/skills/data-formats/SKILL.md) |
| RDF 1.2 triple terms | [RDF 1.2 Concepts](https://www.w3.org/TR/rdf12-concepts/) | [sparql-query](https://github.com/jeswr/sparq/blob/main/skills/sparql-query/SKILL.md) |
| RDFS / OWL-RL / N3 reasoning | [RDFS](https://www.w3.org/TR/rdf-schema/), [OWL 2 RL](https://www.w3.org/TR/owl2-profiles/#OWL_2_RL), [N3](https://w3c.github.io/N3/spec/) | [inference](https://github.com/jeswr/sparq/blob/main/skills/inference/SKILL.md) |
| SHACL validation | [SHACL](https://www.w3.org/TR/shacl/) | [shacl-validation](https://github.com/jeswr/sparq/blob/main/skills/shacl-validation/SKILL.md) |
| Full-text search | BM25 over RDF literals | [full-text-search](https://github.com/jeswr/sparq/blob/main/skills/full-text-search/SKILL.md) |
| GeoSPARQL | [OGC GeoSPARQL](https://www.ogc.org/standard/geosparql/) | [geosparql](https://github.com/jeswr/sparq/blob/main/skills/geosparql/SKILL.md) |
| Vector & similarity search | embedding nearest-neighbour | [vector-search](https://github.com/jeswr/sparq/blob/main/skills/vector-search/SKILL.md) |
| GenAI / NL retrieval | schema introspection + grounded NL→SPARQL | [genai-retrieval](https://github.com/jeswr/sparq/blob/main/skills/genai-retrieval/SKILL.md) |
| HDT archives | [HDT](https://www.w3.org/submissions/HDT/) | [hdt crate](https://github.com/jeswr/sparq/tree/main/crates/sparq-hdt) |
| RDF stream processing | [RSP-QL](https://www.w3.org/community/rsp/) | [streaming-rsp](https://github.com/jeswr/sparq/blob/main/skills/streaming-rsp/SKILL.md) |
| Solid access control | [Solid WAC](https://solidproject.org/TR/wac) / [ACP](https://solidproject.org/TR/acp) | [solid crate](https://github.com/jeswr/sparq/tree/main/crates/sparq-solid) |
| Dataset canonicalization | [RDFC-1.0](https://www.w3.org/TR/rdf-canon/) | [rdf-canon](https://github.com/jeswr/sparq/blob/main/skills/rdf-canon/SKILL.md) |

## A note on SPARQL Update 1.1 vs 1.2

SPARQL 1.1 Update is fully supported: `INSERT DATA`, `DELETE DATA`, `DELETE`/`INSERT … WHERE`
(with `USING` / `WITH`), `LOAD`, `CLEAR`, `CREATE`, `DROP`, `COPY`, `MOVE`, and `ADD`, with
request-level atomicity for `;`-separated bodies.

[SPARQL 1.2 Update](https://www.w3.org/TR/sparql12-update/) adds **no new operations** over 1.1;
its substantive change is the [RDF 1.2](https://www.w3.org/TR/rdf12-concepts/) triple-term
semantics inside `INSERT DATA` / `DELETE DATA` and `DELETE`/`INSERT … WHERE` — inserting or
deleting a reifying triple (`rdf:reifies <<( s p o )>>`) operates on that triple term itself and
does **not** automatically assert or retract the asserted triple it refers to. sparq handles this:
triple terms are stored and matched as structural object terms, so a reifying triple is added or
removed as an exact term with no coupling to its asserted counterpart. This behaviour is exercised
by the engine's write tests (`crates/sparq-engine/tests/rdfstar_write.rs`); it has not been checked
against a formal SPARQL 1.2 conformance suite.

## Research scaffolds (no security guarantee yet)

Two capabilities are **research scaffolds**. They are honest models of the protocols, but they do
**not** yet provide the cryptographic guarantee a relying party would need. Treat any engineering
numbers as indicative, not as an audited cryptographic guarantee. The two maturity caveats below
are single-sourced verbatim (build-time `{{#include}}`) from their canonical guides, so they
cannot drift from the source of truth — see the
[zk-query-proofs guide](https://github.com/jeswr/sparq/blob/main/skills/zk-query-proofs/SKILL.md),
the [mpc guide](https://github.com/jeswr/sparq/blob/main/skills/mpc/SKILL.md), and
[SECURITY.md](https://github.com/jeswr/sparq/blob/main/SECURITY.md) for the full scope.

**Zero-knowledge query proofs** —

{{#include ../../../skills/zk-query-proofs/SKILL.md:scaffold-caveat}}

**Federated MPC** —

{{#include ../../../skills/mpc/SKILL.md:scaffold-caveat}}

## Interfaces

A [CLI](https://github.com/jeswr/sparq/blob/main/skills/cli/SKILL.md), a
[SPARQL 1.1 Protocol](https://www.w3.org/TR/sparql11-protocol/) + Graph Store Protocol
[HTTP server](https://github.com/jeswr/sparq/blob/main/skills/http-server/SKILL.md), a
[JavaScript / WASM build](https://github.com/jeswr/sparq/blob/main/skills/javascript-wasm/SKILL.md),
and a [Python package](https://github.com/jeswr/sparq/blob/main/skills/python/SKILL.md).
