//! RESEARCH SPIKE — what does RE-INTRODUCING a dropped permutation cost?
//!
//! Two paths, both measured against the SPO permutation of a saved raw dir:
//!  1. REBUILD: stream SPO rows (mmap), permute the columns, parallel-sort.
//!     (What a store must do if a dropped perm was never persisted.)
//!  2. RELOAD: sequential read of the persisted perm file from disk into RAM.
//!     (What a store does if the perm file was kept on disk — the cheap path.)
//!
//! usage: rebuild <raw-index-dir> [iters]

use rayon::prelude::*;
use sparq_core::dict::Id;
use std::time::Instant;

fn read_rows(path: &std::path::Path) -> Vec<[Id; 3]> {
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(bytes.len() % 12, 0);
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::PathBuf::from(args.get(1).expect("usage: rebuild <raw-index-dir> [iters]"));
    let iters: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);

    // Orders: canonical [s,p,o] -> stored row for each of the six permutations.
    let orders: [(usize, [usize; 3]); 3] = [
        (1, [0, 2, 1]), // SOP
        (2, [1, 0, 2]), // PSO
        (5, [2, 1, 0]), // OPS
    ];
    let names = ["SPO", "SOP", "PSO", "POS", "OSP", "OPS"];

    // Source: SPO rows, loaded once (charged separately — a live store already has SPO).
    let t = Instant::now();
    let spo = read_rows(&dir.join("perm0.bin"));
    println!("spo_read_s\t{:.3}\trows\t{}", t.elapsed().as_secs_f64(), spo.len());

    for (idx, order) in orders {
        let mut best_permute = f64::INFINITY;
        let mut best_sort = f64::INFINITY;
        for _ in 0..iters {
            let t = Instant::now();
            let mut v: Vec<[Id; 3]> = spo.par_iter().map(|t| [t[order[0]], t[order[1]], t[order[2]]]).collect();
            let permute_s = t.elapsed().as_secs_f64();
            let t = Instant::now();
            v.par_sort_unstable();
            let sort_s = t.elapsed().as_secs_f64();
            best_permute = best_permute.min(permute_s);
            best_sort = best_sort.min(sort_s);
            std::hint::black_box(&v);
        }
        println!(
            "rebuild\t{}\tpermute_s\t{best_permute:.3}\tsort_s\t{best_sort:.3}\ttotal_s\t{:.3}",
            names[idx],
            best_permute + best_sort
        );
    }

    // RELOAD path: sequential read of a persisted perm file (use perm1 SOP — one of the
    // wasm-dropped set). Includes the byte->row copy a real reload would do.
    let mut best = f64::INFINITY;
    let mut first = f64::NAN;
    for i in 0..iters {
        let t = Instant::now();
        let v = read_rows(&dir.join("perm1.bin"));
        let s = t.elapsed().as_secs_f64();
        if i == 0 {
            first = s;
        }
        best = best.min(s);
        std::hint::black_box(&v);
    }
    // NOTE: page-cache warm after (or even before) the first read; cold adds one
    // sequential disk read of the file.
    println!("reload_from_disk\tSOP\tfirst_s\t{first:.3}\tbest_s\t{best:.3}");
}
