//! T3 measurement: 32-bit vs 64-bit permutation rows — the LOSS FLOOR of widening
//! `Id` to `u64` (research/roadmap.md T3). Measures, at olympics scale (~1.8M) and
//! synthetic 10M rows:
//!   1. sort time of `Vec<[u32;3]>` vs `Vec<[u64;3]>` (index build kernel),
//!   2. full-scan time (sum over one column — the bandwidth-bound scan kernel),
//!   3. random binary-search range probes (the pattern-scan kernel),
//!   4. delta+varint compressed size (the SPQCPRM1 block encoding) for:
//!      u32 rows as-is, u64 rows with the SAME id values (pure widening),
//!      and u64 rows where inline ids carry HIGH tag bits (bit 60), with and
//!      without inline-id-heavy object columns.
//!
//! Run: cargo run -p sparq-core --example bench_id_width --release [rows]

use std::time::Instant;

fn put_varint(out: &mut Vec<u8>, mut x: u64) {
    while x >= 0x80 {
        out.push((x as u8) | 0x80);
        x >>= 7;
    }
    out.push(x as u8);
}

/// SPQCPRM1-style block encoding (128-row blocks, lexicographic delta + varint),
/// generic over the id width via u64 row values.
fn encoded_size(rows: &[[u64; 3]]) -> usize {
    let mut bytes = 0usize;
    let mut blocks: Vec<u8> = Vec::new();
    for chunk in rows.chunks(128) {
        blocks.clear();
        put_varint(&mut blocks, chunk.len() as u64);
        put_varint(&mut blocks, chunk[0][0]);
        put_varint(&mut blocks, chunk[0][1]);
        put_varint(&mut blocks, chunk[0][2]);
        for w in chunk.windows(2) {
            let (p, r) = (w[0], w[1]);
            let d0 = r[0] - p[0];
            put_varint(&mut blocks, d0);
            if d0 == 0 {
                let d1 = r[1] - p[1];
                put_varint(&mut blocks, d1);
                if d1 == 0 {
                    put_varint(&mut blocks, r[2] - p[2]);
                } else {
                    put_varint(&mut blocks, r[2]);
                }
            } else {
                put_varint(&mut blocks, r[1]);
                put_varint(&mut blocks, r[2]);
            }
        }
        bytes += blocks.len();
        bytes += 16; // directory entry: first row (u32x3 padded) + offset, approx
    }
    bytes
}

/// xorshift PRNG (deterministic across runs).
fn rng(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// Synthetic POS-shaped permutation rows: `n` rows, `preds` predicates, subjects/objects
/// dense in [1, n/4). `inline_frac_pct`% of objects are inline-integer ids.
/// `inline_base` is where the inline range sits in the id space (u32 scheme: 2^31).
fn gen_rows(n: usize, inline_frac_pct: u64, inline_base: u64) -> Vec<[u64; 3]> {
    let mut st = 0xDEADBEEFCAFEBABEu64;
    let max_ent = (n as u64 / 4).max(1000);
    let mut v: Vec<[u64; 3]> = (0..n)
        .map(|_| {
            let p = 1 + rng(&mut st) % 40;
            let s = 1 + rng(&mut st) % max_ent;
            let o = if rng(&mut st) % 100 < inline_frac_pct {
                inline_base + rng(&mut st) % 100_000 // inline integer value
            } else {
                1 + rng(&mut st) % max_ent
            };
            [p, o, s] // POS order
        })
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

fn narrow(rows: &[[u64; 3]]) -> Vec<[u32; 3]> {
    rows.iter()
        .map(|r| [r[0] as u32, r[1] as u32, r[2] as u32])
        .collect()
}

fn time<R>(f: impl FnOnce() -> R) -> (R, f64) {
    let t = Instant::now();
    let r = f();
    (r, t.elapsed().as_secs_f64() * 1e3)
}

fn bench_width(label: &str, rows64: &[[u64; 3]], rows32: &[[u32; 3]]) {
    let n = rows64.len();
    println!("\n--- {label}: {n} rows ---");
    println!(
        "memory: u32 {} MB | u64 {} MB",
        n * 12 / 1_000_000,
        n * 24 / 1_000_000
    );

    // 1. sort (build kernel) — min of 3
    let mut best32 = f64::MAX;
    let mut best64 = f64::MAX;
    for _ in 0..3 {
        let mut a = rows32.to_vec();
        let mut st = 0x1234_5678u64;
        // shuffle (Fisher-Yates)
        for i in (1..a.len()).rev() {
            a.swap(i, (rng(&mut st) % (i as u64 + 1)) as usize);
        }
        let (_, ms) = time(|| a.sort_unstable());
        best32 = best32.min(ms);
        let mut b = rows64.to_vec();
        let mut st = 0x1234_5678u64;
        for i in (1..b.len()).rev() {
            b.swap(i, (rng(&mut st) % (i as u64 + 1)) as usize);
        }
        let (_, ms) = time(|| b.sort_unstable());
        best64 = best64.min(ms);
    }
    println!(
        "sort:        u32 {best32:8.1} ms | u64 {best64:8.1} ms | ratio {:.2}x",
        best64 / best32
    );

    // 2. full scan (sum col 2)
    let mut s32 = f64::MAX;
    let mut s64 = f64::MAX;
    let mut sink = 0u64;
    for _ in 0..5 {
        let (r, ms) = time(|| rows32.iter().map(|r| r[2] as u64).sum::<u64>());
        sink ^= r;
        s32 = s32.min(ms);
        let (r, ms) = time(|| rows64.iter().map(|r| r[2]).sum::<u64>());
        sink ^= r;
        s64 = s64.min(ms);
    }
    println!(
        "full scan:   u32 {s32:8.1} ms | u64 {s64:8.1} ms | ratio {:.2}x",
        s64 / s32
    );

    // 3. random range probes: 100k binary searches for predicate-bound prefixes + walk 16 rows
    let probes: Vec<u64> = {
        let mut st = 0xABCDEFu64;
        (0..100_000).map(|_| 1 + rng(&mut st) % 40).collect()
    };
    let mut p32 = f64::MAX;
    let mut p64 = f64::MAX;
    for _ in 0..3 {
        let (r, ms) = time(|| {
            let mut acc = 0u64;
            for &p in &probes {
                let lo = rows32.partition_point(|r| r[0] < p as u32);
                for r in rows32[lo..].iter().take(16) {
                    acc = acc.wrapping_add(r[2] as u64);
                }
            }
            acc
        });
        sink ^= r;
        p32 = p32.min(ms);
        let (r, ms) = time(|| {
            let mut acc = 0u64;
            for &p in &probes {
                let lo = rows64.partition_point(|r| r[0] < p);
                for r in rows64[lo..].iter().take(16) {
                    acc = acc.wrapping_add(r[2]);
                }
            }
            acc
        });
        sink ^= r;
        p64 = p64.min(ms);
    }
    println!(
        "100k probes: u32 {p32:8.1} ms | u64 {p64:8.1} ms | ratio {:.2}x   (sink {sink})",
        p64 / p32
    );
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000_000);

    for &(label, rows) in &[("olympics-scale", 1_800_000usize), ("synthetic", 0)] {
        let n = if rows == 0 { n } else { rows };
        // 30% inline objects approximates a literal-heavy graph.
        let rows64 = gen_rows(n, 30, 1u64 << 31);
        let rows32 = narrow(&rows64);
        bench_width(label, &rows64, &rows32);

        // 4. compressed sizes
        let raw32 = rows32.len() * 12;
        let c32 = encoded_size(&rows64); // values < 2^32: identical varints to u32 enc
                                         // pure widening: same values, ids unchanged (dict ids stay dense; inline stays 2^31)
                                         // tag-high: inline ids move to bit 60 (u64 tag scheme, tags in the top nibble)
        let tagged: Vec<[u64; 3]> = {
            let mut t: Vec<[u64; 3]> = rows64
                .iter()
                .map(|r| {
                    let m = |x: u64| {
                        if x >= (1 << 31) {
                            (1u64 << 60) | (x - (1 << 31))
                        } else {
                            x
                        }
                    };
                    [m(r[0]), m(r[1]), m(r[2])]
                })
                .collect();
            t.sort_unstable();
            t
        };
        let chigh = encoded_size(&tagged);
        println!(
            "compressed:  raw u32 {} MB | varint-u32-ids {} MB | varint-u64-tag-bit60 {} MB ({:+.0}% vs u32-ids)",
            raw32 / 1_000_000,
            c32 / 1_000_000,
            chigh / 1_000_000,
            (chigh as f64 / c32 as f64 - 1.0) * 100.0
        );
    }
}
