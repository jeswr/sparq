<!-- [OPUS-4.8] sq-gq28y (issue #1546) re-pinned sq-38lqs [SONNET-4.6] -->
# Vendored: solid-sparql-query query-semantics conformance suite

`manifest.json` + `fixture.nq` here are a **pinned copy** of the upstream conformance suite
for the *Access-Controlled SPARQL Query over a Solid Pod* Editor's Draft. `sparq` runs them
via `crates/sparq-solid/tests/conformance_solid_sparql_query.rs` and passes every case
(the standing "contribute spec tests and pass all of them" directive on `jeswr/sparq#1546`).

| Field | Value |
|-------|-------|
| Upstream repo | <https://github.com/jeswr/solid-sparql-query> |
| Upstream path | `test-suite/query-semantics/` |
| Upstream commit | `5ea9718f13c89fb790a6bb3ada77bffeef926841` |
| Upstream PR | <https://github.com/jeswr/solid-sparql-query/pull/1> |
| Upstream merge date | 2026-07-05 |
| Conformance class | `query-semantics` |
| Cases | 15 (all MUST/MAY for the query-engine seam) |

## Pin note

The commit above is the **merge commit** of upstream PR #1 (merged 2026-07-05).
The file content at the merge commit is byte-for-byte identical to the contribution branch head
(`8d031bb7e1344aeaf9758d27ac0836cdbcf566fc`) — the merge was clean with no fixups applied.
Do NOT hand-edit `manifest.json`/`fixture.nq` in this directory — change them upstream and
re-vendor, so the suite stays a faithful mirror of the spec's own tests.

## Scope

Only the **query-semantics** conformance class is vendored/run here (empty default graph,
union-default-graph opt-in, non-disclosure invariants). The HTTP-**protocol** class (protocol
bindings, Service Description, Update refusal, caching) and the **content-mapping** class
(JSON-LD flatten / no-raw-preserve) are implemented by the Solid HTTP server and the RDF
parser respectively; they are enumerated in the manifest's `outOfClassScenarios`, not run by
this query-engine runner.
