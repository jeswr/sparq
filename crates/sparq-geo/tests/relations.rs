//! geof: spatial functions: a truth table for the eight simple-features
//! relations on hand-built geometries, plus distance / envelope / boundary /
//! convexHull checks.

use sparq_geo::geof::{self, lex, Unit};
use sparq_geo::{parse_wkt_literal, GeoError};

/// (a, b, [equals, disjoint, intersects, touches, crosses, within, contains, overlaps])
const TRUTH_TABLE: &[(&str, &str, [bool; 8])] = &[
    // Identical polygons.
    (
        "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))",
        "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))",
        [true, false, true, false, false, true, true, false],
    ),
    // Same point set, different vertex order: still topologically equal.
    (
        "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))",
        "POLYGON((4 0, 4 4, 0 4, 0 0, 4 0))",
        [true, false, true, false, false, true, true, false],
    ),
    // Small polygon strictly inside a big one.
    (
        "POLYGON((1 1, 2 1, 2 2, 1 2, 1 1))",
        "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))",
        [false, false, true, false, false, true, false, false],
    ),
    // …and the reverse: contains.
    (
        "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))",
        "POLYGON((1 1, 2 1, 2 2, 1 2, 1 1))",
        [false, false, true, false, false, false, true, false],
    ),
    // Far-apart polygons: disjoint.
    (
        "POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))",
        "POLYGON((5 5, 6 5, 6 6, 5 6, 5 5))",
        [false, true, false, false, false, false, false, false],
    ),
    // Edge-adjacent polygons: touches (boundaries meet, interiors don't).
    (
        "POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))",
        "POLYGON((1 0, 2 0, 2 1, 1 1, 1 0))",
        [false, false, true, true, false, false, false, false],
    ),
    // Corner-touching polygons: touches.
    (
        "POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))",
        "POLYGON((1 1, 2 1, 2 2, 1 2, 1 1))",
        [false, false, true, true, false, false, false, false],
    ),
    // Partially overlapping polygons: overlaps.
    (
        "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))",
        "POLYGON((1 1, 3 1, 3 3, 1 3, 1 1))",
        [false, false, true, false, false, false, false, true],
    ),
    // Line crossing a polygon interior: crosses (dim 1 vs dim 2).
    (
        "LINESTRING(-1 1, 3 1)",
        "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))",
        [false, false, true, false, true, false, false, false],
    ),
    // Two lines crossing at a point: crosses.
    (
        "LINESTRING(0 0, 2 2)",
        "LINESTRING(0 2, 2 0)",
        [false, false, true, false, true, false, false, false],
    ),
    // Two lines sharing an endpoint: touches.
    (
        "LINESTRING(0 0, 1 1)",
        "LINESTRING(1 1, 2 0)",
        [false, false, true, true, false, false, false, false],
    ),
    // Collinear overlapping lines: overlaps (same dimension, partial sharing).
    (
        "LINESTRING(0 0, 2 0)",
        "LINESTRING(1 0, 3 0)",
        [false, false, true, false, false, false, false, true],
    ),
    // Point inside a polygon: within (and the polygon would contain it).
    (
        "POINT(1 1)",
        "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))",
        [false, false, true, false, false, true, false, false],
    ),
    // Point on a polygon's boundary: touches, NOT within (interior rule).
    (
        "POINT(0 1)",
        "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))",
        [false, false, true, true, false, false, false, false],
    ),
    // Point vs same point: equals.
    ("POINT(3 4)", "POINT(3 4)", [true, false, true, false, false, true, true, false]),
    // Point vs different point: disjoint.
    ("POINT(3 4)", "POINT(3 5)", [false, true, false, false, false, false, false, false]),
    // Line within a polygon.
    (
        "LINESTRING(0.5 0.5, 1.5 1.5)",
        "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))",
        [false, false, true, false, false, true, false, false],
    ),
];

#[test]
fn simple_features_truth_table() {
    type Rel = fn(&str, &str) -> Result<bool, GeoError>;
    let relations: [(&str, Rel); 8] = [
        ("sfEquals", lex::sf_equals),
        ("sfDisjoint", lex::sf_disjoint),
        ("sfIntersects", lex::sf_intersects),
        ("sfTouches", lex::sf_touches),
        ("sfCrosses", lex::sf_crosses),
        ("sfWithin", lex::sf_within),
        ("sfContains", lex::sf_contains),
        ("sfOverlaps", lex::sf_overlaps),
    ];
    for (a, b, expected) in TRUTH_TABLE {
        for ((name, f), want) in relations.iter().zip(expected) {
            let got = f(a, b).unwrap_or_else(|e| panic!("{name}({a}, {b}): {e}"));
            assert_eq!(got, *want, "{name}({a}, {b})");
        }
    }
}

#[test]
fn intersects_is_negation_of_disjoint_and_relations_are_consistent() {
    for (a, b, _) in TRUTH_TABLE {
        let inter = lex::sf_intersects(a, b).unwrap();
        assert_eq!(inter, !lex::sf_disjoint(a, b).unwrap(), "{a} / {b}");
        // within(a,b) == contains(b,a)
        assert_eq!(
            lex::sf_within(a, b).unwrap(),
            lex::sf_contains(b, a).unwrap(),
            "{a} / {b}"
        );
    }
}

const UOM_METRE: &str = "http://www.opengis.net/def/uom/OGC/1.0/metre";
const UOM_KM: &str = "http://www.opengis.net/def/uom/OGC/1.0/kilometre";
const UOM_DEGREE: &str = "http://www.opengis.net/def/uom/OGC/1.0/degree";
const UOM_RADIAN: &str = "http://www.opengis.net/def/uom/OGC/1.0/radian";

#[test]
fn distance_in_meters_london_to_paris() {
    // Charing Cross -> Notre-Dame, great-circle ≈ 341 km.
    let london = "POINT(-0.1276 51.5074)";
    let paris = "POINT(2.3496 48.8530)";
    let m = lex::distance(london, paris, UOM_METRE).unwrap();
    assert!((m - 341_000.0).abs() < 4_000.0, "got {m}");
    let km = lex::distance(london, paris, UOM_KM).unwrap();
    assert!((km - m / 1000.0).abs() < 1e-9);
}

#[test]
fn distance_in_degrees_and_radians() {
    let d = lex::distance("POINT(0 0)", "POINT(3 4)", UOM_DEGREE).unwrap();
    assert!((d - 5.0).abs() < 1e-12, "got {d}");
    let r = lex::distance("POINT(0 0)", "POINT(3 4)", UOM_RADIAN).unwrap();
    assert!((r - 5.0_f64.to_radians()).abs() < 1e-12, "got {r}");
}

#[test]
fn distance_point_to_polygon_is_to_closest_point() {
    // Point 1 degree of longitude east of the polygon's right edge, on the equator.
    let d = lex::distance("POINT(2 0)", "POLYGON((0 -1, 1 -1, 1 1, 0 1, 0 -1))", UOM_METRE)
        .unwrap();
    // One degree of longitude at the equator on the GRS80 mean sphere ≈ 111.195 km.
    assert!((d - 111_195.0).abs() < 100.0, "got {d}");
    // Point inside the polygon: distance 0.
    let z = lex::distance("POINT(0.5 0)", "POLYGON((0 -1, 1 -1, 1 1, 0 1, 0 -1))", UOM_METRE)
        .unwrap();
    assert_eq!(z, 0.0);
}

#[test]
fn distance_between_extended_geometries_approximates_locally() {
    // Two short parallel vertical lines 0.01° of longitude apart on the equator:
    // the equirectangular approximation should be within a metre of the truth.
    let d = lex::distance(
        "LINESTRING(0 0, 0 0.01)",
        "LINESTRING(0.01 0, 0.01 0.01)",
        UOM_METRE,
    )
    .unwrap();
    assert!((d - 1_111.95).abs() < 1.0, "got {d}");
}

#[test]
fn distance_epsg4326_mixes_with_crs84() {
    // Same two points, one written in EPSG:4326 lat/long order.
    let a = "POINT(-0.1276 51.5074)";
    let b = "<http://www.opengis.net/def/crs/EPSG/0/4326> POINT(48.8530 2.3496)";
    let m = lex::distance(a, b, UOM_METRE).unwrap();
    assert!((m - 341_000.0).abs() < 4_000.0, "got {m}");
}

#[test]
fn distance_errors() {
    // Unknown unit IRI.
    assert!(matches!(
        lex::distance("POINT(0 0)", "POINT(1 1)", "http://example.org/uom/furlong"),
        Err(GeoError::UnknownUnit(_))
    ));
    // Metres in a projected CRS: refused (no projection support).
    let bng = "<http://www.opengis.net/def/crs/EPSG/0/27700> POINT(530000 180000)";
    let bng2 = "<http://www.opengis.net/def/crs/EPSG/0/27700> POINT(531000 180000)";
    assert!(matches!(
        lex::distance(bng, bng2, UOM_METRE),
        Err(GeoError::NonGeographicCrs(_))
    ));
    // …but coordinate-space distance still works within that CRS.
    let d = lex::distance(bng, bng2, UOM_DEGREE).unwrap();
    assert!((d - 1000.0).abs() < 1e-9);
    // CRS mismatch between a projected and a geographic literal.
    assert!(matches!(
        lex::distance(bng, "POINT(0 0)", UOM_DEGREE),
        Err(GeoError::CrsMismatch(_, _))
    ));
}

#[test]
fn relations_require_compatible_crs() {
    let bng = "<http://www.opengis.net/def/crs/EPSG/0/27700> POINT(530000 180000)";
    assert!(matches!(lex::sf_intersects(bng, "POINT(0 0)"), Err(GeoError::CrsMismatch(_, _))));
}

#[test]
fn envelope_boundary_convex_hull() {
    // Envelope of a tilted line: its bounding box as a polygon.
    let env = lex::envelope("LINESTRING(0 0, 2 1, 1 3)").unwrap();
    let env = parse_wkt_literal(&env).unwrap();
    let corners = parse_wkt_literal("POLYGON((0 0, 2 0, 2 3, 0 3, 0 0))").unwrap();
    assert!(geof::sf_equals(&env, &corners).unwrap());

    // Boundary of a polygon with a hole: both rings.
    let b = lex::boundary("POLYGON((0 0, 4 0, 4 4, 0 4, 0 0), (1 1, 2 1, 2 2, 1 2, 1 1))")
        .unwrap();
    assert!(b.starts_with("MULTILINESTRING"), "got {b}");
    let b = parse_wkt_literal(&b).unwrap();
    let rings = parse_wkt_literal(
        "MULTILINESTRING((0 0, 4 0, 4 4, 0 4, 0 0), (1 1, 2 1, 2 2, 1 2, 1 1))",
    )
    .unwrap();
    assert!(geof::sf_equals(&b, &rings).unwrap());

    // Boundary of an open linestring: its two endpoints; of a closed one: empty.
    let b = lex::boundary("LINESTRING(0 0, 1 1, 2 0)").unwrap();
    let b = parse_wkt_literal(&b).unwrap();
    assert!(geof::sf_equals(&b, &parse_wkt_literal("MULTIPOINT(0 0, 2 0)").unwrap()).unwrap());
    let closed = lex::boundary("LINESTRING(0 0, 1 1, 2 0, 0 0)").unwrap();
    assert!(closed.contains("EMPTY"), "got {closed}");

    // Boundary of a point: empty.
    assert!(lex::boundary("POINT(1 2)").unwrap().contains("EMPTY"));

    // Convex hull of a concave polygon: the dent disappears.
    let hull = lex::convex_hull("POLYGON((0 0, 4 0, 4 4, 2 1, 0 4, 0 0))").unwrap();
    let hull = parse_wkt_literal(&hull).unwrap();
    let square = parse_wkt_literal("POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))").unwrap();
    assert!(geof::sf_equals(&hull, &square).unwrap(), "got {}", hull.to_wkt_literal());
}

#[test]
fn unit_iri_aliases() {
    assert_eq!(Unit::from_iri("http://qudt.org/vocab/unit/M").unwrap(), Unit::Metre);
    assert_eq!(Unit::from_iri("http://qudt.org/vocab/unit/KiloM").unwrap(), Unit::Kilometre);
    assert_eq!(Unit::from_iri("http://qudt.org/vocab/unit/DEG").unwrap(), Unit::Degree);
    assert_eq!(Unit::from_iri(UOM_RADIAN).unwrap(), Unit::Radian);
    assert!(Unit::from_iri("metre").is_err());
}
