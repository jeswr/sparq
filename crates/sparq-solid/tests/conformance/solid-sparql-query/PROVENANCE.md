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

<!-- [OPUS-4.8] sq-tlzvw (issue #1654): vendored spec text + machine-readable companion -->
## Spec text (`spec.html`)

`spec.html` is a **pinned, verbatim copy** of the upstream `index.html` (the ReSpec
Editor's Draft) at the same upstream commit `5ea9718…` as the suite above — byte-identical
to the upstream `HEAD` at vendoring time. It is present so the companion's quotes are
fidelity-checkable against spec text **in this tree**, offline. Do NOT hand-edit it;
re-vendor from upstream on any spec change and re-run the fidelity + SHACL checks below.

## Machine-readable companion (`spec.statements.ttl`)

`spec.statements.ttl` is an **additive** normative-statement companion for the spec (the
spec text is untouched), responding to `jeswr/sparq#1654`. It reuses the W3C `spec:`
vocabulary (`http://www.w3.org/ns/spec#`) — the same one the Solid Protocol and Conformance
Test Harness use — with an `sc:` extension for the testability spine and the coverage links.
Each of the **86** RFC 2119 obligations of the spec is one `spec:Requirement` carrying:

- a **verbatim, character-for-character quote** (`spec:statement`) checked against `spec.html`;
- its RFC 2119 level (`spec:requirementLevel`) and actor binding (`spec:requirementSubject`);
- a **testability tag** (`sc:testabilityTag`) — `sc:Enforceable` (E) versus the
  audit-accountable `sc:AuditInternal` (A-int) / `sc:AuditExistential` (A-exist) /
  `sc:Permission` (P), so the companion never overclaims server enforceability;
- a resolvable section anchor (`sc:section` → `spec.html#…`);
- a link to the conformance vector that tests it (`sc:testedBy` → `manifest.json#<case-id>`)
  **or** an honest `sc:testGap` (both, where coverage is partial).

Tag distribution: **E 37 · A-int 11 · A-exist 16 · P 22** (levels: MUST 38 · MUST NOT 25 ·
SHOULD 10 · MAY 13). 25 requirements link at least one conformance vector; 69 carry an
honest gap note (an out-of-class scenario, an audit-only property, or a permission).

### The `sc:` namespace is provisional

The `sc:` namespace IRI (`https://w3id.org/spec-companion#`) follows #1654's `sc:testGap`
term and the maintainer's w3id convention, but the canonical `jeswr/spec-companion`
repository was **not fetchable** at authoring time, so the exact IRI and term shapes should
be reconciled with that vocabulary when it is available. The extension terms are
self-described in `spec-companion.shapes.ttl`.

### Validation (spec-of-specs guardrail)

`spec-companion.shapes.ttl` is the SHACL guardrail: every requirement must carry exactly one
statement / level / subject / tag / section anchor and at least one of `sc:testedBy` /
`sc:testGap`. Validate with sparq's own SHACL engine:

```sh
cargo run -p sparq-shacl --example validate -- \
  spec.statements.ttl spec-companion.shapes.ttl
```

Exit 0 / `Conforms` == validator-0-errors (verified for the current companion; a
spot-check confirms the shapes are non-vacuous — dropping any required field fails).

### Count versus the #1654 inventory

The #1654 proposal cites an **88**-statement inventory; the tree yields **86** distinct
RFC 2119 obligations (counted per keyword occurrence). The 2-statement gap is accounted for
by two non-normative keyword tokens the companion deliberately excludes: the SPARQL
`OPTIONAL` keyword in §Query cost and denial of service ("OPTIONAL-heavy queries") and the
informative back-reference in §The read-oracle class ("makes their mitigation a SHOULD",
which points at invariant 6's SHOULD rather than stating a new obligation). Per the standing
rule, the **tree wins**; the discrepancy is recorded on #1654.
