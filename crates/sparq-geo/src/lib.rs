//! sparq-geo: opt-in GeoSPARQL 1.0/1.1 core for the sparq RDF engine.
//!
//! Three layers, bottom-up:
//!
//! 1. [`literal`] — `geo:wktLiteral` lexical forms <-> [`GeoGeometry`] (a
//!    [`geo_types::Geometry`] tagged with its [`Crs`]). Handles the optional
//!    leading `<CRS-IRI>` per GeoSPARQL Req 10 / section 8.5.1 (default CRS84)
//!    and the EPSG:4326 lat/long axis order.
//! 2. [`geof`] — the `geof:` function namespace: `geof:distance` (with unit
//!    IRIs); the relation families — simple-features `geof:sfEquals` ..
//!    `geof:sfOverlaps`, Egenhofer `geof:eh*`, RCC8 `geof:rcc8*` (all DE-9IM
//!    via `geo`'s `Relate`) plus the generic `geof:relate`; the
//!    geometry-producing `geof:envelope` / `boundary` / `convexHull` /
//!    `buffer` and the point-set operations `geof:intersection` /
//!    `union` / `difference` / `symDifference` (polygon overlay plus the
//!    well-defined line/point cases); and `geof:getSRID`. The
//!    [`geof::lex`] sub-module mirrors every function at the LEXICAL level
//!    (wkt-literal strings in, plain values / wkt-literal strings out) — the
//!    exact shape a SPARQL engine builtin receives; the [`registry`] module
//!    (default-on `engine` feature) packages them as a sparq-engine
//!    [`FunctionRegistry`](sparq_engine::FunctionRegistry), so `geof:`
//!    functions run inside real SPARQL via
//!    [`sparq_engine::query_with_functions`] — see [`geof_registry`].
//! 3. [`index`] — [`GeoIndex`]: extracts `(entity, geometry)` pairs from a
//!    sparq [`sparq_core::Graph`] (via `geo:hasGeometry` /
//!    `geo:hasDefaultGeometry` / `geo:asWKT`; default graph plus named
//!    graphs), bulk-loads an R-tree (`rstar`) over their bounding boxes, and
//!    answers `within_distance` / `nearest` / `intersects` queries returning
//!    entity [`oxrdf::Term`]s (antimeridian-crossing balls handled); deltas
//!    are mirrored incrementally via [`GeoIndex::apply_delta`].
//!
//! The opt-in `reproject` cargo feature adds [`reproject`]: pure-Rust
//! (proj4rs) transformation of projected EPSG literals into CRS84 for a
//! curated EPSG set.
//!
//! No existing sparq crate depends on this one unconditionally (in particular
//! the wasm build carries zero geometry code); spatial support is engaged only
//! by depending on `sparq-geo` — e.g. sparq-server's opt-in `geo` cargo
//! feature, which installs [`geof_registry`] on its SPARQL endpoints.
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

pub mod geof;
pub mod index;
pub mod literal;
#[cfg(feature = "engine")]
pub mod provider;
#[cfg(feature = "engine")]
pub mod registry;
#[cfg(feature = "reproject")]
pub mod reproject;

pub use geof::Unit;
pub use index::GeoIndex;
pub use literal::{parse_wkt_literal, Crs, GeoGeometry};
#[cfg(feature = "engine")]
pub use provider::GeoIndexProvider;
#[cfg(feature = "engine")]
pub use registry::geof_registry;

/// The GeoSPARQL vocabulary IRIs this crate touches.
pub mod vocab {
    /// `geo:` — the GeoSPARQL ontology namespace.
    pub const GEO_NS: &str = "http://www.opengis.net/ont/geosparql#";
    /// `geo:wktLiteral` — the WKT serialization datatype (GeoSPARQL 8.5.1).
    pub const WKT_LITERAL: &str = "http://www.opengis.net/ont/geosparql#wktLiteral";
    /// `geo:asWKT` — geometry node -> WKT serialization (GeoSPARQL 8.5.2).
    pub const AS_WKT: &str = "http://www.opengis.net/ont/geosparql#asWKT";
    /// `geo:hasGeometry` — feature -> geometry node (GeoSPARQL 8.3).
    pub const HAS_GEOMETRY: &str = "http://www.opengis.net/ont/geosparql#hasGeometry";
    /// `geo:hasDefaultGeometry` — feature -> default geometry node (GeoSPARQL 8.3).
    pub const HAS_DEFAULT_GEOMETRY: &str = "http://www.opengis.net/ont/geosparql#hasDefaultGeometry";
    /// `geof:` — the GeoSPARQL function namespace.
    pub const GEOF_NS: &str = "http://www.opengis.net/def/function/geosparql/";
    /// `uom:` — the OGC units-of-measure namespace used by `geof:distance`.
    pub const UOM_NS: &str = "http://www.opengis.net/def/uom/OGC/1.0/";
    /// CRS84 (WGS84 long/lat) — the GeoSPARQL DEFAULT CRS for wktLiterals.
    pub const CRS84: &str = "http://www.opengis.net/def/crs/OGC/1.3/CRS84";
    /// EPSG:4326 (WGS84 LAT/LONG axis order).
    pub const EPSG_4326: &str = "http://www.opengis.net/def/crs/EPSG/0/4326";
}

/// Errors from WKT parsing, `geof:` evaluation, or index queries.
#[derive(Debug, Clone, PartialEq)]
pub enum GeoError {
    /// The wktLiteral lexical form (or its CRS prefix) failed to parse.
    Parse(String),
    /// The two arguments are in incompatible CRSs (no transformation in v1).
    CrsMismatch(String, String),
    /// The operation needs a geographic CRS (CRS84 / EPSG:4326), e.g. metric distance.
    NonGeographicCrs(String),
    /// The unit IRI passed to `geof:distance` is not recognised.
    UnknownUnit(String),
    /// The operation is not defined / not implemented for this geometry.
    Unsupported(String),
}

impl std::fmt::Display for GeoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeoError::Parse(m) => write!(f, "WKT parse error: {m}"),
            GeoError::CrsMismatch(a, b) => write!(f, "CRS mismatch: <{a}> vs <{b}>"),
            GeoError::NonGeographicCrs(c) => {
                write!(f, "operation requires a geographic CRS (CRS84/EPSG:4326), got <{c}>")
            }
            GeoError::UnknownUnit(u) => write!(f, "unknown unit of measure: <{u}>"),
            GeoError::Unsupported(m) => write!(f, "unsupported: {m}"),
        }
    }
}

impl std::error::Error for GeoError {}
