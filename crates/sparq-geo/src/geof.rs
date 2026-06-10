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
//!   `sfOverlaps`) — DE-9IM intersection matrices via `geo`'s `Relate`.
//! - [`envelope`] / [`boundary`] / [`convex_hull`] — `geof:envelope`,
//!   `geof:boundary`, `geof:convexHull`. (`geof:buffer` needs a buffer op the
//!   `geo` crate does not ship at 0.30 — see TODO.md.)
//!
//! The [`lex`] sub-module mirrors every function at the lexical level
//! (wkt-literal strings in, values out) — the shape a SPARQL engine builtin
//! receives, so sparq-engine can register them directly once it grows an
//! extension-function registry (TODO.md records the API needed).

use crate::literal::{Crs, GeoGeometry};
use crate::GeoError;
use geo::{
    BoundingRect, Closest, ConvexHull, CoordsIter, Distance, Euclidean, Haversine,
    HaversineClosestPoint, Intersects, MapCoords, Relate,
};
use geo_types::{
    Coord, Geometry, LineString, MultiLineString, MultiPoint, Point, Polygon,
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
}
