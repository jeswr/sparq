#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

pub mod geof;
pub mod gml; // [OPUS-4.8] sq-zy0: GML-SF geometry parser (geo:gmlLiteral)
pub mod index;
pub mod literal;
#[cfg(feature = "engine")]
pub mod provider;
#[cfg(feature = "engine")]
pub mod registry;
#[cfg(feature = "reproject")]
pub mod reproject;

pub use geof::Unit;
pub use gml::parse_gml_literal; // [OPUS-4.8]
pub use index::GeoIndex;
pub use literal::{
    is_geometry_datatype, parse_geometry_literal, parse_wkt_literal, Crs, GeoGeometry,
}; // [OPUS-4.8]
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
    /// `geo:gmlLiteral` — the GML serialization datatype (GeoSPARQL 8.5.2). [OPUS-4.8]
    pub const GML_LITERAL: &str = "http://www.opengis.net/ont/geosparql#gmlLiteral";
    /// `gml:` — the GML 3.2 namespace (GML-SF geometry elements). [OPUS-4.8]
    pub const GML_NS: &str = "http://www.opengis.net/gml/3.2";
    /// `geo:asWKT` — geometry node -> WKT serialization (GeoSPARQL 8.5.2).
    pub const AS_WKT: &str = "http://www.opengis.net/ont/geosparql#asWKT";
    /// `geo:asGML` — geometry node -> GML serialization (GeoSPARQL 8.5.2). [OPUS-4.8]
    pub const AS_GML: &str = "http://www.opengis.net/ont/geosparql#asGML";
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

/// Errors from WKT/GML parsing, `geof:` evaluation, or index queries.
#[derive(Debug, Clone, PartialEq)]
pub enum GeoError {
    /// A geometry literal lexical form (wktLiteral CRS prefix / WKT body, or a
    /// gmlLiteral GML element) failed to parse.
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
            GeoError::Parse(m) => write!(f, "geometry parse error: {m}"),
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
