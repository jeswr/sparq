<!-- [OPUS-4.8] sq-gq28y (issue #1546) -->
# Vendored: solid-sparql-query query-semantics conformance suite

`manifest.json` + `fixture.nq` here are a **pinned copy** of the upstream conformance suite
for the *Access-Controlled SPARQL Query over a Solid Pod* Editor's Draft. `sparq` runs them
via `crates/sparq-solid/tests/conformance_solid_sparql_query.rs` and passes every case
(the standing "contribute spec tests and pass all of them" directive on `jeswr/sparq#1546`).

| Field | Value |
|-------|-------|
| Upstream repo | <https://github.com/jeswr/solid-sparql-query> |
| Upstream path | `test-suite/query-semantics/` |
| Upstream commit | `8d031bb7e1344aeaf9758d27ac0836cdbcf566fc` |
| Upstream PR | <https://github.com/jeswr/solid-sparql-query/pull/1> |
| Conformance class | `query-semantics` |
| Cases | 15 (all MUST/MAY for the query-engine seam) |

## Pin note

The commit above is the head of the contribution branch
(`sparq-agent/query-semantics-conformance-suite`), pending merge of upstream PR #1. Once that
PR merges, re-vendor from the merge commit and update the row above (the file content is not
expected to change on merge). Do NOT hand-edit `manifest.json`/`fixture.nq` in this directory
— change them upstream and re-vendor, so the suite stays a faithful mirror of the spec's own
tests.

## Scope

Only the **query-semantics** conformance class is vendored/run here (empty default graph,
union-default-graph opt-in, non-disclosure invariants). The HTTP-**protocol** class (protocol
bindings, Service Description, Update refusal, caching) and the **content-mapping** class
(JSON-LD flatten / no-raw-preserve) are implemented by the Solid HTTP server and the RDF
parser respectively; they are enumerated in the manifest's `outOfClassScenarios`, not run by
this query-engine runner.
