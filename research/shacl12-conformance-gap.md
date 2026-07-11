# SHACL 1.2 conformance — gap map (vendored W3C suite)

**Status: gap analysis + ratchet baseline. Date 2026-06-27.** Epic `sq-waf9o`
([EPIC] Full W3C SHACL 1.2 conformance). This record was produced by `sq-6glcr`,
which makes CI gate the **full** vendored SHACL-1.2 test tree (not just
`core/node/`) at a calibrated ratchet floor. It is the honest, sourced map of
exactly what the `sparq-shacl` crate does and does not yet pass, so the
follow-on feature waves are measurable rather than guesswork.

> **[OPUS-4.8] (sq-0mjfd / sq-5q76d / sq-u5rxj) update — the "final achievable"
> wave.** This wave closed the SHACL-SPARQL pre-binding **rejection channel** (the
> 7 `sht:Failure` entries, via `sparq_shacl::validate_strict`), `sh:reifierShape`
> / `sh:reificationRequired` (2 core), the `rdf:dirLangString` base-direction
> `sh:uniqueLang` key (1 core), the shapes-graph `sh:conformanceDisallows` thread
> (1 core), and the node-expr caller-supplied variable scope (`shnex:var`, +1
> node-expr). New measured floors: **core 133/134**, **sparql 24 (0 gap)**,
> **node-expr 63**. The remaining honest divergence is §6 below: the three
> `misc/{deactivated-003,message-002,severity-003}` per-constraint reified-
> annotation OVERRIDE entries. **Correction:** the `{| … |}` RDF-1.2 annotation
> SYNTAX is NOT a blocker — the pinned `oxttl 0.2.3` with the `rdf-12` feature
> (already enabled workspace-wide) parses it and `sparq-core` stores the
> triple-term (verified in-worktree). The blocker is the SHACL-MODEL work to
> interpret a `sh:deactivated`/`sh:message`/`sh:severity` reifier as an override
> of a SPECIFIC constraint statement (bead `sq-pb0wm`, see §6).

<!-- -->

> **[OPUS-4.8] (sq-pb0wm) update — the FINAL core gap is CLOSED.** The
> per-constraint-statement reified-annotation overrides are now interpreted, so
> `misc/{deactivated-003,message-002,severity-003}` move FAIL → PASS. New measured
> floors: **core 136 (default) / 137 (`shacl-af`)**, fail-ceiling **0** in BOTH
> feature states — every in-scope full-core entry passes. The two-sided ratchet is
> kept (not switched to an all-pass assert) so a future regression in either
> direction is still caught. See §6 for the resolved divergence and the one draft
> ambiguity that had to be decided.

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
| `w3c_core_full_shacl12.rs` | `core/manifest.ttl` (all 7 categories) | 137 | **133 / 134** | 3 FAIL (+ 1 SKIP default) — sq-0mjfd/sq-5q76d closed reifierShape×2, uniqueLang-003, conformance-disallows-001 |
| `w3c_sparql_shacl12.rs` | `sparql/manifest.ttl` | 24 | **24 / 24** | 0 FAIL — sq-0mjfd built the `sht:Failure` rejection channel (the 7 expected-failures now PASS) |
| `w3c_node_expr.rs` + `w3c_node_expr_constraints.rs` | `node-expr/` | 65 | **— / 63** | sq-u5rxj threaded the caller-supplied `shnex:var` scope; the previous `sht:scope-*` xfail now PASSES (`shacl-af`) |

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
| 12 | Per-statement reified-annotation overrides | `misc/deactivated-003`, `misc/message-002`, `misc/severity-003` | a `{\| sh:deactivated/message/severity … \|}` annotation on a constraint statement overrides ONLY that occurrence (DONE) | `sq-pb0wm` |
| 13 | `sh:uniqueLang` over `rdf:dirLangString` | `property/uniqueLang-003` | direction-tagged language strings | `sq-0mjfd` |

Clusters 1–8 (eight beads' worth of "value constraints", 13 entries) are `sq-sx15d`;
9–10 (targets, 3 entries) are `sq-rnkdh`; 11–13 (the RDF-1.2 draft/hard zone, 6
entries) are `sq-0mjfd`.

---

## 3. Gap clusters — SPARQL (CLOSED, sq-0mjfd)

The 1.2 SPARQL suite has 24 `sht:Validate` entries; **all 24 now pass.** The 7
`mf:result sht:Failure` entries declare a SPARQL constraint a conformant
processor MUST **reject** (re-binding a pre-bound variable, an unsupported
`MINUS` / `VALUES` / `SERVICE`, or a sub-`SELECT` that drops `$this`). `sq-0mjfd`
built that rejection channel: a build-time pre-binding validity check
(`PreparedSparql::build` / the component-validator path) records the violation,
and `sparq_shacl::validate_strict` returns `Err(ShaclFailure)` for it. The
harness drives `validate_strict` for an `sht:Failure` entry and PASSES iff it
rejects. (The lenient `validate` still skips such a constraint, preserving its
never-fails contract.) The historical gap table is kept below for provenance.

| # | Cluster | Entries | Why it fails | Bead |
| --- | --- | --- | --- | --- |
| 14 | SPARQL constraint result detail | `sparql/node/sparql-001`, `sparql/property/property-select-001` | `sh:select` constraint result message / count detail | `sq-rnkdh` |
| 15 | `sh:sparqlExpr` value-expression constraint | `sparql/property/property-sparqlExpr-001` | SPARQL-expression-valued property constraint | `sq-rnkdh` |
| 16 | SPARQL `sh:target` (targetNode-select) | `sparql/targets/targetNode-select-001` | SPARQL-based target | `sq-rnkdh` |
| 17 | Pre-binding VALUES propagation — **DONE (sq-mue75)** | `sparql/pre-binding/pre-binding-002`, `pre-binding-005`, `pre-binding-007` | the pre-binding algebra-rewrite (UNION / sibling / sub-SELECT scope) — now PASS | `sq-mue75` |
| 18 | Pre-binding **rejection** channel (`sht:Failure`) | `sparql/pre-binding/pre-binding-006`, `unsupported-sparql-001..006` (7) | a query that re-binds a pre-bound variable, or uses unsupported `MINUS`/`SERVICE`, MUST be rejected — the crate has no failure channel | `sq-0mjfd` |

### Per-category SPARQL scoreboard (post-sq-0mjfd)

| category | pass | fail | skip |
| --- | ---: | ---: | ---: |
| component | 3 | 0 | 0 |
| node | 4 | 0 | 0 |
| pre-binding | 13 | 0 | 0 |
| property | 3 | 0 | 0 |
| targets | 1 | 0 | 0 |
| **TOTAL** | **24** | **0** | **0** |

(The whole 1.2 SPARQL suite is green here: the 7 `sht:Failure` rejection entries
are genuine PASSes via `validate_strict`, cluster 18 / `sq-0mjfd` CLOSED.)

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
- **`sq-0mjfd`** — the draft / hard zone (clusters 11, 13, 18; +5q76d/u5rxj):
  `sh:reifierShape` / `sh:reificationRequired` (cluster 11, 2 core),
  `rdf:dirLangString` `uniqueLang` (cluster 13, 1 core), and the `sht:Failure`
  pre-binding **rejection channel** (cluster 18, 7 SPARQL) all PASS.
- **`sq-pb0wm`** — the per-constraint-statement reified-annotation overrides
  (cluster 12, `misc/{deactivated-003,message-002,severity-003}`, 3 core): **DONE**
  (see §6). This was the FINAL core gap; full core is now 136/137, fail-ceiling 0.
- **`sq-5q76d`** — shapes-graph `sh:conformanceDisallows` → `validate()` default
  `conforms` (cluster 8 remainder, `validation-reports/conformance-disallows-001`,
  1 core): DONE.
- **`sq-u5rxj`** — node-expr caller-supplied `shnex:var` scope (the `sht:scope-*`
  entry, +1 node-expr): DONE.

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

The Core validator is correct on the constraints it implements (the full-core
**136/137** + SPARQL **24/24** passing entries are compared strictly — a wrong
report there is a FAIL, never laundered into a SKIP). As of `sq-pb0wm` every
in-scope full-core entry passes (fail-ceiling 0); a handful of node/SPARQL/JS
entries remain SKIP because they use a constraint surface out of scope for the
Core path. Asserting all-pass would force either a false green or disabling real
comparison; the two-sided ratchet keeps the gate honest *and* regression-proof.
When a bead lands, its cluster's entries move FAIL → PASS, the per-runner floor is
bumped in the same commit, and this table shrinks.

---

## 6. Resolved divergence — per-statement reified-annotation overrides (sq-pb0wm)

**[OPUS-4.8] (sq-pb0wm, epic sq-waf9o)** The three `misc/` reified-annotation
OVERRIDE entries — the final core gap — are now CLOSED. Each is matched through
the REAL `validate()`:

| Entry | What it asserts | How it now passes |
| --- | --- | --- |
| `misc/deactivated-003` | `sh:datatype X {\| sh:deactivated true \|}` and `sh:property S {\| sh:deactivated true \|}` deactivate **that specific constraint occurrence** | `parse_shape` resolves the reifier of each constraint triple `(shape, P, O)`; eval skips ONLY that component (the shape keeps validating its others) — conforms |
| `misc/message-002` | `sh:datatype X {\| sh:message "…"@en \|}` attaches a message to **that constraint's** results | the per-statement message is threaded as a `ComponentMeta` override and applied in `result()` for ONLY that occurrence's results |
| `misc/severity-003` | `sh:datatype X {\| sh:severity sh:Warning \|}` overrides **that constraint's** severity | the per-statement severity is applied in `result()`, scoped to that occurrence (a sibling constraint keeps the default `sh:Violation`) |

### Implementation (the threading change)

For each constraint triple `(shape, P, O)` turned into a `Component`,
`ShapesModel::attach_component_meta` looks up the reifier
`?r rdf:reifies <<( shape P O )>>` in the SHAPES graph and captures any
`sh:deactivated` / `sh:message` / `sh:severity` into a `ComponentMeta` held in a
vector PARALLEL to `Shape::components`. `validate_shape` skips a per-statement
deactivated occurrence and sets the active `ComponentMeta` so `result()` applies a
per-occurrence message/severity — distinct from the shape-level
`deactivated`/`messages`/`severity` (which still work). The `{| … |}` syntax is
parsed by the already-pinned `oxttl 0.2.3` (`rdf-12`); **no Cargo dependency
change** was needed.

### Draft ambiguity decided (the brief's `divergences`)

Per-statement annotation semantics are the least-settled corner of the SHACL-1.2
draft; the W3C suite pins only the cases above. Two readings had to be chosen:

1. **Scope of the override = the single annotated occurrence, not the shape.** A
   `{| sh:deactivated true |}` on ONE constraint does NOT deactivate the whole
   shape (that is the shape-level `sh:deactivated`). This is exactly what
   `deactivated-003` encodes (the shape conforms because BOTH its constraints are
   individually deactivated, while a third un-annotated constraint would still
   fire) — implemented as written.
2. **A `{| sh:message |}` / `{| sh:severity |}` on a `sh:property` / `sh:node`
   statement governs the COMPOSITE component's own results, not the nested
   shape's.** The nested shape's results carry the nested shape's own
   metas/severity (it is a separate focus/shape evaluation). The suite does not
   exercise a message/severity annotation on a shape-referencing constraint, so
   this is the conservative, occurrence-local reading; `sh:property` deactivation
   (which IS in the suite) is unambiguous and implemented. Only single-statement
   Core constraints with a faithfully-recoverable object term carry an override;
   list-/path-valued operands (e.g. the disjunctive `sh:datatype ( … )`, an
   RDF-list comparand) are multi-triple and are NOT annotated.

   *[FABLE-5] (sq-1jemy)* This reading was initially only PARTIALLY implemented:
   the nested `conforms()` / `validate_shape()` recursion reset the active
   per-statement meta before the composite arm's `result()` read it, silently
   dropping the override on every recursing composite (`sh:node`, `sh:not`,
   `sh:someValue`, `sh:memberShape`, …). Fixed by saving the caller's meta at
   the single recursion entry (`validate_shape_at`) and restoring it on exit —
   nested evaluation is now transparent to the caller's override, and a take()
   keeps an outer override from leaking INTO nested results (pinned by
   `per_statement_override_on_sh_property_does_not_govern_nested_results`).
   On `sh:property` — which reports the nested shape's results directly and has
   no composite result — only deactivation is observable, as decided above.

### oxttl / oxrdf upgrade assessment (the item-6 question)

The original gap note assumed the `{| … |}` annotation syntax was unparseable
because "oxttl 0.1.8 predates it". **That is no longer true and was not the real
blocker.** Measured in-worktree at the pinned suite commit:

- The workspace pins `oxttl = { version = "0.2", features = ["rdf-12"] }` and
  `oxrdf = "=0.3.3"` (both with `rdf-12`); the resolved versions are `oxttl 0.2.3`
  / `oxrdf 0.3.3`.
- `oxttl 0.2.3` **does** implement the RDF-1.2 `annotationBlock ::= '{|'
  predicateObjectList '|}'` rule (verified in its `terse.rs`), and
  `sparq_shacl::load_turtle_with_base` parses `reifierShape-001.ttl` into the
  triple-term form: `_:r rdf:reifies <<( s p o )>> ; _:r ex:q false`. `oxrdf`
  exposes `Term::Triple(Box<Triple>)` and `Literal::direction()` for
  `rdf:dirLangString` — both used by this wave (no version bump needed).
- **No Cargo.toml/Cargo.lock dependency change was made in this PR** (a hard
  constraint), and none is REQUIRED: `sh:reifierShape` and the
  `rdf:dirLangString` `uniqueLang` key were implementable directly against the
  already-pinned versions.

So the residual work is **NOT** a parser/dependency upgrade — it is the
SHACL-MODEL feature of interpreting a reified-annotation as a per-constraint-
statement override. That requires, for each constraint triple `(shape, P, O)`,
looking up its reifier `?r rdf:reifies <<( shape P O )>>` and applying any
`sh:deactivated` / `sh:message` / `sh:severity` on `?r` to JUST that constraint
occurrence — a threading change through `parse_shape` → `Component` → `result`
(per-occurrence severity/message/deactivation), distinct from a shape-level one.
This is tracked as a dedicated bead (created with this PR). It is genuinely the
hardest remaining SHACL-1.2 surface (per-statement annotation semantics are the
least-settled corner of the 1.2 draft), counted-not-asserted at fail-ceiling 3.
