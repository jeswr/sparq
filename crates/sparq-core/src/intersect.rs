//! Sorted-set intersection kernels for the join paths (merge-join, Leapfrog
//! Triejoin) — the one NEON target the M1 hardware research measured as a real
//! win (`research/hardware/m1-apple-silicon.md` §1: 1.6–2.8× spike on contiguous
//! `&[u32]`), with the explicit caveat that the engine's row-major layouts may
//! eat the gain (`research/hardware/optimization-findings.md` §2). This module
//! exists to settle that measure-first question: it provides
//!
//! * `intersect_scalar` — the branchy two-pointer baseline (the semantic
//!   reference for the differential tests),
//! * `intersect_gallop` — galloping/exponential search for high-skew pairs,
//! * `intersect_simd` — on `aarch64`, a Schlegel/Lemire-style block-vs-block
//!   NEON kernel (4×4 `vceqq` rotations + `vqtbl1q` shuffle-LUT compaction);
//!   elsewhere it falls back to scalar,
//! * `intersect` — the dispatching entry point encoding the measured
//!   gallop-vs-block crossover,
//! * `bench_*` harnesses used by `sparq-cli bench-intersect` to re-validate the
//!   headroom on real id distributions before any call-site wiring.
//!
//! CONTRACT: all inputs must be sorted **strictly increasing** (sets, no
//! duplicates) — exactly what the call sites have (distinct trie keys per LFTJ
//! level, sorted unique id lists). Debug builds assert it. The scalar baseline
//! is additionally well-defined (min-count multiset) on inputs with duplicates,
//! but the SIMD kernel is not claimed for them.

use crate::dict::Id;

/// Debug-only validation of the strictly-increasing input contract.
#[inline]
fn debug_check_strictly_increasing(a: &[Id], b: &[Id]) {
    debug_assert!(a.windows(2).all(|w| w[0] < w[1]), "intersect: `a` not strictly increasing");
    debug_assert!(b.windows(2).all(|w| w[0] < w[1]), "intersect: `b` not strictly increasing");
}

/// [OPUS-4.8] Cheap, branch-light release check that a slice is sorted strictly
/// increasing (sets, no duplicates). This is the SAFETY GATE for the SIMD kernel:
/// the NEON block kernel's capacity reservation (`min(|a|,|b|) + 4`) is only valid
/// when both inputs honour the strictly-increasing contract. On a duplicated `a`
/// (e.g. `a = [5;80]`, `b = [5, 1000..1063]`) the kernel re-emits the same value
/// from every block and writes past the reserved capacity — a heap overflow
/// (roborev High #2155). We therefore validate in RELEASE too and fall back to the
/// always-safe scalar baseline if the contract is broken, so NO input — including
/// duplicates / non-increasing — can ever drive the raw-pointer store out of bounds.
#[inline]
fn is_strictly_increasing(s: &[Id]) -> bool {
    // `all` short-circuits on the first violation; ~1 cmp/elem, predictable branch
    // on honest inputs (the call-site invariant), negligible vs the kernel work.
    s.windows(2).all(|w| w[0] < w[1])
}

// ---- scalar baseline ----------------------------------------------------------

/// Branchy two-pointer merge intersection. Appends the common elements to `out`.
/// Semantic reference for the SIMD kernel (and min-count-correct on multisets).
pub fn intersect_scalar(a: &[Id], b: &[Id], out: &mut Vec<Id>) {
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        let (x, y) = (a[i], b[j]);
        if x < y {
            i += 1;
        } else if y < x {
            j += 1;
        } else {
            out.push(x);
            i += 1;
            j += 1;
        }
    }
}

// ---- galloping (high-skew) ------------------------------------------------------

/// First index in `s[lo..]` with `s[idx] >= x`, by exponential search from `lo`.
#[inline]
fn gallop_lower_bound(s: &[Id], mut lo: usize, x: Id) -> usize {
    let mut step = 1;
    let mut hi = lo;
    while hi < s.len() && s[hi] < x {
        lo = hi + 1;
        hi += step;
        step <<= 1;
    }
    let hi = hi.min(s.len());
    lo + s[lo..hi].partition_point(|&v| v < x)
}

/// Galloping intersection for skewed cardinalities: walks the smaller side,
/// exponential-searches the larger. `a` and `b` may be passed in either order.
pub fn intersect_gallop(a: &[Id], b: &[Id], out: &mut Vec<Id>) {
    let (small, large) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let mut j = 0;
    for &x in small {
        j = gallop_lower_bound(large, j, x);
        if j == large.len() {
            break;
        }
        if large[j] == x {
            out.push(x);
            j += 1;
        }
    }
}

// ---- NEON kernel (aarch64) ------------------------------------------------------

#[cfg(target_arch = "aarch64")]
mod neon {
    use super::Id;
    use core::arch::aarch64::*;

    /// Shuffle LUT: for each 4-bit lane mask, the byte indices that compact the
    /// selected `u32` lanes of a 16-byte vector to the front (0xFF pads ignored —
    /// only the first `popcount(mask)` lanes of the result are stored as valid).
    const fn build_lut() -> [[u8; 16]; 16] {
        let mut lut = [[0xFFu8; 16]; 16];
        let mut mask = 0usize;
        while mask < 16 {
            let mut out_lane = 0usize;
            let mut lane = 0usize;
            while lane < 4 {
                if mask & (1 << lane) != 0 {
                    let mut byte = 0usize;
                    while byte < 4 {
                        lut[mask][out_lane * 4 + byte] = (lane * 4 + byte) as u8;
                        byte += 1;
                    }
                    out_lane += 1;
                }
                lane += 1;
            }
            mask += 1;
        }
        lut
    }
    static LUT: [[u8; 16]; 16] = build_lut();

    /// Block-vs-block NEON intersection of strictly-increasing `u32` slices
    /// (Schlegel et al. / Lemire SIMD set intersection): compare a 4-lane block
    /// of `a` against all 4 rotations of a 4-lane block of `b`, compact the
    /// matching lanes with a `vqtbl1q` shuffle, advance the block whose max is
    /// smaller; scalar tail.
    ///
    /// # Safety
    /// [OPUS-4.8] Both `a` and `b` MUST be sorted **strictly increasing** (no
    /// duplicates). The reservation below (`min(|a|,|b|) + 4`) is only sufficient
    /// under that contract: with duplicates the kernel can emit the same value
    /// from multiple blocks and write past the reserved capacity through the raw
    /// `vst1q_u8` store — a heap buffer overflow (roborev High #2155). This is
    /// `unsafe` and private precisely so the only caller is the validated public
    /// wrapper `intersect_simd`, which falls back to scalar on any violation.
    pub unsafe fn intersect_neon(a: &[Id], b: &[Id], out: &mut Vec<Id>) {
        let (mut i, mut j) = (0usize, 0usize);
        // Worst case appends min(|a|,|b|) elements; +4 slack for the vector store.
        out.reserve(a.len().min(b.len()) + 4);
        unsafe {
            let weights = vld1q_u32([1u32, 2, 4, 8].as_ptr());
            while i + 4 <= a.len() && j + 4 <= b.len() {
                let va = vld1q_u32(a.as_ptr().add(i));
                let vb = vld1q_u32(b.as_ptr().add(j));
                let m0 = vceqq_u32(va, vb);
                let m1 = vceqq_u32(va, vextq_u32::<1>(vb, vb));
                let m2 = vceqq_u32(va, vextq_u32::<2>(vb, vb));
                let m3 = vceqq_u32(va, vextq_u32::<3>(vb, vb));
                let m = vorrq_u32(vorrq_u32(m0, m1), vorrq_u32(m2, m3));
                // 4-bit lane mask: lanes are 0/all-ones → AND with the lane weight
                // {1,2,4,8} keeps the weight of matching lanes, then sum.
                let mask = vaddvq_u32(vandq_u32(m, weights)) as usize;
                if mask != 0 {
                    let shuf = vld1q_u8(LUT[mask].as_ptr());
                    let packed = vqtbl1q_u8(vreinterpretq_u8_u32(va), shuf);
                    let n = mask.count_ones() as usize;
                    let len = out.len();
                    // SAFETY: reserve above guarantees 16 spare bytes past len.
                    vst1q_u8(out.as_mut_ptr().add(len) as *mut u8, packed);
                    out.set_len(len + n);
                }
                let amax = *a.get_unchecked(i + 3);
                let bmax = *b.get_unchecked(j + 3);
                // Branchless dual advance (both advance on equal maxima).
                i += if amax <= bmax { 4 } else { 0 };
                j += if bmax <= amax { 4 } else { 0 };
            }
        }
        // Scalar tail (also handles inputs shorter than one block).
        let (mut i, mut j) = (i, j);
        while i < a.len() && j < b.len() {
            let (x, y) = (a[i], b[j]);
            if x < y {
                i += 1;
            } else if y < x {
                j += 1;
            } else {
                out.push(x);
                i += 1;
                j += 1;
            }
        }
    }
}

/// SIMD intersection where the ISA has a measured kernel (aarch64 NEON), the
/// scalar baseline elsewhere.
///
/// Inputs are *expected* to be sorted strictly increasing (the call-site
/// invariant). [OPUS-4.8] This wrapper is nevertheless **memory-safe for every
/// input**: before dispatching to the unchecked NEON kernel it verifies the
/// strictly-increasing contract in RELEASE as well as debug, and on any
/// violation (duplicates, out-of-order — the roborev High #2155 OOB shape) it
/// falls back to the always-safe scalar baseline. The scalar fallback is the
/// documented multiset reference, so the result also stays well-defined.
#[inline]
pub fn intersect_simd(a: &[Id], b: &[Id], out: &mut Vec<Id>) {
    debug_check_strictly_increasing(a, b);
    #[cfg(target_arch = "aarch64")]
    {
        // [OPUS-4.8] SAFETY GATE for #2155: the raw-pointer NEON store assumes a
        // capacity bound that only holds for strictly-increasing sets. Validate
        // here (release-checked) and degrade to the safe scalar merge otherwise,
        // so no input can drive `intersect_neon` past its reserved capacity.
        if is_strictly_increasing(a) && is_strictly_increasing(b) {
            // SAFETY: both inputs verified strictly increasing immediately above,
            // satisfying `intersect_neon`'s documented precondition.
            unsafe { neon::intersect_neon(a, b, out); }
        } else {
            intersect_scalar(a, b, out);
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        intersect_scalar(a, b, out);
    }
}

/// Size-ratio above which galloping beats the linear kernels (measured on M1 by
/// `bench-intersect ratio`; see research/hardware/optimization-findings.md).
pub const GALLOP_RATIO: usize = 32;

/// Dispatching sorted-set intersection: galloping for high-skew pairs, the SIMD
/// block kernel (scalar off-aarch64) for comparable cardinalities. Inputs must be
/// sorted strictly increasing; appends the common elements to `out`.
#[inline]
pub fn intersect(a: &[Id], b: &[Id], out: &mut Vec<Id>) {
    let (s, l) = if a.len() <= b.len() { (a.len(), b.len()) } else { (b.len(), a.len()) };
    if s == 0 {
        return;
    }
    if l / s.max(1) >= GALLOP_RATIO {
        debug_check_strictly_increasing(a, b);
        intersect_gallop(a, b, out);
    } else {
        intersect_simd(a, b, out);
    }
}

// ---- microbench harness (sparq-cli bench-intersect) -----------------------------

/// Deterministic xorshift for the bench generators (no rand dep).
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }
    fn next(&mut self) -> u64 {
        let x = &mut self.0;
        *x ^= *x << 13;
        *x ^= *x >> 7;
        *x ^= *x << 17;
        *x
    }
}

/// Sorted strictly-increasing random set of `n` ids drawn from `[0, universe)`.
fn random_set(rng: &mut Rng, n: usize, universe: u64) -> Vec<Id> {
    let mut v: Vec<Id> = (0..n * 2).map(|_| (rng.next() % universe) as Id).collect();
    v.sort_unstable();
    v.dedup();
    v.truncate(n);
    v
}

/// A set of `n` ids of which roughly `hit_frac` come from `base` (the overlap)
/// and the rest are fresh draws — controls intersection selectivity.
fn overlapping_set(rng: &mut Rng, base: &[Id], n: usize, hit_frac: f64, universe: u64) -> Vec<Id> {
    let hits = ((n as f64) * hit_frac) as usize;
    let mut v: Vec<Id> = Vec::with_capacity(n * 2);
    for _ in 0..hits {
        v.push(base[(rng.next() as usize) % base.len()]);
    }
    for _ in 0..(n.saturating_sub(hits)) * 2 {
        v.push((rng.next() % universe) as Id);
    }
    v.sort_unstable();
    v.dedup();
    v
}

fn time_best<F: FnMut() -> usize>(iters: usize, mut f: F) -> (f64, usize) {
    let mut best = f64::INFINITY;
    let mut n = 0;
    for _ in 0..iters.max(1) {
        let t = std::time::Instant::now();
        n = f();
        let s = t.elapsed().as_secs_f64();
        if s < best {
            best = s;
        }
    }
    (best, n)
}

/// One measured row: best-of-`iters` time per variant on the same (a, b) pair.
/// Returns (label, |a|, |b|, |a∩b|, scalar_s, gallop_s, simd_s).
pub fn bench_pair(label: &str, a: &[Id], b: &[Id], iters: usize) -> (String, usize, usize, usize, f64, f64, f64) {
    let mut out = Vec::new();
    let (ts, n) = time_best(iters, || {
        out.clear();
        intersect_scalar(a, b, &mut out);
        std::hint::black_box(out.len())
    });
    let (tg, ng) = time_best(iters, || {
        out.clear();
        intersect_gallop(a, b, &mut out);
        std::hint::black_box(out.len())
    });
    let (tv, nv) = time_best(iters, || {
        out.clear();
        intersect_simd(a, b, &mut out);
        std::hint::black_box(out.len())
    });
    assert_eq!(n, ng, "gallop result size mismatch on {label}");
    assert_eq!(n, nv, "simd result size mismatch on {label}");
    (label.to_string(), a.len(), b.len(), n, ts, tg, tv)
}

fn print_row(r: &(String, usize, usize, usize, f64, f64, f64)) {
    let (label, la, lb, n, ts, tg, tv) = r;
    println!(
        "{label:<34} |a|={la:<9} |b|={lb:<9} |∩|={n:<9} scalar={:>9.1}µs gallop={:>9.1}µs simd={:>9.1}µs  simd/scalar={:>5.2}x gallop/scalar={:>5.2}x",
        ts * 1e6,
        tg * 1e6,
        tv * 1e6,
        ts / tv,
        ts / tg,
    );
}

/// Synthetic-distribution grid: selectivity × skew, sizes representative of the
/// join inputs (adjacency lists and trie key columns), and a row-strided variant
/// measuring the extract-column cost the row-major layouts impose.
pub fn bench_grid(iters: usize) {
    let mut rng = Rng::new(0xC0FFEE);
    println!("# contiguous strictly-increasing u32 sets (the kernel's ideal shape)");
    for &(nb, label_sz) in &[(1usize << 14, "16K"), (1 << 17, "128K"), (1 << 20, "1M")] {
        for &(hf, label_sel) in &[(0.9, "dense-overlap"), (0.5, "mid-overlap"), (0.05, "sparse-overlap")] {
            let universe = (nb as u64) * 8;
            let b = random_set(&mut rng, nb, universe);
            let a = overlapping_set(&mut rng, &b, nb, hf, universe);
            let r = bench_pair(&format!("equal-size {label_sz} {label_sel}"), &a, &b, iters);
            print_row(&r);
        }
    }
    println!("\n# skew (gallop-vs-kernel crossover): |b| = 1M, |a| = |b|/ratio");
    let nb = 1usize << 20;
    let universe = (nb as u64) * 8;
    let b = random_set(&mut rng, nb, universe);
    for &ratio in &[2usize, 4, 8, 16, 32, 64, 256, 1024, 4096] {
        let na = (nb / ratio).max(4);
        let a = overlapping_set(&mut rng, &b, na, 0.5, universe);
        let r = bench_pair(&format!("skew 1:{ratio}"), &a, &b, iters);
        print_row(&r);
    }
    println!("\n# row-strided keys (the engine's ACTUAL layouts): extract-then-simd vs direct scalar");
    bench_strided(iters);
    println!("\n# many tiny intersections (LFTJ inner-level shape: degree-sized adjacency)");
    bench_tiny(iters);
}

/// The engine's actual layouts: join keys strided inside rows. Measures the
/// gather/extract pass the research doc warned about, against intersecting
/// in place with the scalar two-pointer over strided keys.
fn bench_strided(iters: usize) {
    let mut rng = Rng::new(0xBEEF);
    let n = 1usize << 20;
    let universe = (n as u64) * 8;
    let keys_a = random_set(&mut rng, n, universe);
    let keys_b = overlapping_set(&mut rng, &keys_a, n, 0.5, universe);
    for &width in &[2usize, 3, 4] {
        // Row-major rows of `width` ids, key in column 0 (Vec<[Id;w]> ≈ SmallVec inline rows).
        let rows_a: Vec<Vec<Id>> = keys_a.iter().map(|&k| {
            let mut r = vec![k]; r.extend((1..width).map(|c| c as Id)); r
        }).collect();
        let rows_b: Vec<Vec<Id>> = keys_b.iter().map(|&k| {
            let mut r = vec![k]; r.extend((1..width).map(|c| c as Id)); r
        }).collect();
        // Flat strided (the [[u32;3]] store-scan shape) — contiguous rows, key every `width`.
        let flat_a: Vec<Id> = rows_a.iter().flatten().copied().collect();
        let flat_b: Vec<Id> = rows_b.iter().flatten().copied().collect();

        // (1) direct scalar two-pointer over the strided keys (current engine behavior).
        let (t_direct, n_direct) = time_best(iters, || {
            let (mut i, mut j, mut c) = (0usize, 0usize, 0usize);
            let (la, lb) = (flat_a.len() / width, flat_b.len() / width);
            while i < la && j < lb {
                let (x, y) = (flat_a[i * width], flat_b[j * width]);
                if x < y { i += 1; } else if y < x { j += 1; } else { c += 1; i += 1; j += 1; }
            }
            std::hint::black_box(c)
        });
        // (2) extract both key columns to contiguous buffers, then SIMD kernel.
        let mut out = Vec::new();
        let (t_extract, n_extract) = time_best(iters, || {
            let ka: Vec<Id> = flat_a.iter().step_by(width).copied().collect();
            let kb: Vec<Id> = flat_b.iter().step_by(width).copied().collect();
            out.clear();
            intersect_simd(&ka, &kb, &mut out);
            std::hint::black_box(out.len())
        });
        // (3) SIMD kernel given ALREADY-contiguous keys (the columnar-future shape).
        let (t_cont, n_cont) = time_best(iters, || {
            out.clear();
            intersect_simd(&keys_a, &keys_b, &mut out);
            std::hint::black_box(out.len())
        });
        // (4) pointer-chased rows (the LFTJ Trie Vec<Vec<Id>> shape), scalar.
        let (t_chase, n_chase) = time_best(iters, || {
            let (mut i, mut j, mut c) = (0usize, 0usize, 0usize);
            while i < rows_a.len() && j < rows_b.len() {
                let (x, y) = (rows_a[i][0], rows_b[j][0]);
                if x < y { i += 1; } else if y < x { j += 1; } else { c += 1; i += 1; j += 1; }
            }
            std::hint::black_box(c)
        });
        assert_eq!(n_direct, n_extract);
        assert_eq!(n_direct, n_cont);
        assert_eq!(n_direct, n_chase);
        println!(
            "width={width}: strided-scalar={:>8.1}µs  Vec<Vec>-scalar={:>8.1}µs  extract+simd={:>8.1}µs  contiguous-simd={:>8.1}µs  → extract+simd/strided={:.2}x  contiguous/strided={:.2}x",
            t_direct * 1e6, t_chase * 1e6, t_extract * 1e6, t_cont * 1e6,
            t_direct / t_extract, t_direct / t_cont,
        );
    }
}

/// Degree-sized intersections (what LFTJ actually does per inner level on a graph):
/// per-call overhead dominates — measures whether the kernel helps or hurts there.
fn bench_tiny(iters: usize) {
    let mut rng = Rng::new(0xACE);
    for &deg in &[8usize, 32, 128] {
        let universe = (deg as u64) * 16;
        let pairs: Vec<(Vec<Id>, Vec<Id>)> = (0..4096)
            .map(|_| {
                let b = random_set(&mut rng, deg, universe);
                let a = overlapping_set(&mut rng, &b, deg, 0.3, universe);
                (a, b)
            })
            .collect();
        let mut out = Vec::new();
        let (ts, ns) = time_best(iters, || {
            let mut c = 0;
            for (a, b) in &pairs {
                out.clear();
                intersect_scalar(a, b, &mut out);
                c += out.len();
            }
            std::hint::black_box(c)
        });
        let (tv, nv) = time_best(iters, || {
            let mut c = 0;
            for (a, b) in &pairs {
                out.clear();
                intersect_simd(a, b, &mut out);
                c += out.len();
            }
            std::hint::black_box(c)
        });
        assert_eq!(ns, nv);
        println!(
            "deg≈{deg:<4} 4096 calls: scalar={:>8.1}µs simd={:>8.1}µs  simd/scalar={:.2}x",
            ts * 1e6, tv * 1e6, ts / tv
        );
    }
}

/// Real-distribution bench: loads an N-Triples/Turtle file, picks the most
/// frequent predicate, builds its sorted distinct subject/object adjacency
/// structure, and times the intersection shapes the join paths actually produce.
pub fn bench_real(graph: &crate::Graph, iters: usize) {
    use std::collections::HashMap;
    // Most frequent predicate + its (s, o) pairs.
    let mut pred_count: HashMap<Id, usize> = HashMap::new();
    for t in graph.iter_ids() {
        *pred_count.entry(t[1]).or_insert(0) += 1;
    }
    let (&p, &cnt) = pred_count.iter().max_by_key(|(_, &c)| c).expect("non-empty graph");
    println!("# real data: {} triples, most frequent predicate id={p} ({cnt} triples)", graph.len());
    let mut out_adj: HashMap<Id, Vec<Id>> = HashMap::new();
    let mut in_adj: HashMap<Id, Vec<Id>> = HashMap::new();
    for t in graph.iter_ids() {
        if t[1] == p {
            out_adj.entry(t[0]).or_default().push(t[2]);
            in_adj.entry(t[2]).or_default().push(t[0]);
        }
    }
    for v in out_adj.values_mut().chain(in_adj.values_mut()) {
        v.sort_unstable();
        v.dedup();
    }
    let mut subjects: Vec<Id> = out_adj.keys().copied().collect();
    subjects.sort_unstable();
    let mut objects: Vec<Id> = in_adj.keys().copied().collect();
    objects.sort_unstable();

    // Shape 1: the top-level LFTJ intersection (distinct subjects ∩ distinct objects).
    let r = bench_pair("real top-level subj∩obj", &subjects, &objects, iters);
    print_row(&r);

    // Shape 2: per-node adjacency intersections (LFTJ inner level; degree-sized).
    let both: Vec<Id> = {
        let mut v = Vec::new();
        intersect_scalar(&subjects, &objects, &mut v);
        v
    };
    let sample: Vec<(&Vec<Id>, &Vec<Id>)> = both
        .iter()
        .take(100_000)
        .filter_map(|n| Some((out_adj.get(n)?, in_adj.get(n)?)))
        .collect();
    let mut out = Vec::new();
    let (ts, ns) = time_best(iters, || {
        let mut c = 0;
        for (a, b) in &sample {
            out.clear();
            intersect_scalar(a, b, &mut out);
            c += out.len();
        }
        std::hint::black_box(c)
    });
    let (tv, nv) = time_best(iters, || {
        let mut c = 0;
        for (a, b) in &sample {
            out.clear();
            intersect_simd(a, b, &mut out);
            c += out.len();
        }
        std::hint::black_box(c)
    });
    assert_eq!(ns, nv);
    let degs: Vec<usize> = sample.iter().map(|(a, b)| a.len().min(b.len())).collect();
    let mean_deg = degs.iter().sum::<usize>() as f64 / degs.len().max(1) as f64;
    println!(
        "real adjacency out(n)∩in(n) ({} pairs, mean min-degree {:.1}): scalar={:.1}µs simd={:.1}µs  simd/scalar={:.2}x",
        sample.len(), mean_deg, ts * 1e6, tv * 1e6, ts / tv
    );

    // Shape 3: skew — one node's adjacency vs the full distinct-subject column.
    if let Some(big) = out_adj.values().max_by_key(|v| v.len()) {
        let r = bench_pair("real adj(max-degree-node) ∩ subj", big, &subjects, iters);
        print_row(&r);
    }
}

// ---- tests ----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ref_intersect(a: &[Id], b: &[Id]) -> Vec<Id> {
        let mut out = Vec::new();
        intersect_scalar(a, b, &mut out);
        out
    }

    #[test]
    fn empty_disjoint_identical() {
        let e: Vec<Id> = vec![];
        let a = vec![1, 5, 9];
        let b = vec![2, 6, 10];
        type IntersectFn = fn(&[Id], &[Id], &mut Vec<Id>);
        for f in [intersect_simd as IntersectFn, intersect_gallop as IntersectFn, intersect as IntersectFn] {
            let mut out = Vec::new();
            f(&e, &a, &mut out);
            assert!(out.is_empty());
            out.clear();
            f(&a, &e, &mut out);
            assert!(out.is_empty());
            out.clear();
            f(&a, &b, &mut out);
            assert!(out.is_empty());
            out.clear();
            f(&a, &a, &mut out);
            assert_eq!(out, a);
        }
    }

    /// Differential: SIMD kernel and gallop == scalar on randomized strictly-
    /// increasing inputs across sizes, selectivities, and skews (>1M cases).
    #[test]
    fn differential_randomized() {
        let mut rng = Rng::new(42);
        let mut cases = 0usize;
        // Many small cases (block-boundary / tail edge coverage)…
        for n in 0..2048usize {
            for _ in 0..512 {
                let universe = 1 + (rng.next() % 64);
                let na = (rng.next() as usize) % (n % 24 + 1);
                let nb = (rng.next() as usize) % (n % 24 + 1);
                let a = random_set(&mut rng, na, universe);
                let b = random_set(&mut rng, nb, universe);
                let expect = ref_intersect(&a, &b);
                let mut got = Vec::new();
                intersect_simd(&a, &b, &mut got);
                assert_eq!(got, expect, "simd a={a:?} b={b:?}");
                got.clear();
                intersect_gallop(&a, &b, &mut got);
                assert_eq!(got, expect, "gallop a={a:?} b={b:?}");
                got.clear();
                intersect(&a, &b, &mut got);
                assert_eq!(got, expect, "dispatch a={a:?} b={b:?}");
                cases += 3;
            }
        }
        // …plus larger overlapping cases.
        for _ in 0..512 {
            let universe = 1 + (rng.next() % 100_000);
            let nb = (rng.next() as usize) % 4096;
            let b = random_set(&mut rng, nb, universe);
            let hf = (rng.next() % 100) as f64 / 100.0;
            let na = (rng.next() as usize) % 4096;
            let a = if b.is_empty() {
                random_set(&mut rng, 64, universe)
            } else {
                overlapping_set(&mut rng, &b, na, hf, universe)
            };
            let expect = ref_intersect(&a, &b);
            let mut got = Vec::new();
            intersect_simd(&a, &b, &mut got);
            assert_eq!(got, expect);
            got.clear();
            intersect(&a, &b, &mut got);
            assert_eq!(got, expect);
            cases += 2;
        }
        assert!(cases > 1_000_000, "only {cases} differential cases ran");
    }

    /// The scalar baseline keeps min-count multiset semantics on duplicate inputs
    /// (the SIMD kernel is not claimed for duplicates — contract is sets).
    #[test]
    fn scalar_multiset_semantics() {
        let mut out = Vec::new();
        intersect_scalar(&[1, 1, 2, 2, 2, 3], &[1, 2, 2, 4], &mut out);
        assert_eq!(out, vec![1, 2, 2]);
    }

    /// [OPUS-4.8] roborev High #2155 regression: the exact OOB-trigger shape.
    /// `a = [5; 80]` (duplicates), `b = [5, 1000..1063]`. The raw NEON kernel
    /// reserved only `min(|a|,|b|)+4 = 68` but, treating each `[5,5,5,5]` block
    /// of `a` as four matches against the single `5` in `b`, would emit 80 ids —
    /// a 12-element (48-byte) heap overflow AND `len(80) > capacity(68)` (an
    /// instant Vec-invariant UB). The validated public wrapper must instead route
    /// this to the scalar baseline and return the correct set intersection `[5]`,
    /// with `len <= capacity` preserved. Run under release (`--release`) too —
    /// that is where the original `debug_assert!` gate evaporated.
    #[test]
    fn oob_2155_duplicate_block_is_memory_safe() {
        let a: Vec<Id> = vec![5u32; 80];
        let mut b: Vec<Id> = vec![5u32];
        b.extend(1000u32..1063); // |b| = 64, so min = 64, old reserve = 68
        let mut out: Vec<Id> = Vec::new();
        intersect_simd(&a, &b, &mut out);
        // Correct multiset/set intersection of these inputs is a single 5.
        assert_eq!(out, vec![5]);
        // The Vec invariant must hold: the fix must never let len exceed capacity.
        assert!(out.len() <= out.capacity(), "len {} > cap {} — OOB still live", out.len(), out.capacity());
        // Dispatch entry point must be safe on the same shape.
        let mut out2: Vec<Id> = Vec::new();
        intersect(&a, &b, &mut out2);
        assert!(out2.len() <= out2.capacity());
    }

    /// [OPUS-4.8] Adversarial fuzz: `intersect_simd` / `intersect` must (a) never
    /// violate the Vec capacity invariant and (b) equal the scalar reference for
    /// ALL inputs — strictly-increasing, with duplicates, non-monotone, and
    /// empty — not just the contract-honouring set inputs. Inputs that break the
    /// contract are routed to the scalar baseline, so the safe API stays total.
    #[test]
    fn fuzz_arbitrary_inputs_safe_and_correct() {
        let mut rng = Rng::new(0xDEADBEEF);
        let mut cases = 0usize;
        for _ in 0..200_000 {
            // Small, dup-heavy universes maximise duplicate blocks (the OOB shape).
            let universe = 1 + (rng.next() % 12);
            let na = (rng.next() as usize) % 96;
            let nb = (rng.next() as usize) % 96;
            // Sorted but NOT deduped → duplicates are common; sometimes shuffle a
            // pair to also exercise the non-monotone fallback branch.
            let mut a: Vec<Id> = (0..na).map(|_| (rng.next() % universe) as Id).collect();
            let mut b: Vec<Id> = (0..nb).map(|_| (rng.next() % universe) as Id).collect();
            a.sort_unstable();
            b.sort_unstable();
            if na >= 2 && rng.next() & 1 == 0 {
                let i = (rng.next() as usize) % (na - 1);
                a.swap(i, i + 1); // make `a` non-strictly-increasing on purpose
            }
            // Reference is the scalar baseline on the SAME (possibly duplicated) input.
            let expect = ref_intersect(&a, &b);
            let mut got = Vec::new();
            intersect_simd(&a, &b, &mut got);
            assert!(got.len() <= got.capacity(), "capacity invariant violated (simd)");
            assert_eq!(got, expect, "simd != scalar on a={a:?} b={b:?}");
            // `intersect` dispatch must also be safe + match scalar.
            let mut got2 = Vec::new();
            intersect(&a, &b, &mut got2);
            assert!(got2.len() <= got2.capacity(), "capacity invariant violated (dispatch)");
            // dispatch may pick gallop (skew) or simd; both reduce to the scalar
            // semantics for these set/multiset inputs.
            assert_eq!(got2, expect, "dispatch != scalar on a={a:?} b={b:?}");
            cases += 2;
        }
        assert!(cases >= 400_000, "only {cases} fuzz cases ran");
    }

    /// [OPUS-4.8] Explicit duplicate-block correctness spot-checks against the
    /// scalar reference (kept small for readable failures).
    #[test]
    fn simd_matches_scalar_on_duplicates() {
        let dup_cases: &[(&[Id], &[Id])] = &[
            (&[7, 7, 7, 7], &[5, 6, 7, 8]),
            (&[1, 1, 1, 1, 1, 1, 1, 1], &[1, 1, 1, 1]),
            (&[2, 2, 4, 4, 6, 6], &[2, 4, 4, 6, 6, 8]),
            (&[5; 80], &[5, 9, 13, 17]),
            (&[], &[1, 1, 2]),
            (&[1, 1, 2], &[]),
        ];
        for (a, b) in dup_cases {
            let mut want = Vec::new();
            intersect_scalar(a, b, &mut want);
            let mut got = Vec::new();
            intersect_simd(a, b, &mut got);
            assert!(got.len() <= got.capacity());
            assert_eq!(got, want, "simd != scalar on a={a:?} b={b:?}");
        }
    }

    #[test]
    fn gallop_lower_bound_basics() {
        let s = [2u32, 4, 6, 8, 10, 12, 14, 16];
        assert_eq!(gallop_lower_bound(&s, 0, 1), 0);
        assert_eq!(gallop_lower_bound(&s, 0, 2), 0);
        assert_eq!(gallop_lower_bound(&s, 0, 9), 4);
        assert_eq!(gallop_lower_bound(&s, 3, 17), 8);
        assert_eq!(gallop_lower_bound(&s, 8, 1), 8);
    }
}
