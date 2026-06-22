//! RESEARCH SPIKE — sustained small-update stream (research/concurrent-serving.md §d.iv):
//! the Solid write workload, one SPARQL UPDATE per resource write, run for tens of
//! thousands of updates against the production `AppState` writer.
//!
//! This is the direct test of the QLever failure mode prod-solid-server designed around
//! (qlever#2481: memory climbing under sustained live updates → OOM): does sparq's
//! generation-ring writer (Wave A4: structural-fork per generation, delta-overlay
//! `apply`, bounded-K retention, the applier's own periodic O(graph) compaction fold)
//! keep BOTH update latency and RSS flat over a long update stream?
//!
//! Each update models prod-solid-server's `putDocument` shape scaled to the default
//! graph (the server's published surface today): DELETE the resource's previous ~9
//! metadata triples + INSERT fresh ones (~350 bytes of SPARQL). Resources are revisited
//! round-robin over a pool, so deletes really hit (an overlay that only grows by
//! inserts would flatter the result).
//!
//! Optional concurrent readers (`--readers N`) pin the current generation
//! ([`AppState::current`]) and run a point query in a loop — checking that reads stay
//! lock-free and never stall the writer under sustained read load.
//!
//! Output: update p50/p99/max per 1k-window, RSS per window, total throughput.
//!
//! Run: `cargo run --release --bin update_stream_spike [--updates N --resources N --readers N]`
//!      in bench/serve.
//!   cargo run --release --bin update_stream_spike -- --json /tmp/upd.json  # strictly-additive
//!
//! [OPUS-4.8] (sq-5vm.1) `--json <path>` writes the SAME per-window + TOTAL figures STDOUT
//! prints as a stable, DEPENDENCY-FREE JSON document, mirroring writer_spike's emit (sq-k5qq).
//! No serde dep is added to the harness (serde_json is a TEST-only dev-dep that parses the emit
//! back). STDOUT is byte-for-byte unchanged whether or not the flag is present; every number is
//! best-effort, MEASURED on the running host — ADVISORY + NON-CANONICAL (stated in the emitted
//! `note`) — nothing is committed.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use sparq_core::Graph;
use sparq_engine::QueryBudget;
use sparq_server::{AppState, ServerConfig};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn rss_mb() -> f64 {
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .expect("ps");
    String::from_utf8_lossy(&out.stdout).trim().parse::<f64>().unwrap_or(0.0) / 1024.0
}

fn put_document_update(res: usize, version: usize) -> String {
    // prod-solid-server putDocument analog: replace the resource's metadata triples.
    // DELETE WHERE clears the previous version's triples; INSERT DATA writes ~9 fresh ones.
    format!(
        "DELETE WHERE {{ <http://pod.ex/r{res}> ?p ?o }} ; \
         INSERT DATA {{ \
           <http://pod.ex/r{res}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/ldp#Resource> . \
           <http://pod.ex/r{res}> <https://pss.dev/ns#contentType> \"text/turtle\" . \
           <http://pod.ex/r{res}> <http://www.w3.org/ns/posix/stat#size> {version} . \
           <http://pod.ex/r{res}> <http://www.w3.org/ns/posix/stat#mtime> {version} . \
           <http://pod.ex/r{res}> <http://purl.org/dc/terms/modified> \"2026-06-12T00:00:{:02}\" . \
           <http://pod.ex/r{res}> <https://pss.dev/ns#etag> \"etag-{res}-{version}\" . \
           <http://pod.ex/r{res}> <https://pss.dev/ns#s3Key> \"k/{res}/{version}\" . \
           <http://pod.ex/parent> <http://www.w3.org/ns/ldp#contains> <http://pod.ex/r{res}> . \
         }}",
        version % 60
    )
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] (sq-5vm.1) --json <path> machine-readable results emit
// ---------------------------------------------------------------------------

/// One 1k-window of the sustained update stream: the index range plus throughput,
/// latency percentiles and RSS at that point — the flat-latency / flat-RSS evidence.
struct WindowRow {
    from: usize,
    to: usize,
    throughput_per_s: f64,
    p50_us: u64,
    p99_us: u64,
    max_us: u64,
    rss_mb: f64,
}

/// Run-level summary (the workload params + the TOTAL line + final state).
#[derive(Default)]
struct Summary {
    updates: usize,
    resources: usize,
    readers: usize,
    base_triples: usize,
    base_rss_mb: f64,
    total_secs: f64,
    sustained_updates_per_s: f64,
    reader_queries: u64,
    final_rss_mb: f64,
    final_graph_triples: usize,
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
fn results_json(sum: &Summary, windows: &[WindowRow]) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str("  \"harness\": \"serve-spikes update_stream_spike\",\n");
    s.push_str(
        "  \"note\": \"sustained small-update stream (QLever-OOM failure mode test); every \
         updates/s, µs-latency and RSS figure is best-effort, MEASURED on the running host — \
         ADVISORY, NON-CANONICAL (this dev box) — do not bake into committed files\",\n",
    );
    s.push_str(&format!("  \"updates\": {},\n", sum.updates));
    s.push_str(&format!("  \"resources\": {},\n", sum.resources));
    s.push_str(&format!("  \"readers\": {},\n", sum.readers));
    s.push_str(&format!("  \"base_triples\": {},\n", sum.base_triples));
    s.push_str(&format!("  \"base_rss_mb\": {:.0},\n", sum.base_rss_mb));
    s.push_str(&format!("  \"total_secs\": {:.3},\n", sum.total_secs));
    s.push_str(&format!(
        "  \"sustained_updates_per_s\": {:.1},\n",
        sum.sustained_updates_per_s
    ));
    s.push_str(&format!("  \"reader_queries\": {},\n", sum.reader_queries));
    s.push_str(&format!("  \"final_rss_mb\": {:.0},\n", sum.final_rss_mb));
    s.push_str(&format!("  \"final_graph_triples\": {},\n", sum.final_graph_triples));
    s.push_str("  \"windows\": [\n");
    for (i, w) in windows.iter().enumerate() {
        let comma = if i + 1 < windows.len() { "," } else { "" };
        s.push_str(&format!(
            "    {{ \"from\": {}, \"to\": {}, \"throughput_per_s\": {:.1}, \"p50_us\": {}, \
             \"p99_us\": {}, \"max_us\": {}, \"rss_mb\": {:.0} }}{comma}\n",
            w.from, w.to, w.throughput_per_s, w.p50_us, w.p99_us, w.max_us, w.rss_mb,
        ));
    }
    s.push_str("  ]\n");
    s.push_str("}\n");
    s
}

fn main() {
    let mut updates = 20_000usize;
    let mut resources = 2_000usize;
    let mut readers = 0usize;
    // Strictly-additive: pull `--json <path>` out before the spike's own flag parse, so the
    // `--updates/--resources/--readers` loop below is unaffected when the flag is absent.
    let (argv, json_path) = take_json_flag(std::env::args().collect());
    let mut args = argv.into_iter().skip(1);
    while let Some(a) = args.next() {
        let mut next = || args.next().expect("flag value").parse::<usize>().expect("number");
        match a.as_str() {
            "--updates" => updates = next(),
            "--resources" => resources = next(),
            "--readers" => readers = next(),
            other => panic!("unknown arg {other}"),
        }
    }
    let mut sum = Summary {
        updates,
        resources,
        readers,
        ..Summary::default()
    };
    let mut windows: Vec<WindowRow> = Vec::new();

    // Base graph: 1M triples of unrelated data + initial resource metadata.
    let mut nt = String::new();
    for i in 0..1_000_000 {
        nt.push_str(&format!("<http://ex/s{}> <http://ex/p{}> \"v{}\" .\n", i % 200_000, i % 50, i));
    }
    let graph = Graph::load_str(&nt, "ntriples").expect("load");
    sum.base_triples = graph.len();
    sum.base_rss_mb = rss_mb();
    println!(
        "base graph: {} triples, RSS {:.0} MB; {updates} updates over {resources} resources, readers={readers}",
        graph.len(),
        sum.base_rss_mb
    );

    // Compaction is no longer a ServerConfig knob: as of Wave A4 the writer's
    // GraphApplier folds each generation's pending overlay flat once it reaches its own
    // compact threshold (DEFAULT_COMPACT_THRESHOLD), entirely inside sparq-serve. This
    // spike drives the PRODUCTION AppState, so it inherits that default — what the HTTP
    // server actually does — and no longer overrides the fold cadence.
    let config = ServerConfig {
        query_timeout: Some(Duration::from_secs(5)),
        ..ServerConfig::default()
    };
    let state = AppState::with_config(graph, config);
    let stop = Arc::new(AtomicBool::new(false));

    let reader_handles: Vec<_> = (0..readers)
        .map(|i| {
            let st = state.clone();
            let stop = stop.clone();
            std::thread::spawn(move || {
                let b = QueryBudget::unlimited();
                let mut n = 0u64;
                while !stop.load(Ordering::Relaxed) {
                    // Pin the current generation (lock-free ~10–20ns); never blocked by the
                    // writer. Hold `pin` for the whole query so its snapshot stays readable.
                    let pin = st.current();
                    let g = pin.snapshot();
                    let s = (n as usize).wrapping_mul(2654435761).wrapping_add(i) % 200_000;
                    let q = format!("SELECT ?o WHERE {{ <http://ex/s{s}> ?p ?o }} LIMIT 5");
                    std::hint::black_box(sparq_engine::query_json_with_budget(g, &q, &b).ok());
                    n += 1;
                }
                n
            })
        })
        .collect();

    let window = 1_000usize;
    let mut lat = Vec::with_capacity(window);
    let run_start = Instant::now();
    let mut window_start = Instant::now();
    for i in 0..updates {
        let res = i % resources;
        let upd = put_document_update(res, i / resources + 1);
        let t = Instant::now();
        state.apply_update(&upd).expect("update");
        lat.push(t.elapsed().as_micros() as u64);
        if lat.len() == window {
            lat.sort_unstable();
            let from = i + 1 - window;
            let to = i + 1;
            let throughput = window as f64 / window_start.elapsed().as_secs_f64();
            let p50 = lat[window / 2];
            let p99 = lat[window * 99 / 100];
            let max = lat[window - 1];
            let rss = rss_mb();
            println!(
                "updates {:>6}-{:>6}: thr={:>6.0}/s p50={:>5}us p99={:>6}us max={:>8}us  RSS={:>5.0} MB",
                from, to, throughput, p50, p99, max, rss
            );
            windows.push(WindowRow {
                from,
                to,
                throughput_per_s: throughput,
                p50_us: p50,
                p99_us: p99,
                max_us: max,
                rss_mb: rss,
            });
            lat.clear();
            window_start = Instant::now();
        }
    }
    stop.store(true, Ordering::Relaxed);
    let reads: u64 = reader_handles.into_iter().map(|h| h.join().unwrap()).sum();
    sum.total_secs = run_start.elapsed().as_secs_f64();
    sum.sustained_updates_per_s = updates as f64 / sum.total_secs;
    sum.reader_queries = reads;
    sum.final_rss_mb = rss_mb();
    println!(
        "TOTAL: {updates} updates in {:.1}s = {:.0} updates/s sustained; reader queries completed: {reads}; final RSS {:.0} MB",
        sum.total_secs,
        sum.sustained_updates_per_s,
        sum.final_rss_mb
    );
    // Final published graph sanity: each resource should have exactly 8 triples + parent containment.
    let pin = state.current();
    sum.final_graph_triples = pin.snapshot().len();
    println!("final graph: {} triples", sum.final_graph_triples);

    // [OPUS-4.8] (sq-5vm.1) Strictly-additive JSON emit: only when `--json <path>` was given.
    // STDOUT above (the per-window + TOTAL lines) is the unchanged human/research output.
    if let Some(path) = json_path {
        let doc = results_json(&sum, &windows);
        if let Err(e) = std::fs::write(&path, doc) {
            eprintln!("error writing --json results to {path}: {e}");
            std::process::exit(1);
        }
        eprintln!("wrote {} window rows + summary to {path}", windows.len());
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
        let argv: Vec<String> = ["update_stream_spike", "--updates", "5000", "--json", "/tmp/o.json"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let (positional, path) = take_json_flag(argv);
        assert_eq!(positional, vec!["update_stream_spike", "--updates", "5000"]);
        assert_eq!(path.as_deref(), Some("/tmp/o.json"));
        let plain: Vec<String> = ["update_stream_spike", "--updates", "5000"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let (p2, none) = take_json_flag(plain.clone());
        assert_eq!(p2, plain);
        assert!(none.is_none());
    }

    #[test]
    fn results_json_round_trips() {
        let sum = Summary {
            updates: 20_000,
            resources: 2_000,
            readers: 4,
            base_triples: 1_000_000,
            base_rss_mb: 250.0,
            total_secs: 12.5,
            sustained_updates_per_s: 1600.0,
            reader_queries: 5_000_000,
            final_rss_mb: 300.0,
            final_graph_triples: 1_016_001,
        };
        let windows = vec![
            WindowRow {
                from: 0,
                to: 1_000,
                throughput_per_s: 1700.0,
                p50_us: 400,
                p99_us: 900,
                max_us: 5_000,
                rss_mb: 260.0,
            },
            WindowRow {
                from: 1_000,
                to: 2_000,
                throughput_per_s: 1650.0,
                p50_us: 410,
                p99_us: 950,
                max_us: 6_000,
                rss_mb: 265.0,
            },
        ];
        let doc = results_json(&sum, &windows);
        // The dependency-free emit must round-trip through a REAL serde_json parse.
        let v: serde_json::Value = serde_json::from_str(&doc).expect("emit must be valid JSON");
        assert_eq!(v["harness"], "serve-spikes update_stream_spike");
        assert!(v["note"].as_str().unwrap().contains("NON-CANONICAL"));
        assert_eq!(v["updates"], 20_000);
        assert_eq!(v["readers"], 4);
        assert!(v["sustained_updates_per_s"].is_number());
        assert_eq!(v["final_graph_triples"], 1_016_001);
        let w = v["windows"].as_array().expect("windows is an array");
        assert_eq!(w.len(), 2);
        assert_eq!(w[0]["from"], 0);
        assert_eq!(w[0]["to"], 1_000);
        assert_eq!(w[1]["p50_us"], 410);
        assert!(w[0]["throughput_per_s"].is_number());

        // Empty window set must still be valid JSON (no trailing comma).
        let empty = results_json(&Summary::default(), &[]);
        let v2: serde_json::Value = serde_json::from_str(&empty).expect("empty is valid JSON");
        assert!(v2["windows"].as_array().unwrap().is_empty());
    }
}
