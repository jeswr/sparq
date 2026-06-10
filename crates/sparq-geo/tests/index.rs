//! GeoIndex correctness: R-tree query results vs brute force over random points.

use geo_types::Point;
use oxrdf::Term;
use rand::{rngs::StdRng, Rng, SeedableRng};
use sparq_core::Graph;
use sparq_geo::{geof, parse_wkt_literal, GeoIndex};

/// A graph of `n` random points (seeded) in the given long/lat window, with
/// `geo:asWKT` attached directly to each entity.
fn random_point_graph(n: usize, seed: u64, lon: (f64, f64), lat: (f64, f64)) -> Graph {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut nt = String::new();
    for i in 0..n {
        let x = rng.random_range(lon.0..lon.1);
        let y = rng.random_range(lat.0..lat.1);
        nt.push_str(&format!(
            "<http://example.org/e{i}> <http://www.opengis.net/ont/geosparql#asWKT> \
             \"POINT({x} {y})\"^^<http://www.opengis.net/ont/geosparql#wktLiteral> .\n"
        ));
    }
    Graph::load_str(&nt, "ntriples").unwrap()
}

/// Brute-force (entity, metres) pairs, nearest first.
fn brute_force(index: &GeoIndex, center: Point<f64>) -> Vec<(Term, f64)> {
    let mut all: Vec<(Term, f64)> = index
        .entries()
        .iter()
        .map(|e| {
            let d = geof::point_to_geometry_meters(center, &e.geometry.geometry).unwrap();
            (e.entity.clone(), d)
        })
        .collect();
    all.sort_by(|a, b| a.1.total_cmp(&b.1));
    all
}

#[test]
fn within_distance_matches_brute_force() {
    let graph = random_point_graph(2000, 42, (-1.0, 1.0), (50.0, 52.0));
    let index = GeoIndex::build(&graph);
    assert_eq!(index.len(), 2000);
    assert_eq!(index.skipped(), 0);

    let mut rng = StdRng::seed_from_u64(7);
    for _ in 0..50 {
        let center = Point::new(rng.random_range(-1.2..1.2), rng.random_range(49.8..52.2));
        let radius = rng.random_range(500.0..40_000.0);
        let got = index.within_distance(center, radius, None);
        let want: Vec<(Term, f64)> = brute_force(&index, center)
            .into_iter()
            .take_while(|(_, d)| *d <= radius)
            .collect();
        assert_eq!(got.len(), want.len(), "center {center:?} radius {radius}");
        for ((ge, gd), (we, wd)) in got.iter().zip(&want) {
            assert_eq!(*ge, we, "center {center:?} radius {radius}");
            assert!((gd - wd).abs() < 1e-9);
        }
    }
}

#[test]
fn nearest_matches_brute_force() {
    let graph = random_point_graph(2000, 99, (-1.0, 1.0), (50.0, 52.0));
    let index = GeoIndex::build(&graph);

    let mut rng = StdRng::seed_from_u64(13);
    for _ in 0..50 {
        let center = Point::new(rng.random_range(-1.5..1.5), rng.random_range(49.5..52.5));
        let k = rng.random_range(1..40);
        let got = index.nearest(center, k);
        let want = brute_force(&index, center);
        assert_eq!(got.len(), k);
        for (i, (ge, gd)) in got.iter().enumerate() {
            // Compare by distance (ties may order differently than the brute sort).
            assert!(
                (gd - want[i].1).abs() < 1e-9,
                "k={k} i={i}: got {ge}@{gd}, want {}@{}",
                want[i].0,
                want[i].1
            );
        }
    }
}

#[test]
fn nearest_with_k_larger_than_index() {
    let graph = random_point_graph(5, 1, (-1.0, 1.0), (50.0, 52.0));
    let index = GeoIndex::build(&graph);
    let got = index.nearest(Point::new(0.0, 51.0), 50);
    assert_eq!(got.len(), 5);
    // Sorted ascending.
    assert!(got.windows(2).all(|w| w[0].1 <= w[1].1));
}

#[test]
fn within_distance_limit_truncates_nearest_first() {
    let graph = random_point_graph(500, 3, (-1.0, 1.0), (50.0, 52.0));
    let index = GeoIndex::build(&graph);
    let center = Point::new(0.0, 51.0);
    let all = index.within_distance(center, 1e7, None);
    let five = index.within_distance(center, 1e7, Some(5));
    assert_eq!(all.len(), 500);
    assert_eq!(five, all[..5]);
}

#[test]
fn intersects_matches_brute_force() {
    let graph = random_point_graph(2000, 17, (-1.0, 1.0), (50.0, 52.0));
    let index = GeoIndex::build(&graph);
    let polygon =
        parse_wkt_literal("POLYGON((-0.5 50.5, 0.5 50.5, 0.5 51.5, -0.5 51.5, -0.5 50.5))")
            .unwrap();
    let mut got: Vec<Term> = index.intersects(&polygon).into_iter().cloned().collect();
    let mut want: Vec<Term> = index
        .entries()
        .iter()
        .filter(|e| geof::sf_intersects(&e.geometry, &polygon).unwrap())
        .map(|e| e.entity.clone())
        .collect();
    got.sort_by_key(|t| t.to_string());
    want.sort_by_key(|t| t.to_string());
    assert!(!want.is_empty());
    assert_eq!(got, want);
}

#[test]
fn empty_and_geo_free_graphs_yield_empty_indexes() {
    let graph = Graph::load_str("", "ntriples").unwrap();
    let index = GeoIndex::build(&graph);
    assert!(index.is_empty());
    assert!(index.nearest(Point::new(0.0, 0.0), 3).is_empty());
    assert!(index.within_distance(Point::new(0.0, 0.0), 1000.0, None).is_empty());

    let graph =
        Graph::load_str("<http://e.org/a> <http://e.org/p> <http://e.org/b> .", "ntriples")
            .unwrap();
    assert!(GeoIndex::build(&graph).is_empty());
}

#[test]
fn unparseable_and_non_geographic_literals_are_skipped() {
    let nt = r##"<http://e.org/good> <http://www.opengis.net/ont/geosparql#asWKT> "POINT(1 2)"^^<http://www.opengis.net/ont/geosparql#wktLiteral> .
<http://e.org/bad> <http://www.opengis.net/ont/geosparql#asWKT> "POINT(broken"^^<http://www.opengis.net/ont/geosparql#wktLiteral> .
<http://e.org/plain> <http://www.opengis.net/ont/geosparql#asWKT> "POINT(3 4)" .
<http://e.org/bng> <http://www.opengis.net/ont/geosparql#asWKT> "<http://www.opengis.net/def/crs/EPSG/0/27700> POINT(530000 180000)"^^<http://www.opengis.net/ont/geosparql#wktLiteral> .
<http://e.org/empty> <http://www.opengis.net/ont/geosparql#asWKT> "POINT EMPTY"^^<http://www.opengis.net/ont/geosparql#wktLiteral> .
"##;
    let graph = Graph::load_str(nt, "ntriples").unwrap();
    let index = GeoIndex::build(&graph);
    assert_eq!(index.len(), 1);
    assert_eq!(index.skipped(), 4);
}
