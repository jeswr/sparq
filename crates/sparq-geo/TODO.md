# sparq-geo — gaps / follow-ups

## Engine extension-function registry (the wiring this crate is waiting for)

sparq-engine was deliberately NOT modified (T24b scope: new opt-in crate only;
the engine is owned by another agent). To run `geof:` functions inside SPARQL
`FILTER`/`BIND`, the engine needs an extension-function registry. The exact
seam, recorded for the engine maintainer:

- `spargebra` already parses unknown function IRIs as
  `spargebra::algebra::Function::Custom(NamedNode)`.
- `crates/sparq-engine/src/exec.rs` -> `eval_function(...)` (~line 4655)
  currently handles `F::Custom(nn)` ONLY as the single-argument XSD
  constructor casts (~line 4907) and errors on everything else — that arm is
  the dispatch point.

A minimal registry API that fits the existing code shape:

```rust
// sparq-engine
/// Extension function: TERM-level in/out. `None` args are unbound/errored
/// expressions; returning Err -> SPARQL expression error (row filtered).
pub type ExtFn = dyn Fn(&[Term]) -> Result<Term, String> + Send + Sync;

pub struct FunctionRegistry { map: HashMap<String /* IRI */, Box<ExtFn>> }
impl FunctionRegistry {
    pub fn register(&mut self, iri: &str, f: impl Fn(&[Term]) -> Result<Term, String> + Send + Sync + 'static);
}

// New entry points threading the registry to eval_function (alongside the
// existing query/ask/count *_with_budget variants):
pub fn query_with_functions(graph: &Graph, sparql: &str, fns: &FunctionRegistry) -> Result<QueryResult, String>;
```

In `eval_function`'s `F::Custom(nn)` arm: after the XSD-cast fast path misses,
look up `nn.as_str()` in the registry, evaluate each `args[i]` to a `Value`,
materialise to `Term`s (`Value::Term` is already there; numerics/bools via
their literal forms), call, wrap the result back into `Value::Term`.

sparq-geo's side is ready: `sparq_geo::geof::lex::*` are exactly that shape —
e.g. register `geof:distance` as a 3-arg wrapper over
`geof::lex::distance(&a_lex, &b_lex, &unit_iri)` (extract the literal lexical
form of args 0/1, the IRI string of arg 2; return an `xsd:double` literal),
and each `geof:sf*` over `geof::lex::sf_*` returning `xsd:boolean`. The IRIs
are `vocab::GEOF_NS` + `distance` / `sfEquals` / `sfDisjoint` / `sfIntersects`
/ `sfTouches` / `sfCrosses` / `sfWithin` / `sfContains` / `sfOverlaps` /
`envelope` / `boundary` / `convexHull`.

A registry alone gives FILTER-level GeoSPARQL (correct, post-hoc). Pushing
`geof:` filters down into a `GeoIndex` window query (the performant plan
shape) additionally needs a planner hook — much bigger; not specified here.

## geof: functions not in v1

- `geof:buffer` — `geo` 0.30 ships no buffer operation. Upstream work exists
  (geo-buffer crates; a `Buffer` trait landed in newer geo lines); revisit on
  the next geo bump.
- `geof:intersection` / `union` / `difference` / `symDifference` — `geo`'s
  `BooleanOps` covers polygon/multipolygon pairs only; a general
  geometry-pair implementation needs case analysis (line/line overlay etc.).
  Cheap to add for the polygonal subset if needed.
- `geof:getSRID` — trivial (`GeoGeometry::crs.iri()`), just needs the literal
  in/out plumbing once the registry exists.
- `geof:relate` (generic DE-9IM pattern test) — `geo`'s `IntersectionMatrix`
  has `matches(pattern)`; easy follow-up.
- Egenhofer (`geof:eh*`) and RCC8 (`geof:rcc8*`) relation families — same
  DE-9IM machinery, different matrix patterns.

## CRS support

- No coordinate transformation: literals in projected CRSs (e.g.
  EPSG:27700) only combine with literals in the SAME CRS, and metric distance
  is refused for them. Real reprojection means `proj` (C dependency — heavy,
  breaks pure-Rust/wasm) or a pure-Rust subset (e.g. `proj4rs`).
- Only EPSG:4326 gets the lat/long axis-order normalisation; other
  geographic EPSG codes (4269, …) are treated as opaque `Crs::Other`.

## Index gaps

- Antimeridian: query balls and geometries crossing ±180° longitude are not
  wrapped — a ball window is clamped to [-180, 180] instead of splitting into
  two windows. Fix: split the window and run two tree queries.
- Default graph only: named graphs (`Graph::named`) are not scanned.
- Static index: `Graph::apply_delta` updates do not flow into a built
  `GeoIndex` (rebuild after compaction; `rstar` does support incremental
  insert/remove if incremental maintenance is ever needed).
- `geo:gmlLiteral` geometries are ignored (WKT only).
- Distance to an extended geometry uses the spherical closest point of the
  geometry as stored (segments are treated as great-circle arcs by
  `HaversineClosestPoint`); planar-vs-spherical edge discrepancies are
  negligible at the segment lengths typical of RDF data.

## Conformance

Formal OGC GeoSPARQL conformance (the official test suite) is skipped — it
requires a SPARQL-protocol endpoint harness and the full `geor:` query-rewrite
machinery. The implemented subset is tabulated against spec sections in
README.md.
