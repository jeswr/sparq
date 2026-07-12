//! Opt-in CRS reprojection (`reproject` cargo feature): transform a
//! [`GeoGeometry`] written in a PROJECTED or non-default GEOGRAPHIC CRS into
//! CRS84 (WGS84 long/lat), so it can join the geographic machinery — metric
//! `geof:distance`, the [`GeoIndex`](crate::GeoIndex), buffer in metres.
//!
//! Transformation is pure Rust via [`proj4rs`] (a Rust port of proj.4 — NOT
//! the C `proj` library, keeping the workspace pure-Rust and wasm-friendly).
//! proj4rs evaluates proj-strings but ships no EPSG database, so this module
//! carries a SMALL CURATED TABLE of proj4 definitions ([`proj4_definition`])
//! for the CRSs that actually show up in published RDF.
//!
//! Projected CRSs (metre-valued coordinates):
//!
//! - EPSG:27700 — British National Grid (OSGB36, 7-param Helmert; verified
//!   against the Ordnance Survey worked example to ~1e-6°),
//! - EPSG:3857 — Web Mercator,
//! - EPSG:2154 — RGF93 / Lambert-93 (France),
//! - EPSG:25832 / 25833 — ETRS89 / UTM 32N, 33N (German/Nordic open data),
//! - EPSG:32601–32660 and 32701–32760 — WGS84 / UTM north + south zones.
//!
//! Geographic CRSs (degree-valued coordinates, sq-ove): the EPSG registry
//! defines these with LAT/LONG axis order, which GeoSPARQL honours — the
//! EPSG:4326 convention, but parsing only axis-normalises 4326 itself (other
//! codes are [`Crs::Other`], kept verbatim). The curated axis-order map
//! ([`geographic_axis_order`]) lets [`to_crs84`] normalise them too:
//!
//! - EPSG:4258 — ETRS89, EPSG:4269 — NAD83, EPSG:4283 — GDA94,
//!   EPSG:4171 — RGF93, EPSG:4490 — CGCS2000 (all lat/long).
//!
//! These datums are treated as COINCIDENT with WGS84 (`+towgs84=0,0,0`) — the
//! standard proj convention, accurate to a metre or two (plate motion since
//! each datum's epoch). EPSG:4267 (NAD27) is deliberately NOT curated: a
//! correct NAD27 transform needs NADCON grid shifts (tens of metres), which
//! this crate does not ship — it stays an explicit [`GeoError::Unsupported`].
//!
//! CRS84 / EPSG:4326 inputs pass through unchanged (parsing already
//! normalised them to long/lat). Anything else is a
//! [`GeoError::Unsupported`] naming the missing EPSG code — extend the table
//! as needed; correctness of a new entry is one worked example away.

use crate::literal::{Crs, GeoGeometry};
use crate::GeoError;
use geo::MapCoords;
use geo_types::Coord;
use proj4rs::Proj;

/// The coordinate axis order a GEOGRAPHIC CRS's authority definition uses for
/// its lexical forms (GeoSPARQL honours the authority order in wktLiterals).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisOrder {
    /// Latitude first (the EPSG registry convention for geographic 2D CRSs).
    LatLong,
    /// Longitude first (the OGC CRS84 / internal convention).
    LongLat,
}

/// The wktLiteral axis order of a curated GEOGRAPHIC (degree-valued) EPSG CRS.
///
/// `Some(order)` marks the code as geographic — [`to_crs84`] converts its
/// coordinates from `order` degrees into long/lat radians before the proj4
/// transform. `None` means "not a curated geographic CRS" (projected codes
/// like 27700 land here: their metre coordinates are fed to proj4 verbatim).
///
/// Hand-maintained alongside [`proj4_definition`]; every geographic entry
/// there must appear here (that pairing is unit-tested).
pub fn geographic_axis_order(epsg: u32) -> Option<AxisOrder> {
    match epsg {
        // WGS84 itself — axis-normalised on parse, listed for introspection.
        4326 => Some(AxisOrder::LatLong),
        // ETRS89, NAD83, GDA94, RGF93, CGCS2000 — EPSG geographic 2D,
        // lat/long per the registry.
        4258 | 4269 | 4283 | 4171 | 4490 => Some(AxisOrder::LatLong),
        _ => None,
    }
}

/// The proj4 definition string for a supported EPSG code.
pub fn proj4_definition(epsg: u32) -> Option<String> {
    match epsg {
        // British National Grid (OSGB36 -> WGS84 7-param Helmert).
        27700 => Some(
            "+proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 \
             +ellps=airy +towgs84=446.448,-125.157,542.06,0.15,0.247,0.842,-20.489 \
             +units=m +no_defs"
                .to_string(),
        ),
        // Web Mercator (spherical, the tile-map CRS).
        3857 => Some(
            "+proj=merc +a=6378137 +b=6378137 +lat_ts=0 +lon_0=0 +x_0=0 +y_0=0 +k=1 \
             +units=m +nadgrids=@null +no_defs"
                .to_string(),
        ),
        // RGF93 / Lambert-93 (ETRS89-based; null Helmert to WGS84).
        2154 => Some(
            "+proj=lcc +lat_0=46.5 +lon_0=3 +lat_1=49 +lat_2=44 +x_0=700000 +y_0=6600000 \
             +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs"
                .to_string(),
        ),
        // ETRS89 / UTM zones 32N, 33N.
        25832 | 25833 => Some(format!(
            "+proj=utm +zone={} +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs",
            epsg - 25800
        )),
        // WGS84 / UTM north (326xx) and south (327xx) zones.
        32601..=32660 => Some(format!(
            "+proj=utm +zone={} +datum=WGS84 +units=m +no_defs",
            epsg - 32600
        )),
        32701..=32760 => Some(format!(
            "+proj=utm +zone={} +south +datum=WGS84 +units=m +no_defs",
            epsg - 32700
        )),
        // Geographic CRSs (see geographic_axis_order). WGS84 itself:
        4326 => Some("+proj=longlat +datum=WGS84 +no_defs".to_string()),
        // ETRS89 / NAD83 / GDA94 / RGF93 / CGCS2000: GRS80-family ellipsoids
        // (CGCS2000's ellipsoid shares GRS80's a and 1/f to the published
        // precision), null Helmert — coincident with WGS84 at metre level.
        4258 | 4269 | 4283 | 4171 | 4490 => {
            Some("+proj=longlat +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +no_defs".to_string())
        }
        _ => None,
    }
}

/// The EPSG code of an OGC CRS IRI (`…/def/crs/EPSG/0/<code>`), if it is one.
pub fn epsg_code(crs: &Crs) -> Option<u32> {
    crs.iri()
        .strip_prefix("http://www.opengis.net/def/crs/EPSG/0/")
        .and_then(|c| c.parse().ok())
}

/// Reprojects a geometry into CRS84 (WGS84 long/lat degrees).
///
/// CRS84 / EPSG:4326 inputs are returned as-is (re-tagged CRS84 —
/// coordinates were already normalised to long/lat on parse). Anything else
/// must be an EPSG CRS in the curated [`proj4_definition`] table — projected
/// codes are transformed from their metre coordinates; curated GEOGRAPHIC
/// codes (EPSG:4258/4269/4283/4171/4490) are first axis-normalised from the
/// registry's lat/long order ([`geographic_axis_order`]) and then
/// datum-shifted. Anything else is [`GeoError::Unsupported`].
pub fn to_crs84(g: &GeoGeometry) -> Result<GeoGeometry, GeoError> {
    if g.crs.is_geographic() {
        return Ok(GeoGeometry {
            crs: Crs::Crs84,
            geometry: g.geometry.clone(),
        });
    }
    let epsg = epsg_code(&g.crs).ok_or_else(|| {
        GeoError::Unsupported(format!(
            "reprojection needs an EPSG CRS IRI ({}/<code>), got <{}>",
            "http://www.opengis.net/def/crs/EPSG/0",
            g.crs.iri()
        ))
    })?;
    let def = proj4_definition(epsg).ok_or_else(|| {
        GeoError::Unsupported(format!(
            "no built-in proj4 definition for EPSG:{epsg} (supported: 27700, 3857, 2154, \
             25832, 25833, 326xx/327xx UTM; geographic 4258, 4269, 4283, 4171, 4490)"
        ))
    })?;
    let src = Proj::from_proj_string(&def)
        .map_err(|e| GeoError::Parse(format!("proj4 definition for EPSG:{epsg}: {e}")))?;
    let dst = Proj::from_proj_string("+proj=longlat +datum=WGS84 +no_defs")
        .map_err(|e| GeoError::Parse(format!("proj4 WGS84 definition: {e}")))?;
    let src_axis = geographic_axis_order(epsg);

    // proj4rs transforms in place; failures (e.g. coordinates outside the
    // projection's domain) abort the whole geometry.
    let geometry = g
        .geometry
        .try_map_coords(|c: Coord<f64>| -> Result<Coord<f64>, GeoError> {
            // Normalise the input into proj's convention: geographic sources are
            // written in the authority axis order in DEGREES but proj4rs consumes
            // long/lat RADIANS; projected sources are metres, fed verbatim.
            let (x, y) = match src_axis {
                Some(AxisOrder::LatLong) => (c.y.to_radians(), c.x.to_radians()),
                Some(AxisOrder::LongLat) => (c.x.to_radians(), c.y.to_radians()),
                None => (c.x, c.y),
            };
            let mut p = (x, y, 0.0);
            proj4rs::transform::transform(&src, &dst, &mut p).map_err(|e| {
                GeoError::Unsupported(format!("EPSG:{epsg} -> CRS84 transform failed: {e}"))
            })?;
            // Geographic outputs are radians in proj convention.
            Ok(Coord {
                x: p.0.to_degrees(),
                y: p.1.to_degrees(),
            })
        })?;
    Ok(GeoGeometry {
        crs: Crs::Crs84,
        geometry,
    })
}

/// Lexical-level mirror of [`to_crs84`]: a wktLiteral lexical form in, the
/// reprojected CRS84 lexical form out — the shape an engine builtin or a
/// pre-indexing normalisation pass consumes.
pub fn to_crs84_lex(lex: &str) -> Result<String, GeoError> {
    Ok(to_crs84(&crate::parse_wkt_literal(lex)?)?.to_wkt_literal())
}
