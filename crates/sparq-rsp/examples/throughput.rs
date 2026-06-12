//! Throughput measurement for the README: pushes synthetic sensor readings
//! through a registered continuous query and reports triples/s and windows/s,
//! for each window-materialisation strategy ([`EvalMode`]).
//!
//! The LIBRARY never reads a clock; this harness (an example, not the crate)
//! times the run from outside. Run with:
//!
//! ```sh
//! cargo run --release -p sparq-rsp --example throughput
//! ```

use std::time::Instant;

use oxrdf::{Literal, NamedNode, Term};
use sparq_rsp::{ContinuousQuery, EvalMode, WindowSpec, WindowedStream};

const N_TRIPLES: u64 = 1_000_000;
const N_SENSORS: u64 = 100;

fn reading(sensor: u64, v: u64) -> [Term; 3] {
    [
        NamedNode::new_unchecked(format!("http://ex/sensor/{sensor}")).into(),
        NamedNode::new_unchecked("http://ex/value").into(),
        Literal::from(v as i64).into(),
    ]
}

/// One scenario × one strategy: windows of `range`/`step`, one triple per tick.
fn run(label: &str, range: u64, step: u64, sparql: &str, mode: EvalMode) {
    let mut q = ContinuousQuery::register(sparql, WindowSpec::time(range, step))
        .expect("valid query")
        .with_mode(mode);
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
        "{label:<48} {:<16} {:>7.2} Mtriples/s  {:>9.0} windows/s  ({windows} windows, {rows} rows, {dt:.2}s)",
        format!("{mode:?}"),
        N_TRIPLES as f64 / dt / 1e6,
        windows as f64 / dt,
    );
}

fn run_all_modes(label: &str, range: u64, step: u64, sparql: &str) {
    for mode in [EvalMode::Rebuild, EvalMode::PersistentDict, EvalMode::Delta] {
        run(label, range, step, sparql, mode);
    }
    println!();
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
        "{:<48} {:<16} {:>7.2} Mtriples/s  {:>9.0} windows/s  ({windows} windows, {dt:.2}s)\n",
        format!("windowing only (no query), RANGE {window_ticks}"),
        "-",
        N_TRIPLES as f64 / dt / 1e6,
        windows as f64 / dt,
    );
}

fn main() {
    println!("{N_TRIPLES} triples, {N_SENSORS} sensors, 1 triple/tick, time windows\n");
    run_windowing_only(1_000);
    let avg = "SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/value> ?v }";
    run_all_modes("AVG, RANGE 100 STEP 100 (tumbling)", 100, 100, avg);
    run_all_modes("AVG, RANGE 1000 STEP 1000 (tumbling)", 1_000, 1_000, avg);
    run_all_modes("AVG, RANGE 10000 STEP 10000 (tumbling)", 10_000, 10_000, avg);
    run_all_modes("AVG, RANGE 1000 STEP 100 (sliding 10x overlap)", 1_000, 100, avg);
    run_all_modes("AVG, RANGE 10000 STEP 1000 (sliding 10x overlap)", 10_000, 1_000, avg);
    run_all_modes(
        "AVG per sensor (GROUP BY), RANGE 1000 STEP 1000",
        1_000,
        1_000,
        "SELECT ?s (AVG(?v) AS ?avg) WHERE { ?s <http://ex/value> ?v } GROUP BY ?s",
    );
}
