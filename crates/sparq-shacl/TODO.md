# sparq-shacl — known gaps / follow-ups

## Spec coverage
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
  with `load_turtle_with_base` (oxttl parse + `Graph::from_parts`). A
  `load_str_with_base` on sparq-core would remove the workaround.
- `sparq_core::Graph` exposes no term-level triple iterator; the `GraphView`
  wrapper here materialises `oxrdf::Term`s per scan row. Fine for validation
  workloads; a borrowing iterator (via `term_parts`) would cut allocations.

## Implementation niceties
- Severity-aware conformance: `sh:conforms` is currently `results.is_empty()`
  regardless of severity, which matches every suite expectation (including
  `misc/severity-*`); some implementations expose a "violations only" toggle.
- The W3C runner compares expected blank-node values as wildcards (blank
  nodes have no cross-graph identity). A graph-isomorphism comparison would
  be stricter; with result counts + per-property matching it has not been
  needed.
- Performance: `conforms()` re-validates shared shapes per (focus, shape)
  pair without memoisation. The suite (incl. the SHACL-SHACL meta test) runs
  in ~1.6s; memoise `(focus, shape) -> bool` if real workloads need it.
