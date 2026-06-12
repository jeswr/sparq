//! RESEARCH SPIKE — cost of ACCESS TRACKING on the scan hot path.
//!
//! The cheapest workload-awareness signal is a per-permutation (or per-block) access
//! counter bumped on every `rows_in` probe. This bin measures, on a realistic probe
//! loop (binary-search of a random bound-subject prefix over a 10M-row sorted perm):
//!   a) the bare probe                       (baseline)
//!   b) probe + relaxed AtomicU64 fetch_add  (per-perm counter, uncontended)
//!   c) probe + SAMPLED counter (1/64 via a thread-local u64)
//!   d) 4 threads hammering ONE shared counter while probing (contended worst case)
//! plus the raw fetch_add cost in isolation.
//!
//! usage: atomic_cost <raw-index-dir> [probes]

use sparq_core::dict::Id;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

fn read_rows(path: &std::path::Path) -> Vec<[Id; 3]> {
    let bytes = std::fs::read(path).unwrap();
    let mut rows = Vec::with_capacity(bytes.len() / 12);
    for c in bytes.chunks_exact(12) {
        rows.push([
            u32::from_le_bytes(c[0..4].try_into().unwrap()),
            u32::from_le_bytes(c[4..8].try_into().unwrap()),
            u32::from_le_bytes(c[8..12].try_into().unwrap()),
        ]);
    }
    rows
}

#[inline]
fn probe(rows: &[[Id; 3]], s: Id) -> usize {
    let lo = [s, Id::MIN, Id::MIN];
    let hi = [s, Id::MAX, Id::MAX];
    rows.partition_point(|r| *r <= hi) - rows.partition_point(|r| *r < lo)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::PathBuf::from(args.get(1).expect("usage: atomic_cost <raw-index-dir> [probes]"));
    let probes: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(2_000_000);

    let rows = read_rows(&dir.join("perm0.bin"));
    let max_s = rows.last().unwrap()[0];
    println!("rows\t{}\tprobes\t{probes}", rows.len());

    // Deterministic pseudo-random keys (xorshift), same sequence for every variant.
    let keys: Vec<Id> = {
        let mut state = 0x9e3779b9u32;
        (0..probes)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                1 + state % max_s
            })
            .collect()
    };

    let bench = |name: &str, f: &dyn Fn() -> u64| {
        let mut best = f64::INFINITY;
        for _ in 0..5 {
            let t = Instant::now();
            let acc = f();
            let s = t.elapsed().as_secs_f64();
            std::hint::black_box(acc);
            best = best.min(s);
        }
        println!("{name}\t{:.2} ns/probe\t({:.3} s best-of-5)", best * 1e9 / probes as f64, best);
        best
    };

    // a) baseline probe loop
    let base = bench("baseline_probe", &|| keys.iter().map(|&k| probe(&rows, k) as u64).sum());

    // b) + relaxed per-perm atomic counter
    let counters: [AtomicU64; 6] = Default::default();
    let with_atomic = bench("probe_plus_atomic", &|| {
        keys.iter()
            .map(|&k| {
                counters[0].fetch_add(1, Ordering::Relaxed);
                probe(&rows, k) as u64
            })
            .sum()
    });

    // c) + sampled counter: bump the atomic only every 64th probe (local u64 in between)
    let with_sampled = bench("probe_plus_sampled64", &|| {
        let mut local = 0u64;
        keys.iter()
            .map(|&k| {
                local += 1;
                if local & 63 == 0 {
                    counters[1].fetch_add(64, Ordering::Relaxed);
                }
                probe(&rows, k) as u64
            })
            .sum()
    });

    // raw fetch_add in isolation
    let t = Instant::now();
    let n = 100_000_000u64;
    for _ in 0..n {
        counters[2].fetch_add(1, Ordering::Relaxed);
    }
    let raw = t.elapsed().as_secs_f64();
    println!("raw_fetch_add\t{:.2} ns/op", raw * 1e9 / n as f64);

    // d) contended: 4 threads, one shared counter, each probing
    let shared = AtomicU64::new(0);
    let t = Instant::now();
    std::thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                let mut acc = 0u64;
                for &k in &keys {
                    shared.fetch_add(1, Ordering::Relaxed);
                    acc += probe(&rows, k) as u64;
                }
                std::hint::black_box(acc);
            });
        }
    });
    let contended = t.elapsed().as_secs_f64();
    println!("contended_4thr_probe_plus_atomic\t{:.2} ns/probe (per-thread)", contended * 1e9 / probes as f64);

    println!(
        "overhead\tatomic\t{:.1}%\tsampled64\t{:.1}%",
        100.0 * (with_atomic - base) / base,
        100.0 * (with_sampled - base) / base
    );
}
