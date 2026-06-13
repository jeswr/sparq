# sparq-shacl — known gaps / follow-ups

## SHACL-SPARQL (`sh:sparql`, W3C SHACL §5.2) — DONE (sq-1rr) [OPUS-4.8]
The `sh:sparql` constraint component is implemented (`src/sparql.rs` + the
`Component::Sparql` arm in `eval.rs`), routing the `sh:select` query through
`sparq-engine`. Per focus node the query runs with `$this` pre-bound (by an
**algebra-level** `VALUES (?this)` injection — robust to query layout, unlike a
textual prepend); each solution is one validation result. Covered:
  - `sh:prefixes` (`sh:declare` → `sh:prefix`/`sh:namespace`), with `owl:imports`
    chasing of the prefix-declarations resource (cycle-guarded);
  - `$PATH` pre-binding on property shapes (the path's SPARQL property-path form
    is substituted for `$PATH`/`?PATH` as a whole token);
  - solution → result mapping: `sh:focusNode` = `$this`; `sh:value` = `?value`
    when projected, else the focus node (per the W3C suite); `sh:resultPath` from
    a bound `?path` (IRI) else the property shape's path; `sh:resultMessage` from
    the constraint's `sh:message` (with `{?var}`/`{$var}` substitution), else a
    bound `?message`, else a default;
  - `sh:deactivated` on the constraint; ill-formed `sh:select` skipped (lenient).
Pinned by `tests/sparql_constraints.rs` (10 cases) and the W3C `sparql/node` +
`sparql/property` sub-suites via `tests/w3c_sparql.rs` (5/5).

### SHACL-SPARQL — still deferred (honest scope)
- **SPARQL-based constraint COMPONENTS** (`sh:ConstraintComponent` declarations
  with `sh:parameter` + `sh:validator`/`sh:nodeValidator`/`sh:propertyValidator`,
  `sh:SPARQLAskValidator`/`sh:SPARQLSelectValidator`): NOT implemented. This is
  the larger §6 machinery — a component registry keyed on the parameter
  predicates a shape uses, multi-parameter pre-binding (`$paramName`), the
  ASK-validator-per-value-node firing rule, and `?value`/`$value` binding for
  ASK validators. The suite's `sparql/component/*` entries also `owl:imports` the
  external `http://datashapes.org/dash` vocabulary, which the offline test
  harness cannot resolve. A follow-up would add a `Component::CustomSparql`
  carrying the resolved validator + bound parameters and a component-discovery
  pass over the shapes graph.
- **Full pre-binding semantics** (`sparql/pre-binding/*`): only `$this`/`$PATH`
  pre-binding is done. The suite's pre-binding section additionally requires
  *rejecting* queries that would re-bind a pre-bound variable (e.g. a `BIND` or
  inner `VALUES`/sub-SELECT projecting `?this`) as `sht:Failure`, and binding
  `$shapesGraph`/`$currentShape`. `$currentShape` could be bound cheaply; the
  re-binding rejection and `$shapesGraph` (no named-graph handle to the shapes
  graph at query time) are not done. These entries are not walked by the runner.
- `sh:pattern` uses Rust `regex` semantics, not XPath/XQuery regex. The
  divergences (e.g. `\i`, `\c` character classes, the `q` flag) don't appear
  in the core suite, but a strict implementation would translate or implement
  XPath regexes.
- XSD date/time comparison treats timezoned vs untimezoned values as
  incomparable outright; XSD's ±14h determinate window (a timezoned value can
  still be ordered against an untimezoned one when they are >14h apart) is not
  implemented. No core test exercises the window.
- Ill-formed *shapes graphs* are handled leniently (malformed constructs are
  skipped) rather than reported as `sht:Failure`. The core suite contains no
  Failure entries; the lenient choice keeps `validate()`'s signature
  infallible per the roadmap API.
- `sh:shapesGraph`/owl:imports resolution of shapes graphs is not done — the
  caller assembles the shapes graph.
- Focus-node/value-node sets containing RDF 1.2 triple terms are untested
  (the dictionary supports them; the SHACL spec predates them).

## Upstream (sparq-core / sparq-engine) gaps noticed — NOT changed here
- ~~`Graph::load_str` has no base-IRI parameter~~ **DONE (engine-seams wave)**:
  `Graph::load_str_with_base` / `parse_to_triples_with_base` landed in sparq-core
  (Turtle/TriG; a document's own `@base` still wins). This crate's
  `load_turtle_with_base` workaround can be swapped for it by this crate's owner.
- ~~`sparq_core::Graph` exposes no term-level triple iterator~~ **DONE
  (engine-seams wave)**: `Graph::iter_ids` / `iter_ids_sorted(col)` yield
  canonical `[Id; 3]` rows borrowing the index (overlay-merged, zero alloc per
  row); resolve lazily via `Dict::term_parts` (borrowing) or `Dict::term`.
  `GraphView` can iterate ids and materialise only the terms it actually
  compares — this crate's owner's call.

## Implementation niceties — status
- ~~Severity-aware conformance~~ **DONE**:
  `ValidationReport::conforms_violations_only()` (true iff no result carries
  `sh:Violation` — the "warnings don't fail the build" toggle) and
  `results_with_severity(iri)`. The spec's `sh:conforms` field is untouched
  (`results.is_empty()` regardless of severity — every suite expectation,
  including `misc/severity-*`, still pinned); the API stays infallible.
- The W3C runner compares expected blank-node values as wildcards (blank
  nodes have no cross-graph identity). A graph-isomorphism comparison would
  be stricter; with result counts + per-property matching it has not been
  needed.
- ~~Performance: `conforms()` re-validates shared shapes per (focus, shape)
  pair without memoisation~~ **DONE**: `(focus, shape) → bool` memo with a
  cycle-soundness rule that does NOT weaken the recursion guard — the guard
  is unchanged; the memo additionally tracks the lowest stack depth any
  guard re-entry pointed at, and a frame's result is stored only when no
  re-entry escaped below it (a frame closing its own cycle is context-free
  and may be stored; one reached mid-cycle may not). Pinned by a cyclic
  sh:node test (conforming and violating cycles, per-route reporting
  preserved) and the full W3C suite (98/98 unchanged). Measured on the suite
  runner (release, quiet M1, 5 runs of the test binary): typical 0.11 s →
  0.08 s — small in absolute terms because suite fixtures are tiny and the
  wall time is dominated by manifest parsing; the memo's real payoff is
  workloads where shared shapes are reached from many focus nodes/routes
  (DAG-shaped models, sh:and/sh:or member reuse).
