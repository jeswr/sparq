# sparq-geo

Opt-in GeoSPARQL 1.0/1.1 core for the [sparq](https://github.com/jeswr/sparq)
RDF engine: `geo:wktLiteral` parsing, the `geof:` spatial functions, and an
R-tree `GeoIndex` over sparq `Graph`s.

This is a **separate crate** by design: no existing sparq crate (and in
particular not the wasm build) depends on it unconditionally — spatial support
is engaged only by adding `sparq-geo` as a dependency. The `geof:` functions
are exposed both as plain Rust (`geof::*` over parsed geometries, `geof::lex::*`
over wktLiteral lexical forms) and — behind the default-on `engine` feature —
as a sparq-engine **extension-function registry**, so they run inside real
SPARQL `FILTER`/`BIND` expressions (see "Running `geof:` inside SPARQL" below).

Geometry parsing and algorithms wrap the standard pure-Rust geo stack
([`wkt`](https://crates.io/crates/wkt), [`geo`](https://crates.io/crates/geo) /
`geo-types`); the spatial index wraps [`rstar`](https://crates.io/crates/rstar).

## What's implemented (GeoSPARQL spec subset)

| Spec section (1.0 / 1.1) | Feature | Status |
| --- | --- | --- |
| 8.5.1 (Req 10–12) | `geo:wktLiteral` lexical form: WKT body, optional leading `<CRS-IRI>`, default CRS84 | ✅ `parse_wkt_literal` / `GeoGeometry::to_wkt_literal` |
| — | WKT geometry types | ✅ POINT, LINESTRING, POLYGON (holes), MULTIPOINT/-LINESTRING/-POLYGON, GEOMETRYCOLLECTION; empties parse to geo-types' empty representations |
| — | CRS | ✅ CRS84 (default), EPSG:4326 (lat/long axis order normalised internally); ✅ other CRS IRIs carried verbatim (relations work within the same CRS); ❌ no CRS transformation / projected-metric support |
| 8.7 / F.1 | `geof:distance(g1, g2, units)` | ✅ unit IRIs: OGC `uom:metre`/`kilometre`/`degree`/`radian` + QUDT `M`/`KiloM`/`MI`/`DEG`/`RAD` (see accuracy notes below) |
| 9.3–9.5 (Req 22–24) | Simple-features relation functions `geof:sfEquals/sfDisjoint/sfIntersects/sfTouches/sfCrosses/sfWithin/sfContains/sfOverlaps` | ✅ DE-9IM via `geo`'s `Relate` (planar, in coordinate/degree space) |
| 8.7 | `geof:envelope`, `geof:boundary`, `geof:convexHull` | ✅ (`boundary` of a GEOMETRYCOLLECTION unsupported) |
| 8.7 | `geof:buffer` | ❌ `geo` 0.30 has no buffer op (TODO.md) |
| 8.7 | `geof:intersection/union/difference/symDifference`, `geof:getSRID` | ❌ not in v1 (TODO.md) |
| 8.3/8.4 | Core RDF shape: `geo:hasGeometry`, `geo:hasDefaultGeometry`, `geo:asWKT` | ✅ extracted by `GeoIndex::build` |
| 8.5.2 | `geo:gmlLiteral` | ❌ WKT only |
| 7 / 9 | RIF/SPARQL rewrite rules, `geor:` query rewriting, Egenhofer/RCC8 relation families | ❌ out of scope for v1 |

Formal OGC conformance testing is **skipped** (the official suite needs a full
SPARQL endpoint harness); the table above is the implemented subset.

### Distance semantics & accuracy

Metric units require a geographic CRS (CRS84/EPSG:4326) and measure
**great-circle** distance on the GRS80 mean sphere (R = 6 371 008.8 m, the
same sphere as `geo`'s haversine):

- point ↔ point: exact haversine;
- point ↔ extended geometry: haversine to the spherical closest point
  (`geo::HaversineClosestPoint`);
- extended ↔ extended: **local equirectangular approximation** about the
  geometries' mean latitude (accurate at local scale, degrades for
  continent-spanning geometry pairs).

`uom:degree` / `uom:radian` measure euclidean coordinate-space distance
(degrees of arc for geographic CRSs; raw units for other CRSs).

## Library use

```rust
use geo_types::Point;
use sparq_core::Graph;
use sparq_geo::{geof, GeoIndex, Unit, parse_wkt_literal};

let graph = Graph::load_str(ttl, "turtle")?;

// geof: functions over wktLiteral lexical forms (the engine-builtin shape):
let km = geof::lex::distance(
    "POINT(-0.1276 51.5074)", "POINT(2.3496 48.8530)",
    "http://www.opengis.net/def/uom/OGC/1.0/kilometre")?;        // ≈ 341
let yes = geof::lex::sf_within("POINT(1 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))")?;

// Spatial index over the graph's geo:hasGeometry / geo:asWKT shape:
let index = GeoIndex::build(&graph);
let nearby = index.within_distance(Point::new(-0.1276, 51.5074), 250_000.0, None);
let top10  = index.nearest(Point::new(-0.1276, 51.5074), 10);
let hits   = index.intersects_wkt("POLYGON((1 48, 3 48, 3 49.5, 1 49.5, 1 48))")?;
```

## Running `geof:` inside SPARQL

`geof_registry()` (the default-on `engine` cargo feature) packages every
implemented `geof:` function as a
[`sparq_engine::FunctionRegistry`](../../docs/extension-functions.md) —
sparq-engine's SPARQL 17.6 extension-function mechanism:

```rust
use sparq_core::Graph;
use sparq_engine::query_with_functions;
use sparq_geo::geof_registry;

let g = Graph::load_str(ttl, "turtle")?;
let r = query_with_functions(&g, r#"
    PREFIX geof: <http://www.opengis.net/def/function/geosparql/>
    PREFIX uom:  <http://www.opengis.net/def/uom/OGC/1.0/>
    SELECT ?city WHERE {
      <http://ex/london> <http://ex/loc> ?here .
      ?city <http://ex/loc> ?there .
      FILTER(geof:distance(?here, ?there, uom:kilometre) < 400)
    }"#, &geof_registry())?;
```

Registered IRIs (all under `http://www.opengis.net/def/function/geosparql/`):
`distance` (3rd arg a unit IRI, result `xsd:double`), the eight
simple-features relations `sfEquals` … `sfOverlaps` (`xsd:boolean`), and
`envelope` / `boundary` / `convexHull` (`geo:wktLiteral`). Geometry arguments
must be `geo:wktLiteral` literals. Any geo error (malformed WKT, wrong
datatype, CRS mismatch, unknown unit, wrong arity) is a per-row SPARQL
*expression* error — the row is dropped by a `FILTER`, left unbound by a
`BIND` — exactly like the builtin functions; an IRI not in the registry (e.g.
`geof:buffer`, not in v1) stays a hard query error.

Build the registry **once** and reuse it: it is cheaply cloneable and
`Send + Sync`. sparq-server exposes exactly this wiring behind its opt-in
`geo` cargo feature (`cargo build -p sparq-server --features geo`), which
installs the registry on the `/sparql` query, update, and subscription paths.

The registry evaluates `geof:` *post-hoc* (after pattern matching, per row).
For large spatial selections, pre-filter with the R-tree `GeoIndex` below and
feed the candidates to the query (e.g. via `VALUES`); pushing `geof:` filters
down into index windows automatically needs a planner hook (TODO.md).

## Index design

`GeoIndex::build` scans `geo:asWKT` once (plus one scan per `hasGeometry`
predicate to map geometry nodes back to their owning features), parses each
wktLiteral, and bulk-loads an `rstar` R-tree (packed STR build) of
`{entry-index, AABB}` leaves in long/lat degree space; parsed geometries live
in a flat side array. Queries prune by bounding box and refine against the
true geometry:

- `within_distance(center, meters, limit)` — pole-safe long/lat window around
  the great-circle ball, then exact great-circle refinement; results sorted
  nearest-first.
- `nearest(center, k)` — expanding-radius (×4) search over `within_distance`,
  seeded from the data extent; exact under the same metric.
- `intersects(geometry)` — AABB window + `geo::Intersects` refinement.

Geometries crossing the **antimeridian** are not wrapped in v1, and the index
covers the **default graph** only (see TODO.md).

## Benchmark

`cargo run --release -p sparq-geo --example bench_geo` — 100 000 random CRS84
points over an 8°×8° window (country-sized), 1 000 query points, Apple M-class
laptop (2026-06-10):

```
graph load     : 100000 asWKT triples in 81.05ms
index build    : 100000 entries in 96.15ms (1.04 Mentries/s)
within     1km :      1.1 µs/query (0.7 avg hits)
within    10km :     16.9 µs/query (63.0 avg hits)
within    50km :    396.6 µs/query (1483.1 avg hits)
nearest k=1     :      5.5 µs/query
nearest k=10    :     32.5 µs/query
nearest k=100   :    236.7 µs/query
intersects box :     34.4 µs/query (389.8 avg hits, 0.5°x0.5° boxes)
```

Query cost is dominated by the number of true hits refined (the 50 km radius
touches ~1.5 k points); pure pruning (1 km / k=1) is a microsecond or two.
