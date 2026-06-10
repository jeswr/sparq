//! GeoIndex benchmark: build + query latency on ~100k random points.
//!
//! Run with:
//! ```sh
//! cargo run --release -p sparq-geo --example bench_geo
//! ```
//!
//! Generates N random CRS84 points (seeded) over a country-sized window
//! (8° x 8°), loads them into a sparq Graph as `geo:asWKT` N-Triples, builds
//! the GeoIndex, then measures `within_distance` / `nearest` / `intersects`
//! latency against query points drawn from the same window. Numbers land in
//! the crate README.

use std::time::Instant;

use geo_types::Point;
use rand::{rngs::StdRng, Rng, SeedableRng};
use sparq_core::Graph;
use sparq_geo::{parse_wkt_literal, GeoIndex};

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(100_000);
    let mut rng = StdRng::seed_from_u64(20260610);
    let (lon, lat) = ((-4.0, 4.0), (47.0, 55.0)); // ~8°x8°, France-ish

    // ---- data generation + graph load ------------------------------------------
    let mut nt = String::with_capacity(n * 140);
    for i in 0..n {
        let x = rng.random_range(lon.0..lon.1);
        let y = rng.random_range(lat.0..lat.1);
        nt.push_str(&format!(
            "<http://example.org/e{i}> <http://www.opengis.net/ont/geosparql#asWKT> \
             \"POINT({x} {y})\"^^<http://www.opengis.net/ont/geosparql#wktLiteral> .\n"
        ));
    }
    let t = Instant::now();
    let graph = Graph::load_str(&nt, "ntriples").unwrap();
    let load = t.elapsed();
    println!("graph load     : {n} asWKT triples in {load:.2?}");

    // ---- index build -------------------------------------------------------------
    let t = Instant::now();
    let index = GeoIndex::build(&graph);
    let build = t.elapsed();
    println!(
        "index build    : {} entries in {build:.2?} ({:.2} Mentries/s)",
        index.len(),
        index.len() as f64 / build.as_secs_f64() / 1e6
    );

    let mut query_points: Vec<Point<f64>> = Vec::new();
    for _ in 0..1000 {
        query_points
            .push(Point::new(rng.random_range(lon.0..lon.1), rng.random_range(lat.0..lat.1)));
    }

    // ---- within_distance ----------------------------------------------------------
    for radius in [1_000.0, 10_000.0, 50_000.0] {
        let t = Instant::now();
        let mut total = 0usize;
        for &p in &query_points {
            total += index.within_distance(p, radius, None).len();
        }
        let el = t.elapsed();
        println!(
            "within {:>5}km : {:>8.1} µs/query ({:.1} avg hits)",
            radius / 1000.0,
            el.as_micros() as f64 / query_points.len() as f64,
            total as f64 / query_points.len() as f64
        );
    }

    // ---- nearest -------------------------------------------------------------------
    for k in [1, 10, 100] {
        let t = Instant::now();
        let mut total = 0usize;
        for &p in &query_points {
            total += index.nearest(p, k).len();
        }
        let el = t.elapsed();
        assert_eq!(total, k * query_points.len());
        println!(
            "nearest k={k:<4}  : {:>8.1} µs/query",
            el.as_micros() as f64 / query_points.len() as f64
        );
    }

    // ---- intersects ------------------------------------------------------------------
    let mut polygons = Vec::new();
    for _ in 0..100 {
        let x = rng.random_range(lon.0..lon.1 - 0.5);
        let y = rng.random_range(lat.0..lat.1 - 0.5);
        let (w, h) = (0.5, 0.5); // ~55km x 55km boxes
        polygons.push(
            parse_wkt_literal(&format!(
                "POLYGON(({x} {y}, {x2} {y}, {x2} {y2}, {x} {y2}, {x} {y}))",
                x2 = x + w,
                y2 = y + h
            ))
            .unwrap(),
        );
    }
    let t = Instant::now();
    let mut total = 0usize;
    for poly in &polygons {
        total += index.intersects(poly).len();
    }
    let el = t.elapsed();
    println!(
        "intersects box : {:>8.1} µs/query ({:.1} avg hits, 0.5°x0.5° boxes)",
        el.as_micros() as f64 / polygons.len() as f64,
        total as f64 / polygons.len() as f64
    );
}
