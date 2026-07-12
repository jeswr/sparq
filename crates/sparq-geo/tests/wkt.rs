//! wktLiteral parsing: lexical forms, CRS prefixes, axis order, round-trips.

use geo_types::{Geometry, Point};
use sparq_geo::{parse_wkt_literal, vocab, Crs, GeoError};

#[test]
fn parses_point_default_crs() {
    let g = parse_wkt_literal("POINT(-83.38 33.95)").unwrap();
    assert_eq!(g.crs, Crs::Crs84);
    assert_eq!(g.geometry, Geometry::Point(Point::new(-83.38, 33.95)));
}

#[test]
fn parses_each_simple_features_type() {
    for lex in [
        "POINT(1 2)",
        "LINESTRING(0 0, 1 1, 2 0)",
        "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))",
        "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0), (1 1, 2 1, 2 2, 1 2, 1 1))",
        "MULTIPOINT((0 0), (1 1))",
        "MULTIPOINT(0 0, 1 1)",
        "MULTILINESTRING((0 0, 1 1), (2 2, 3 3))",
        "MULTIPOLYGON(((0 0, 1 0, 1 1, 0 0)), ((5 5, 6 5, 6 6, 5 5)))",
        "GEOMETRYCOLLECTION(POINT(1 2), LINESTRING(0 0, 1 1))",
    ] {
        let g = parse_wkt_literal(lex).unwrap_or_else(|e| panic!("{lex}: {e}"));
        assert_eq!(g.crs, Crs::Crs84, "{lex}");
    }
}

#[test]
fn parses_crs84_prefix() {
    let g = parse_wkt_literal("<http://www.opengis.net/def/crs/OGC/1.3/CRS84> POINT(-83.38 33.95)")
        .unwrap();
    assert_eq!(g.crs, Crs::Crs84);
    assert_eq!(g.geometry, Geometry::Point(Point::new(-83.38, 33.95)));
    // CRS84 (the default) round-trips WITHOUT the prefix.
    assert_eq!(g.to_wkt_literal(), "POINT(-83.38 33.95)");
}

#[test]
fn epsg_4326_swaps_axis_order() {
    // EPSG:4326 is LAT/LONG: the lexical form writes latitude first…
    let g = parse_wkt_literal("<http://www.opengis.net/def/crs/EPSG/0/4326> POINT(33.95 -83.38)")
        .unwrap();
    assert_eq!(g.crs, Crs::Epsg4326);
    // …but internally x must be LONGITUDE.
    assert_eq!(g.geometry, Geometry::Point(Point::new(-83.38, 33.95)));
    // And serialisation swaps back, keeping the CRS prefix.
    assert_eq!(
        g.to_wkt_literal(),
        format!("<{}> POINT(33.95 -83.38)", vocab::EPSG_4326)
    );
}

#[test]
fn other_crs_is_preserved_verbatim() {
    let iri = "http://www.opengis.net/def/crs/EPSG/0/27700"; // British National Grid
    let g = parse_wkt_literal(&format!("<{iri}> POINT(530000 180000)")).unwrap();
    assert_eq!(g.crs, Crs::Other(iri.to_string()));
    assert!(!g.crs.is_geographic());
    assert_eq!(g.geometry, Geometry::Point(Point::new(530000.0, 180000.0)));
    assert_eq!(g.to_wkt_literal(), format!("<{iri}> POINT(530000 180000)"));
}

#[test]
fn leading_whitespace_is_tolerated() {
    let g = parse_wkt_literal("  \t<http://www.opengis.net/def/crs/OGC/1.3/CRS84>  POINT(1 2)")
        .unwrap();
    assert_eq!(g.geometry, Geometry::Point(Point::new(1.0, 2.0)));
}

#[test]
fn round_trips_reparse_to_equal_geometry() {
    for lex in [
        "POINT(1.5 -2.25)",
        "LINESTRING(0 0, 1 1, 2 0)",
        "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0), (1 1, 2 1, 2 2, 1 2, 1 1))",
        "MULTIPOINT(0 0, 1 1)",
        "MULTILINESTRING((0 0, 1 1), (2 2, 3 3))",
        "MULTIPOLYGON(((0 0, 1 0, 1 1, 0 0)))",
        "GEOMETRYCOLLECTION(POINT(1 2), LINESTRING(0 0, 1 1))",
        "<http://www.opengis.net/def/crs/EPSG/0/4326> LINESTRING(33 -83, 34 -84)",
        "<http://example.org/crs/local> POINT(7 8)",
    ] {
        let once = parse_wkt_literal(lex).unwrap_or_else(|e| panic!("{lex}: {e}"));
        let twice = parse_wkt_literal(&once.to_wkt_literal())
            .unwrap_or_else(|e| panic!("re-parse of {lex}: {e}"));
        assert_eq!(once, twice, "round-trip of {lex}");
    }
}

#[test]
fn rejects_garbage_and_unterminated_crs() {
    assert!(matches!(
        parse_wkt_literal("POINT(1)"),
        Err(GeoError::Parse(_))
    ));
    assert!(matches!(
        parse_wkt_literal("FOO(1 2)"),
        Err(GeoError::Parse(_))
    ));
    assert!(matches!(parse_wkt_literal(""), Err(GeoError::Parse(_))));
    assert!(matches!(
        parse_wkt_literal("<http://example.org/crs POINT(1 2)"),
        Err(GeoError::Parse(_))
    ));
}

#[test]
fn empty_geometries_parse_to_empty_representations() {
    // geo-types has no empty point: the wkt crate maps POINT EMPTY to an
    // empty MULTIPOINT. Such geometries have no bounding box and are skipped
    // by the GeoIndex (tested in index.rs).
    let g = parse_wkt_literal("POINT EMPTY").unwrap();
    assert_eq!(
        g.geometry,
        Geometry::MultiPoint(geo_types::MultiPoint(vec![]))
    );
    let g = parse_wkt_literal("GEOMETRYCOLLECTION EMPTY").unwrap();
    assert!(matches!(g.geometry, Geometry::GeometryCollection(c) if c.is_empty()));
}
