//! End-to-end: load a Turtle document with the GeoSPARQL core shape
//! (feature -> geo:hasGeometry -> geometry node -> geo:asWKT), build the
//! index, run nearest / within-distance / intersects queries.

use geo_types::Point;
use oxrdf::{NamedNode, Term};
use sparq_core::Graph;
use sparq_geo::GeoIndex;

const TTL: &str = r#"
@prefix geo:  <http://www.opengis.net/ont/geosparql#> .
@prefix ex:   <http://example.org/> .

# Features with geometry nodes (the canonical GeoSPARQL shape).
ex:london a ex:City ;
    geo:hasGeometry ex:londonGeom .
ex:londonGeom a geo:Geometry ;
    geo:asWKT "POINT(-0.1276 51.5074)"^^geo:wktLiteral .

ex:paris a ex:City ;
    geo:hasDefaultGeometry ex:parisGeom .
ex:parisGeom geo:asWKT "POINT(2.3496 48.8530)"^^geo:wktLiteral .

ex:brussels a ex:City ;
    geo:hasGeometry ex:brusselsGeom .
ex:brusselsGeom geo:asWKT "POINT(4.3517 50.8503)"^^geo:wktLiteral .

# EPSG:4326 literal (lat/long axis order): Amsterdam.
ex:amsterdam a ex:City ;
    geo:hasGeometry ex:amsterdamGeom .
ex:amsterdamGeom geo:asWKT "<http://www.opengis.net/def/crs/EPSG/0/4326> POINT(52.3676 4.9041)"^^geo:wktLiteral .

# A bare geometry: asWKT attached straight to the entity (no hasGeometry).
ex:thames geo:asWKT "LINESTRING(-0.50 51.45, -0.12 51.50, 0.30 51.47)"^^geo:wktLiteral .

# A polygon feature: an approximate box around inner London.
ex:innerLondon a ex:Region ;
    geo:hasGeometry ex:innerLondonGeom .
ex:innerLondonGeom geo:asWKT "POLYGON((-0.25 51.45, 0.05 51.45, 0.05 51.57, -0.25 51.57, -0.25 51.45))"^^geo:wktLiteral .
"#;

fn iri(suffix: &str) -> Term {
    Term::NamedNode(NamedNode::new_unchecked(format!(
        "http://example.org/{suffix}"
    )))
}

#[test]
fn turtle_to_index_to_queries() {
    let graph = Graph::load_str(TTL, "turtle").unwrap();
    let index = GeoIndex::build(&graph);
    // 6 geometries, all indexed; entities resolve through hasGeometry /
    // hasDefaultGeometry, or stay the subject for the bare ex:thames.
    assert_eq!(index.len(), 6);
    assert_eq!(index.skipped(), 0);

    let entities: Vec<&Term> = index.entries().map(|e| &e.entity).collect();
    for ent in [
        "london",
        "paris",
        "brussels",
        "amsterdam",
        "thames",
        "innerLondon",
    ] {
        assert!(entities.contains(&&iri(ent)), "missing {ent}");
    }
    // The geometry NODES are not the reported entities.
    assert!(!entities.contains(&&iri("londonGeom")));

    // Nearest to central London: the Thames line and inner-London polygon
    // touch/contain the point (distance 0), then London itself, then the
    // continental cities — Brussels before Paris from London, Amsterdam last.
    let center = Point::new(-0.1276, 51.5074); // = ex:london
    let got: Vec<(&Term, f64)> = index.nearest(center, 6);
    let names: Vec<&Term> = got.iter().map(|(t, _)| *t).collect();
    assert_eq!(got.len(), 6);
    assert_eq!(
        names[3..],
        [&iri("brussels"), &iri("paris"), &iri("amsterdam")]
    );
    assert!(names[..3].contains(&&iri("london")));
    assert!(names[..3].contains(&&iri("thames")));
    assert!(names[..3].contains(&&iri("innerLondon")));
    // Paris ≈ 341 km from London; Amsterdam ≈ 358 km.
    let paris_d = got.iter().find(|(t, _)| *t == &iri("paris")).unwrap().1;
    assert!((paris_d - 341_000.0).abs() < 4_000.0, "got {paris_d}");

    // Within 250 km of London: only the London-area entities.
    let within: Vec<&Term> = index
        .within_distance(center, 250_000.0, None)
        .into_iter()
        .map(|(t, _)| t)
        .collect();
    assert_eq!(within.len(), 3);
    assert!(!within.contains(&&iri("paris")));

    // Intersects a box over the Channel + Paris: Paris only.
    let hits = index
        .intersects_wkt("POLYGON((1.0 48.0, 3.0 48.0, 3.0 49.5, 1.0 49.5, 1.0 48.0))")
        .unwrap();
    assert_eq!(hits, [&iri("paris")]);

    // Intersects a thin box across the Thames line: the line, the inner-London
    // polygon, and the London point itself all fall inside it.
    let mut hits = index
        .intersects_wkt(
            "POLYGON((-0.13 51.40, -0.11 51.40, -0.11 51.60, -0.13 51.60, -0.13 51.40))",
        )
        .unwrap();
    hits.sort_by_key(|t| t.to_string());
    assert_eq!(hits, [&iri("innerLondon"), &iri("london"), &iri("thames")]);
}

#[test]
fn shared_geometry_node_reports_every_owning_feature() {
    let ttl = r#"
@prefix geo: <http://www.opengis.net/ont/geosparql#> .
@prefix ex:  <http://example.org/> .
ex:a geo:hasGeometry ex:g .
ex:b geo:hasGeometry ex:g .
ex:g geo:asWKT "POINT(1 1)"^^geo:wktLiteral .
"#;
    let graph = Graph::load_str(ttl, "turtle").unwrap();
    let index = GeoIndex::build(&graph);
    assert_eq!(index.len(), 2);
    let mut entities: Vec<String> = index.entries().map(|e| e.entity.to_string()).collect();
    entities.sort();
    assert_eq!(
        entities,
        ["<http://example.org/a>", "<http://example.org/b>"]
    );
}
