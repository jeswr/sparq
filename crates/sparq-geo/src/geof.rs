//! The `geof:` function namespace (GeoSPARQL 1.0 §8.7 / 1.1 §10).
//!
//! Implemented over [`GeoGeometry`] values:
//!
//! - [`distance`] — `geof:distance(geom1, geom2, units)`. Units are selected
//!   by IRI ([`Unit::from_iri`]). Metric units on geographic-CRS geometries
//!   use the haversine great-circle distance (exact for point/point and
//!   point/geometry via the spherical closest point; a local equirectangular
//!   approximation between two EXTENDED geometries — see [`distance_meters`]).
//! - the eight simple-features relations (`geof:sfEquals`, `sfDisjoint`,
//!   `sfIntersects`, `sfTouches`, `sfCrosses`, `sfWithin`, `sfContains`,
//!   `sfOverlaps`) — DE-9IM intersection matrices via `geo`'s `Relate` — plus
//!   the generic [`relate`] (`geof:relate`, arbitrary DE-9IM pattern) and the
//!   Egenhofer (`geof:eh*`) and RCC8 (`geof:rcc8*`) families (the GeoSPARQL
//!   1.0 Req 25/26 matrix patterns over the same machinery).
//! - [`envelope`] / [`boundary`] / [`convex_hull`] — `geof:envelope`,
//!   `geof:boundary`, `geof:convexHull` — and [`buffer`] (`geof:buffer`, geo
//!   0.33's `Buffer`; metric radii via a local equirectangular frame).
//! - the set operations [`intersection`] / [`union`] / [`difference`] /
//!   [`sym_difference`] — POLYGONAL operands only (`geo`'s `BooleanOps`).
//!
//! The [`lex`] sub-module mirrors every function at the lexical level
//! (wkt-literal strings in, values out) — the shape a SPARQL engine builtin
//! receives. [`crate::registry::geof_registry`] packages these directly as a
//! `sparq_engine::FunctionRegistry`, so they run inside SPARQL FILTER/BIND via
//! `sparq_engine::query_with_functions` (default-on `engine` cargo feature).

use crate::literal::{Crs, GeoGeometry};
use crate::GeoError;
use geo::{
    BooleanOps, BoundingRect, Buffer, Closest, ConvexHull, CoordsIter, Distance, Euclidean,
    Haversine, HaversineClosestPoint, Intersects, MapCoords, Relate,
};
use geo_types::{
    Coord, Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};

/// Mean-Earth-radius metres per degree of arc (GRS80 mean radius 6 371 008.8 m,
/// the same sphere `geo`'s `Haversine` measures on): π·R/180.
const METERS_PER_DEGREE: f64 = std::f64::consts::PI * 6_371_008.8 / 180.0;

/// A unit of measure accepted by `geof:distance`, selected by IRI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Metre,
    Kilometre,
    Mile,
    Degree,
    Radian,
}

impl Unit {
    /// Resolves a unit IRI: the OGC `uom:` registry entries plus the common
    /// QUDT aliases.
    pub fn from_iri(iri: &str) -> Result<Unit, GeoError> {
        match iri {
            "http://www.opengis.net/def/uom/OGC/1.0/metre" | "http://qudt.org/vocab/unit/M" => {
                Ok(Unit::Metre)
            }
            "http://www.opengis.net/def/uom/OGC/1.0/kilometre"
            | "http://qudt.org/vocab/unit/KiloM" => Ok(Unit::Kilometre),
            "http://qudt.org/vocab/unit/MI" => Ok(Unit::Mile),
            "http://www.opengis.net/def/uom/OGC/1.0/degree"
            | "http://qudt.org/vocab/unit/DEG" => Ok(Unit::Degree),
            "http://www.opengis.net/def/uom/OGC/1.0/radian"
            | "http://qudt.org/vocab/unit/RAD" => Ok(Unit::Radian),
            other => Err(GeoError::UnknownUnit(other.to_string())),
        }
    }

    /// Metres per 1.0 of this unit (metric units only).
    fn meters_scale(self) -> Option<f64> {
        match self {
            Unit::Metre => Some(1.0),
            Unit::Kilometre => Some(1000.0),
            Unit::Mile => Some(1609.344),
            Unit::Degree | Unit::Radian => None,
        }
    }
}

/// Both arguments must be in the same coordinate space: the two geographic
/// CRSs (CRS84 / EPSG:4326) are mutually compatible because parsing
/// normalised them to long/lat; any other CRS must match exactly.
fn ensure_compatible(a: &GeoGeometry, b: &GeoGeometry) -> Result<(), GeoError> {
    let compatible = match (&a.crs, &b.crs) {
        (x, y) if x.is_geographic() && y.is_geographic() => true,
        (Crs::Other(x), Crs::Other(y)) => x == y,
        _ => false,
    };
    if compatible {
        Ok(())
    } else {
        Err(GeoError::CrsMismatch(a.crs.iri().to_string(), b.crs.iri().to_string()))
    }
}

/// Euclidean coordinate-space distance between two geometries (degrees for
/// geographic CRSs). Dispatches one side so `geo`'s `Euclidean` pairwise
/// impls (every concrete type vs `&Geometry`) apply.
pub fn euclidean_distance(a: &Geometry<f64>, b: &Geometry<f64>) -> f64 {
    match a {
        Geometry::Point(g) => Euclidean.distance(g, b),
        Geometry::Line(g) => Euclidean.distance(g, b),
        Geometry::LineString(g) => Euclidean.distance(g, b),
        Geometry::Polygon(g) => Euclidean.distance(g, b),
        Geometry::MultiPoint(g) => Euclidean.distance(g, b),
        Geometry::MultiLineString(g) => Euclidean.distance(g, b),
        Geometry::MultiPolygon(g) => Euclidean.distance(g, b),
        Geometry::GeometryCollection(g) => Euclidean.distance(g, b),
        Geometry::Rect(g) => Euclidean.distance(g, b),
        Geometry::Triangle(g) => Euclidean.distance(g, b),
    }
}

/// Great-circle metres from a point to the nearest point of a geometry
/// (haversine on the GRS80 mean sphere; exact for points, spherical
/// closest-point for extended geometries).
pub fn point_to_geometry_meters(p: Point<f64>, g: &Geometry<f64>) -> Result<f64, GeoError> {
    match g.haversine_closest_point(&p) {
        Closest::Intersection(_) => Ok(0.0),
        Closest::SinglePoint(c) => Ok(Haversine.distance(p, c)),
        Closest::Indeterminate => Err(GeoError::Unsupported(
            "closest point is indeterminate (degenerate geometry)".to_string(),
        )),
    }
}

/// Great-circle metres between two geometries in a GEOGRAPHIC CRS (long/lat
/// degrees). Exact (haversine) when either side is a point; between two
/// extended geometries it falls back to a local equirectangular projection
/// about the geometries' mean latitude — accurate at local scale, documented
/// as an approximation in the README.
pub fn distance_meters(a: &Geometry<f64>, b: &Geometry<f64>) -> Result<f64, GeoError> {
    if a.intersects(b) {
        return Ok(0.0);
    }
    match (a, b) {
        (Geometry::Point(p), _) => point_to_geometry_meters(*p, b),
        (_, Geometry::Point(p)) => point_to_geometry_meters(*p, a),
        _ => {
            let (ra, rb) = match (a.bounding_rect(), b.bounding_rect()) {
                (Some(ra), Some(rb)) => (ra, rb),
                _ => {
                    return Err(GeoError::Unsupported(
                        "distance between empty geometries".to_string(),
                    ))
                }
            };
            let lat0 = (ra.center().y + rb.center().y) / 2.0;
            let kx = METERS_PER_DEGREE * lat0.to_radians().cos();
            let project = |g: &Geometry<f64>| {
                g.map_coords(|c: Coord<f64>| Coord { x: c.x * kx, y: c.y * METERS_PER_DEGREE })
            };
            Ok(euclidean_distance(&project(a), &project(b)))
        }
    }
}

/// `geof:distance(geom1, geom2, units)`.
///
/// - Metric units ([`Unit::Metre`] / `Kilometre` / `Mile`) require a
///   geographic CRS and measure great-circle distance (see
///   [`distance_meters`]).
/// - [`Unit::Degree`] / [`Unit::Radian`] measure euclidean coordinate-space
///   distance (degrees of arc for geographic CRSs; raw coordinate units for
///   [`Crs::Other`]).
pub fn distance(a: &GeoGeometry, b: &GeoGeometry, unit: Unit) -> Result<f64, GeoError> {
    ensure_compatible(a, b)?;
    match unit.meters_scale() {
        Some(scale) => {
            if !a.crs.is_geographic() {
                return Err(GeoError::NonGeographicCrs(a.crs.iri().to_string()));
            }
            Ok(distance_meters(&a.geometry, &b.geometry)? / scale)
        }
        None => {
            let d = euclidean_distance(&a.geometry, &b.geometry);
            Ok(match unit {
                Unit::Radian => d.to_radians(),
                _ => d,
            })
        }
    }
}

// ---- Simple-features relations (DE-9IM via geo's Relate) -------------------------

macro_rules! sf_relation {
    ($(#[$doc:meta])* $name:ident, $pred:ident) => {
        $(#[$doc])*
        pub fn $name(a: &GeoGeometry, b: &GeoGeometry) -> Result<bool, GeoError> {
            ensure_compatible(a, b)?;
            Ok(a.geometry.relate(&b.geometry).$pred())
        }
    };
}

sf_relation!(
    /// `geof:sfEquals` — topologically equal (same point set).
    sf_equals, is_equal_topo
);
sf_relation!(
    /// `geof:sfDisjoint` — no points in common.
    sf_disjoint, is_disjoint
);
sf_relation!(
    /// `geof:sfIntersects` — at least one point in common.
    sf_intersects, is_intersects
);
sf_relation!(
    /// `geof:sfTouches` — boundaries touch, interiors do not intersect.
    sf_touches, is_touches
);
sf_relation!(
    /// `geof:sfCrosses` — interiors intersect with lower-dimensional result.
    sf_crosses, is_crosses
);
sf_relation!(
    /// `geof:sfWithin` — `a` lies in the interior+boundary of `b`.
    sf_within, is_within
);
sf_relation!(
    /// `geof:sfContains` — `b` lies in the interior+boundary of `a` (and
    /// their interiors intersect).
    sf_contains, is_contains
);
sf_relation!(
    /// `geof:sfOverlaps` — same dimension, interiors intersect, neither
    /// contains the other.
    sf_overlaps, is_overlaps
);

// ---- Generic DE-9IM + Egenhofer / RCC8 relation families ---------------------------

/// `geof:relate` — generic DE-9IM pattern test (GeoSPARQL 1.0 §9 / Simple
/// Features `Relate`): `true` iff the intersection matrix of `a` against `b`
/// matches `pattern` (nine of `T` / `F` / `*` / `0` / `1` / `2`).
pub fn relate(a: &GeoGeometry, b: &GeoGeometry, pattern: &str) -> Result<bool, GeoError> {
    ensure_compatible(a, b)?;
    a.geometry
        .relate(&b.geometry)
        .matches(pattern)
        .map_err(|e| GeoError::Parse(format!("invalid DE-9IM pattern {pattern:?}: {e}")))
}

/// `true` iff the DE-9IM matrix of `a` vs `b` matches ANY of `patterns`
/// (the spec defines some relations as a disjunction of matrices).
fn relate_any(a: &GeoGeometry, b: &GeoGeometry, patterns: &[&str]) -> Result<bool, GeoError> {
    ensure_compatible(a, b)?;
    let matrix = a.geometry.relate(&b.geometry);
    for p in patterns {
        // Patterns are compile-time constants below — a failure is a crate bug.
        if matrix.matches(p).expect("valid built-in DE-9IM pattern") {
            return Ok(true);
        }
    }
    Ok(false)
}

macro_rules! de9im_relation {
    ($(#[$doc:meta])* $name:ident, [$($pattern:literal),+]) => {
        $(#[$doc])*
        pub fn $name(a: &GeoGeometry, b: &GeoGeometry) -> Result<bool, GeoError> {
            relate_any(a, b, &[$($pattern),+])
        }
    };
}

// The Egenhofer relation family (GeoSPARQL 1.0 Req 25 / 1.1 §9 — the standard
// DE-9IM matrix patterns for each relation).
de9im_relation!(
    /// `geof:ehEquals` — Egenhofer equal.
    eh_equals, ["TFFFTFFFT"]
);
de9im_relation!(
    /// `geof:ehDisjoint` — Egenhofer disjoint.
    eh_disjoint, ["FF*FF****"]
);
de9im_relation!(
    /// `geof:ehMeet` — Egenhofer meet (boundaries in contact, interiors not).
    eh_meet, ["FT*******", "F**T*****", "F***T****"]
);
de9im_relation!(
    /// `geof:ehOverlap` — Egenhofer overlap.
    eh_overlap, ["T*T***T**"]
);
de9im_relation!(
    /// `geof:ehCovers` — Egenhofer covers.
    eh_covers, ["T*TFT*FF*"]
);
de9im_relation!(
    /// `geof:ehCoveredBy` — Egenhofer coveredBy.
    eh_covered_by, ["TFF*TFT**"]
);
de9im_relation!(
    /// `geof:ehInside` — Egenhofer inside.
    eh_inside, ["TFF*FFT**"]
);
de9im_relation!(
    /// `geof:ehContains` — Egenhofer contains.
    eh_contains, ["T*TFF*FF*"]
);

// The RCC8 relation family (GeoSPARQL 1.0 Req 26 / 1.1 §9). RCC8 is defined
// over REGIONS (non-empty interiors); the matrices below are the spec's.
de9im_relation!(
    /// `geof:rcc8eq` — equal.
    rcc8_eq, ["TFFFTFFFT"]
);
de9im_relation!(
    /// `geof:rcc8dc` — disconnected.
    rcc8_dc, ["FFTFFTTTT"]
);
de9im_relation!(
    /// `geof:rcc8ec` — externally connected (boundaries touch).
    rcc8_ec, ["FFTFTTTTT"]
);
de9im_relation!(
    /// `geof:rcc8po` — partially overlapping.
    rcc8_po, ["TTTTTTTTT"]
);
de9im_relation!(
    /// `geof:rcc8tppi` — tangential proper part inverse.
    rcc8_tppi, ["TTTFTTFFT"]
);
de9im_relation!(
    /// `geof:rcc8tpp` — tangential proper part.
    rcc8_tpp, ["TFFTTFTTT"]
);
de9im_relation!(
    /// `geof:rcc8ntpp` — non-tangential proper part.
    rcc8_ntpp, ["TFFTFFTTT"]
);
de9im_relation!(
    /// `geof:rcc8ntppi` — non-tangential proper part inverse.
    rcc8_ntppi, ["TTTFFTFFT"]
);

// ---- Geometry-producing functions -------------------------------------------------

/// `geof:envelope` — the minimum bounding rectangle, as a polygon (degenerate
/// for points / vertical / horizontal inputs, matching the spec's "envelope").
pub fn envelope(g: &GeoGeometry) -> Result<GeoGeometry, GeoError> {
    let rect = g
        .geometry
        .bounding_rect()
        .ok_or_else(|| GeoError::Unsupported("empty geometry has no envelope".to_string()))?;
    Ok(GeoGeometry { crs: g.crs.clone(), geometry: Geometry::Polygon(rect.to_polygon()) })
}

/// `geof:convexHull` — the convex hull of the geometry's coordinates.
pub fn convex_hull(g: &GeoGeometry) -> Result<GeoGeometry, GeoError> {
    if g.geometry.coords_count() == 0 {
        return Err(GeoError::Unsupported("empty geometry has no convex hull".to_string()));
    }
    let points: MultiPoint<f64> = g.geometry.coords_iter().map(Point::from).collect();
    Ok(GeoGeometry { crs: g.crs.clone(), geometry: Geometry::Polygon(points.convex_hull()) })
}

/// `geof:boundary` — the simple-features boundary:
///
/// - points / multipoints: empty (an empty MULTIPOINT),
/// - curves: the end points appearing an ODD number of times (mod-2 rule;
///   closed curves have an empty boundary),
/// - surfaces: the exterior + interior rings as a MULTILINESTRING,
/// - geometry collections: unsupported in v1 (heterogeneous boundary).
pub fn boundary(g: &GeoGeometry) -> Result<GeoGeometry, GeoError> {
    let geometry = boundary_geometry(&g.geometry)?;
    Ok(GeoGeometry { crs: g.crs.clone(), geometry })
}

fn boundary_geometry(g: &Geometry<f64>) -> Result<Geometry<f64>, GeoError> {
    /// Mod-2 endpoint rule over a set of curves.
    fn curve_boundary<'a>(curves: impl Iterator<Item = &'a LineString<f64>>) -> Geometry<f64> {
        let mut counts: Vec<(Coord<f64>, usize)> = Vec::new();
        let mut bump = |c: Coord<f64>| match counts.iter_mut().find(|(x, _)| *x == c) {
            Some((_, n)) => *n += 1,
            None => counts.push((c, 1)),
        };
        for ls in curves {
            if ls.0.len() < 2 || ls.is_closed() {
                continue;
            }
            bump(ls.0[0]);
            bump(*ls.0.last().unwrap());
        }
        let pts: Vec<Point<f64>> =
            counts.into_iter().filter(|(_, n)| n % 2 == 1).map(|(c, _)| Point::from(c)).collect();
        Geometry::MultiPoint(MultiPoint(pts))
    }

    fn rings(p: &Polygon<f64>) -> impl Iterator<Item = LineString<f64>> + '_ {
        std::iter::once(p.exterior().clone()).chain(p.interiors().iter().cloned())
    }

    Ok(match g {
        Geometry::Point(_) | Geometry::MultiPoint(_) => Geometry::MultiPoint(MultiPoint(vec![])),
        Geometry::Line(l) => {
            Geometry::MultiPoint(MultiPoint(vec![l.start_point(), l.end_point()]))
        }
        Geometry::LineString(ls) => curve_boundary(std::iter::once(ls)),
        Geometry::MultiLineString(mls) => curve_boundary(mls.0.iter()),
        Geometry::Polygon(p) => Geometry::MultiLineString(MultiLineString(rings(p).collect())),
        Geometry::MultiPolygon(mp) => {
            Geometry::MultiLineString(MultiLineString(mp.0.iter().flat_map(rings).collect()))
        }
        Geometry::Rect(r) => boundary_geometry(&Geometry::Polygon(r.to_polygon()))?,
        Geometry::Triangle(t) => boundary_geometry(&Geometry::Polygon(t.to_polygon()))?,
        Geometry::GeometryCollection(_) => {
            return Err(GeoError::Unsupported(
                "geof:boundary of a GEOMETRYCOLLECTION".to_string(),
            ))
        }
    })
}

// ---- Set operations (polygonal subset via geo's BooleanOps) -----------------------

/// The geometry as a multipolygon, if it is polygonal (POLYGON, MULTIPOLYGON,
/// or the rect/triangle shorthands). `None` for points, curves, collections.
fn polygonal(g: &Geometry<f64>) -> Option<MultiPolygon<f64>> {
    match g {
        Geometry::Polygon(p) => Some(MultiPolygon(vec![p.clone()])),
        Geometry::MultiPolygon(mp) => Some(mp.clone()),
        Geometry::Rect(r) => Some(MultiPolygon(vec![r.to_polygon()])),
        Geometry::Triangle(t) => Some(MultiPolygon(vec![t.to_polygon()])),
        _ => None,
    }
}

/// Shared front end of the four set operations: CRS compatibility + the
/// POLYGONAL-OPERANDS-ONLY restriction (geo's `BooleanOps` is a polygon
/// overlay; general line/point overlays are not implemented — see TODO.md).
fn polygonal_pair(
    name: &str,
    a: &GeoGeometry,
    b: &GeoGeometry,
) -> Result<(MultiPolygon<f64>, MultiPolygon<f64>), GeoError> {
    ensure_compatible(a, b)?;
    match (polygonal(&a.geometry), polygonal(&b.geometry)) {
        (Some(pa), Some(pb)) => Ok((pa, pb)),
        _ => Err(GeoError::Unsupported(format!(
            "geof:{name} supports polygonal operands only (POLYGON/MULTIPOLYGON), got {} and {}",
            wkt_type_name(&a.geometry),
            wkt_type_name(&b.geometry)
        ))),
    }
}

/// The WKT keyword for a geometry variant (for error messages).
fn wkt_type_name(g: &Geometry<f64>) -> &'static str {
    match g {
        Geometry::Point(_) => "POINT",
        Geometry::Line(_) | Geometry::LineString(_) => "LINESTRING",
        Geometry::Polygon(_) | Geometry::Rect(_) | Geometry::Triangle(_) => "POLYGON",
        Geometry::MultiPoint(_) => "MULTIPOINT",
        Geometry::MultiLineString(_) => "MULTILINESTRING",
        Geometry::MultiPolygon(_) => "MULTIPOLYGON",
        Geometry::GeometryCollection(_) => "GEOMETRYCOLLECTION",
    }
}

macro_rules! set_operation {
    ($(#[$doc:meta])* $name:ident, $geof_name:literal, $op:ident) => {
        $(#[$doc])*
        ///
        /// POLYGONAL operands only (the result is a MULTIPOLYGON in the
        /// operands' CRS); any other operand type is a [`GeoError::Unsupported`].
        pub fn $name(a: &GeoGeometry, b: &GeoGeometry) -> Result<GeoGeometry, GeoError> {
            let (pa, pb) = polygonal_pair($geof_name, a, b)?;
            Ok(GeoGeometry { crs: a.crs.clone(), geometry: Geometry::MultiPolygon(pa.$op(&pb)) })
        }
    };
}

set_operation!(
    /// `geof:intersection` — points in both `a` and `b`.
    intersection, "intersection", intersection
);
set_operation!(
    /// `geof:union` — points in `a` or `b`.
    union, "union", union
);
set_operation!(
    /// `geof:difference` — points in `a` but not in `b`.
    difference, "difference", difference
);
set_operation!(
    /// `geof:symDifference` — points in exactly one of `a`, `b`.
    sym_difference, "symDifference", xor
);

// ---- geof:buffer -------------------------------------------------------------------

/// `geof:buffer(geom, radius, units)` — all points within `radius` of the
/// geometry, as a MULTIPOLYGON (geo 0.33's `Buffer`, rounded joins/caps).
///
/// - Metric units require a geographic CRS: the geometry is projected into a
///   LOCAL EQUIRECTANGULAR metre frame about its mean latitude, buffered
///   there, and unprojected — accurate at local scale (the same approximation
///   as extended-extended [`distance_meters`]), increasingly distorted for
///   continent-scale geometries or near the poles.
/// - [`Unit::Degree`] / [`Unit::Radian`] buffer in euclidean coordinate space
///   (degrees for geographic CRSs, raw units for [`Crs::Other`]).
pub fn buffer(g: &GeoGeometry, radius: f64, unit: Unit) -> Result<GeoGeometry, GeoError> {
    let buffered = match unit.meters_scale() {
        Some(scale) => {
            if !g.crs.is_geographic() {
                return Err(GeoError::NonGeographicCrs(g.crs.iri().to_string()));
            }
            let rect = g.geometry.bounding_rect().ok_or_else(|| {
                GeoError::Unsupported("geof:buffer of an empty geometry".to_string())
            })?;
            // Local equirectangular frame about the geometry's mean latitude.
            let kx = METERS_PER_DEGREE * rect.center().y.to_radians().cos();
            if kx <= 0.0 {
                return Err(GeoError::Unsupported(
                    "geof:buffer with metric units at the poles".to_string(),
                ));
            }
            let meters = radius * scale;
            let projected =
                g.geometry.map_coords(|c| Coord { x: c.x * kx, y: c.y * METERS_PER_DEGREE });
            projected
                .buffer(meters)
                .map_coords(|c| Coord { x: c.x / kx, y: c.y / METERS_PER_DEGREE })
        }
        None => {
            let d = match unit {
                Unit::Radian => radius.to_degrees(),
                _ => radius,
            };
            g.geometry.buffer(d)
        }
    };
    Ok(GeoGeometry { crs: g.crs.clone(), geometry: Geometry::MultiPolygon(buffered) })
}

// ---- Lexical-level mirrors (the engine-builtin shape) ------------------------------

/// Every `geof:` function at the LEXICAL level: arguments and results are
/// `geo:wktLiteral` lexical forms (plus plain `f64` / `bool`), exactly what a
/// SPARQL engine builtin sees after evaluating its argument expressions to
/// literals. sparq-engine can register these one-to-one once it has an
/// extension-function registry (the required API is recorded in TODO.md).
pub mod lex {
    use super::Unit;
    use crate::literal::parse_wkt_literal;
    use crate::GeoError;

    /// `geof:distance(?a, ?b, ?unitIri)`.
    pub fn distance(a: &str, b: &str, unit_iri: &str) -> Result<f64, GeoError> {
        super::distance(&parse_wkt_literal(a)?, &parse_wkt_literal(b)?, Unit::from_iri(unit_iri)?)
    }

    macro_rules! lex_relation {
        ($(#[$doc:meta])* $name:ident) => {
            $(#[$doc])*
            pub fn $name(a: &str, b: &str) -> Result<bool, GeoError> {
                super::$name(&parse_wkt_literal(a)?, &parse_wkt_literal(b)?)
            }
        };
    }

    lex_relation!(
        /// `geof:sfEquals(?a, ?b)`.
        sf_equals
    );
    lex_relation!(
        /// `geof:sfDisjoint(?a, ?b)`.
        sf_disjoint
    );
    lex_relation!(
        /// `geof:sfIntersects(?a, ?b)`.
        sf_intersects
    );
    lex_relation!(
        /// `geof:sfTouches(?a, ?b)`.
        sf_touches
    );
    lex_relation!(
        /// `geof:sfCrosses(?a, ?b)`.
        sf_crosses
    );
    lex_relation!(
        /// `geof:sfWithin(?a, ?b)`.
        sf_within
    );
    lex_relation!(
        /// `geof:sfContains(?a, ?b)`.
        sf_contains
    );
    lex_relation!(
        /// `geof:sfOverlaps(?a, ?b)`.
        sf_overlaps
    );

    macro_rules! lex_geometry_fn {
        ($(#[$doc:meta])* $name:ident) => {
            $(#[$doc])*
            pub fn $name(a: &str) -> Result<String, GeoError> {
                Ok(super::$name(&parse_wkt_literal(a)?)?.to_wkt_literal())
            }
        };
    }

    lex_geometry_fn!(
        /// `geof:envelope(?a)` -> wktLiteral lexical form.
        envelope
    );
    lex_geometry_fn!(
        /// `geof:convexHull(?a)` -> wktLiteral lexical form.
        convex_hull
    );
    lex_geometry_fn!(
        /// `geof:boundary(?a)` -> wktLiteral lexical form.
        boundary
    );

    /// `geof:getSRID(?a)` -> the geometry's CRS IRI (an `xsd:anyURI` value).
    pub fn get_srid(a: &str) -> Result<String, GeoError> {
        Ok(parse_wkt_literal(a)?.crs.iri().to_string())
    }

    /// `geof:relate(?a, ?b, ?de9imPattern)`.
    pub fn relate(a: &str, b: &str, pattern: &str) -> Result<bool, GeoError> {
        super::relate(&parse_wkt_literal(a)?, &parse_wkt_literal(b)?, pattern)
    }

    lex_relation!(
        /// `geof:ehEquals(?a, ?b)`.
        eh_equals
    );
    lex_relation!(
        /// `geof:ehDisjoint(?a, ?b)`.
        eh_disjoint
    );
    lex_relation!(
        /// `geof:ehMeet(?a, ?b)`.
        eh_meet
    );
    lex_relation!(
        /// `geof:ehOverlap(?a, ?b)`.
        eh_overlap
    );
    lex_relation!(
        /// `geof:ehCovers(?a, ?b)`.
        eh_covers
    );
    lex_relation!(
        /// `geof:ehCoveredBy(?a, ?b)`.
        eh_covered_by
    );
    lex_relation!(
        /// `geof:ehInside(?a, ?b)`.
        eh_inside
    );
    lex_relation!(
        /// `geof:ehContains(?a, ?b)`.
        eh_contains
    );
    lex_relation!(
        /// `geof:rcc8eq(?a, ?b)`.
        rcc8_eq
    );
    lex_relation!(
        /// `geof:rcc8dc(?a, ?b)`.
        rcc8_dc
    );
    lex_relation!(
        /// `geof:rcc8ec(?a, ?b)`.
        rcc8_ec
    );
    lex_relation!(
        /// `geof:rcc8po(?a, ?b)`.
        rcc8_po
    );
    lex_relation!(
        /// `geof:rcc8tppi(?a, ?b)`.
        rcc8_tppi
    );
    lex_relation!(
        /// `geof:rcc8tpp(?a, ?b)`.
        rcc8_tpp
    );
    lex_relation!(
        /// `geof:rcc8ntpp(?a, ?b)`.
        rcc8_ntpp
    );
    lex_relation!(
        /// `geof:rcc8ntppi(?a, ?b)`.
        rcc8_ntppi
    );

    macro_rules! lex_set_operation {
        ($(#[$doc:meta])* $name:ident) => {
            $(#[$doc])*
            pub fn $name(a: &str, b: &str) -> Result<String, GeoError> {
                Ok(super::$name(&parse_wkt_literal(a)?, &parse_wkt_literal(b)?)?.to_wkt_literal())
            }
        };
    }

    lex_set_operation!(
        /// `geof:intersection(?a, ?b)` -> wktLiteral lexical form (polygonal operands only).
        intersection
    );
    lex_set_operation!(
        /// `geof:union(?a, ?b)` -> wktLiteral lexical form (polygonal operands only).
        union
    );
    lex_set_operation!(
        /// `geof:difference(?a, ?b)` -> wktLiteral lexical form (polygonal operands only).
        difference
    );
    lex_set_operation!(
        /// `geof:symDifference(?a, ?b)` -> wktLiteral lexical form (polygonal operands only).
        sym_difference
    );

    /// `geof:buffer(?a, ?radius, ?unitIri)` -> wktLiteral lexical form.
    pub fn buffer(a: &str, radius: f64, unit_iri: &str) -> Result<String, GeoError> {
        Ok(super::buffer(&parse_wkt_literal(a)?, radius, Unit::from_iri(unit_iri)?)?
            .to_wkt_literal())
    }
}
