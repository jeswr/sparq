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
//!   [`sym_difference`] — point-set operations over `geo`'s `BooleanOps`
//!   (polygon overlay) plus directly-implemented line/point cases: point-in/on
//!   tests, line-to-polygon clipping, and line∩line via `geo`'s
//!   `line_intersection`. The genuinely-intractable combinations (1-D
//!   set-subtraction) return an honest [`GeoError::Unsupported`]; see each
//!   function's docs for the supported matrix.
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
    Haversine, HaversineClosestPoint, Intersects, LineIntersection, MapCoords, Relate,
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

// ---- Set operations (GeoSPARQL §10.x point-set ops) -------------------------------
//
// GeoSPARQL defines `geof:intersection` / `union` / `difference` /
// `symDifference` as the set-theoretic operations on the geometries' point
// sets, returning the result geometry. `geo`'s `BooleanOps` realises this for
// the POLYGON×POLYGON case (a polygon-overlay algorithm). For lower-dimension
// operands we implement the well-defined cases directly over `geo`'s
// primitives (`line_intersection`, `CoordinatePosition`) and return an honest
// [`GeoError::Unsupported`] for the genuinely-intractable combinations (notably
// line−line / line∆line, which need linear-referencing subtraction `geo` does
// not provide) rather than a wrong answer. See README / TODO.md for the table.

use geo::coordinate_position::CoordPos;
use geo::line_intersection::line_intersection;
use geo::CoordinatePosition;
use geo_types::Line;

/// The topological dimension of a geometry: 0 = point(s), 1 = curve(s),
/// 2 = surface(s). `None` for an empty geometry or a heterogeneous
/// GEOMETRYCOLLECTION (which the dimension-keyed set ops do not handle).
fn dimension(g: &Geometry<f64>) -> Option<u8> {
    match g {
        Geometry::Point(_) => Some(0),
        Geometry::MultiPoint(mp) if !mp.0.is_empty() => Some(0),
        Geometry::Line(_) | Geometry::LineString(_) => Some(1),
        Geometry::MultiLineString(mls) if !mls.0.is_empty() => Some(1),
        Geometry::Polygon(_)
        | Geometry::MultiPolygon(_)
        | Geometry::Rect(_)
        | Geometry::Triangle(_) => Some(2),
        // Empty MULTIPOINT / MULTILINESTRING (geo-types' empty representations).
        Geometry::MultiPoint(_) | Geometry::MultiLineString(_) => None,
        Geometry::GeometryCollection(_) => None,
    }
}

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

/// An empty geometry of the given dimension (geo-types has no empty Point /
/// LineString / Polygon, so 0/1 use the empty MULTI* and 2 the empty
/// MULTIPOLYGON — matching how the rest of the crate models emptiness).
fn empty_of_dimension(dim: u8) -> Geometry<f64> {
    match dim {
        0 => Geometry::MultiPoint(MultiPoint(vec![])),
        1 => Geometry::MultiLineString(MultiLineString(vec![])),
        _ => Geometry::MultiPolygon(MultiPolygon(vec![])),
    }
}

/// Every constituent [`Coord`] of a 0-dimensional operand (POINT / MULTIPOINT).
fn point_coords(g: &Geometry<f64>) -> Vec<Coord<f64>> {
    match g {
        Geometry::Point(p) => vec![p.0],
        Geometry::MultiPoint(mp) => mp.0.iter().map(|p| p.0).collect(),
        _ => Vec::new(),
    }
}

/// Every constituent [`LineString`] of a 1-dimensional operand. A bare `Line`
/// is promoted to a two-vertex `LineString`.
fn line_strings(g: &Geometry<f64>) -> Vec<LineString<f64>> {
    match g {
        Geometry::Line(l) => vec![LineString(vec![l.start, l.end])],
        Geometry::LineString(ls) => vec![ls.clone()],
        Geometry::MultiLineString(mls) => mls.0.clone(),
        _ => Vec::new(),
    }
}

/// Two coords equal within a tiny epsilon (intersection arithmetic rarely lands
/// on an exact bit pattern; we de-duplicate result points robustly).
fn coord_eq(a: Coord<f64>, b: Coord<f64>) -> bool {
    const EPS: f64 = 1e-12;
    (a.x - b.x).abs() <= EPS && (a.y - b.y).abs() <= EPS
}

/// Collect a MULTIPOINT result, de-duplicating coincident coords.
fn collect_points(coords: Vec<Coord<f64>>) -> Geometry<f64> {
    let mut out: Vec<Point<f64>> = Vec::new();
    for c in coords {
        if !out.iter().any(|p| coord_eq(p.0, c)) {
            out.push(Point::from(c));
        }
    }
    Geometry::MultiPoint(MultiPoint(out))
}

/// Collect line pieces as the narrowest single 1-D geometry: a lone piece as a
/// LINESTRING, several as a MULTILINESTRING, an empty set as an empty
/// MULTILINESTRING.
fn collect_lines(lines: Vec<LineString<f64>>) -> Geometry<f64> {
    match lines.len() {
        0 => empty_of_dimension(1),
        1 => Geometry::LineString(lines.into_iter().next().unwrap()),
        _ => Geometry::MultiLineString(MultiLineString(lines)),
    }
}

// ---- intersection -----------------------------------------------------------------

/// `geof:intersection` — the geometry of the points common to `a` and `b`.
///
/// Supported operand combinations (either argument order):
///
/// - polygon ∩ polygon → MULTIPOLYGON (`geo`'s `BooleanOps`),
/// - point ∩ anything → the point if it lies on the other geometry, else an
///   empty MULTIPOINT (point-in/on test via `CoordinatePosition`),
/// - line ∩ polygon → the portion(s) of the line inside-or-on the polygon,
///   as a (MULTI)LINESTRING (line clipping),
/// - line ∩ line → the crossing points and any overlapping (collinear)
///   segments, per `geo`'s `line_intersection`.
///
/// The result is the lowest-dimensional geometry that captures the
/// intersection (e.g. a line just *touching* a polygon at one point yields a
/// MULTIPOINT). Result CRS follows the operands.
pub fn intersection(a: &GeoGeometry, b: &GeoGeometry) -> Result<GeoGeometry, GeoError> {
    ensure_compatible(a, b)?;
    let geometry = intersection_geometry(&a.geometry, &b.geometry)?;
    Ok(GeoGeometry { crs: a.crs.clone(), geometry })
}

fn intersection_geometry(a: &Geometry<f64>, b: &Geometry<f64>) -> Result<Geometry<f64>, GeoError> {
    // Polygon × polygon: the BooleanOps overlay.
    if let (Some(pa), Some(pb)) = (polygonal(a), polygonal(b)) {
        return Ok(Geometry::MultiPolygon(pa.intersection(&pb)));
    }
    match (dimension(a), dimension(b)) {
        // Either operand empty (or a heterogeneous collection): no intersection.
        (None, _) | (_, None) => Ok(empty_of_dimension(0)),
        // Point on the LOWER-dimension side: keep the points that lie on the other.
        (Some(0), _) => Ok(points_on(point_coords(a), b)),
        (_, Some(0)) => Ok(points_on(point_coords(b), a)),
        // Line ∩ line.
        (Some(1), Some(1)) => Ok(line_line_intersection(a, b)),
        // Line ∩ polygon (clip the line to the polygon).
        (Some(1), Some(2)) => Ok(clip_lines_to_polygon(a, &polygonal(b).unwrap())),
        (Some(2), Some(1)) => Ok(clip_lines_to_polygon(b, &polygonal(a).unwrap())),
        _ => Err(GeoError::Unsupported(format!(
            "geof:intersection does not support {} ∩ {}",
            wkt_type_name(a),
            wkt_type_name(b)
        ))),
    }
}

/// The subset of `coords` lying inside-or-on `g` (a MULTIPOINT).
fn points_on(coords: Vec<Coord<f64>>, g: &Geometry<f64>) -> Geometry<f64> {
    let kept = coords
        .into_iter()
        .filter(|c| g.coordinate_position(c) != CoordPos::Outside)
        .collect();
    collect_points(kept)
}

/// Line ∩ line over every constituent segment pair: crossing points and any
/// overlapping collinear segments. Collinear overlaps are emitted as
/// LINESTRINGs; isolated crossings as a MULTIPOINT; a mix as a
/// GEOMETRYCOLLECTION.
fn line_line_intersection(a: &Geometry<f64>, b: &Geometry<f64>) -> Geometry<f64> {
    let mut points: Vec<Coord<f64>> = Vec::new();
    let mut segments: Vec<LineString<f64>> = Vec::new();
    for la in line_strings(a) {
        for sa in la.lines() {
            for lb in line_strings(b) {
                for sb in lb.lines() {
                    match line_intersection(sa, sb) {
                        Some(LineIntersection::SinglePoint { intersection, .. }) => {
                            points.push(intersection)
                        }
                        Some(LineIntersection::Collinear { intersection }) => {
                            segments.push(LineString(vec![intersection.start, intersection.end]))
                        }
                        None => {}
                    }
                }
            }
        }
    }
    // Drop crossing points that already lie on an overlapping segment.
    points.retain(|p| !segments.iter().any(|s| point_on_linestring(*p, s)));
    match (segments.is_empty(), points.is_empty()) {
        (true, true) => empty_of_dimension(0),
        (false, true) => collect_lines(segments),
        (true, false) => collect_points(points),
        (false, false) => {
            // Mixed dimensions (crossings + overlaps): a GEOMETRYCOLLECTION.
            let geoms = vec![collect_lines(segments), collect_points(points)];
            Geometry::GeometryCollection(geo_types::GeometryCollection(geoms))
        }
    }
}

/// Whether `c` lies on the linestring `ls` (any segment), within epsilon.
fn point_on_linestring(c: Coord<f64>, ls: &LineString<f64>) -> bool {
    ls.lines().any(|seg| seg.coordinate_position(&c) != CoordPos::Outside)
}

/// Clip a 1-D operand to a polygon: the portions of each segment whose
/// midpoint lies inside-or-on the polygon, after splitting every segment at its
/// boundary crossings. Returns a (MULTI)LINESTRING (or, when the line only
/// touches the polygon at isolated points, a MULTIPOINT of those points).
fn clip_lines_to_polygon(line: &Geometry<f64>, poly: &MultiPolygon<f64>) -> Geometry<f64> {
    let mut pieces: Vec<LineString<f64>> = Vec::new();
    let mut touch_points: Vec<Coord<f64>> = Vec::new();
    // Boundary segments of the polygon, for crossing computation.
    let boundary: Vec<Line<f64>> = poly
        .0
        .iter()
        .flat_map(|p| {
            std::iter::once(p.exterior().clone())
                .chain(p.interiors().iter().cloned())
                .flat_map(|r| r.lines().collect::<Vec<_>>())
        })
        .collect();
    for ls in line_strings(line) {
        for seg in ls.lines() {
            // Parametric positions (t in [0,1]) where this segment meets the
            // polygon boundary, plus the endpoints.
            let mut ts: Vec<f64> = vec![0.0, 1.0];
            for bseg in &boundary {
                if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                    line_intersection(seg, *bseg)
                {
                    ts.push(param_of(seg, intersection));
                }
            }
            ts.retain(|t| t.is_finite() && (0.0..=1.0).contains(t));
            ts.sort_by(|x, y| x.partial_cmp(y).unwrap());
            ts.dedup_by(|x, y| (*x - *y).abs() <= 1e-12);
            // Keep each sub-segment whose midpoint is inside-or-on the polygon.
            for w in ts.windows(2) {
                let (t0, t1) = (w[0], w[1]);
                if t1 - t0 <= 1e-12 {
                    continue;
                }
                let mid = at_param(seg, (t0 + t1) / 2.0);
                if poly.coordinate_position(&mid) != CoordPos::Outside {
                    pieces.push(LineString(vec![at_param(seg, t0), at_param(seg, t1)]));
                }
            }
        }
    }
    merge_pieces(&mut pieces);
    if pieces.is_empty() {
        // Possibly the line only grazes the polygon at isolated vertices.
        for ls in line_strings(line) {
            for c in ls.0 {
                if poly.coordinate_position(&c) != CoordPos::Outside {
                    touch_points.push(c);
                }
            }
        }
        return collect_points(touch_points);
    }
    collect_lines(pieces)
}

/// The parameter t∈[0,1] of `c` along `seg` (projection onto the longer axis;
/// `seg` is assumed non-degenerate here).
fn param_of(seg: Line<f64>, c: Coord<f64>) -> f64 {
    let dx = seg.end.x - seg.start.x;
    let dy = seg.end.y - seg.start.y;
    if dx.abs() >= dy.abs() {
        if dx == 0.0 {
            0.0
        } else {
            (c.x - seg.start.x) / dx
        }
    } else {
        (c.y - seg.start.y) / dy
    }
}

/// The coord at parameter t along `seg`.
fn at_param(seg: Line<f64>, t: f64) -> Coord<f64> {
    Coord {
        x: seg.start.x + (seg.end.x - seg.start.x) * t,
        y: seg.start.y + (seg.end.y - seg.start.y) * t,
    }
}

/// Stitch consecutive clip pieces that share an endpoint back into longer
/// linestrings (so a straight clipped span returns one LINESTRING, not many
/// two-vertex hops). A simple greedy chain; good enough for the clip output.
fn merge_pieces(pieces: &mut Vec<LineString<f64>>) {
    let mut merged: Vec<LineString<f64>> = Vec::new();
    for piece in pieces.drain(..) {
        let mut coords = piece.0;
        if coords.len() < 2 {
            continue;
        }
        match merged.last_mut() {
            Some(prev) if coord_eq(*prev.0.last().unwrap(), coords[0]) => {
                prev.0.extend_from_slice(&coords[1..]);
            }
            _ => merged.push(LineString(std::mem::take(&mut coords))),
        }
    }
    *pieces = merged;
}

// ---- union ------------------------------------------------------------------------

/// `geof:union` — the geometry of the points in `a` OR `b`.
///
/// - polygon ∪ polygon → MULTIPOLYGON (`geo`'s `BooleanOps`),
/// - point ∪ point → MULTIPOINT (the union of the coordinate sets),
/// - line ∪ line → MULTILINESTRING (the constituent curves, concatenated; this
///   is a valid OGC union — overlapping curves are NOT dissolved/noded, which
///   `geo` cannot do for 1-D geometry),
/// - mixed-dimension union → GEOMETRYCOLLECTION of the two operands.
pub fn union(a: &GeoGeometry, b: &GeoGeometry) -> Result<GeoGeometry, GeoError> {
    ensure_compatible(a, b)?;
    let geometry = union_geometry(&a.geometry, &b.geometry)?;
    Ok(GeoGeometry { crs: a.crs.clone(), geometry })
}

fn union_geometry(a: &Geometry<f64>, b: &Geometry<f64>) -> Result<Geometry<f64>, GeoError> {
    if let (Some(pa), Some(pb)) = (polygonal(a), polygonal(b)) {
        return Ok(Geometry::MultiPolygon(pa.union(&pb)));
    }
    match (dimension(a), dimension(b)) {
        // Empty operand: the union is the other operand (cloned).
        (None, _) => Ok(b.clone()),
        (_, None) => Ok(a.clone()),
        (Some(0), Some(0)) => {
            let mut coords = point_coords(a);
            coords.extend(point_coords(b));
            Ok(collect_points(coords))
        }
        (Some(1), Some(1)) => {
            let mut lines = line_strings(a);
            lines.extend(line_strings(b));
            Ok(collect_lines(lines))
        }
        // Mixed dimension: a heterogeneous union is exactly a GEOMETRYCOLLECTION.
        _ => Ok(Geometry::GeometryCollection(geo_types::GeometryCollection(vec![
            a.clone(),
            b.clone(),
        ]))),
    }
}

// ---- difference / symDifference ---------------------------------------------------

/// `geof:difference` — the geometry of the points in `a` but NOT in `b`.
///
/// - polygon − polygon → MULTIPOLYGON (`geo`'s `BooleanOps`),
/// - point − anything → the points of `a` that do NOT lie on `b` (MULTIPOINT),
/// - point − point (special-cased by the above): exact set subtraction.
///
/// 1-D − anything (line − line / line − polygon) needs linear-referencing
/// subtraction that `geo` does not provide, so it is a clean
/// [`GeoError::Unsupported`] rather than a wrong answer.
pub fn difference(a: &GeoGeometry, b: &GeoGeometry) -> Result<GeoGeometry, GeoError> {
    ensure_compatible(a, b)?;
    let geometry = difference_geometry(&a.geometry, &b.geometry)?;
    Ok(GeoGeometry { crs: a.crs.clone(), geometry })
}

fn difference_geometry(a: &Geometry<f64>, b: &Geometry<f64>) -> Result<Geometry<f64>, GeoError> {
    if let (Some(pa), Some(pb)) = (polygonal(a), polygonal(b)) {
        return Ok(Geometry::MultiPolygon(pa.difference(&pb)));
    }
    match (dimension(a), dimension(b)) {
        // Empty `a`: nothing to keep. Empty `b`: `a` unchanged.
        (None, _) => Ok(empty_of_dimension(0)),
        (Some(_), None) => Ok(a.clone()),
        // Point − anything: keep the coords of `a` not lying on `b`.
        (Some(0), _) => {
            let kept = point_coords(a)
                .into_iter()
                .filter(|c| b.coordinate_position(c) == CoordPos::Outside)
                .collect();
            Ok(collect_points(kept))
        }
        // 1-D / 2-D minus a lower-or-equal non-polygonal operand is not tractable
        // with geo's primitives (no linear-referencing subtraction).
        _ => Err(GeoError::Unsupported(format!(
            "geof:difference does not support {} − {} (no linear-referencing subtraction in geo)",
            wkt_type_name(a),
            wkt_type_name(b)
        ))),
    }
}

/// `geof:symDifference` — the geometry of the points in exactly one of `a`, `b`
/// (i.e. (a−b) ∪ (b−a)).
///
/// - polygon ∆ polygon → MULTIPOLYGON (`geo`'s `BooleanOps`),
/// - point ∆ point → MULTIPOINT (the symmetric difference of the coord sets),
/// - point ∆ line/polygon (either order) → the points of the 0-D operand not on
///   the other, unioned with the other operand (a GEOMETRYCOLLECTION when mixed
///   dimension).
///
/// line ∆ line / line ∆ polygon are unsupported for the same reason as
/// [`difference`].
pub fn sym_difference(a: &GeoGeometry, b: &GeoGeometry) -> Result<GeoGeometry, GeoError> {
    ensure_compatible(a, b)?;
    let geometry = sym_difference_geometry(&a.geometry, &b.geometry)?;
    Ok(GeoGeometry { crs: a.crs.clone(), geometry })
}

fn sym_difference_geometry(
    a: &Geometry<f64>,
    b: &Geometry<f64>,
) -> Result<Geometry<f64>, GeoError> {
    if let (Some(pa), Some(pb)) = (polygonal(a), polygonal(b)) {
        return Ok(Geometry::MultiPolygon(pa.xor(&pb)));
    }
    match (dimension(a), dimension(b)) {
        (None, None) => Ok(empty_of_dimension(0)),
        (None, Some(_)) => Ok(b.clone()),
        (Some(_), None) => Ok(a.clone()),
        // Same-dimension 0-D: exact coordinate-set symmetric difference.
        (Some(0), Some(0)) => {
            let ca = point_coords(a);
            let cb = point_coords(b);
            let mut out: Vec<Coord<f64>> =
                ca.iter().filter(|c| !cb.iter().any(|d| coord_eq(**c, *d))).copied().collect();
            out.extend(cb.iter().filter(|c| !ca.iter().any(|d| coord_eq(**c, *d))).copied());
            Ok(collect_points(out))
        }
        // Point ∆ (line | polygon): (point − other) ∪ other.
        (Some(0), Some(_)) => {
            let pts = difference_geometry(a, b)?;
            union_geometry(&pts, b)
        }
        (Some(_), Some(0)) => {
            let pts = difference_geometry(b, a)?;
            union_geometry(a, &pts)
        }
        _ => Err(GeoError::Unsupported(format!(
            "geof:symDifference does not support {} ∆ {} (no linear-referencing subtraction in geo)",
            wkt_type_name(a),
            wkt_type_name(b)
        ))),
    }
}

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
        /// `geof:intersection(?a, ?b)` -> wktLiteral lexical form (polygon/line/point
        /// operands; see [`super::intersection`] for the supported matrix).
        intersection
    );
    lex_set_operation!(
        /// `geof:union(?a, ?b)` -> wktLiteral lexical form (polygon/line/point
        /// operands; see [`super::union`]).
        union
    );
    lex_set_operation!(
        /// `geof:difference(?a, ?b)` -> wktLiteral lexical form (polygon − polygon
        /// and point − anything; see [`super::difference`]).
        difference
    );
    lex_set_operation!(
        /// `geof:symDifference(?a, ?b)` -> wktLiteral lexical form (see
        /// [`super::sym_difference`]).
        sym_difference
    );

    /// `geof:buffer(?a, ?radius, ?unitIri)` -> wktLiteral lexical form.
    pub fn buffer(a: &str, radius: f64, unit_iri: &str) -> Result<String, GeoError> {
        Ok(super::buffer(&parse_wkt_literal(a)?, radius, Unit::from_iri(unit_iri)?)?
            .to_wkt_literal())
    }
}
