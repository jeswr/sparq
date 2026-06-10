# sparq-geo — gaps / follow-ups

## Engine extension-function registry — DONE

sparq-engine grew the extension-function registry this section originally
specified (`FunctionRegistry` / `ExtFn` / `query_with_functions` /
`with_functions`; see `docs/extension-functions.md`), and `src/registry.rs`
registers all 12 implemented `geof:` functions through it (the default-on
`engine` cargo feature; `tests/registry_sparql.rs` exercises them through
real SPARQL). sparq-server installs the registry behind its opt-in `geo`
cargo feature.

Still open from that design note: the registry evaluates `geof:` post-hoc
(per row, after pattern matching). Pushing `geof:` filters down into a
`GeoIndex` window query (the performant plan shape) additionally needs a
planner hook — much bigger; not specified here.

## geof: functions not in v1

- `geof:buffer` — `geo` 0.30 ships no buffer operation. Upstream work exists
  (geo-buffer crates; a `Buffer` trait landed in newer geo lines); revisit on
  the next geo bump.
- `geof:intersection` / `union` / `difference` / `symDifference` — `geo`'s
  `BooleanOps` covers polygon/multipolygon pairs only; a general
  geometry-pair implementation needs case analysis (line/line overlay etc.).
  Cheap to add for the polygonal subset if needed.
- `geof:getSRID` — trivial (`GeoGeometry::crs.iri()`), just needs the literal
  in/out plumbing in src/registry.rs (the registry now exists).
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
