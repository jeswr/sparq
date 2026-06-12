# sparq-geo — gaps / follow-ups

Status audit 2026-06-12: every item from the previous revision of this file is
either IMPLEMENTED (with tests) or explicitly DEFERRED below with its reason.

## Engine extension-function registry — DONE

sparq-engine grew the extension-function registry this section originally
specified (`FunctionRegistry` / `ExtFn` / `query_with_functions` /
`with_functions`; see `docs/extension-functions.md`), and `src/registry.rs`
registers all 35 implemented `geof:` functions through it (the default-on
`engine` cargo feature; `tests/registry_sparql.rs` exercises every function
through real SPARQL). sparq-server installs the registry behind its opt-in
`geo` cargo feature.

## geof: functions — DONE (was "not in v1")

All previously-deferred functions landed (2026-06-12):

- `geof:getSRID` — implemented (`xsd:anyURI` result; `lex::get_srid`).
- `geof:relate` — implemented (generic DE-9IM pattern via geo's
  `IntersectionMatrix::matches`; malformed patterns are per-row expression
  errors).
- Egenhofer `geof:eh*` (8) and RCC8 `geof:rcc8*` (8) — implemented with the
  GeoSPARQL 1.0 Req 25/26 matrix patterns over the same DE-9IM machinery;
  partition + inverse-consistency truth tables in `tests/relations.rs`.
- `geof:intersection` / `union` / `difference` / `symDifference` — implemented
  for the POLYGONAL subset (geo's `BooleanOps`: polygon/multipolygon, plus the
  rect/triangle shorthands); result is a MULTIPOLYGON.
- `geof:buffer` — implemented (geo bumped 0.30 → 0.33, which ships the
  `Buffer` trait). Metric radii require a geographic CRS and buffer in a
  LOCAL EQUIRECTANGULAR metre frame about the geometry's mean latitude (same
  approximation class as extended–extended `geof:distance`); degree/radian
  radii buffer in coordinate space.

## CRS support

- DONE (opt-in): the `reproject` cargo feature (`src/reproject.rs`) adds
  pure-Rust reprojection into CRS84 via `proj4rs` for a CURATED EPSG table —
  27700 (verified to ~1e-6° against the OS worked example), 3857, 2154,
  25832/25833, UTM 326xx/327xx. One transitive dep (thiserror); off by
  default and out of the wasm graph.
- DEFERRED — full EPSG database: proj4rs ships no EPSG registry; embedding
  one (thousands of definitions) is real product surface with licensing/data
  questions. The curated table is one worked example per new code to extend.
- DEFERRED — automatic reprojection inside `geof:` functions / GeoIndex
  extraction: a semantics decision (GeoSPARQL says compute in the literal's
  CRS), not a code gap; callers can normalise with `reproject::to_crs84`
  before indexing.
- DEFERRED — lat/long axis-order normalisation for geographic EPSG codes
  other than 4326 (4269, …): needs an axis-order registry (same database
  problem as above); they remain opaque `Crs::Other`.

## Index gaps

- Antimeridian — DONE: query balls crossing ±180° split into two windows
  (two tree walks, deduped + merged); `nearest` inherits the fix. Regression
  tests straddle the seam against brute force.
- Named graphs — DONE: `GeoIndex::build` scans the default graph plus every
  `Graph::named` entry; `Entry::graph` records the origin.
- Incremental updates — DONE: `GeoIndex::apply_delta(graph, inserts, deletes)`
  mirrors a `Graph::apply_delta` batch (rstar incremental insert/remove,
  O(batch·log n)), re-extracting affected geometry nodes including
  `geo:hasGeometry` ownership re-keying; random-churn test asserts
  equivalence with a fresh build. Deltas affect the default graph, matching
  `Graph::apply_delta` semantics.
- DEFERRED — `geo:gmlLiteral` (WKT only): needs a GML (XML) geometry parser;
  no maintained pure-Rust GML parser exists, and hand-rolling XML geometry
  parsing is a project of its own. Revisit if a wrappable crate appears.
- Distance to an extended geometry uses the spherical closest point of the
  geometry as stored (segments are treated as great-circle arcs by
  `HaversineClosestPoint`); planar-vs-spherical edge discrepancies are
  negligible at the segment lengths typical of RDF data. (Documented
  accuracy note, not a planned change.)

## Deferred: planner pushdown (needs-engine-seam)

The registry evaluates `geof:` post-hoc (per row, after pattern matching).
Pushing `geof:` filters down into a `GeoIndex` window query (the performant
plan shape) needs a planner hook in sparq-engine — an engine seam, out of
scope for this leaf crate. Recorded for the engine roadmap; until then,
pre-filter with `GeoIndex` and feed candidates via `VALUES`.

## Conformance

Formal OGC GeoSPARQL conformance (the official test suite) is skipped — it
requires a SPARQL-protocol endpoint harness and the full `geor:` query-rewrite
machinery (also an engine-level feature). The implemented subset is tabulated
against spec sections in README.md.
