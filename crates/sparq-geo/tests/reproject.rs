//! CRS reprojection (the opt-in `reproject` feature): projected EPSG literals
//! -> CRS84, verified against published reference coordinates.
#![cfg(feature = "reproject")]

use sparq_geo::reproject::{epsg_code, proj4_definition, to_crs84, to_crs84_lex};
use sparq_geo::{geof, parse_wkt_literal, Crs, GeoError, Unit};

const BNG: &str = "http://www.opengis.net/def/crs/EPSG/0/27700";

#[test]
fn bng_matches_the_ordnance_survey_worked_example() {
    // OS guide "A guide to coordinate systems in Great Britain" worked example:
    // E 651409.903, N 313177.270 == ETRS89 lat 52°39′28.723″N, lon 1°42′57.787″E
    // (52.657979, 1.716052). The 7-param Helmert is itself ~5 m accurate; the
    // proj4 path reproduces the standard proj answer to ~1e-6°.
    let g = parse_wkt_literal(&format!("<{BNG}> POINT(651409.903 313177.270)")).unwrap();
    let out = to_crs84(&g).unwrap();
    assert_eq!(out.crs, Crs::Crs84);
    let geo_types::Geometry::Point(p) = out.geometry else {
        panic!("point in, point out")
    };
    assert!((p.x() - 1.716052).abs() < 1e-4, "lon {}", p.x());
    assert!((p.y() - 52.657979).abs() < 1e-4, "lat {}", p.y());
}

#[test]
fn reprojected_geometries_join_the_geographic_machinery() {
    // Two BNG points 1 km apart on the easting axis: after reprojection,
    // geof:distance in metres must report ~1 km (BNG's scale factor 0.9996
    // and the sphere/ellipsoid difference allow a few metres of slack).
    let a =
        to_crs84(&parse_wkt_literal(&format!("<{BNG}> POINT(530000 180000)")).unwrap()).unwrap();
    let b =
        to_crs84(&parse_wkt_literal(&format!("<{BNG}> POINT(531000 180000)")).unwrap()).unwrap();
    let d = geof::distance(&a, &b, Unit::Metre).unwrap();
    assert!((d - 1000.0).abs() < 5.0, "got {d}");
}

#[test]
fn web_mercator_and_utm_definitions_work() {
    // EPSG:3857 has a closed form: x = R·lon. London x=-14226.6, y=6711542 ->
    // lon -0.1278, lat 51.5074.
    let g = parse_wkt_literal(
        "<http://www.opengis.net/def/crs/EPSG/0/3857> POINT(-14226.63 6711542.48)",
    )
    .unwrap();
    let out = to_crs84(&g).unwrap();
    let geo_types::Geometry::Point(p) = out.geometry else {
        panic!("point")
    };
    assert!((p.x() - -0.1278).abs() < 1e-4, "lon {}", p.x());
    assert!((p.y() - 51.5074).abs() < 1e-4, "lat {}", p.y());

    // UTM 32N (EPSG:32632): 500000 E is the central meridian, 9°E. At the
    // equator northing 0 -> lat 0.
    let g =
        parse_wkt_literal("<http://www.opengis.net/def/crs/EPSG/0/32632> POINT(500000 0)").unwrap();
    let out = to_crs84(&g).unwrap();
    let geo_types::Geometry::Point(p) = out.geometry else {
        panic!("point")
    };
    assert!((p.x() - 9.0).abs() < 1e-6, "lon {}", p.x());
    assert!(p.y().abs() < 1e-6, "lat {}", p.y());
}

#[test]
fn geographic_inputs_pass_through_and_polygons_keep_their_shape() {
    // CRS84 / EPSG:4326 pass through (4326 was already axis-normalised on parse).
    let out = to_crs84_lex("POINT(1 2)").unwrap();
    assert_eq!(out, "POINT(1 2)");
    let out = to_crs84_lex("<http://www.opengis.net/def/crs/EPSG/0/4326> POINT(51.5074 -0.1278)")
        .unwrap();
    assert!(out.starts_with("POINT(-0.1278"), "got {out}");

    // A BNG polygon stays a polygon, and its reprojected envelope is sane
    // (a ~2 km square near Charing Cross).
    let out = to_crs84_lex(&format!(
        "<{BNG}> POLYGON((529000 179000, 531000 179000, 531000 181000, 529000 181000, 529000 179000))"
    ))
    .unwrap();
    let g = parse_wkt_literal(&out).unwrap();
    assert!(matches!(g.geometry, geo_types::Geometry::Polygon(_)));
    let within = geof::sf_within(
        &g,
        &parse_wkt_literal(
            "POLYGON((-0.25 51.45, 0.0 51.45, 0.0 51.56, -0.25 51.56, -0.25 51.45))",
        )
        .unwrap(),
    )
    .unwrap();
    assert!(within, "got {out}");
}

#[test]
fn unsupported_crs_errors_are_explicit() {
    // EPSG code without a curated definition.
    let g = parse_wkt_literal("<http://www.opengis.net/def/crs/EPSG/0/2056> POINT(0 0)").unwrap();
    assert!(matches!(to_crs84(&g), Err(GeoError::Unsupported(m)) if m.contains("EPSG:2056")));
    // Non-EPSG CRS IRI.
    let g = parse_wkt_literal("<http://example.org/my-crs> POINT(0 0)").unwrap();
    assert!(matches!(to_crs84(&g), Err(GeoError::Unsupported(_))));
    // Table introspection.
    assert!(proj4_definition(27700).is_some());
    assert!(proj4_definition(2056).is_none());
    assert_eq!(
        epsg_code(&Crs::Other(
            "http://www.opengis.net/def/crs/EPSG/0/27700".into()
        )),
        Some(27700)
    );
    assert_eq!(epsg_code(&Crs::Other("http://example.org/x".into())), None);
}

// ---- Geographic (non-4326) axis-order normalisation, sq-ove ----------------

use sparq_geo::reproject::{geographic_axis_order, AxisOrder};

const EPSG: &str = "http://www.opengis.net/def/crs/EPSG/0";

#[test]
fn geographic_axis_order_map_is_curated_and_paired_with_definitions() {
    // Direct unit coverage of the public map (one entry per curated code).
    for code in [4326, 4258, 4269, 4283, 4171, 4490] {
        assert_eq!(
            geographic_axis_order(code),
            Some(AxisOrder::LatLong),
            "EPSG:{code}"
        );
        // Pairing invariant: every geographic axis-order entry has a proj4
        // definition (and it is a longlat one).
        let def = proj4_definition(code).unwrap_or_else(|| panic!("no proj4 def for {code}"));
        assert!(def.starts_with("+proj=longlat"), "EPSG:{code}: {def}");
    }
    // Projected + unknown codes carry NO geographic axis order.
    assert_eq!(geographic_axis_order(27700), None);
    assert_eq!(geographic_axis_order(3857), None);
    assert_eq!(geographic_axis_order(99999), None);
}

#[test]
fn nad83_lat_long_normalises_into_crs84_long_lat() {
    // EPSG:4269 (NAD83) is lat/long per the EPSG registry: Atlanta written
    // (lat 34.3, lon -83.4). After to_crs84 the internal convention is
    // x = longitude, y = latitude; NAD83 is coincident with WGS84 at metre
    // level, so the DEGREE VALUES survive to ~1e-9 (null Helmert).
    let g = parse_wkt_literal(&format!("<{EPSG}/4269> POINT(34.3 -83.4)")).unwrap();
    let out = to_crs84(&g).unwrap();
    assert_eq!(out.crs, Crs::Crs84);
    let geo_types::Geometry::Point(p) = out.geometry else {
        panic!("point in, point out")
    };
    assert!((p.x() - -83.4).abs() < 1e-9, "lon {}", p.x());
    assert!((p.y() - 34.3).abs() < 1e-9, "lat {}", p.y());
}

#[test]
fn geographic_codes_agree_with_the_same_point_written_as_4326() {
    // The same physical point written as EPSG:4326 (axis-swapped on parse)
    // and as ETRS89 / RGF93 / CGCS2000 (normalised by to_crs84) must land on
    // identical CRS84 coordinates.
    let via_4326 =
        to_crs84(&parse_wkt_literal(&format!("<{EPSG}/4326> POINT(48.8566 2.3522)")).unwrap())
            .unwrap();
    for code in [4258, 4171, 4490] {
        let via_geo = to_crs84(
            &parse_wkt_literal(&format!("<{EPSG}/{code}> POINT(48.8566 2.3522)")).unwrap(),
        )
        .unwrap();
        let (geo_types::Geometry::Point(a), geo_types::Geometry::Point(b)) =
            (&via_4326.geometry, &via_geo.geometry)
        else {
            panic!("points in, points out")
        };
        assert!(
            (a.x() - b.x()).abs() < 1e-9,
            "EPSG:{code} lon {} vs {}",
            a.x(),
            b.x()
        );
        assert!(
            (a.y() - b.y()).abs() < 1e-9,
            "EPSG:{code} lat {} vs {}",
            a.y(),
            b.y()
        );
    }
}

#[test]
fn normalised_geographic_geometries_join_the_metric_machinery() {
    // Two GDA94 points ~1° of longitude apart at lat -33.86 (Sydney):
    // haversine distance ≈ 111.32 km × cos(lat) ≈ 92.4 km. Knocking out the
    // axis swap would place the points near (2.3°, …) with a wildly different
    // separation, so this is red without the normalisation.
    let a = to_crs84(&parse_wkt_literal(&format!("<{EPSG}/4283> POINT(-33.86 151.21)")).unwrap())
        .unwrap();
    let b = to_crs84(&parse_wkt_literal(&format!("<{EPSG}/4283> POINT(-33.86 152.21)")).unwrap())
        .unwrap();
    let d = geof::distance(&a, &b, Unit::Metre).unwrap();
    assert!((d - 92_400.0).abs() < 500.0, "got {d}");
}

#[test]
fn lex_roundtrip_drops_the_geographic_crs_prefix_after_normalisation() {
    // Lexical mirror: an ETRS89 literal comes back as a bare CRS84 lexical
    // form (no CRS prefix) with the axes swapped into long/lat.
    let out = to_crs84_lex(&format!("<{EPSG}/4258> POINT(52.37 4.9)")).unwrap();
    assert!(out.starts_with("POINT(4.9"), "got {out}");
    assert!(!out.contains('<'), "got {out}");
}

#[test]
fn nad27_stays_an_explicit_unsupported_error() {
    // EPSG:4267 (NAD27) needs NADCON grid shifts we do not ship — it must
    // fail loudly, not silently reproject with tens of metres of error.
    assert_eq!(geographic_axis_order(4267), None);
    assert!(proj4_definition(4267).is_none());
    let g = parse_wkt_literal(&format!("<{EPSG}/4267> POINT(34.3 -83.4)")).unwrap();
    assert!(matches!(to_crs84(&g), Err(GeoError::Unsupported(m)) if m.contains("EPSG:4267")));
}
