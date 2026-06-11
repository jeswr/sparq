//! Kernel correctness vs the scalar CPU reference on random data.
//!
//! Every test acquires its own [`Gpu`] and SKIPS (passes, with a note on
//! stderr) when no adapter exists — CI runs without a GPU and must stay green.
//! Counts/sums over ids are exact (u32/u64 arithmetic); the f64 filter is also
//! exact by construction (bit-pattern total-order comparison, not float math on
//! the device), so it is asserted exactly too — including NaN/±inf/-0.0 edges.

use sparq_gpu::{cpu, Gpu, EMPTY_KEY, MAX_GROUPS};

/// Deterministic xorshift64* stream (same generator the wgpu spike used).
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn u32(&mut self) -> u32 {
        (self.next() >> 32) as u32
    }
}

/// `Some(gpu)` or skip: prints the reason so a skipped CI run is auditable.
fn gpu_or_skip(test: &str) -> Option<Gpu> {
    match Gpu::new() {
        Some(g) => {
            eprintln!("[{test}] running on {} ({:?})", g.info.name, g.info.backend);
            Some(g)
        }
        None => {
            eprintln!("[{test}] SKIPPED — no GPU adapter available");
            None
        }
    }
}

const N: usize = 300_003; // deliberately not a multiple of the 256-wide workgroup

#[test]
fn filter_u32_matches_cpu() {
    let Some(gpu) = gpu_or_skip("filter_u32") else {
        return;
    };
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let col: Vec<u32> = (0..N).map(|_| rng.u32()).collect();
    let resident = gpu.upload_u32(&col);

    for (lo, hi) in [
        (0u32, u32::MAX / 8), // ~12.5% selectivity (the bench predicate)
        (0, u32::MAX),        // ~everything
        (5, 5),               // empty range
        (u32::MAX / 2, u32::MAX / 2 + 1_000_000), // narrow band
    ] {
        let expect = cpu::filter_count_u32(&col, lo, hi);
        assert_eq!(
            gpu.filter_count_u32(&resident, lo, hi),
            expect,
            "lo={lo} hi={hi}"
        );
    }
}

#[test]
fn filter_f64_matches_cpu_including_ieee_edges() {
    let Some(gpu) = gpu_or_skip("filter_f64") else {
        return;
    };
    let mut rng = Rng(0xDEAD_BEEF_CAFE_F00D);
    let mut col: Vec<f64> = (0..N)
        .map(|_| (rng.u32() as f64 - (u32::MAX / 2) as f64) / 1e3)
        .collect();
    // Salt with every awkward IEEE-754 citizen.
    let edges = [
        f64::NAN,
        -f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        0.0,
        -0.0,
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        5e-324, // smallest subnormal
        -5e-324,
        f64::MAX,
        f64::MIN,
    ];
    for (i, &e) in edges.iter().enumerate() {
        let idx = i * 1009 % col.len();
        col[idx] = e;
    }
    let resident = gpu.upload_f64(&col);

    for t in [
        0.0,
        -0.0,
        1.5,
        -123456.789,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        5e-324,
    ] {
        let expect = cpu::filter_count_f64_gt(&col, t);
        assert_eq!(gpu.filter_count_f64_gt(&resident, t), expect, "t={t}");
    }
}

#[test]
fn hash_probe_matches_cpu() {
    let Some(gpu) = gpu_or_skip("hash_probe") else {
        return;
    };
    let mut rng = Rng(0x1234_5678_9ABC_DEF0);
    let n_build = N / 4;
    // Build keys drawn from a domain twice the build size => duplicates exist
    // (1:N join rows); probe drawn from a wider domain => ~50% miss rate.
    let build_keys: Vec<u32> = (0..n_build)
        .map(|_| rng.u32() % (2 * n_build as u32))
        .collect();
    let payloads: Vec<u32> = (0..n_build).map(|_| rng.u32()).collect();
    let probe: Vec<u32> = (0..N)
        .map(|_| {
            let k = rng.u32() % (4 * n_build as u32);
            if k == EMPTY_KEY {
                0
            } else {
                k
            }
        })
        .collect();

    let slots = cpu::build_hash_table(&build_keys, &payloads);
    let table = gpu.upload_table(&slots);
    let probe_col = gpu.upload_u32(&probe);

    let expect = cpu::hash_probe(&slots, &probe);
    assert_eq!(gpu.hash_probe(&table, &probe_col), expect);
}

#[test]
fn group_aggregate_matches_cpu() {
    let Some(gpu) = gpu_or_skip("group_aggregate") else {
        return;
    };
    let mut rng = Rng(0x0F0F_F0F0_1337_4242);
    for groups in [1u32, 7, MAX_GROUPS] {
        let keys: Vec<u32> = (0..N).map(|_| rng.u32() % groups).collect();
        // Full-range u32 values: exercises the lo/hi carry emulation hard
        // (group sums far exceed u32).
        let vals: Vec<u32> = (0..N).map(|_| rng.u32()).collect();
        let keys_col = gpu.upload_u32(&keys);
        let vals_col = gpu.upload_u32(&vals);

        let expect = cpu::group_aggregate(&keys, &vals, groups);
        let got = gpu.group_aggregate(&keys_col, &vals_col, groups);
        assert_eq!(got, expect, "groups={groups}");
        let total: u64 = got.iter().map(|(c, _)| c).sum();
        assert_eq!(total, N as u64, "every row lands in exactly one group");
    }
}
