---
name: geosparql
description: Use when adding GeoSPARQL spatial support to the sparq RDF/SPARQL engine — parsing geo:wktLiteral geometries, calling geof: functions (distance, sf*/eh*/rcc8* DE-9IM relations, envelope/boundary/convexHull/buffer, intersection/union/difference/symDifference, getSRID) inside SPARQL FILTER/BIND/SELECT via the sparq-engine extension-function registry, or building an R-tree GeoIndex over a Graph for within_distance / nearest / intersects queries. Crate: sparq-geo.
---

# sparq-geosparql

`sparq-geo` is the OPT-IN GeoSPARQL 1.0/1.1 **core** for the sparq engine: it parses `geo:wktLiteral` lexical forms, evaluates the `geof:` spatial functions, packages those functions as a `sparq_engine::FunctionRegistry` so they run inside real SPARQL `FILTER`/`BIND`/`SELECT`, and builds an R-tree `GeoIndex` over a `sparq_core::Graph` for distance/nearest/intersection queries. It is a separate crate so the core engine and the wasm build carry zero geometry code — you engage spatial support only by depending on `sparq-geo`.

## Quickstart

`Cargo.toml`:
```toml
[dependencies]
sparq-geo    = { path = "../sparq-geo" }   # default features include `engine`
sparq-core   = { path = "../sparq-core" }
sparq-engine = { path = "../sparq-engine" }
geo-types    = "0.7"                        # only if you use GeoIndex (Point<f64>)
```

Run `geof:` functions inside a SPARQL query:
```rust
use sparq_core::Graph;
use sparq_engine::query_with_functions;
use sparq_geo::geof_registry;

let g = Graph::load_str(r#"
    @prefix geo: <http://www.opengis.net/ont/geosparql#> .
    <http://ex/london> <http://ex/loc> "POINT(-0.1278 51.5074)"^^geo:wktLiteral .
    <http://ex/paris>  <http://ex/loc> "POINT(2.3522 48.8566)"^^geo:wktLiteral .
"#, "turtle").unwrap();

let reg = geof_registry();               // 35 functions; clone-cheap, Send + Sync
let r = query_with_functions(&g,
    "PREFIX geof: <http://www.opengis.net/def/function/geosparql/>
     PREFIX uom:  <http://www.opengis.net/def/uom/OGC/1.0/>
     SELECT ?a ?b WHERE {
       ?a <http://ex/loc> ?ga . ?b <http://ex/loc> ?gb .
       FILTER(STR(?a) < STR(?b) && geof:distance(?ga, ?gb, uom:kilometre) < 400)
     }", &reg).unwrap();
assert_eq!(r.len(), 1);                   // London–Paris ≈ 343.6 km
```

`POINT` / WKT coordinates are **longitude latitude** (CRS84, the GeoSPARQL default). A leading `<CRS-IRI>` before the WKT selects a CRS; `EPSG:4326` literals are LAT/LONG and are axis-swapped to internal long/lat on parse and back on output.

## Key APIs

```rust
// --- WKT literals (always available) ---
pub fn parse_wkt_literal(lex: &str) -> Result<GeoGeometry, GeoError>;
pub struct GeoGeometry { pub crs: Crs, pub geometry: geo_types::Geometry<f64> }
impl GeoGeometry { pub fn new(g: Geometry<f64>) -> Self;        // CRS84
                   pub fn to_wkt_literal(&self) -> String; }    // lexical form
pub enum Crs { Crs84, Epsg4326, Other(String) }                 // .iri(), .is_geographic()

// --- geof: as the SPARQL extension-function registry (default-on `engine` feature) ---
pub fn geof_registry() -> sparq_engine::FunctionRegistry;
// drive with:  sparq_engine::query_with_functions(&graph, sparql, &reg) -> Result<QueryResult, String>
//   or scope another entry point: sparq_engine::with_functions(&reg, || sparq_engine::ask(&g, q))

// --- geof: as plain Rust over GeoGeometry (always available) ---
pub fn distance(a: &GeoGeometry, b: &GeoGeometry, unit: Unit) -> Result<f64, GeoError>;
pub fn sf_within(a, b) -> Result<bool, GeoError>;   // + sf_equals/disjoint/intersects/touches/
                                                    //   crosses/contains/overlaps, eh_*, rcc8_*
pub fn relate(a, b, pattern: &str) -> Result<bool, GeoError>;   // generic 9-char DE-9IM
pub fn envelope|boundary|convex_hull(g: &GeoGeometry) -> Result<GeoGeometry, GeoError>;
pub fn buffer(g: &GeoGeometry, radius: f64, unit: Unit) -> Result<GeoGeometry, GeoError>;
pub fn intersection|union|difference|sym_difference(a, b) -> Result<GeoGeometry, GeoError>;
pub enum Unit { Metre, Kilometre, Mile, Degree, Radian }        // Unit::from_iri(uom_iri)
// geof::lex::* mirrors every function at the wktLiteral-string level (str in, value/str out).

// --- GeoIndex: R-tree over a Graph (always available) ---
pub fn GeoIndex::build(graph: &sparq_core::Graph) -> GeoIndex;
pub fn within_distance(&self, center: Point<f64>, meters: f64, limit: Option<usize>) -> Vec<(&Term, f64)>;
pub fn nearest(&self, center: Point<f64>, k: usize) -> Vec<(&Term, f64)>;
pub fn intersects(&self, g: &GeoGeometry) -> Vec<&Term>;        // + intersects_wkt(&str)
pub fn apply_delta(&mut self, graph: &Graph, inserts: &[[Term;3]], deletes: &[[Term;3]]);
pub fn entries(&self) -> impl Iterator<Item=&Entry>;  // len() / is_empty() / skipped()
```

Function IRIs are all under `http://www.opengis.net/def/function/geosparql/`. Result types in SPARQL: `geof:distance` → `xsd:double`; the relation families → `xsd:boolean`; `envelope`/`boundary`/`convexHull`/`buffer`/the four set ops → `geo:wktLiteral`; `getSRID` → `xsd:anyURI`.

## Common recipes

**Spatial join — which point is within which polygon (SPARQL):**
```rust
let r = query_with_functions(&g,
  "PREFIX geof: <http://www.opengis.net/def/function/geosparql/>
   SELECT ?city ?region WHERE {
     ?city <http://ex/loc> ?pt . ?region <http://ex/area> ?poly .
     FILTER(geof:sfWithin(?pt, ?poly)) }", &geof_registry()).unwrap();
```

**Chain geometry-producing + relation functions in one expression:** `geof:envelope` / `geof:buffer` return a `geo:wktLiteral` that feeds straight back into another `geof:` call:
```sparql
FILTER(geof:sfWithin(?there, geof:buffer(?here, 400, uom:kilometre)))   # 400 km buffer
FILTER(geof:sfContains(geof:envelope(?poly), ?pt))
```

**Build an R-tree index and run k-NN / radius / intersection queries:**
```rust
use geo_types::Point;
use sparq_geo::{GeoIndex, parse_wkt_literal};

let index = GeoIndex::build(&graph);              // scans geo:asWKT (+ hasGeometry ownership)
let near  = index.nearest(Point::new(2.35, 48.86), 5);            // Vec<(&Term, f64 metres)>
let ball  = index.within_distance(Point::new(2.35, 48.86), 50_000.0, Some(20)); // ≤50 km, top 20
let hits  = index.intersects(&parse_wkt_literal(
                "POLYGON((-1 48, 3 48, 3 49, -1 49, -1 48))").unwrap()); // Vec<&Term>
// Center is a CRS84 long/lat point; distances are great-circle metres, nearest first.
// Antimeridian-crossing balls are handled. Empty/non-geographic literals are skipped() not indexed.
```

**Keep the index in sync with graph updates (no rebuild):** apply the SAME batch to the graph, then to the index:
```rust
graph.apply_delta(&inserts, &deletes).unwrap();   // sparq_core::Graph, &[[Term;3]]
index.apply_delta(&graph, &inserts, &deletes);     // O(batch × log n); non-geo triples are a no-op
```

**Call geof: directly as Rust (no SPARQL engine, works with `--no-default-features`):**
```rust
use sparq_geo::{geof, geof::Unit, parse_wkt_literal};
let a = parse_wkt_literal("POINT(-0.1278 51.5074)").unwrap();
let b = parse_wkt_literal("POINT(2.3522 48.8566)").unwrap();
let km = geof::distance(&a, &b, Unit::Kilometre).unwrap();        // ≈ 343.6
let inside = geof::sf_within(&b, &parse_wkt_literal(
                "POLYGON((-1 42.5,7 42.5,7 51,-1 51,-1 42.5))").unwrap()).unwrap();
```

**Reproject a projected literal into CRS84 (opt-in `reproject` feature):**
```rust
// Cargo: sparq-geo = { path = "...", features = ["reproject"] }
use sparq_geo::reproject::{to_crs84, to_crs84_lex};
let crs84 = to_crs84_lex(
  "<http://www.opengis.net/def/crs/EPSG/0/27700> POINT(530000 180000)").unwrap();
// British National Grid → CRS84 long/lat; then index / metric-distance it.
```

## Gotchas / feature flags / prerequisites

- **`engine` feature (default ON)** pulls in `sparq-engine` and exposes `geof_registry()`. Disable default features (`default-features = false`) for the pure geometry library (WKT literals, `geof::*` as plain Rust, `GeoIndex`) with no engine in the dependency graph — `geof_registry` / the registry then do not exist.
- **`reproject` feature (OFF by default)** adds the `reproject` module (pure-Rust `proj4rs`). It ships only a **small curated EPSG table**: 27700 (British National Grid), 3857 (Web Mercator), 2154 (Lambert-93), 25832/25833 (ETRS89 UTM 32/33N), and WGS84 UTM zones 326xx/327xx. Any other code is `GeoError::Unsupported`.
- **In SPARQL, every `geof:` failure is a per-row EXPRESSION error, not a hard query error**: a wrong datatype (must be a `geo:wktLiteral`), unparseable WKT, CRS mismatch, unknown unit IRI, or an unsupported operand combo drops the row in a `FILTER` / leaves the variable unbound in a `BIND` — the query still succeeds. But an **unregistered** `geof:` IRI (e.g. `geof:gmlToWkt`, which is not implemented) is the engine's usual hard "unsupported SPARQL function" error.
- **Relations are PLANAR DE-9IM** (via `geo`'s `Relate`) in coordinate/degree space, not geodesic. `geof:distance` with metric units (`metre`/`kilometre`/`mile`) requires a geographic CRS (CRS84/EPSG:4326) and is exact haversine when either operand is a point, but uses a **local equirectangular approximation between two extended geometries** (same for `geof:buffer` in metres) — accurate locally, distorted at continental scale or near the poles. `Unit::Degree`/`Radian` measure euclidean coordinate-space distance.
- **Set ops are partial by design.** Polygon×polygon `intersection`/`union`/`difference`/`symDifference` are exact (`geo` `BooleanOps`); point and line cases are handled where well-defined; **1-D set SUBTRACTION (line−line, line−polygon `difference`/`symDifference`) returns an honest `GeoError::Unsupported`** rather than a wrong answer.
- **`GeoIndex` only indexes geographic-CRS, non-empty geometries** reached via `geo:asWKT` (default graph + named graphs), with the indexed *entity* resolved through `geo:hasGeometry` / `geo:hasDefaultGeometry` when present (else the `asWKT` subject itself). Other CRSs, empties, non-`wktLiteral` objects, and unparseable WKT are counted in `index.skipped()` and excluded — check it to detect mis-typed data. `intersects` also requires a geographic-CRS argument.
- **No GML literals, no RIF/`geor:` query rewriting, no `.hdt` write-out.** This is the GeoSPARQL *core*; see the crate `TODO.md` for the boundary.
- **Server wiring:** `sparq-server` exposes exactly this behind its opt-in `geo` cargo feature (`cargo build -p sparq-server --features geo`), which installs `geof_registry()` on every SPARQL endpoint via `with_functions`. With the feature off, the server and the wasm build carry no geometry code.
- **Reuse the registry:** `geof_registry()` is clone-cheap (`Arc`-shared fns) and `Send + Sync` — build it once (e.g. a `OnceLock`) and share across queries and threads.

## See also

- `serve` / `sparql-query` sibling skills for running the engine and writing SPARQL.
- `hdt-format` and `fused-decompress-parse` for getting RDF into the `Graph` that `GeoIndex::build` and `geof_registry()` operate on.
