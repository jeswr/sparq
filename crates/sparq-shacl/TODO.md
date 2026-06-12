# sparq-shacl — known gaps / follow-ups

## Spec coverage (documented out of scope — unchanged)
- **SHACL-SPARQL** (`sh:sparql`, SPARQL-based constraint components,
  pre-binding) is not implemented — the suite's `sparql/` section is out of
  scope for this crate (Core only). Doing it well would route generated
  SELECTs through `sparq-engine`.
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
- `Graph::load_str` has no base-IRI parameter, so documents with relative
  IRIs can't be loaded directly into a `Graph`; this crate works around it
  with `load_turtle_with_base` (oxttl parse + `Graph::from_parts`).
  **BLOCKED-ON-ENGINE-SEAM** (handoff recorded): the engine-seams work is
  adding `load_str_with_base` on sparq-core; when it lands, swap
  `load_turtle_with_base` to delegate to it (keep the public helper for
  API stability) — do NOT remove the workaround before then.
- `sparq_core::Graph` exposes no term-level triple iterator; the `GraphView`
  wrapper here materialises `oxrdf::Term`s per scan row. Fine for validation
  workloads; **BLOCKED-ON-ENGINE-SEAM** (same handoff): the seams agent is
  adding a borrowing triple/term iterator (via `term_parts`) which would cut
  allocations in `GraphView`.

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
