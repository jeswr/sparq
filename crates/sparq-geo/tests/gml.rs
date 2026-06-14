//! `geo:gmlLiteral` (GML Simple-Features) parsing + equivalence to its WKT twin.
//! [OPUS-4.8] sq-zy0

use geo_types::{
    Coord, Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};
use sparq_geo::{
    is_geometry_datatype, parse_geometry_literal, parse_gml_literal, parse_wkt_literal, vocab, Crs,
    GeoError,
};

// ---- Each GML-SF geometry type -> hand-computed geo geometry ----------------

#[test]
fn parses_point_pos_default_crs() {
    let g = parse_gml_literal(
        r#"<gml:Point><gml:pos>-83.38 33.95</gml:pos></gml:Point>"#,
    )
    .unwrap();
    assert_eq!(g.crs, Crs::Crs84);
    assert_eq!(g.geometry, Geometry::Point(Point::new(-83.38, 33.95)));
}

#[test]
fn parses_point_gml2_coordinates() {
    // GML 2 legacy spelling: <gml:coordinates>x,y</gml:coordinates>.
    let g = parse_gml_literal(
        r#"<gml:Point><gml:coordinates>1.5,-2.25</gml:coordinates></gml:Point>"#,
    )
    .unwrap();
    assert_eq!(g.geometry, Geometry::Point(Point::new(1.5, -2.25)));
}

#[test]
fn parses_linestring_poslist() {
    let g = parse_gml_literal(
        r#"<gml:LineString><gml:posList>0 0 1 1 2 0</gml:posList></gml:LineString>"#,
    )
    .unwrap();
    let expected = LineString::new(vec![
        Coord { x: 0.0, y: 0.0 },
        Coord { x: 1.0, y: 1.0 },
        Coord { x: 2.0, y: 0.0 },
    ]);
    assert_eq!(g.geometry, Geometry::LineString(expected));
}

#[test]
fn parses_linestring_repeated_pos() {
    let g = parse_gml_literal(
        r#"<gml:LineString>
             <gml:pos>0 0</gml:pos>
             <gml:pos>1 1</gml:pos>
           </gml:LineString>"#,
    )
    .unwrap();
    let expected =
        LineString::new(vec![Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 }]);
    assert_eq!(g.geometry, Geometry::LineString(expected));
}

#[test]
fn parses_polygon_exterior_only() {
    let g = parse_gml_literal(
        r#"<gml:Polygon>
             <gml:exterior>
               <gml:LinearRing>
                 <gml:posList>0 0 4 0 4 4 0 4 0 0</gml:posList>
               </gml:LinearRing>
             </gml:exterior>
           </gml:Polygon>"#,
    )
    .unwrap();
    let exterior = LineString::new(vec![
        Coord { x: 0.0, y: 0.0 },
        Coord { x: 4.0, y: 0.0 },
        Coord { x: 4.0, y: 4.0 },
        Coord { x: 0.0, y: 4.0 },
        Coord { x: 0.0, y: 0.0 },
    ]);
    assert_eq!(g.geometry, Geometry::Polygon(Polygon::new(exterior, vec![])));
}

#[test]
fn parses_polygon_with_interior_ring() {
    let g = parse_gml_literal(
        r#"<gml:Polygon>
             <gml:exterior><gml:LinearRing>
               <gml:posList>0 0 4 0 4 4 0 4 0 0</gml:posList>
             </gml:LinearRing></gml:exterior>
             <gml:interior><gml:LinearRing>
               <gml:posList>1 1 2 1 2 2 1 2 1 1</gml:posList>
             </gml:LinearRing></gml:interior>
           </gml:Polygon>"#,
    )
    .unwrap();
    let exterior = LineString::new(vec![
        Coord { x: 0.0, y: 0.0 },
        Coord { x: 4.0, y: 0.0 },
        Coord { x: 4.0, y: 4.0 },
        Coord { x: 0.0, y: 4.0 },
        Coord { x: 0.0, y: 0.0 },
    ]);
    let interior = LineString::new(vec![
        Coord { x: 1.0, y: 1.0 },
        Coord { x: 2.0, y: 1.0 },
        Coord { x: 2.0, y: 2.0 },
        Coord { x: 1.0, y: 2.0 },
        Coord { x: 1.0, y: 1.0 },
    ]);
    assert_eq!(
        g.geometry,
        Geometry::Polygon(Polygon::new(exterior, vec![interior]))
    );
}

#[test]
fn parses_polygon_gml2_boundary_spellings() {
    // GML 2 used outerBoundaryIs / innerBoundaryIs instead of exterior/interior.
    let g = parse_gml_literal(
        r#"<gml:Polygon>
             <gml:outerBoundaryIs><gml:LinearRing>
               <gml:coordinates>0,0 4,0 4,4 0,4 0,0</gml:coordinates>
             </gml:LinearRing></gml:outerBoundaryIs>
           </gml:Polygon>"#,
    )
    .unwrap();
    let exterior = LineString::new(vec![
        Coord { x: 0.0, y: 0.0 },
        Coord { x: 4.0, y: 0.0 },
        Coord { x: 4.0, y: 4.0 },
        Coord { x: 0.0, y: 4.0 },
        Coord { x: 0.0, y: 0.0 },
    ]);
    assert_eq!(g.geometry, Geometry::Polygon(Polygon::new(exterior, vec![])));
}

#[test]
fn parses_multipoint() {
    let g = parse_gml_literal(
        r#"<gml:MultiPoint>
             <gml:pointMember><gml:Point><gml:pos>0 0</gml:pos></gml:Point></gml:pointMember>
             <gml:pointMember><gml:Point><gml:pos>1 1</gml:pos></gml:Point></gml:pointMember>
           </gml:MultiPoint>"#,
    )
    .unwrap();
    let expected = MultiPoint::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)]);
    assert_eq!(g.geometry, Geometry::MultiPoint(expected));
}

#[test]
fn parses_multicurve_of_linestrings() {
    let g = parse_gml_literal(
        r#"<gml:MultiCurve>
             <gml:curveMember><gml:LineString>
               <gml:posList>0 0 1 1</gml:posList>
             </gml:LineString></gml:curveMember>
             <gml:curveMember><gml:LineString>
               <gml:posList>2 2 3 3</gml:posList>
             </gml:LineString></gml:curveMember>
           </gml:MultiCurve>"#,
    )
    .unwrap();
    let expected = MultiLineString::new(vec![
        LineString::new(vec![Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 }]),
        LineString::new(vec![Coord { x: 2.0, y: 2.0 }, Coord { x: 3.0, y: 3.0 }]),
    ]);
    assert_eq!(g.geometry, Geometry::MultiLineString(expected));
}

#[test]
fn parses_multisurface_of_polygons() {
    let g = parse_gml_literal(
        r#"<gml:MultiSurface>
             <gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing>
               <gml:posList>0 0 1 0 1 1 0 0</gml:posList>
             </gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
             <gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing>
               <gml:posList>5 5 6 5 6 6 5 5</gml:posList>
             </gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
           </gml:MultiSurface>"#,
    )
    .unwrap();
    let p1 = Polygon::new(
        LineString::new(vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 1.0, y: 1.0 },
            Coord { x: 0.0, y: 0.0 },
        ]),
        vec![],
    );
    let p2 = Polygon::new(
        LineString::new(vec![
            Coord { x: 5.0, y: 5.0 },
            Coord { x: 6.0, y: 5.0 },
            Coord { x: 6.0, y: 6.0 },
            Coord { x: 5.0, y: 5.0 },
        ]),
        vec![],
    );
    assert_eq!(g.geometry, Geometry::MultiPolygon(MultiPolygon::new(vec![p1, p2])));
}

#[test]
fn namespace_prefix_is_irrelevant() {
    // A document binding the GML namespace to a different prefix, or the default
    // namespace, parses just the same (we match on the local name).
    for lex in [
        r#"<Point xmlns="http://www.opengis.net/gml/3.2"><pos>1 2</pos></Point>"#,
        r#"<g:Point xmlns:g="http://www.opengis.net/gml/3.2"><g:pos>1 2</g:pos></g:Point>"#,
    ] {
        let g = parse_gml_literal(lex).unwrap_or_else(|e| panic!("{lex}: {e}"));
        assert_eq!(g.geometry, Geometry::Point(Point::new(1.0, 2.0)));
    }
}

// ---- CRS / srsName handling consistent with the WKT path --------------------

#[test]
fn srs_name_crs84_default_equivalent() {
    let with = parse_gml_literal(
        &format!(r#"<gml:Point srsName="{}"><gml:pos>-83.38 33.95</gml:pos></gml:Point>"#, vocab::CRS84),
    )
    .unwrap();
    assert_eq!(with.crs, Crs::Crs84);
    assert_eq!(with.geometry, Geometry::Point(Point::new(-83.38, 33.95)));
}

#[test]
fn epsg_4326_srs_name_swaps_axis_order() {
    // EPSG:4326 is LAT/LONG, like the WKT path: the lexical form writes latitude
    // first but internal x must be longitude. Multiple srsName spellings count.
    for srs in [
        "http://www.opengis.net/def/crs/EPSG/0/4326",
        "urn:ogc:def:crs:EPSG::4326",
        "EPSG:4326",
    ] {
        let g = parse_gml_literal(&format!(
            r#"<gml:Point srsName="{srs}"><gml:pos>33.95 -83.38</gml:pos></gml:Point>"#
        ))
        .unwrap();
        assert_eq!(g.crs, Crs::Epsg4326, "{srs}");
        assert_eq!(
            g.geometry,
            Geometry::Point(Point::new(-83.38, 33.95)),
            "{srs}"
        );
    }
}

#[test]
fn other_crs_kept_verbatim() {
    let iri = "http://www.opengis.net/def/crs/EPSG/0/27700"; // British National Grid
    let g = parse_gml_literal(&format!(
        r#"<gml:Point srsName="{iri}"><gml:pos>530000 180000</gml:pos></gml:Point>"#
    ))
    .unwrap();
    assert_eq!(g.crs, Crs::Other(iri.to_string()));
    assert!(!g.crs.is_geographic());
    assert_eq!(g.geometry, Geometry::Point(Point::new(530000.0, 180000.0)));
}

// ---- Malformed GML -> clean GeoError, never a panic -------------------------

#[test]
fn malformed_gml_is_a_clean_error() {
    let cases = [
        // Not XML at all.
        "POINT(1 2)",
        // Unknown / non-geometry root element.
        r#"<gml:Feature><gml:name>x</gml:name></gml:Feature>"#,
        // Point with no coordinates.
        r#"<gml:Point></gml:Point>"#,
        // Odd ordinate count (only 2-D supported).
        r#"<gml:Point><gml:pos>1 2 3</gml:pos></gml:Point>"#,
        // Non-numeric ordinate.
        r#"<gml:Point><gml:pos>foo bar</gml:pos></gml:Point>"#,
        // Polygon missing its exterior ring.
        r#"<gml:Polygon><gml:interior><gml:LinearRing><gml:posList>0 0 1 0 1 1 0 0</gml:posList></gml:LinearRing></gml:interior></gml:Polygon>"#,
        // LinearRing too short to close.
        r#"<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>0 0 1 1</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>"#,
        // Truncated / unbalanced XML.
        r#"<gml:Point><gml:pos>1 2</gml:pos>"#,
        // Empty string.
        "",
    ];
    for lex in cases {
        match parse_gml_literal(lex) {
            Err(GeoError::Parse(_)) | Err(GeoError::Unsupported(_)) => {}
            other => panic!("expected a clean error for {lex:?}, got {other:?}"),
        }
    }
}

#[test]
fn deferred_types_are_clean_unsupported() {
    // gml:Envelope is deliberately deferred (a bead) -> a clean Unsupported,
    // not a parse failure or a panic.
    let env = r#"<gml:Envelope srsName="http://www.opengis.net/def/crs/OGC/1.3/CRS84">
        <gml:lowerCorner>0 0</gml:lowerCorner><gml:upperCorner>4 4</gml:upperCorner></gml:Envelope>"#;
    assert!(matches!(parse_gml_literal(env), Err(GeoError::Unsupported(_))));
}

// ---- Equivalence with the WKT twin ------------------------------------------

/// Every GML geometry parses to the SAME `GeoGeometry` as the equivalent WKT
/// literal — proving GML rides the identical downstream pipeline.
#[test]
fn gml_equals_its_wkt_twin() {
    let pairs: &[(&str, &str)] = &[
        (
            r#"<gml:Point><gml:pos>1 2</gml:pos></gml:Point>"#,
            "POINT(1 2)",
        ),
        (
            r#"<gml:LineString><gml:posList>0 0 1 1 2 0</gml:posList></gml:LineString>"#,
            "LINESTRING(0 0, 1 1, 2 0)",
        ),
        (
            r#"<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>0 0 4 0 4 4 0 4 0 0</gml:posList></gml:LinearRing></gml:exterior><gml:interior><gml:LinearRing><gml:posList>1 1 2 1 2 2 1 2 1 1</gml:posList></gml:LinearRing></gml:interior></gml:Polygon>"#,
            "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0), (1 1, 2 1, 2 2, 1 2, 1 1))",
        ),
        (
            r#"<gml:MultiPoint><gml:pointMember><gml:Point><gml:pos>0 0</gml:pos></gml:Point></gml:pointMember><gml:pointMember><gml:Point><gml:pos>1 1</gml:pos></gml:Point></gml:pointMember></gml:MultiPoint>"#,
            "MULTIPOINT(0 0, 1 1)",
        ),
        (
            r#"<gml:MultiCurve><gml:curveMember><gml:LineString><gml:posList>0 0 1 1</gml:posList></gml:LineString></gml:curveMember><gml:curveMember><gml:LineString><gml:posList>2 2 3 3</gml:posList></gml:LineString></gml:curveMember></gml:MultiCurve>"#,
            "MULTILINESTRING((0 0, 1 1), (2 2, 3 3))",
        ),
        (
            r#"<gml:MultiSurface><gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>0 0 1 0 1 1 0 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember></gml:MultiSurface>"#,
            "MULTIPOLYGON(((0 0, 1 0, 1 1, 0 0)))",
        ),
    ];
    for (gml, wkt) in pairs {
        let from_gml = parse_gml_literal(gml).unwrap_or_else(|e| panic!("{gml}: {e}"));
        let from_wkt = parse_wkt_literal(wkt).unwrap_or_else(|e| panic!("{wkt}: {e}"));
        assert_eq!(from_gml, from_wkt, "GML {gml} != WKT {wkt}");
    }
}

#[test]
fn parse_geometry_literal_dispatches_by_datatype() {
    assert!(is_geometry_datatype(vocab::WKT_LITERAL));
    assert!(is_geometry_datatype(vocab::GML_LITERAL));
    assert!(!is_geometry_datatype("http://www.w3.org/2001/XMLSchema#string"));

    let from_dispatch = parse_geometry_literal(
        r#"<gml:Point><gml:pos>1 2</gml:pos></gml:Point>"#,
        vocab::GML_LITERAL,
    )
    .unwrap();
    assert_eq!(from_dispatch, parse_wkt_literal("POINT(1 2)").unwrap());

    let wkt_dispatch = parse_geometry_literal("POINT(1 2)", vocab::WKT_LITERAL).unwrap();
    assert_eq!(wkt_dispatch.geometry, Geometry::Point(Point::new(1.0, 2.0)));

    // An unrecognised geometry datatype is a clean Unsupported.
    assert!(matches!(
        parse_geometry_literal("POINT(1 2)", "http://example.org/notGeometry"),
        Err(GeoError::Unsupported(_))
    ));
}

// ---- A gmlLiteral round-trips through SPARQL geof: like its WKT twin ---------

#[cfg(feature = "engine")]
mod sparql {
    use sparq_core::Graph;
    use sparq_engine::query_with_functions;
    use sparq_geo::{geof_registry, GeoIndex};

    const PREFIXES: &str = "PREFIX geof: <http://www.opengis.net/def/function/geosparql/> \
                            PREFIX uom:  <http://www.opengis.net/def/uom/OGC/1.0/> \
                            PREFIX ex:   <http://ex/> ";

    /// London/Paris/Lyon as POINTS, but each city's location is given BOTH as a
    /// wktLiteral (ex:wloc) and as the equivalent gmlLiteral (ex:gloc); the UK
    /// box is a wktLiteral polygon.
    fn cities() -> Graph {
        Graph::load_str(
            r#"
            @prefix geo: <http://www.opengis.net/ont/geosparql#> .
            @prefix ex:  <http://ex/> .
            ex:london ex:wloc "POINT(-0.1278 51.5074)"^^geo:wktLiteral .
            ex:london ex:gloc "<gml:Point><gml:pos>-0.1278 51.5074</gml:pos></gml:Point>"^^geo:gmlLiteral .
            ex:paris  ex:wloc "POINT(2.3522 48.8566)"^^geo:wktLiteral .
            ex:paris  ex:gloc "<gml:Point><gml:pos>2.3522 48.8566</gml:pos></gml:Point>"^^geo:gmlLiteral .
            ex:lyon   ex:wloc "POINT(4.8357 45.7640)"^^geo:wktLiteral .
            ex:lyon   ex:gloc "<gml:Point><gml:pos>4.8357 45.7640</gml:pos></gml:Point>"^^geo:gmlLiteral .
            ex:uk     ex:area "POLYGON((-6 50, 2 50, 2 59, -6 59, -6 50))"^^geo:wktLiteral .
            "#,
            "turtle",
        )
        .unwrap()
    }

    fn names(r: &sparq_engine::QueryResult, col: usize) -> Vec<String> {
        r.rows.iter().map(|row| row[col].as_ref().unwrap().to_string()).collect()
    }

    #[test]
    fn distance_gml_equals_wkt() {
        // Cities within 400 km of London computed over the GML locations:
        // same answer as over WKT — Paris in, Lyon out.
        let r = query_with_functions(
            &cities(),
            &format!(
                "{PREFIXES} SELECT ?city WHERE {{ \
                   ex:london ex:gloc ?here . ?city ex:gloc ?there . \
                   FILTER(?city != ex:london && geof:distance(?here, ?there, uom:kilometre) < 400) \
                 }}"
            ),
            &geof_registry(),
        )
        .unwrap();
        assert_eq!(names(&r, 0), vec!["<http://ex/paris>"]);
    }

    #[test]
    fn mixed_gml_and_wkt_arguments_interoperate() {
        // One argument a gmlLiteral, the other a wktLiteral: geof: treats both
        // serializations identically (GeoSPARQL §8.5), so this still finds Paris.
        let r = query_with_functions(
            &cities(),
            &format!(
                "{PREFIXES} SELECT ?city WHERE {{ \
                   ex:london ex:wloc ?here . ?city ex:gloc ?there . \
                   FILTER(?city != ex:london && geof:distance(?here, ?there, uom:kilometre) < 400) \
                 }}"
            ),
            &geof_registry(),
        )
        .unwrap();
        assert_eq!(names(&r, 0), vec!["<http://ex/paris>"]);
    }

    #[test]
    fn sf_within_gml_point_in_wkt_polygon() {
        // The London GML point is within the UK wktLiteral polygon.
        let r = query_with_functions(
            &cities(),
            &format!(
                "{PREFIXES} SELECT ?city WHERE {{ \
                   ?city ex:gloc ?pt . ex:uk ex:area ?poly . \
                   FILTER(geof:sfWithin(?pt, ?poly)) \
                 }}"
            ),
            &geof_registry(),
        )
        .unwrap();
        assert_eq!(names(&r, 0), vec!["<http://ex/london>"]);
    }

    #[test]
    fn geof_buffer_of_gml_feeds_sf_within() {
        // A geometry-producing geof: over a GML arg returns a wktLiteral that
        // feeds straight back into another geof: call — Paris is within a 400 km
        // buffer of the London GML point.
        let r = query_with_functions(
            &cities(),
            &format!(
                "{PREFIXES} SELECT ?city WHERE {{ \
                   ex:london ex:gloc ?here . ?city ex:gloc ?there . \
                   FILTER(?city != ex:london && geof:sfWithin(?there, geof:buffer(?here, 400, uom:kilometre))) \
                 }}"
            ),
            &geof_registry(),
        )
        .unwrap();
        assert_eq!(names(&r, 0), vec!["<http://ex/paris>"]);
    }

    /// The GeoIndex extracts geometries from BOTH geo:asWKT and geo:asGML, and a
    /// GML-sourced point answers the same spatial query as a WKT-sourced one.
    #[test]
    fn geo_index_extracts_as_gml() {
        let g = Graph::load_str(
            r#"
            @prefix geo: <http://www.opengis.net/ont/geosparql#> .
            @prefix ex:  <http://ex/> .
            ex:london geo:asWKT "POINT(-0.1278 51.5074)"^^geo:wktLiteral .
            ex:paris  geo:asGML "<gml:Point><gml:pos>2.3522 48.8566</gml:pos></gml:Point>"^^geo:gmlLiteral .
            "#,
            "turtle",
        )
        .unwrap();
        let index = GeoIndex::build(&g);
        assert_eq!(index.len(), 2, "both asWKT and asGML geometries indexed");
        assert_eq!(index.skipped(), 0);

        use geo_types::Point;
        // Within 400 km of London: London itself + Paris (the GML-sourced one).
        let near = index.within_distance(Point::new(-0.1278, 51.5074), 400_000.0, None);
        let mut hit: Vec<String> = near.iter().map(|(t, _)| t.to_string()).collect();
        hit.sort();
        assert_eq!(hit, vec!["<http://ex/london>", "<http://ex/paris>"]);
    }
}
