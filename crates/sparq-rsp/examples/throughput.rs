//! Throughput measurement for the README: pushes synthetic sensor readings
//! through a registered continuous query and reports triples/s and windows/s.
//!
//! The LIBRARY never reads a clock; this harness (an example, not the crate)
//! times the run from outside. Run with:
//!
//! ```sh
//! cargo run --release -p sparq-rsp --example throughput
//! ```

use std::time::Instant;

use oxrdf::{Literal, NamedNode, Term};
use sparq_rsp::{ContinuousQuery, WindowSpec, WindowedStream};

const N_TRIPLES: u64 = 1_000_000;
const N_SENSORS: u64 = 100;

fn reading(sensor: u64, v: u64) -> [Term; 3] {
    [
        NamedNode::new_unchecked(format!("http://ex/sensor/{sensor}")).into(),
        NamedNode::new_unchecked("http://ex/value").into(),
        Literal::from(v as i64).into(),
    ]
}

/// One scenario: tumbling windows of `window_ticks`, one triple per tick.
fn run(label: &str, window_ticks: u64, sparql: &str) {
    let mut q = ContinuousQuery::register(sparql, WindowSpec::time(window_ticks, window_ticks))
        .expect("valid query");
    let mut windows = 0u64;
    let mut rows = 0u64;
    let start = Instant::now();
    for ts in 0..N_TRIPLES {
        q.push(reading(ts % N_SENSORS, ts % 50), ts, |r| {
            windows += 1;
            rows += r.rows.len() as u64;
        })
        .expect("eval");
    }
    q.flush(|r| {
        windows += 1;
        rows += r.rows.len() as u64;
    })
    .expect("eval");
    let dt = start.elapsed().as_secs_f64();
    println!(
        "{label:<55} {:>7.2} Mtriples/s  {:>9.0} windows/s  ({windows} windows, {rows} result rows, {dt:.2}s)",
        N_TRIPLES as f64 / dt / 1e6,
        windows as f64 / dt,
    );
}

/// Windowing only (no query): the bare S2R operator cost.
fn run_windowing_only(window_ticks: u64) {
    let mut ws = WindowedStream::empty(WindowSpec::time(window_ticks, window_ticks));
    let mut windows = 0u64;
    let start = Instant::now();
    for ts in 0..N_TRIPLES {
        ws.push(reading(ts % N_SENSORS, ts % 50), ts);
        windows += ws.take_closed().len() as u64;
    }
    windows += ws.flush().len() as u64;
    let dt = start.elapsed().as_secs_f64();
    println!(
        "{:<55} {:>7.2} Mtriples/s  {:>9.0} windows/s  ({windows} windows, {dt:.2}s)",
        format!("windowing only (no query), RANGE {window_ticks}"),
        N_TRIPLES as f64 / dt / 1e6,
        windows as f64 / dt,
    );
}

fn main() {
    println!("{N_TRIPLES} triples, {N_SENSORS} sensors, 1 triple/tick, tumbling windows\n");
    run_windowing_only(1_000);
    let avg = "SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/value> ?v }";
    run("AVG per window, RANGE 100 (100 triples/window)", 100, avg);
    run("AVG per window, RANGE 1000 (1k triples/window)", 1_000, avg);
    run("AVG per window, RANGE 10000 (10k triples/window)", 10_000, avg);
    run(
        "AVG per sensor per window (GROUP BY), RANGE 1000",
        1_000,
        "SELECT ?s (AVG(?v) AS ?avg) WHERE { ?s <http://ex/value> ?v } GROUP BY ?s",
    );
}
