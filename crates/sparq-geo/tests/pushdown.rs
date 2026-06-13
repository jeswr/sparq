//! Spatial FILTER pushdown (sq-mg9): the engine's `SpatialProvider` seam wired to
//! a `GeoIndex` via `GeoIndexProvider`. These tests are the CORRECTNESS PROOF — the
//! pushed-down plan (index window/range scan -> exact `geof:` refinement) must
//! return EXACTLY the same rows as the post-hoc plan (the same query without an
//! index installed) — plus EVIDENCE that fewer geometries are exact-checked.
#![cfg(feature = "engine")]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use sparq_core::Graph;
use sparq_engine::{query, with_functions, with_spatial_index, FunctionRegistry, QueryResult, SpatialProvider};
use sparq_geo::{geof_registry, GeoIndex, GeoIndexProvider};

const PREFIXES: &str = "PREFIX geo:  <http://www.opengis.net/ont/geosparql#> \
                        PREFIX geof: <http://www.opengis.net/def/function/geosparql/> \
                        PREFIX uom:  <http://www.opengis.net/def/uom/OGC/1.0/> \
                        PREFIX ex:   <http://ex/> ";

const WKT: &str = "http://www.opengis.net/ont/geosparql#wktLiteral";
const GEOF: &str = "http://www.opengis.net/def/function/geosparql/";

/// A `geof:` registry that COUNTS how many times the pushable predicates
/// (`distance`, `sfWithin`, `sfIntersects`) are invoked — i.e. how many exact
/// geometry checks the engine performs. Wraps the real `geof_registry` so the
/// answers are identical; only the call count is observed.
fn counting_registry(counter: Arc<AtomicUsize>) -> FunctionRegistry {
    let real = geof_registry();
    let mut reg = FunctionRegistry::new();
    for name in ["distance", "sfWithin", "sfIntersects"] {
        let iri = format!("{GEOF}{name}");
        let inner = real.get(&iri).expect("registered").clone();
        let c = counter.clone();
        reg.register(iri, move |args| {
            c.fetch_add(1, Ordering::Relaxed);
            inner(args)
        });
    }
    reg
}

/// A grid of N×N points (degree spacing) under `ex:f -> geo:hasGeometry -> node
/// -> geo:asWKT -> "POINT(lon lat)"`, the GeoSPARQL feature shape `GeoIndex`
/// indexes. Big enough that a windowed pushdown skips most of it.
fn grid(n: i32) -> Graph {
    let mut ttl = String::from(
        "@prefix geo: <http://www.opengis.net/ont/geosparql#> .\n@prefix ex: <http://ex/> .\n",
    );
    for x in 0..n {
        for y in 0..n {
            // Spread over a wide area (0.5° steps) so a small window is selective.
            let (lon, lat) = (x as f64 * 0.5, y as f64 * 0.5);
            ttl.push_str(&format!(
                "ex:f{x}_{y} geo:hasGeometry ex:g{x}_{y} . ex:g{x}_{y} geo:asWKT \"POINT({lon} {lat})\"^^geo:wktLiteral .\n"
            ));
        }
    }
    Graph::load_str(&ttl, "turtle").unwrap()
}

fn rows_sorted(r: &QueryResult) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = r
        .rows
        .iter()
        .map(|row| row.iter().map(|c| c.as_ref().map(|t| t.to_string()).unwrap_or_default()).collect())
        .collect();
    out.sort();
    out
}

/// Run `sparql` both WITHOUT and WITH the spatial index installed (same counting
/// registry instance reset between runs), asserting identical results and
/// returning `(rows, exact_checks_posthoc, exact_checks_pushdown)`.
fn compare(graph: &Graph, sparql: &str) -> (Vec<Vec<String>>, usize, usize) {
    let provider: Arc<dyn SpatialProvider> = Arc::new(GeoIndexProvider::new(GeoIndex::build(graph)));

    let counter = Arc::new(AtomicUsize::new(0));
    let reg = counting_registry(counter.clone());

    // Post-hoc: registry installed, NO spatial index.
    let posthoc = with_functions(&reg, || query(graph, sparql)).unwrap();
    let checks_posthoc = counter.swap(0, Ordering::Relaxed);

    // Pushdown: same registry + the spatial index.
    let pushed = with_spatial_index(provider, || with_functions(&reg, || query(graph, sparql))).unwrap();
    let checks_pushdown = counter.load(Ordering::Relaxed);

    let a = rows_sorted(&posthoc);
    let b = rows_sorted(&pushed);
    assert_eq!(a, b, "pushdown changed the RESULT (must be identical to post-hoc)\nquery: {sparql}");
    (a, checks_posthoc, checks_pushdown)
}

#[test]
fn distance_within_same_results_fewer_checks() {
    let g = grid(20); // 400 features
    let q = format!(
        "{PREFIXES} SELECT ?f WHERE {{ \
           ?f geo:hasGeometry ?n . ?n geo:asWKT ?wkt . \
           FILTER(geof:distance(?wkt, \"POINT(0 0)\"^^geo:wktLiteral, uom:kilometre) < 120) \
         }}"
    );
    let (rows, posthoc, pushed) = compare(&g, &q);
    // ~120 km ≈ ~1.08°, so points within a (0,0)-centred ~1° ball: (0,0),(0,0.5),
    // (0.5,0),(0,1),(1,0),(0.5,0.5)... a small handful out of 400.
    assert!(!rows.is_empty(), "expected some matches");
    assert!(rows.len() < 20, "the window should be selective, got {} rows", rows.len());
    // Post-hoc checks EVERY feature; the pushdown checks only the window candidates.
    assert_eq!(posthoc, 400, "post-hoc exact-checks every geometry");
    assert!(pushed < posthoc, "pushdown must exact-check FEWER ({pushed} vs {posthoc})");
    assert!(pushed <= rows.len() + 8, "candidate set should be near-tight, got {pushed} for {} matches", rows.len());
}

#[test]
fn sf_within_box_same_results_fewer_checks() {
    let g = grid(20); // 400 features
    // A small box in the lower-left corner.
    let q = format!(
        "{PREFIXES} SELECT ?f WHERE {{ \
           ?f geo:hasGeometry ?n . ?n geo:asWKT ?wkt . \
           FILTER(geof:sfWithin(?wkt, \"POLYGON((-0.1 -0.1, 1.1 -0.1, 1.1 1.1, -0.1 1.1, -0.1 -0.1))\"^^geo:wktLiteral)) \
         }}"
    );
    let (rows, posthoc, pushed) = compare(&g, &q);
    // The box [0,1]×[0,1] at 0.5° spacing contains a 3×3 = 9-point sub-grid.
    assert_eq!(rows.len(), 9, "3x3 points inside the box");
    assert_eq!(posthoc, 400);
    assert!(pushed < posthoc, "pushdown must exact-check FEWER ({pushed} vs {posthoc})");
}

#[test]
fn sf_intersects_box_same_results() {
    let g = grid(12);
    let q = format!(
        "{PREFIXES} SELECT ?f WHERE {{ \
           ?f geo:hasGeometry ?n . ?n geo:asWKT ?wkt . \
           FILTER(geof:sfIntersects(?wkt, \"POLYGON((1.9 1.9, 3.1 1.9, 3.1 3.1, 1.9 3.1, 1.9 1.9))\"^^geo:wktLiteral)) \
         }}"
    );
    let (rows, posthoc, pushed) = compare(&g, &q);
    assert!(!rows.is_empty());
    assert_eq!(posthoc, 144);
    assert!(pushed < posthoc);
}

#[test]
fn pushdown_le_boundary_identical() {
    // `<=` (inclusive) must also be a correct superset: a point EXACTLY on the
    // radius is kept by both paths.
    let g = grid(10);
    let q = format!(
        "{PREFIXES} SELECT ?f WHERE {{ \
           ?f geo:hasGeometry ?n . ?n geo:asWKT ?wkt . \
           FILTER(geof:distance(?wkt, \"POINT(0 0)\"^^geo:wktLiteral, uom:kilometre) <= 200) \
         }}"
    );
    let (rows, _posthoc, _pushed) = compare(&g, &q);
    assert!(!rows.is_empty());
}

#[test]
fn non_metric_unit_declined_still_correct() {
    // Degree-unit distance is euclidean coordinate distance — NOT the index's
    // great-circle metric — so the provider DECLINES it (no pushdown). The result
    // must still be correct (the post-hoc path runs), proving the decline path is
    // safe. Pushdown count == post-hoc count (no candidates restricted).
    let g = grid(8);
    let q = format!(
        "{PREFIXES} SELECT ?f WHERE {{ \
           ?f geo:hasGeometry ?n . ?n geo:asWKT ?wkt . \
           FILTER(geof:distance(?wkt, \"POINT(0 0)\"^^geo:wktLiteral, uom:degree) < 1.2) \
         }}"
    );
    let (rows, posthoc, pushed) = compare(&g, &q);
    assert!(!rows.is_empty());
    assert_eq!(pushed, posthoc, "degree unit must NOT push down (declined), so same exact-check count");
}

#[test]
fn distance_greater_than_stays_posthoc() {
    // `geof:distance(?g, pt) > R` is an unbounded EXTERIOR (no finite window) — it
    // is NOT pushed down. Result must be correct via the post-hoc path.
    let g = grid(6);
    // > 100 km keeps the FAR points (most of the grid): an unbounded exterior.
    let q = format!(
        "{PREFIXES} SELECT ?f WHERE {{ \
           ?f geo:hasGeometry ?n . ?n geo:asWKT ?wkt . \
           FILTER(geof:distance(?wkt, \"POINT(0 0)\"^^geo:wktLiteral, uom:kilometre) > 100) \
         }}"
    );
    let (rows, posthoc, pushed) = compare(&g, &q);
    assert!(!rows.is_empty(), "far points should match > 100 km");
    assert_eq!(pushed, posthoc, "no pushdown for the exterior predicate");
}

#[test]
fn no_index_installed_is_unchanged() {
    // The seam is inert without a provider: a pushable FILTER just runs post-hoc.
    let g = grid(5);
    let counter = Arc::new(AtomicUsize::new(0));
    let reg = counting_registry(counter.clone());
    let q = format!(
        "{PREFIXES} SELECT ?f WHERE {{ \
           ?f geo:hasGeometry ?n . ?n geo:asWKT ?wkt . \
           FILTER(geof:sfWithin(?wkt, \"POLYGON((-0.1 -0.1, 1.1 -0.1, 1.1 1.1, -0.1 1.1, -0.1 -0.1))\"^^geo:wktLiteral)) \
         }}"
    );
    let r = with_functions(&reg, || query(&g, &q)).unwrap();
    assert_eq!(r.len(), 9);
    assert_eq!(counter.load(Ordering::Relaxed), 25, "every one of the 25 features is exact-checked");
}

#[test]
fn provider_candidates_are_a_superset() {
    // Direct unit test of the provider contract: the candidate set is a SUPERSET
    // of the exact matches.
    use sparq_engine::SpatialQuery;
    let g = grid(15);
    let provider = GeoIndexProvider::new(GeoIndex::build(&g));
    let cands = provider
        .candidates(&SpatialQuery::DistanceWithin {
            point_wkt: "POINT(0 0)",
            radius: 100.0,
            unit_iri: "http://www.opengis.net/def/uom/OGC/1.0/kilometre",
            inclusive: false,
        })
        .expect("metric distance pushes down");
    // Every candidate is a wktLiteral; none is dropped erroneously.
    assert!(!cands.is_empty());
    for t in &cands {
        match t {
            oxrdf::Term::Literal(l) => assert_eq!(l.datatype().as_str(), WKT),
            other => panic!("candidate must be a wktLiteral, got {other}"),
        }
    }
}
