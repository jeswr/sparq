//! Scalar CPU reference implementations of the GPU kernels.
//!
//! These are the *correctness oracles* for the tests and the single-thread
//! baselines for `examples/gpu_bench.rs` (the rayon-parallel baselines live in
//! the example — the library carries no thread-pool dependency). They are
//! deliberately the same tight scalar loops a sparq scan thread runs today.

use crate::{hash32, EMPTY_KEY};

/// Counts elements with `lo <= v < hi`.
pub fn filter_count_u32(col: &[u32], lo: u32, hi: u32) -> u64 {
    let mut c = 0u64;
    for &v in col {
        if v >= lo && v < hi {
            c += 1;
        }
    }
    c
}

/// Counts elements with `v > t` (IEEE: NaN compares false on either side).
pub fn filter_count_f64_gt(col: &[f64], t: f64) -> u64 {
    let mut c = 0u64;
    for &v in col {
        if v > t {
            c += 1;
        }
    }
    c
}

/// Builds the open-addressing (linear-probing) hash table both the CPU and GPU
/// probes walk: capacity = next power of two >= 2×keys (load factor <= 0.5),
/// `EMPTY_KEY` marks free slots, duplicates occupy separate slots (1:N join
/// semantics). Keys must not equal `EMPTY_KEY`.
pub fn build_hash_table(keys: &[u32], payloads: &[u32]) -> Vec<[u32; 2]> {
    assert_eq!(keys.len(), payloads.len());
    let cap = (keys.len() * 2).next_power_of_two().max(2);
    let mask = (cap - 1) as u32;
    let mut slots = vec![[EMPTY_KEY, 0u32]; cap];
    for (&k, &p) in keys.iter().zip(payloads) {
        assert_ne!(
            k, EMPTY_KEY,
            "EMPTY_KEY is reserved as the empty-slot sentinel"
        );
        let mut slot = (hash32(k) & mask) as usize;
        while slots[slot][0] != EMPTY_KEY {
            slot = (slot + 1) & mask as usize;
        }
        slots[slot] = [k, p];
    }
    slots
}

/// Probes every id against the table; returns (matches, sum of matched
/// payloads) — identical walk order and semantics to the WGSL kernel.
pub fn hash_probe(slots: &[[u32; 2]], probe: &[u32]) -> (u64, u64) {
    let mask = (slots.len() - 1) as u32;
    let mut matches = 0u64;
    let mut sum = 0u64;
    for &k in probe {
        let mut slot = (hash32(k) & mask) as usize;
        loop {
            let [key, payload] = slots[slot];
            if key == EMPTY_KEY {
                break;
            }
            if key == k {
                matches += 1;
                sum += u64::from(payload);
            }
            slot = (slot + 1) & mask as usize;
        }
    }
    (matches, sum)
}

/// COUNT + SUM GROUP BY; `keys[i] < groups`. Returns `(count, sum)` per group.
pub fn group_aggregate(keys: &[u32], vals: &[u32], groups: u32) -> Vec<(u64, u64)> {
    assert_eq!(keys.len(), vals.len());
    let mut out = vec![(0u64, 0u64); groups as usize];
    for (&k, &v) in keys.iter().zip(vals) {
        let e = &mut out[k as usize];
        e.0 += 1;
        e.1 += u64::from(v);
    }
    out
}
