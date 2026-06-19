//! RESEARCH SPIKE — point-BGP lookup ceiling (research/concurrent-serving.md §a/§d).
//!
//! The NON-cached fast path: a fully-bound-subject single-pattern SELECT executed on the
//! engine against an immutable `Arc<Graph>` snapshot. This is what a cache MISS on a
//! point query costs end-to-end inside the process (parse + plan + index probe + JSON
//! serialisation), i.e. the per-core ceiling of the "executor fast tier" when the result
//! cache cannot answer.
//!
//! Measures, single-thread and N-thread (distinct subjects, Zipf-free — the worst case
//! for any cache, the best case for the index):
//!   * `query_json` end-to-end (what the server's SELECT path pays after snapshot);
//!   * parse-only (spargebra) for the same strings — isolating parse share;
//!   * `ask` on a bound triple (the cheapest possible engine round-trip).
//!
//! Run: `cargo run --release --bin point_spike [threads]` in bench/serve.
//!   cargo run --release --bin point_spike -- [threads] --json /tmp/point.json  # strictly-additive
//!
//! [OPUS-4.8] (sq-5vm.1) `--json <path>` writes the SAME parse / point-SELECT / ASK / N-thread
//! figures STDOUT prints as a stable, DEPENDENCY-FREE JSON document, mirroring writer_spike's
//! emit (sq-k5qq). No serde dep is added to the harness (serde_json is a TEST-only dev-dep that
//! parses the emit back). STDOUT is byte-for-byte unchanged whether or not the flag is present;
//! every number is best-effort, MEASURED on the running host — ADVISORY + NON-CANONICAL (stated
//! in the emitted `note`) — nothing is committed.

use std::sync::Arc;
use std::time::Instant;

use sparq_core::Graph;
use sparq_engine::QueryBudget;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// ---------------------------------------------------------------------------
// [OPUS-4.8] (sq-5vm.1) --json <path> machine-readable results emit
// ---------------------------------------------------------------------------

/// All the figures this spike prints, captured once so STDOUT and the optional JSON
/// emit share a single source of truth.
#[derive(Default)]
struct Results {
    triples: usize,
    threads: usize,
    parse_us_per_op: f64,
    point_select_1t_us_per_op: f64,
    point_select_1t_k_ops_per_s: f64,
    bound_ask_1t_us_per_op: f64,
    bound_ask_1t_k_ops_per_s: f64,
    point_select_nt_m_ops_per_s: f64,
    point_select_nt_us_per_op_per_thread: f64,
}

/// Extracts `--json <path>` from argv, returning argv WITHOUT the flag pair. A bare
/// `--json` is a usage error (exit 2), mirroring `sparq-cli` / `writer_spike`'s flag.
fn take_json_flag(args: Vec<String>) -> (Vec<String>, Option<String>) {
    let mut out = Vec::with_capacity(args.len());
    let mut json_path = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--json" {
            match args.get(i + 1) {
                Some(p) => {
                    json_path = Some(p.clone());
                    i += 2;
                    continue;
                }
                None => {
                    eprintln!("`--json` requires a path argument: --json <path>");
                    std::process::exit(2);
                }
            }
        }
        out.push(args[i].clone());
        i += 1;
    }
    (out, json_path)
}

/// Serialise the run to stable, dependency-free JSON. Every metric is ADVISORY +
/// NON-CANONICAL (stated in `note`).
fn results_json(r: &Results) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str("  \"harness\": \"serve-spikes point_spike\",\n");
    s.push_str(
        "  \"note\": \"point-BGP lookup ceiling (cache MISS fast path); every µs/op, K-ops/s and \
         M-ops/s figure is best-effort, MEASURED on the running host — ADVISORY, NON-CANONICAL \
         (this dev box) — do not bake into committed files\",\n",
    );
    s.push_str(&format!("  \"triples\": {},\n", r.triples));
    s.push_str(&format!("  \"threads\": {},\n", r.threads));
    s.push_str(&format!("  \"parse_us_per_op\": {:.3},\n", r.parse_us_per_op));
    s.push_str(&format!(
        "  \"point_select_1t_us_per_op\": {:.3},\n",
        r.point_select_1t_us_per_op
    ));
    s.push_str(&format!(
        "  \"point_select_1t_k_ops_per_s\": {:.1},\n",
        r.point_select_1t_k_ops_per_s
    ));
    s.push_str(&format!("  \"bound_ask_1t_us_per_op\": {:.3},\n", r.bound_ask_1t_us_per_op));
    s.push_str(&format!(
        "  \"bound_ask_1t_k_ops_per_s\": {:.1},\n",
        r.bound_ask_1t_k_ops_per_s
    ));
    s.push_str(&format!(
        "  \"point_select_nt_m_ops_per_s\": {:.3},\n",
        r.point_select_nt_m_ops_per_s
    ));
    s.push_str(&format!(
        "  \"point_select_nt_us_per_op_per_thread\": {:.3}\n",
        r.point_select_nt_us_per_op_per_thread
    ));
    s.push_str("}\n");
    s
}

fn main() {
    // Strictly-additive: pull `--json <path>` out before reading the optional thread count,
    // so the positional `[threads]` argument is unaffected when the flag is absent.
    let (args, json_path) = take_json_flag(std::env::args().collect());
    let threads: usize = args.into_iter().nth(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    let mut res = Results { threads, ..Results::default() };
    let mut nt = String::new();
    for i in 0..1_000_000 {
        nt.push_str(&format!(
            "<http://ex/s{}> <http://xmlns.com/foaf/0.1/name> \"person number {}\" .\n",
            i, i
        ));
    }
    let graph = Arc::new(Graph::load_str(&nt, "ntriples").expect("load"));
    res.triples = graph.len();
    println!("graph: {} triples", graph.len());

    let q_of = |i: usize| {
        format!("SELECT ?n WHERE {{ <http://ex/s{i}> <http://xmlns.com/foaf/0.1/name> ?n }}")
    };

    // parse-only baseline for these exact strings
    let iters = 50_000u64;
    let t = Instant::now();
    for i in 0..iters {
        let q = q_of(i as usize % 1_000_000);
        std::hint::black_box(spargebra::SparqlParser::new().parse_query(&q).unwrap());
    }
    res.parse_us_per_op = t.elapsed().as_nanos() as f64 / iters as f64 / 1000.0;
    println!(
        "parse only (incl. format!): {:.2} us/op",
        res.parse_us_per_op
    );

    // single-thread end-to-end point SELECT → JSON
    let b = QueryBudget::unlimited();
    let iters = 50_000u64;
    let t = Instant::now();
    let mut sink = 0usize;
    for i in 0..iters {
        let q = q_of((i as usize).wrapping_mul(2654435761) % 1_000_000);
        sink = sink.wrapping_add(sparq_engine::query_json_with_budget(&graph, &q, &b).unwrap().len());
    }
    let el = t.elapsed();
    res.point_select_1t_us_per_op = el.as_nanos() as f64 / iters as f64 / 1000.0;
    res.point_select_1t_k_ops_per_s = iters as f64 / el.as_secs_f64() / 1e3;
    println!(
        "point SELECT->JSON, 1 thread: {:.2} us/op = {:.0} K ops/s [{sink}]",
        res.point_select_1t_us_per_op,
        res.point_select_1t_k_ops_per_s
    );

    // ASK on a fully bound triple — cheapest engine round-trip
    let t = Instant::now();
    for i in 0..iters {
        let s = (i as usize).wrapping_mul(2654435761) % 1_000_000;
        let q = format!(
            "ASK {{ <http://ex/s{s}> <http://xmlns.com/foaf/0.1/name> \"person number {s}\" }}"
        );
        std::hint::black_box(sparq_engine::ask_with_budget(&graph, &q, &b).unwrap());
    }
    let el = t.elapsed();
    res.bound_ask_1t_us_per_op = el.as_nanos() as f64 / iters as f64 / 1000.0;
    res.bound_ask_1t_k_ops_per_s = iters as f64 / el.as_secs_f64() / 1e3;
    println!(
        "bound ASK, 1 thread: {:.2} us/op = {:.0} K ops/s",
        res.bound_ask_1t_us_per_op,
        res.bound_ask_1t_k_ops_per_s
    );

    // N-thread point SELECT → JSON on shared snapshot
    let per_thread = 30_000u64;
    let t = Instant::now();
    let handles: Vec<_> = (0..threads)
        .map(|ti| {
            let g = graph.clone();
            std::thread::spawn(move || {
                let b = QueryBudget::unlimited();
                let mut sink = 0usize;
                for i in 0..per_thread {
                    let s = ((i as usize) ^ (ti * 7919)).wrapping_mul(2654435761) % 1_000_000;
                    let q = format!(
                        "SELECT ?n WHERE {{ <http://ex/s{s}> <http://xmlns.com/foaf/0.1/name> ?n }}"
                    );
                    sink = sink
                        .wrapping_add(sparq_engine::query_json_with_budget(&g, &q, &b).unwrap().len());
                }
                sink
            })
        })
        .collect();
    let mut sink = 0usize;
    for h in handles {
        sink = sink.wrapping_add(h.join().unwrap());
    }
    let el = t.elapsed();
    let total = per_thread * threads as u64;
    res.point_select_nt_m_ops_per_s = total as f64 / el.as_secs_f64() / 1e6;
    res.point_select_nt_us_per_op_per_thread =
        el.as_nanos() as f64 / (total as f64 / threads as f64) / 1000.0;
    println!(
        "point SELECT->JSON, {threads} threads: {:.2} M ops/s total ({:.2} us/op/thread) [{sink}]",
        res.point_select_nt_m_ops_per_s,
        res.point_select_nt_us_per_op_per_thread
    );

    // [OPUS-4.8] (sq-5vm.1) Strictly-additive JSON emit: only when `--json <path>` was given.
    // STDOUT above (the free-text metric lines) is the unchanged human/research output.
    if let Some(path) = json_path {
        let doc = results_json(&res);
        if let Err(e) = std::fs::write(&path, doc) {
            eprintln!("error writing --json results to {path}: {e}");
            std::process::exit(1);
        }
        eprintln!("wrote point_spike results to {path}");
    }
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] (sq-5vm.1) --json emit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_flag_extraction() {
        let argv: Vec<String> = ["point_spike", "16", "--json", "/tmp/o.json"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let (positional, path) = take_json_flag(argv);
        assert_eq!(positional, vec!["point_spike", "16"]);
        assert_eq!(path.as_deref(), Some("/tmp/o.json"));
        let plain: Vec<String> = ["point_spike"].iter().map(|s| s.to_string()).collect();
        let (p2, none) = take_json_flag(plain.clone());
        assert_eq!(p2, plain);
        assert!(none.is_none());
    }

    #[test]
    fn results_json_round_trips() {
        let r = Results {
            triples: 1_000_000,
            threads: 8,
            parse_us_per_op: 1.1,
            point_select_1t_us_per_op: 4.2,
            point_select_1t_k_ops_per_s: 238.0,
            bound_ask_1t_us_per_op: 2.0,
            bound_ask_1t_k_ops_per_s: 500.0,
            point_select_nt_m_ops_per_s: 1.5,
            point_select_nt_us_per_op_per_thread: 5.3,
        };
        let doc = results_json(&r);
        // The dependency-free emit must round-trip through a REAL serde_json parse.
        let v: serde_json::Value = serde_json::from_str(&doc).expect("emit must be valid JSON");
        assert_eq!(v["harness"], "serve-spikes point_spike");
        assert!(v["note"].as_str().unwrap().contains("NON-CANONICAL"));
        assert_eq!(v["triples"], 1_000_000);
        assert_eq!(v["threads"], 8);
        assert!(v["parse_us_per_op"].is_number());
        assert!(v["point_select_1t_us_per_op"].is_number());
        assert!(v["bound_ask_1t_k_ops_per_s"].is_number());
        assert!(v["point_select_nt_m_ops_per_s"].is_number());
        assert!(v["point_select_nt_us_per_op_per_thread"].is_number());

        // A defaulted (all-zero) Results is still valid JSON.
        let empty = results_json(&Results::default());
        let v2: serde_json::Value = serde_json::from_str(&empty).expect("default is valid JSON");
        assert_eq!(v2["triples"], 0);
    }
}
