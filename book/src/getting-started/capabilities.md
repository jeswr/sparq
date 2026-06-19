<!-- [OPUS-4.8] sq-h0tr — scaffold page. Distilled from the README "Features" list
(no lorem). The ZK / MPC entries KEEP the research-scaffold / not-externally-audited
caveats from the source — do not relabel them as sound or maliciously secure (privacy-claims-allow: meta-instruction WARNING against relabeling as sound/maliciously-secure, not an achieved-property claim; sq-toze.35)
(privacy-claims gate). The include-wiring bead (sq-im8u) will single-source each
capability description from its skills/<surface>/SKILL.md.
[OPUS-4.8] sq-tfpq / issue #813 — Update row now links SPARQL 1.2 Update alongside 1.1
and a short note documents the 1.2 triple-term delta (no new ops); honest scoping kept
(engine write tests, not a formal conformance run). -->

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
numbers as indicative, not as an audited cryptographic guarantee.

- **Zero-knowledge query proofs** — models proving a query result is correct without revealing the
  data. The v1 verifier is **research-grade and not externally audited**; it provides **no**
  soundness guarantee to a relying party pending external audit. See the
  [security caveat](https://github.com/jeswr/sparq/blob/main/SECURITY.md) and the
  [zk-query-proofs guide](https://github.com/jeswr/sparq/blob/main/skills/zk-query-proofs/SKILL.md).
- **Federated MPC** — models evaluating SPARQL across parties with multi-party computation. It is
  honest-majority semi-honest, **not** maliciously secure. <!-- privacy-claims-allow: NEGATIVE caveat — explicitly denies malicious security (semi-honest only); sq-qhy4 -->
  See
  [SECURITY.md](https://github.com/jeswr/sparq/blob/main/SECURITY.md) and the
  [mpc guide](https://github.com/jeswr/sparq/blob/main/skills/mpc/SKILL.md).

## Interfaces

A [CLI](https://github.com/jeswr/sparq/blob/main/skills/cli/SKILL.md), a
[SPARQL 1.1 Protocol](https://www.w3.org/TR/sparql11-protocol/) + Graph Store Protocol
[HTTP server](https://github.com/jeswr/sparq/blob/main/skills/http-server/SKILL.md), a
[JavaScript / WASM build](https://github.com/jeswr/sparq/blob/main/skills/javascript-wasm/SKILL.md),
and a [Python package](https://github.com/jeswr/sparq/blob/main/skills/python/SKILL.md).
