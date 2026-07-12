//! Deterministic and property-level witnesses for `geof:simplify`.

use geo::CoordsIter;
use geo_types::{Coord, Geometry, LineString};
use proptest::prelude::*;
use sparq_geo::{geof, parse_wkt_literal, GeoError};

#[test]
fn nearly_collinear_midpoint_is_dropped_exactly() {
    let input = parse_wkt_literal("LINESTRING(0 0,1 0.1,2 0)").unwrap();
    let output = geof::simplify(&input, 0.2).unwrap();

    assert_eq!(output.to_wkt_literal(), "LINESTRING(0 0,2 0)");
}

#[test]
fn polygon_ring_is_simplified_exactly() {
    let input = parse_wkt_literal("POLYGON((0 0,1 0.1,2 0,2 2,0 2,0 0))").unwrap();
    let output = geof::simplify(&input, 0.2).unwrap();

    assert_eq!(output.to_wkt_literal(), "POLYGON((0 0,2 0,2 2,0 2,0 0))");
}

#[test]
fn zero_tolerance_is_byte_identical() {
    let input = parse_wkt_literal(
        "<http://www.opengis.net/def/crs/EPSG/0/27700> LINESTRING(0 0,1 0.1,2 0)",
    )
    .unwrap();

    assert_eq!(
        geof::simplify(&input, 0.0).unwrap().to_wkt_literal(),
        input.to_wkt_literal()
    );
    assert_eq!(geof::simplify(&input, -1.0).unwrap(), input);
}

#[test]
fn point_is_returned_unchanged() {
    let input = parse_wkt_literal("POINT(3 4)").unwrap();

    assert_eq!(geof::simplify(&input, 100.0).unwrap(), input);
}

#[test]
fn crs_is_preserved_and_output_is_deterministic() {
    let input = parse_wkt_literal(
        "<http://www.opengis.net/def/crs/EPSG/0/27700> LINESTRING(0 0,1 0.1,2 0)",
    )
    .unwrap();
    let first = geof::simplify(&input, 0.2).unwrap();
    let second = geof::simplify(&input, 0.2).unwrap();

    assert_eq!(first.crs, input.crs);
    assert_eq!(first.to_wkt_literal(), second.to_wkt_literal());
}

#[test]
fn geometry_collection_is_honestly_unsupported() {
    let input = parse_wkt_literal("GEOMETRYCOLLECTION(POINT(0 0),LINESTRING(0 0,1 1))").unwrap();

    assert!(matches!(
        geof::simplify(&input, 1.0),
        Err(GeoError::Unsupported(message)) if message.contains("GEOMETRYCOLLECTION")
    ));
    assert!(matches!(
        geof::simplify(&input, f64::INFINITY),
        Err(GeoError::Unsupported(message)) if message.contains("finite tolerance")
    ));
    assert!(matches!(
        geof::simplify(&input, f64::NAN),
        Err(GeoError::Unsupported(message)) if message.contains("finite tolerance")
    ));
}

proptest! {
    #[test]
    fn line_simplification_never_adds_vertices(
        points in prop::collection::vec((-10_000i16..=10_000, -10_000i16..=10_000), 2..64),
        tolerance in 0.0f64..10_000.0,
    ) {
        let line = LineString::new(
            points
                .into_iter()
                .map(|(x, y)| Coord { x: f64::from(x), y: f64::from(y) })
                .collect(),
        );
        let input = sparq_geo::GeoGeometry::new(Geometry::LineString(line));
        let before = input.geometry.coords_count();
        let output = geof::simplify(&input, tolerance).unwrap();

        prop_assert!(output.geometry.coords_count() <= before);
    }
}
