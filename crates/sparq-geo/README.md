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
| — | CRS | ✅ CRS84 (default), EPSG:4326 (lat/long axis order normalised internally); ✅ other CRS IRIs carried verbatim (relations work within the same CRS); ✅ opt-in reprojection to CRS84 (`reproject` cargo feature, pure-Rust proj4rs) for a curated EPSG set: 27700, 3857, 2154, 25832/25833, UTM 326xx/327xx |
| 8.7 / F.1 | `geof:distance(g1, g2, units)` | ✅ unit IRIs: OGC `uom:metre`/`kilometre`/`degree`/`radian` + QUDT `M`/`KiloM`/`MI`/`DEG`/`RAD` (see accuracy notes below) |
| 9.3–9.5 (Req 22–24) | Simple-features relation functions `geof:sfEquals/sfDisjoint/sfIntersects/sfTouches/sfCrosses/sfWithin/sfContains/sfOverlaps` | ✅ DE-9IM via `geo`'s `Relate` (planar, in coordinate/degree space) |
| 9.4 (Req 25) | Egenhofer relation functions `geof:ehEquals/ehDisjoint/ehMeet/ehOverlap/ehCovers/ehCoveredBy/ehInside/ehContains` | ✅ the spec's DE-9IM matrix patterns over the same machinery |
| 9.5 (Req 26) | RCC8 relation functions `geof:rcc8eq/rcc8dc/rcc8ec/rcc8po/rcc8tppi/rcc8tpp/rcc8ntpp/rcc8ntppi` | ✅ ditto |
| 9 | `geof:relate(g1, g2, pattern)` — generic DE-9IM test | ✅ `IntersectionMatrix::matches` |
| 8.7 | `geof:envelope`, `geof:boundary`, `geof:convexHull` | ✅ (`boundary` of a GEOMETRYCOLLECTION unsupported) |
| 8.7 | `geof:buffer(g, radius, units)` | ✅ geo 0.33 `Buffer`; metric radii via a local equirectangular metre frame (accuracy notes below); result MULTIPOLYGON |
| 8.7 | `geof:intersection/union/difference/symDifference` | ✅ point-set ops: polygon overlay (geo `BooleanOps`) **plus** the well-defined line/point cases — see the table below; genuinely-intractable combos (1-D set subtraction) are a per-row expression error |
| 8.7 | `geof:getSRID` | ✅ the CRS IRI as `xsd:anyURI` |
| 8.3/8.4 | Core RDF shape: `geo:hasGeometry`, `geo:hasDefaultGeometry`, `geo:asWKT` | ✅ extracted by `GeoIndex::build` (default + named graphs) |
| 8.5.2 | `geo:gmlLiteral` | ❌ WKT only (no maintained pure-Rust GML parser — TODO.md) |
| 7 / 9 | RIF/SPARQL rewrite rules, `geor:` query rewriting | ❌ needs engine-level query rewriting (TODO.md) |

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

Registered IRIs (35, all under `http://www.opengis.net/def/function/geosparql/`):
`distance` (3rd arg a unit IRI, result `xsd:double`); the relation families
`sfEquals` … `sfOverlaps`, `ehEquals` … `ehContains`, `rcc8eq` … `rcc8ntppi`
plus the generic `relate(g1, g2, de9imPattern)` (all `xsd:boolean`);
`envelope` / `boundary` / `convexHull` and `buffer(g, radius, unitIri)` /
`intersection` / `union` / `difference` / `symDifference` (`geo:wktLiteral`);
`getSRID` (`xsd:anyURI`). Geometry arguments must be `geo:wktLiteral`
literals. Any geo error (malformed WKT, wrong datatype, CRS mismatch, unknown
unit, an unsupported set-operation operand pair, wrong arity) is a per-row
SPARQL *expression* error — the row is dropped by a `FILTER`, left unbound by a
`BIND` — exactly like the builtin functions; an IRI not in the registry stays
a hard query error.

### Set-operation operand matrix

The four set operations are GeoSPARQL point-set operations. `geo`'s
`BooleanOps` realises polygon overlay; the lower-dimension cases are
implemented directly over `geo`'s `line_intersection` / `CoordinatePosition`.
"point" covers `POINT`/`MULTIPOINT`, "line" `LINESTRING`/`MULTILINESTRING`,
"polygon" `POLYGON`/`MULTIPOLYGON` (and the rect/triangle shorthands).

| operands | `intersection` | `union` | `difference` (a − b) | `symDifference` |
|---|---|---|---|---|
| polygon × polygon | ✅ overlay | ✅ overlay | ✅ overlay | ✅ overlay |
| point × point | ✅ shared points | ✅ MULTIPOINT | ✅ set subtraction | ✅ set sym-diff |
| point × line/polygon | ✅ points on the other | ✅ GEOMETRYCOLLECTION | ✅ points not on the other | ✅ (point−other) ∪ other |
| line × line | ✅ crossings + collinear overlaps | ✅ MULTILINESTRING | ❌ no linear-referencing in `geo` | ❌ ditto |
| line × polygon | ✅ line clipped to the polygon | ✅ GEOMETRYCOLLECTION | ❌ ditto | ❌ ditto |

A ❌ cell is a clean `GeoError::Unsupported` (a per-row expression error through
the registry), never a wrong answer or panic. Results are the lowest-dimension
geometry that captures the answer (a line merely *touching* a polygon at a
point yields a `MULTIPOINT`; a line∩line that is purely a crossing yields a
`MULTIPOINT`/`POINT`), and serialise back to `geo:wktLiteral`. Line/line and
line/polygon unions are NOT noded/dissolved (`geo` has no 1-D overlay), so an
overlapping line∪line keeps both curves.

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

Query balls crossing the **antimeridian** split into two longitude windows
(two tree walks, merged + deduped); the index covers the default graph **and
every named graph** (each `Entry` records its origin graph and `geo:asWKT`
node). `GeoIndex::apply_delta(graph, inserts, deletes)` mirrors a
`Graph::apply_delta` batch incrementally (rstar insert/remove, O(batch·log n)
— no rebuild), including `geo:hasGeometry` ownership re-keying.

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
