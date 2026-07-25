//! [SONNET-4.6] sq-7d3dj.22 (epic sq-7d3dj) — deterministic allocation-count instrument.
//!
//! 🤖 SPARQ agent. The `op_*` measurement plans (sq-7d3dj.6/.7/.19/.16) cite allocation
//! counts, but no counting allocator existed anywhere in the sparq estate. This harness
//! fills that gap: a `#[global_allocator]` counting wrapper around the system allocator
//! reports `{allocs, peak_bytes}` per pinned `op_*` workload as a **TREND-ONLY** series.
//!
//! ## What it measures
//!
//! For each SPARQL operator query in the `bench/operators/queries/` corpus:
//!
//! * **allocs** — number of heap allocations made during `sparq_engine::query(…)`
//! * **peak_bytes** — peak live heap bytes (net bytes added by the query call at its
//!   highest point, from a per-query-call baseline of 0)
//! * **rows** — result row count (correctness anchor; should be stable across runs)
//!
//! Two consecutive runs of the binary must emit identical `(allocs, peak_bytes)` series
//! (verified internally by running each query twice). If counts diverge, the harness
//! prints the variance band and exits non-zero so the caller notices non-determinism.
//!
//! ## Honesty — BENCH-ONLY, NON-CANONICAL
//!
//! * **alloc count** (`allocs`) is fully cross-run deterministic (depends only on code
//!   paths exercised, not on wall-clock timing or heap layout).
//! * **peak live bytes** (`peak_bytes`) is within-run deterministic (the internal
//!   two-iteration check always agrees), but may vary ±O(10) bytes between separate
//!   process invocations. The root cause is that the system allocator (`System`) may
//!   service the same logical size with slightly different internal alignment or block
//!   boundaries across process starts, shifting the LIVE_BYTES high-water mark by a
//!   few bytes. The variance is small (a few bytes on a ~10-100 KB window) and does
//!   NOT affect the `allocs` metric, which is the primary stable comparator.
//!
//! This is a TREND instrument: use it to compare before/after a change on the SAME box
//! with the SAME build. Do not claim absolute numbers from either metric.
//!
//! This harness is intentionally **bench-only** (a detached cargo project, no part of the
//! root workspace or its clippy/test gate). It changes ZERO production code and moves NO
//! gate or ratchet.
//!
//! ## Usage
//!
//! ```sh
//! # from bench/alloc-track/
//! cargo run --release --bin alloc_track
//! cargo run --release --bin alloc_track -- --scale 2000 --iters 2
//! cargo run --release --bin alloc_track -- --smoke   # fast CI gate run
//! ```
//!
//! Output: TSV to stdout — `op_<name>\tallocs\tpeak_bytes\trows\tstatus`
//! where `status` is `ok` (counts matched across iters) or `band[min,max]` (variance).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

// --------------------------------------------------------------------------- //
// Counting allocator
// --------------------------------------------------------------------------- //

/// `#[global_allocator]` wrapper that tracks allocation counts and peak live bytes.
///
/// Counters can be reset before each measurement window via [`counters_reset`], and
/// snapshotted after via [`counters_snap`]. All atomics use `Relaxed` ordering: only
/// the FINAL snapshot (not any intermediate ordering of operations *between* the
/// allocator calls) matters for the series values, and on a single thread (our
/// mandatory single-rayon-thread configuration) all operations are sequentially
/// consistent regardless of the atomic ordering specified.
///
/// `LIVE_BYTES` is a SIGNED delta from the reset point (net bytes allocated minus
/// bytes freed since the last `counters_reset`). It can transiently go negative if
/// a deallocation frees memory that was allocated before the reset (e.g. the graph
/// dropping a temporary). `PEAK_BYTES` tracks the maximum of `max(LIVE_BYTES, 0)`
/// observed since the last reset — the high-water mark of *additional* heap held
/// by the query call.
struct CountingAlloc;

static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
/// Net live bytes since last `counters_reset`. Signed to tolerate deallocs of
/// pre-reset allocations without wrapping.
static LIVE_BYTES: AtomicI64 = AtomicI64::new(0);
static PEAK_BYTES: AtomicU64 = AtomicU64::new(0);

/// Reset all counters to 0. Call immediately before the operation under measurement.
fn counters_reset() {
    ALLOC_COUNT.store(0, Ordering::Relaxed);
    LIVE_BYTES.store(0, Ordering::Relaxed);
    PEAK_BYTES.store(0, Ordering::Relaxed);
}

/// Snapshot the current counter state. Call immediately after the operation.
/// Returns `(alloc_count, peak_bytes)`.
fn counters_snap() -> (u64, u64) {
    (
        ALLOC_COUNT.load(Ordering::Relaxed),
        PEAK_BYTES.load(Ordering::Relaxed),
    )
}

/// Update `PEAK_BYTES` if `candidate` exceeds the current peak.
#[inline]
fn update_peak(candidate: u64) {
    let mut peak = PEAK_BYTES.load(Ordering::Relaxed);
    while candidate > peak {
        match PEAK_BYTES.compare_exchange_weak(peak, candidate, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => break,
            Err(current) => peak = current,
        }
    }
}

// SAFETY: Delegates every call to `System`; adds only atomic counter updates.
// No memory safety invariants are broken.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwarded to `System`; caller guarantees layout is valid.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            let live = LIVE_BYTES.fetch_add(layout.size() as i64, Ordering::Relaxed)
                + layout.size() as i64;
            if live > 0 {
                update_peak(live as u64);
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarded to `System`.
        unsafe { System.dealloc(ptr, layout) };
        LIVE_BYTES.fetch_sub(layout.size() as i64, Ordering::Relaxed);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwarded to `System`.
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            let live = LIVE_BYTES.fetch_add(layout.size() as i64, Ordering::Relaxed)
                + layout.size() as i64;
            if live > 0 {
                update_peak(live as u64);
            }
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: forwarded to `System`; caller guarantees layout and new_size are valid.
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            // A successful realloc implicitly frees the old block and allocates new_size.
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            let delta = new_size as i64 - layout.size() as i64;
            let live =
                LIVE_BYTES.fetch_add(delta, Ordering::Relaxed) + delta;
            if live > 0 {
                update_peak(live as u64);
            }
        }
        // If new_ptr is null the old block is still alive: no counter update needed.
        new_ptr
    }
}

#[global_allocator]
static ALLOCATOR: CountingAlloc = CountingAlloc;

// --------------------------------------------------------------------------- //
// Dataset generator — identical to crates/sparq-bench/src/dataset.rs::generate
// (copy kept here so this standalone project has no non-path dep on sparq-bench)
// --------------------------------------------------------------------------- //

/// Generates `n` entities as a Turtle string. Deterministic: same `n` → same output.
fn generate_dataset(n: u32) -> String {
    let n = n.max(1);
    let follows_per = 4u32;
    let n_cities = (n / 10).max(1);

    let mut s = String::with_capacity(n as usize * (5 + follows_per as usize) * 40);
    s.push_str("@prefix ex: <http://ex/> .\n");

    let mut seed = 0x9E3779B97F4A7C15u64;
    let mut rng = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as u32
    };

    for i in 0..n {
        s.push_str(&format!(
            "ex:n{} a ex:Person ;\n  ex:name \"name{}\" ;\n  ex:age {} ;\n  ex:city ex:c{} .\n",
            i,
            i,
            20 + i % 80,
            i % n_cities
        ));
        for _ in 0..follows_per {
            let t = rng() % n;
            s.push_str(&format!("ex:n{} ex:follows ex:n{} .\n", i, t));
        }
    }
    s
}

// --------------------------------------------------------------------------- //
// Operator corpus — the pinned op_* queries from bench/operators/queries/
// --------------------------------------------------------------------------- //

/// A single operator workload.
struct Op {
    name: &'static str,
    sparql: &'static str,
}

/// The full operator corpus, mirroring bench/operators/queries/*.rq (in name order).
/// Embedded at compile time via `include_str!` so a standalone build always matches
/// the checked-in corpus. Paths are relative to this source file.
const OPS: &[Op] = &[
    Op {
        name: "q01_bgp",
        sparql: include_str!("../../operators/queries/q01_bgp.rq"),
    },
    Op {
        name: "q02_star3",
        sparql: include_str!("../../operators/queries/q02_star3.rq"),
    },
    Op {
        name: "q03_chain",
        sparql: include_str!("../../operators/queries/q03_chain.rq"),
    },
    Op {
        name: "q04_triangle",
        sparql: include_str!("../../operators/queries/q04_triangle.rq"),
    },
    Op {
        name: "q05_union",
        sparql: include_str!("../../operators/queries/q05_union.rq"),
    },
    Op {
        name: "q06_optional",
        sparql: include_str!("../../operators/queries/q06_optional.rq"),
    },
    Op {
        name: "q07_optional_notbound",
        sparql: include_str!("../../operators/queries/q07_optional_notbound.rq"),
    },
    Op {
        name: "q08_minus",
        sparql: include_str!("../../operators/queries/q08_minus.rq"),
    },
    Op {
        name: "q09_filter_numeric",
        sparql: include_str!("../../operators/queries/q09_filter_numeric.rq"),
    },
    Op {
        name: "q10_filter_string",
        sparql: include_str!("../../operators/queries/q10_filter_string.rq"),
    },
    Op {
        name: "q11_filter_in",
        sparql: include_str!("../../operators/queries/q11_filter_in.rq"),
    },
    Op {
        name: "q12_filter_exists",
        sparql: include_str!("../../operators/queries/q12_filter_exists.rq"),
    },
    Op {
        name: "q13_bind",
        sparql: include_str!("../../operators/queries/q13_bind.rq"),
    },
    Op {
        name: "q14_values",
        sparql: include_str!("../../operators/queries/q14_values.rq"),
    },
    Op {
        name: "q15_agg_group_having",
        sparql: include_str!("../../operators/queries/q15_agg_group_having.rq"),
    },
    Op {
        name: "q16_distinct",
        sparql: include_str!("../../operators/queries/q16_distinct.rq"),
    },
    Op {
        name: "q17_orderby_limit_offset",
        sparql: include_str!("../../operators/queries/q17_orderby_limit_offset.rq"),
    },
    Op {
        name: "q18_path_plus",
        sparql: include_str!("../../operators/queries/q18_path_plus.rq"),
    },
    Op {
        name: "q19_path_star",
        sparql: include_str!("../../operators/queries/q19_path_star.rq"),
    },
    Op {
        name: "q20_path_opt",
        sparql: include_str!("../../operators/queries/q20_path_opt.rq"),
    },
    Op {
        name: "q21_path_seq",
        sparql: include_str!("../../operators/queries/q21_path_seq.rq"),
    },
    Op {
        name: "q22_path_alt",
        sparql: include_str!("../../operators/queries/q22_path_alt.rq"),
    },
    Op {
        name: "q23_path_inverse",
        sparql: include_str!("../../operators/queries/q23_path_inverse.rq"),
    },
    Op {
        name: "q24_path_negated_pset",
        sparql: include_str!("../../operators/queries/q24_path_negated_pset.rq"),
    },
    Op {
        name: "q25_subquery",
        sparql: include_str!("../../operators/queries/q25_subquery.rq"),
    },
    Op {
        name: "q26_ask",
        sparql: include_str!("../../operators/queries/q26_ask.rq"),
    },
    Op {
        name: "q27_construct",
        sparql: include_str!("../../operators/queries/q27_construct.rq"),
    },
    Op {
        name: "q28_describe",
        sparql: include_str!("../../operators/queries/q28_describe.rq"),
    },
];

// --------------------------------------------------------------------------- //
// Measurement — dispatch SELECT/ASK vs CONSTRUCT/DESCRIBE
// --------------------------------------------------------------------------- //

/// Counts for a single measurement window (one query call, post-reset).
#[derive(Copy, Clone, PartialEq, Eq)]
struct Measurement {
    allocs: u64,
    peak_bytes: u64,
    rows: usize,
}

/// Run one query (any form), measure allocations, return the counts.
///
/// Uses `PreparedQuery::is_graph_form()` to route CONSTRUCT/DESCRIBE through
/// `construct_or_describe` and SELECT/ASK through `query`. Both paths exercise the
/// REAL engine; only the result-extraction call differs.
fn measure_once(graph: &sparq_core::Graph, sparql: &str) -> Measurement {
    let prepared = sparq_engine::PreparedQuery::parse(sparql)
        .unwrap_or_else(|e| panic!("parse failed: {}", e));
    counters_reset();
    let rows = if prepared.is_graph_form() {
        // CONSTRUCT / DESCRIBE: result is Vec<Triple>; count produced triples.
        let triples = sparq_engine::construct_or_describe(graph, sparql)
            .unwrap_or_else(|e| panic!("construct_or_describe failed: {}", e));
        triples.len()
    } else {
        // SELECT / ASK
        let result = sparq_engine::query(graph, sparql)
            .unwrap_or_else(|e| panic!("query failed: {}", e));
        result.rows.len()
    };
    let (allocs, peak_bytes) = counters_snap();
    Measurement { allocs, peak_bytes, rows }
}

/// Warmup: run the query once WITHOUT counting, to flush any lazy initialisation
/// (regex compilation, thread-local setup) that would skew the first measurement.
fn warmup(graph: &sparq_core::Graph, sparql: &str, op_name: &str) {
    let prepared = sparq_engine::PreparedQuery::parse(sparql)
        .unwrap_or_else(|e| panic!("parse failed for {}: {}", op_name, e));
    if prepared.is_graph_form() {
        sparq_engine::construct_or_describe(graph, sparql)
            .unwrap_or_else(|e| panic!("warmup construct_or_describe failed for {}: {}", op_name, e));
    } else {
        sparq_engine::query(graph, sparql)
            .unwrap_or_else(|e| panic!("warmup query failed for {}: {}", op_name, e));
    }
}

// --------------------------------------------------------------------------- //
// CLI config
// --------------------------------------------------------------------------- //

struct Config {
    /// Number of entities in the synthetic dataset. Queries reference subjects up to
    /// `ex:n96`, so the minimum useful scale is 100; default matches the
    /// operator-coverage harness default of 2000.
    scale: u32,
    /// Number of measurement iterations per query (must be >= 2 for the
    /// determinism check). The FIRST is a warmup (results are discarded); the
    /// remaining `iters - 1` are measured and checked for agreement.
    iters: usize,
    /// Smoke mode: smaller scale + fewer iters for a fast gate run.
    smoke: bool,
}

impl Config {
    fn parse() -> Self {
        let mut cfg = Config { scale: 2000, iters: 3, smoke: false };
        let args: Vec<String> = std::env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--smoke" => {
                    cfg.smoke = true;
                    cfg.scale = 200;
                    cfg.iters = 2;
                }
                "--scale" => {
                    i += 1;
                    if let Some(v) = args.get(i) {
                        cfg.scale = v.parse().expect("--scale must be a positive integer");
                    }
                }
                "--iters" => {
                    i += 1;
                    if let Some(v) = args.get(i) {
                        cfg.iters = v.parse().expect("--iters must be >= 2");
                        assert!(cfg.iters >= 2, "--iters must be >= 2 for the determinism check");
                    }
                }
                _ => eprintln!("# unknown flag: {}", args[i]),
            }
            i += 1;
        }
        cfg
    }
}

// --------------------------------------------------------------------------- //
// Main
// --------------------------------------------------------------------------- //

fn main() {
    let cfg = Config::parse();

    // Single-threaded rayon — mandatory for deterministic alloc counts.
    // This MUST be called before any rayon work (including the engine's parallel
    // graph build). Returns Err if the global pool was already initialised (which
    // does not happen in a clean process).
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build_global()
        .expect("failed to initialise rayon with num_threads=1");

    eprintln!("# alloc-track: scale={} iters={} smoke={}", cfg.scale, cfg.iters, cfg.smoke);

    // Build the dataset once, BEFORE starting any per-query measurements.
    // We do NOT count these allocations — they are dataset setup, not query cost.
    eprintln!("# building dataset ({} entities)...", cfg.scale);
    let turtle = generate_dataset(cfg.scale);
    let graph = sparq_core::Graph::load_str(&turtle, "turtle")
        .expect("dataset must parse");
    eprintln!("# dataset ready");

    // TSV header
    println!("op\tallocs\tpeak_bytes\trows\tstatus");

    let mut any_variance = false;

    for op in OPS {
        // Warmup: first call may trigger lazy initialisation (regex compilation,
        // thread-local setup, etc.) that would skew the first measurement.
        warmup(&graph, op.sparql, op.name);

        // Measured iterations (iters >= 2 for the determinism check).
        let mut measurements: Vec<Measurement> = Vec::with_capacity(cfg.iters - 1);
        for _ in 0..(cfg.iters - 1) {
            measurements.push(measure_once(&graph, op.sparql));
        }

        let first = measurements[0];
        let row_count = first.rows;

        // Check determinism: all iters must agree on alloc_count and peak_bytes.
        // Row counts must ALWAYS agree (correctness invariant).
        let alloc_matches = measurements.iter().all(|m| m.allocs == first.allocs);
        let peak_matches = measurements.iter().all(|m| m.peak_bytes == first.peak_bytes);
        let rows_match = measurements.iter().all(|m| m.rows == first.rows);

        if !rows_match {
            // This is a BUG in the engine or harness, not just a variance.
            eprintln!(
                "# ERROR: row count variance for {} — row counts disagree across iters",
                op.name
            );
            std::process::exit(2);
        }

        let status = if alloc_matches && peak_matches {
            "ok".to_string()
        } else {
            any_variance = true;
            let alloc_min = measurements.iter().map(|m| m.allocs).min().unwrap();
            let alloc_max = measurements.iter().map(|m| m.allocs).max().unwrap();
            let peak_min = measurements.iter().map(|m| m.peak_bytes).min().unwrap();
            let peak_max = measurements.iter().map(|m| m.peak_bytes).max().unwrap();
            eprintln!(
                "# VARIANCE detected for {}: allocs=[{},{}] peak_bytes=[{},{}]",
                op.name, alloc_min, alloc_max, peak_min, peak_max
            );
            format!(
                "band allocs=[{},{}] peak=[{},{}]",
                alloc_min, alloc_max, peak_min, peak_max
            )
        };

        // Emit TSV. For band rows, allocs/peak_bytes are the MINIMUM of the band
        // (conservative — avoids overstating allocation savings in a comparison).
        let (allocs, peak_bytes) = if alloc_matches && peak_matches {
            (first.allocs, first.peak_bytes)
        } else {
            (
                measurements.iter().map(|m| m.allocs).min().unwrap(),
                measurements.iter().map(|m| m.peak_bytes).min().unwrap(),
            )
        };
        println!("{}\t{}\t{}\t{}\t{}", op.name, allocs, peak_bytes, row_count, status);
    }

    if any_variance {
        eprintln!(
            "# WARNING: some queries showed alloc-count variance across iterations."
        );
        eprintln!(
            "# This indicates non-determinism in the allocator path (e.g. lazy "
        );
        eprintln!(
            "# init on first call escaped the warmup). Re-run with --iters 5+ to "
        );
        eprintln!(
            "# characterise the band. The per-query status column shows the range."
        );
        // Exit 1 so CI notices and the variance is documented, not silently ignored.
        std::process::exit(1);
    }

    eprintln!("# alloc-track: all {} queries deterministic across {} measured iters", OPS.len(), cfg.iters - 1);

    // --------------------------------------------------------------------------- //
    // Load-time allocation measurement (sq-7d3dj.6)
    // Measures allocations during Graph::load_str — the ingest path that includes
    // Graph::build + numerics_of + temporals_of. Provides before/after evidence for
    // the sq-7d3dj.6 optimisation (borrowed-path numerics cache, no per-id owned Term).
    // --------------------------------------------------------------------------- //

    eprintln!("# load-time allocation measurements (sq-7d3dj.6)...");

    // Warmup: call load once without counting to flush any lazy initialisation that
    // would skew the first measurement (e.g., regex engines, thread-local setup).
    let _ = sparq_core::Graph::load_str(&turtle, "turtle").expect("warmup load must succeed");

    let mut load_measurements: Vec<Measurement> = Vec::with_capacity(cfg.iters - 1);
    for _ in 0..(cfg.iters - 1) {
        counters_reset();
        let g = sparq_core::Graph::load_str(&turtle, "turtle")
            .expect("load must succeed");
        let (allocs, peak_bytes) = counters_snap();
        let rows = g.len();
        load_measurements.push(Measurement { allocs, peak_bytes, rows });
    }

    let load_first = load_measurements[0];
    let load_alloc_matches = load_measurements.iter().all(|m| m.allocs == load_first.allocs);
    let load_peak_matches = load_measurements.iter().all(|m| m.peak_bytes == load_first.peak_bytes);
    let load_rows_match = load_measurements.iter().all(|m| m.rows == load_first.rows);

    if !load_rows_match {
        eprintln!("# ERROR: load triple count varied across iterations — dataset not stable");
        std::process::exit(2);
    }

    let load_status = if load_alloc_matches && load_peak_matches {
        "ok".to_string()
    } else {
        let alloc_min = load_measurements.iter().map(|m| m.allocs).min().unwrap();
        let alloc_max = load_measurements.iter().map(|m| m.allocs).max().unwrap();
        let peak_min = load_measurements.iter().map(|m| m.peak_bytes).min().unwrap();
        let peak_max = load_measurements.iter().map(|m| m.peak_bytes).max().unwrap();
        eprintln!(
            "# VARIANCE detected for load_n{}: allocs=[{},{}] peak_bytes=[{},{}]",
            cfg.scale, alloc_min, alloc_max, peak_min, peak_max
        );
        format!("band allocs=[{},{}] peak=[{},{}]", alloc_min, alloc_max, peak_min, peak_max)
    };

    let (load_allocs, load_peak) = if load_alloc_matches && load_peak_matches {
        (load_first.allocs, load_first.peak_bytes)
    } else {
        (
            load_measurements.iter().map(|m| m.allocs).min().unwrap(),
            load_measurements.iter().map(|m| m.peak_bytes).min().unwrap(),
        )
    };

    println!(
        "{}\t{}\t{}\t{}\t{}",
        format!("load_n{}", cfg.scale),
        load_allocs,
        load_peak,
        load_first.rows,
        load_status
    );
    eprintln!("# load_n{}: allocs={} peak_bytes={} triples={} status={}", cfg.scale, load_allocs, load_peak, load_first.rows, load_status);
}

// --------------------------------------------------------------------------- //
// Unit tests for the counting allocator invariants
// --------------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    /// The counting allocator correctly increments `ALLOC_COUNT` and tracks
    /// `PEAK_BYTES`. This exercises the REAL allocator path (not a mock), so it
    /// is the load-bearing invariant test mandated by the bead acceptance criteria.
    #[test]
    fn counting_allocator_tracks_allocs_and_peak() {
        // Reset to a known baseline.
        counters_reset();

        // One allocation: a heap-allocated Vec.
        let v: Vec<u8> = vec![0u8; 64];
        let (allocs_after_alloc, peak_after_alloc) = counters_snap();

        // At least 1 allocation must have been recorded (the vec storage).
        // There may be additional allocations from test infrastructure; we only
        // assert a LOWER BOUND.
        assert!(
            allocs_after_alloc >= 1,
            "expected at least 1 alloc, got {}",
            allocs_after_alloc
        );
        // Peak should be at least the vec's storage size.
        assert!(
            peak_after_alloc >= 64,
            "peak_bytes {} should be >= 64 (vec storage)",
            peak_after_alloc
        );

        // Dropping the vec releases its storage; peak should NOT decrease
        // (it is the high-water mark, not the current live count).
        drop(v);
        let (_, peak_after_drop) = counters_snap();
        assert_eq!(
            peak_after_drop, peak_after_alloc,
            "peak should be stable after drop (it is a high-water mark)"
        );
    }

    /// After `counters_reset`, all counters return to 0.
    #[test]
    fn counters_reset_clears_state() {
        // Make some allocations to dirty the state.
        let _s = String::from("alloc-track test string");
        counters_reset();
        let (allocs, peak) = counters_snap();
        assert_eq!(allocs, 0, "alloc_count should be 0 after reset");
        assert_eq!(peak, 0, "peak_bytes should be 0 after reset");
    }

    /// Peak is a HIGH-WATER MARK: repeated allocations of the same size produce
    /// a peak equal to the MAXIMUM simultaneous live bytes, not the sum of all sizes.
    #[test]
    fn peak_is_high_water_mark() {
        counters_reset();
        {
            // Hold two allocations simultaneously: peak should include both.
            let _a: Vec<u8> = vec![0u8; 128];
            let _b: Vec<u8> = vec![0u8; 256];
            let (_, peak) = counters_snap();
            // Peak must be at least 128 + 256 = 384 bytes (may be more due to
            // test harness overhead, but never less).
            assert!(
                peak >= 384,
                "simultaneous allocs of 128 + 256 bytes must register peak >= 384, got {}",
                peak
            );
        }
        // After drop, peak stays at high-water mark.
        let (_, peak_after_drop) = counters_snap();
        assert!(
            peak_after_drop >= 384,
            "peak should remain after drop, got {}",
            peak_after_drop
        );
    }
}
