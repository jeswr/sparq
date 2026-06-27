# SHACL 1.2 conformance — gap map (vendored W3C suite)

**Status: gap analysis + ratchet baseline. Date 2026-06-27.** Epic `sq-waf9o`
([EPIC] Full W3C SHACL 1.2 conformance). This record was produced by `sq-6glcr`,
which makes CI gate the **full** vendored SHACL-1.2 test tree (not just
`core/node/`) at a calibrated ratchet floor. It is the honest, sourced map of
exactly what the `sparq-shacl` crate does and does not yet pass, so the
follow-on feature waves are measurable rather than guesswork.

The vendored suite is the W3C `w3c/data-shapes` repo pinned at
`b6e73695d6196f33d7ce3ba47094a10fbc298e65` (`crates/sparq-shacl/fetch-shacl-tests.sh`),
under `crates/sparq-shacl/tests/shacl/data-shapes/shacl12-test-suite/tests/`.

All counts below are **measured** by running the harnesses in-worktree in both
feature states (default = SHACL Core + SHACL-SPARQL; `shacl-af` = SHACL-AF
`sh:rule` / `sh:expression` / `sh:nodeByExpression`). No figure is estimated.

---

## 1. What is gated now

Three manifest-driven runners walk the 1.2 tree, each asserting a **ratchet**
(pass must not drop; the gap must not grow) using the same strict report
comparison as `w3c_core.rs` — `sh:conforms` must match and the
`sh:ValidationResult` multisets must correspond 1:1 on `sh:focusNode` /
`sh:resultPath` / `sh:value` / `sh:resultSeverity` /
`sh:sourceConstraintComponent` / `sh:sourceShape` / `sh:resultMessage`
(blank-node-tolerant, via backtracking bipartite matching).

| Runner (`crates/sparq-shacl/tests/`) | Manifest tree | Entries | PASS (default / `shacl-af`) | Gap |
| --- | --- | --- | --- | --- |
| `w3c_core_full_shacl12.rs` | `core/manifest.ttl` (all 7 categories) | 137 | **129 / 130** | 7 FAIL (+ 1 SKIP default) |
| `w3c_sparql_shacl12.rs` | `sparql/manifest.ttl` | 24 | **17 / 17** | 0 FAIL + 7 expected-failure (sq-mue75 closed the 3 pre-binding FAILs) |
| `w3c_node_expr.rs` + `w3c_node_expr_constraints.rs` | `node-expr/` | 65 | **— / 62 + 1 xfail** | the xfail is the harness `sht:scope-*` var entry (sq-mue75 drives the REAL `eval_node_expression`; `shacl-af`) |

The two runners that gate `core` and `sparql` are NOT feature-gated as a whole —
they run in both states. The `shacl-af`-only delta in `core` is one entry
(`nodeByExpression-001`, a SKIP by default → PASS under `shacl-af`).

### Per-category core scoreboard (`shacl-af` ON)

| category | pass | fail | skip |
| --- | ---: | ---: | ---: |
| complex | 2 | 0 | 0 |
| misc | 5 | 5 | 0 |
| node | 45 | 0 | 0 |
| path | 11 | 0 | 0 |
| property | 43 | 13 | 0 |
| targets | 7 | 3 | 0 |
| validation-reports | 2 | 1 | 0 |
| **TOTAL** | **115** | **22** | **0** |

The harnesses do **not** assert all-pass — that would be a false claim of full
SHACL-1.2 conformance. They assert a two-sided ratchet so a regression in either
direction (a passing entry breaking, or a previously-correct entry flipping)
reds CI, while the documented gap entries below stay counted-not-asserted.
Closing a gap moves a FAIL → PASS and bumps the floor in the same commit.

---

## 2. Gap clusters — core (22 FAIL)

Each cluster is a coherent SHACL-1.2 feature the Core validator does not yet
implement; the entry produces a structurally-wrong report (usually
`conforms=true` because an unrecognised constraint predicate is silently
ignored, or a wrong result count / message). Mapped to the implementation beads
under the epic.

| # | Cluster | Entries | Predicate(s) | Bead |
| --- | --- | --- | --- | --- |
| 1 | Path-list comparands of `sh:disjoint` / `sh:equals` | `property/disjoint-002`, `property/equals-002` | comparand is a property-path list, not a single IRI | `sq-sx15d` |
| 2 | Path-list comparands of `sh:lessThan` / `sh:lessThanOrEquals` | `property/lessThan-003`, `property/lessThanOrEquals-002` | same, ordered comparison | `sq-sx15d` |
| 3 | Disjunctive `sh:class` (ClassIn) | `property/class-002` | `sh:class ( C1 C2 )` list + `rdfs:subClassOf` | `sq-sx15d` |
| 4 | `sh:subsetOf` constraint | `property/subsetOf-001`, `property/subsetOf-002` | `sh:SubsetOfConstraintComponent` | `sq-sx15d` |
| 5 | `sh:someValue` constraint | `property/someValue-001` | `sh:SomeValueConstraintComponent` | `sq-sx15d` |
| 6 | `sh:singleLine` constraint | `property/singleLine-001` | `sh:SingleLineConstraintComponent` | `sq-sx15d` |
| 7 | `sh:rootClass` constraint | `property/rootClass-001` | `sh:RootClassConstraintComponent` | `sq-sx15d` |
| 8 | Severity-threshold conformance | `misc/severity-003`, `misc/severity-004`, `misc/severity-005`, `validation-reports/conformance-disallows-001` | `sh:Debug` / `sh:Info` severities + `sh:conformanceDisallows` (a result below the disallowed threshold still `conforms`) | `sq-sx15d` |
| 9 | `sh:targetWhere` target | `targets/targetWhere-001` | SPARQL-ish target selector | `sq-rnkdh` |
| 10 | `sh:shape` implicit / `sh:ShapeClass` target | `targets/shape-001`, `targets/targetClassImplicit-002` | `sh:shape` value-target + `sh:ShapeClass` implicit class target | `sq-rnkdh` |
| 11 | `sh:reifierShape` constraint | `property/reifierShape-001`, `property/reifierShape-002` | `sh:ReifierShapeConstraintComponent` (RDF-1.2 reifiers) | `sq-0mjfd` |
| 12 | RDF-1.2 reified-annotation parsing | `misc/deactivated-003`, `misc/message-002` | the `{\| sh:deactivated true \|}` / `{\| sh:message ... \|}` triple-term annotation syntax on a constraint | `sq-0mjfd` |
| 13 | `sh:uniqueLang` over `rdf:dirLangString` | `property/uniqueLang-003` | direction-tagged language strings | `sq-0mjfd` |

Clusters 1–8 (eight beads' worth of "value constraints", 13 entries) are `sq-sx15d`;
9–10 (targets, 3 entries) are `sq-rnkdh`; 11–13 (the RDF-1.2 draft/hard zone, 6
entries) are `sq-0mjfd`.

---

## 3. Gap clusters — SPARQL (7 FAIL + 7 expected-failure)

The 1.2 SPARQL suite has 24 `sht:Validate` entries; 10 pass. Of the 14
non-passing, **7 carry `mf:result sht:Failure`** — they declare a SPARQL
constraint a conformant processor MUST **reject** (re-binding a pre-bound
variable, or an unsupported `MINUS` / `SERVICE`). The crate has no rejection /
failure channel — `validate` always returns a report — so the runner records
those as a distinct **ExpectedFailure** outcome (counted in the gap, not as
PASS) and asserts there are exactly 7 of them.

| # | Cluster | Entries | Why it fails | Bead |
| --- | --- | --- | --- | --- |
| 14 | SPARQL constraint result detail | `sparql/node/sparql-001`, `sparql/property/property-select-001` | `sh:select` constraint result message / count detail | `sq-rnkdh` |
| 15 | `sh:sparqlExpr` value-expression constraint | `sparql/property/property-sparqlExpr-001` | SPARQL-expression-valued property constraint | `sq-rnkdh` |
| 16 | SPARQL `sh:target` (targetNode-select) | `sparql/targets/targetNode-select-001` | SPARQL-based target | `sq-rnkdh` |
| 17 | Pre-binding VALUES propagation — **DONE (sq-mue75)** | `sparql/pre-binding/pre-binding-002`, `pre-binding-005`, `pre-binding-007` | the pre-binding algebra-rewrite (UNION / sibling / sub-SELECT scope) — now PASS | `sq-mue75` |
| 18 | Pre-binding **rejection** channel (`sht:Failure`) | `sparql/pre-binding/pre-binding-006`, `unsupported-sparql-001..006` (7) | a query that re-binds a pre-bound variable, or uses unsupported `MINUS`/`SERVICE`, MUST be rejected — the crate has no failure channel | `sq-0mjfd` |

### Per-category SPARQL scoreboard

| category | pass | fail | xfail | skip |
| --- | ---: | ---: | ---: | ---: |
| component | 3 | 0 | 0 | 0 |
| node | 4 | 0 | 0 | 0 |
| pre-binding | 6 | 0 | 7 | 0 |
| property | 3 | 0 | 0 | 0 |
| targets | 1 | 0 | 0 | 0 |
| **TOTAL** | **17** | **0** | **7** | **0** |

(Post-sq-mue75 / sq-rnkdh: the only remaining SPARQL gap is the 7 `sht:Failure`
expected-rejection entries — the rejection channel is unbuilt, cluster 18 / `sq-0mjfd`.)

---

## 4. The 16 clusters and the implementation beads

The 13 core + 3 SPARQL = **16 clusters** above are the buildable backlog,
grouped into four implementation beads under epic `sq-waf9o`:

- **`sq-sx15d`** — SHACL 1.2 core value constraints (clusters 1–8, 13 entries):
  path-comparands, disjunctive `sh:class`, `sh:subsetOf` / `sh:someValue` /
  `sh:singleLine` / `sh:rootClass`, and severity-threshold conformance.
- **`sq-rnkdh`** — SHACL 1.2 targets + SPARQL constraint extras (clusters 9–10,
  14–16, 6 entries): `sh:targetWhere` / `sh:shape` / `sh:ShapeClass`, SPARQL
  targets, SPARQL constraint result detail, and `sh:select` / `sh:sparqlExpr`.
- **`sq-mue75`** — SHACL-SPARQL pre-binding VALUES propagation (cluster 17, 3
  entries) + node-expr harness fidelity + `sh:sourceConstraint`.
- **`sq-0mjfd`** — the draft / hard zone (clusters 11–13, 18; 6 core + 7
  SPARQL): RDF-1.2 reified-annotation parsing, `sh:reifierShape`,
  `rdf:dirLangString` `uniqueLang`, and the `sht:Failure` pre-binding **rejection
  channel**.

### Genuinely-draft zones (do not over-invest until the spec settles)

- **RDF-1.2 triple-term annotation parsing** (`{\| ... \|}` reifier syntax) — the
  Turtle parser must surface the reified triple term so `sh:deactivated` /
  `sh:message` attached to a constraint are honoured. This is RDF-1.2 (still
  CR-stage) surface, not SHACL-1.2 proper; `sq-0mjfd` is `P3` for that reason.
- **The 7 `sht:Failure` tests** assert that a processor *rejects* certain
  queries. SHACL's failure semantics (what a "failure" report looks like, and
  when a constraint MUST fail vs MAY warn) are the least-settled part of the
  1.2 SPARQL chapter. Building a fail-closed rejection channel is `sq-0mjfd`;
  until then the harness counts these as a known, asserted-exactly gap (7).

---

## 5. Why a ratchet, not all-pass

The Core validator is correct on the constraints it implements (the 115/137
core + 10/24 SPARQL passing entries are compared strictly — a wrong report
there is a FAIL, never laundered into a SKIP). The remaining entries exercise
SHACL-1.2 features that simply are not built yet. Asserting all-pass would force
either a false green or disabling real comparison; the two-sided ratchet keeps
the gate honest *and* regression-proof. When a bead lands, its cluster's entries
move FAIL → PASS, the per-runner floor is bumped in the same commit, and this
table shrinks.
