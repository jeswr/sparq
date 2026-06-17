// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! RESEARCH SPIKE — Wave A2 sequenced writer + group-commit
//! (research/concurrent-serving.md §6.5).
//!
//! Measures the A2 group-commit writer against the A1 baseline (publish-per-update
//! straight onto the generation ring, no batching):
//!
//!   1. WRITER THROUGHPUT (updates/s) at max_batch 1 / 16 / 256 — closed-loop
//!      drain: a single feeder submits N updates back-to-back via Writer::submit
//!      (sync), so the rate is bounded by commit cost, not arrival. Batch 1 IS the
//!      A2-shaped equivalent of the A1 baseline (one generation per update); 16 and
//!      256 show the group-commit win.
//!   2. READER LATENCY p50/p99 of ring.current() — OPEN-LOOP, coordinated-
//!      omission-safe: a sampler reads on a FIXED schedule (every `read_period`),
//!      and each sample's latency is measured from its INTENDED tick, not from when
//!      the thread actually got to issue it. A reader blocked behind the write path
//!      shows up as schedule slip, not as a hidden gap. Measured twice: with a
//!      concurrent writer hammering, and idle.
//!   3. A1 BASELINE: the same reader-latency measurement while a feeder calls
//!      ring.publish() once per update directly (the pre-A2 path).
//!
//! Honest reporting: if group-commit LOSES to A1 publish-per-update anywhere
//! (e.g. tiny single-update windows where the channel + window add latency), the
//! numbers say so — read them, do not assume the batched path always wins.
//!
//! Run: `cargo run --release --bin writer_spike` in bench/serve.
//! Env: WRITER_UPDATES (default 30000), WRITER_SECS reader window (default 2).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use sparq_core::Graph;
use sparq_serve::{
    ApplyUpdates, GenerationRing, GraphApplier, PodId, Writer, WriterConfig,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn base_graph(n: usize) -> Graph {
    let mut nt = String::with_capacity(n * 40);
    for i in 0..n {
        nt.push_str(&format!("<http://ex/s{i}> <http://ex/p> \"v{i}\" .\n"));
    }
    Graph::load_str(&nt, "ntriples").expect("base graph")
}

/// A small Solid-write-shaped INSERT DATA (one fresh resource triple).
fn update(i: usize) -> String {
    format!("INSERT DATA {{ <http://pod.ex/r{i}> <http://ex/p> \"w{i}\" }}")
}

fn pct(sorted_ns: &[u64], p: f64) -> u64 {
    if sorted_ns.is_empty() {
        return 0;
    }
    let idx = ((sorted_ns.len() as f64 * p) as usize).min(sorted_ns.len() - 1);
    sorted_ns[idx]
}

/// Closed-loop writer throughput: one feeder submits `updates` sync, measuring
/// wall time. Returns (updates/s, final generation count).
fn writer_throughput(updates: usize, max_batch: usize) -> (f64, u64) {
    let ring = Arc::new(GenerationRing::new(base_graph(1_000)));
    let writer = Writer::spawn(
        ring.clone(),
        GraphApplier::new(),
        WriterConfig { window: Duration::from_millis(3), max_batch, ..WriterConfig::default() },
    );
    // Multiple concurrent submitters so the window actually fills (a single sync
    // submitter would serialise one-update-per-window and never batch). We use
    // `max_batch` feeders so the writer can collect up to a full batch per window.
    let n_feeders = max_batch.clamp(1, 64);
    let per = updates / n_feeders;
    let next = Arc::new(AtomicU64::new(0));
    let writer = Arc::new(writer);
    let start = Instant::now();
    let feeders: Vec<_> = (0..n_feeders)
        .map(|_| {
            let writer = writer.clone();
            let next = next.clone();
            std::thread::spawn(move || {
                for _ in 0..per {
                    let i = next.fetch_add(1, Ordering::Relaxed) as usize;
                    let _ = writer.submit(update(i), [PodId::from("pod:w")]);
                }
            })
        })
        .collect();
    for f in feeders {
        f.join().unwrap();
    }
    let elapsed = start.elapsed();
    let total = next.load(Ordering::Relaxed);
    let gens = ring.current().number();
    drop(writer);
    (total as f64 / elapsed.as_secs_f64(), gens)
}

/// Open-loop, coordinated-omission-safe reader latency. `sampler` reads
/// `ring.current()` on a fixed `read_period` schedule for `dur`; each sample's
/// latency is measured from its intended tick. Returns sorted latencies (ns).
fn reader_latencies(
    ring: Arc<GenerationRing<Graph>>,
    read_period: Duration,
    dur: Duration,
) -> Vec<u64> {
    let mut lat = Vec::with_capacity((dur.as_nanos() / read_period.as_nanos().max(1)) as usize + 16);
    let start = Instant::now();
    let mut tick = 0u64;
    loop {
        let intended = start + read_period * (tick as u32);
        let now = Instant::now();
        if now < intended {
            std::thread::sleep(intended - now);
        }
        // Latency from the INTENDED tick: if a prior read slipped us late, that
        // slip is charged here (coordinated-omission-safe).
        let g = ring.current();
        std::hint::black_box(g.number());
        let done = Instant::now();
        lat.push(done.saturating_duration_since(intended).as_nanos() as u64);
        tick += 1;
        if done.saturating_duration_since(start) >= dur {
            break;
        }
    }
    lat.sort_unstable();
    lat
}

/// Run a feeder that drives the A2 writer (or the A1 ring) flat-out for `dur`
/// while the calling thread samples reader latency. `mode` selects the path.
fn reader_under_writer(max_batch: usize, dur: Duration, mode: Mode) -> Vec<u64> {
    let ring = Arc::new(GenerationRing::new(base_graph(1_000)));
    let stop = Arc::new(AtomicBool::new(false));

    let feeder = match mode {
        Mode::A2 => {
            let writer = Arc::new(Writer::spawn(
                ring.clone(),
                GraphApplier::new(),
                WriterConfig { window: Duration::from_millis(3), max_batch, ..WriterConfig::default() },
            ));
            let stop = stop.clone();
            // Bounded in-flight: a few feeder threads each use SYNC submit, so the
            // number of outstanding (un-committed) updates is capped by the feeder
            // count — the writer is never buried under an unbounded backlog it must
            // drain on shutdown, and the load is paced to its commit rate
            // (open-loop with natural backpressure). One full batch can still fill
            // per window because there are at least `max_batch`-many… no — we cap
            // feeders at 64 (clamp below) so windows fill to min(max_batch, 64).
            let n_feeders = max_batch.clamp(1, 64);
            let next = Arc::new(AtomicU64::new(0));
            let handles: Vec<_> = (0..n_feeders)
                .map(|_| {
                    let writer = writer.clone();
                    let stop = stop.clone();
                    let next = next.clone();
                    std::thread::spawn(move || {
                        while !stop.load(Ordering::Relaxed) {
                            let i = next.fetch_add(1, Ordering::Relaxed) as usize;
                            if writer.submit(update(i), [PodId::from("pod:w")]).is_err() {
                                break;
                            }
                        }
                    })
                })
                .collect();
            FeederHandle::A2(writer, handles)
        }
        Mode::A1 => {
            // A1 baseline: publish-per-update straight onto the ring, no batching,
            // no group-commit window — the pre-A2 write path. We fork+apply+seal
            // inline (the same per-update work the A2 writer does, minus batching).
            let ring2 = ring.clone();
            let stop = stop.clone();
            FeederHandle::A1(std::thread::spawn(move || {
                let mut applier = GraphApplier::new();
                let mut i = 0usize;
                while !stop.load(Ordering::Relaxed) {
                    let base = ring2.current();
                    let mut w = applier.fork(base.snapshot()).unwrap();
                    if applier.apply(&mut w, &update(i)).is_ok() {
                        // seal now returns Result (the applier's compaction-fold can fail on a
                        // durable-backed store); this in-memory applier never errs, so a failure
                        // here is a real bug — surface it rather than skip the publish.
                        let s = applier.seal(w).expect("in-memory seal never fails");
                        ring2.publish(s, [PodId::from("pod:w")]);
                    }
                    i += 1;
                }
            }))
        }
    };

    let lat = reader_latencies(ring.clone(), Duration::from_micros(200), dur);
    stop.store(true, Ordering::Relaxed);
    feeder.join();
    lat
}

enum Mode {
    A1,
    A2,
}

enum FeederHandle {
    A1(std::thread::JoinHandle<()>),
    A2(Arc<Writer<String>>, Vec<std::thread::JoinHandle<()>>),
}
impl FeederHandle {
    fn join(self) {
        match self {
            FeederHandle::A1(h) => h.join().unwrap(),
            FeederHandle::A2(w, hs) => {
                for h in hs {
                    h.join().unwrap();
                }
                drop(w);
            }
        }
    }
}

fn main() {
    let updates: usize = std::env::var("WRITER_UPDATES").ok().and_then(|v| v.parse().ok()).unwrap_or(30_000);
    let secs: u64 = std::env::var("WRITER_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    let dur = Duration::from_secs(secs);

    println!("# writer_spike — Wave A2 group-commit vs A1 publish-per-update");
    println!("# updates={updates}, reader window={secs}s, read_period=200µs (open-loop)\n");

    println!("## 1. Writer throughput (closed-loop drain)");
    println!("| max_batch | updates/s | generations | updates/gen |");
    println!("|---|---:|---:|---:|");
    for &b in &[1usize, 16, 256] {
        let (ups, gens) = writer_throughput(updates, b);
        let upg = updates as f64 / gens.max(1) as f64;
        println!("| {b} | {ups:.0} | {gens} | {upg:.1} |");
    }

    println!("\n## 2. Reader latency p50/p99 (open-loop, coordinated-omission-safe)");
    println!("| scenario | p50 (µs) | p99 (µs) | max (µs) | samples |");
    println!("|---|---:|---:|---:|---:|");

    // Idle reader (no writer): the floor.
    {
        let ring = Arc::new(GenerationRing::new(base_graph(1_000)));
        let lat = reader_latencies(ring, Duration::from_micros(200), dur);
        println!(
            "| idle (no writer) | {:.2} | {:.2} | {:.2} | {} |",
            pct(&lat, 0.50) as f64 / 1000.0,
            pct(&lat, 0.99) as f64 / 1000.0,
            *lat.last().unwrap() as f64 / 1000.0,
            lat.len()
        );
    }

    for &b in &[1usize, 16, 256] {
        let lat = reader_under_writer(b, dur, Mode::A2);
        println!(
            "| A2 writer max_batch={b} | {:.2} | {:.2} | {:.2} | {} |",
            pct(&lat, 0.50) as f64 / 1000.0,
            pct(&lat, 0.99) as f64 / 1000.0,
            *lat.last().unwrap() as f64 / 1000.0,
            lat.len()
        );
    }
    {
        let lat = reader_under_writer(1, dur, Mode::A1);
        println!(
            "| A1 publish-per-update | {:.2} | {:.2} | {:.2} | {} |",
            pct(&lat, 0.50) as f64 / 1000.0,
            pct(&lat, 0.99) as f64 / 1000.0,
            *lat.last().unwrap() as f64 / 1000.0,
            lat.len()
        );
    }

    println!("\n# Interpretation: readers never block the writer in EITHER mode (lock-free");
    println!("# current()); the win of group-commit is in WRITER throughput (§1) — more");
    println!("# updates per generation = fewer fork/seal/publish cycles. If batch-1 A2 is");
    println!("# slower than A1, that is the channel + window overhead with no batching gain.");
}
