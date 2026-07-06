//! Criterion micro-benches for `sparq-substrate`: the four join kernels and the
//! `Num` arithmetic tower.
//!
//! Enabled by `--features join,numeric` (which imply `rows`). The benches measure
//! throughput shapes; **no hard-coded performance numbers are expected** — the bench
//! records measurements to `target/criterion/` for local comparison with Criterion's
//! own baseline mechanism. Do not bake numbers into documentation or tests.
//!
//! # [SONNET-4.6] sq-qonbz.4 — epic sq-qonbz
//!
//! Surfaces covered:
//! - `join::merge_join` — sorted merge join on a single shared key column.
//! - `join::build_table` + `join::hash_probe_serial` — hash build and serial
//!   hash probe over the built `FxHashMap`.
//! - `numeric::Num::binop` — `+` / `*` arithmetic on the `Int` / `Dec` / `Double`
//!   tiers.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sparq_substrate::{
    join::{build_table, hash_probe_serial, merge_join, JoinKeys, NoBudget},
    numeric::{ArithOp, Dec, Num},
    rows::Row,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build two sorted `Row` slices of `n` rows each, joined on column 0 (every
/// key is unique, so the output is exactly `n` combined rows).
fn sorted_rows(n: u32) -> (Vec<Row>, Vec<Row>) {
    let left: Vec<Row> = (0..n)
        .map(|i| {
            let mut r = Row::new();
            r.extend_from_slice(&[i, i + 1000]);
            r
        })
        .collect();
    let right: Vec<Row> = (0..n)
        .map(|i| {
            let mut r = Row::new();
            r.extend_from_slice(&[i, i + 2000]);
            r
        })
        .collect();
    (left, right)
}

/// `JoinKeys` descriptor: single equi-join key on column 0; append right col 1.
fn key_col0() -> JoinKeys {
    JoinKeys { key_cols: vec![(0, 0)], right_only: vec![1] }
}

// ---------------------------------------------------------------------------
// Bench: merge join
// ---------------------------------------------------------------------------

fn bench_merge_join(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_join");
    for n in [100u32, 1_000, 10_000] {
        let (left, right) = sorted_rows(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            let mut out: Vec<Row> = Vec::with_capacity(n as usize);
            b.iter(|| {
                out.clear();
                // lk=0, rk=0, no extra shared columns, append right col 1
                merge_join(&left, 0, &right, 0, &[], &[1], &NoBudget, &mut out);
                assert_eq!(out.len(), n as usize);
            });
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Bench: hash build + probe
// ---------------------------------------------------------------------------

fn bench_hash_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_build");
    for n in [100u32, 1_000, 10_000] {
        let (left, _) = sorted_rows(n);
        let keys = key_col0();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                // [SONNET-4.6] black_box prevents LLVM from eliding the build entirely
                // when the result would otherwise be dropped without being observed.
                black_box(build_table(&left, &keys));
            });
        });
    }
    group.finish();
}

fn bench_hash_probe(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_probe");
    for n in [100u32, 1_000, 10_000] {
        let (left, right) = sorted_rows(n);
        let keys = key_col0();
        let table = build_table(&left, &keys);
        // hash_probe_serial takes &[FxHashMap<Key,Posting>]; wrap in a one-element slice.
        let tables = std::slice::from_ref(&table);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            let mut out: Vec<Row> = Vec::with_capacity(n as usize);
            b.iter(|| {
                out.clear();
                hash_probe_serial(&right, &keys, &left, tables, &[1], &NoBudget, &mut out);
                assert_eq!(out.len(), n as usize);
            });
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Bench: Num arithmetic
// ---------------------------------------------------------------------------

fn bench_num_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("num_arithmetic");

    // Integer addition: `Num::Int(i) + Num::Int(j)`
    {
        let vals: Vec<(Num, Num)> = (0i64..1_000)
            .map(|i| (Num::Int(i), Num::Int(i + 1)))
            .collect();
        group.throughput(Throughput::Elements(vals.len() as u64));
        group.bench_function("int_add", |b| {
            b.iter(|| {
                let mut acc: i64 = 0;
                for &(a, bv) in &vals {
                    if let Some(Num::Int(v)) = a.binop(bv, ArithOp::Add) {
                        acc = acc.wrapping_add(v);
                    }
                }
                acc
            });
        });
    }

    // Decimal multiplication: `Dec * Dec`
    {
        let vals: Vec<(Num, Num)> = (1i128..=1_000)
            .map(|i| {
                (
                    Num::Dec(Dec { mant: i, scale: 2 }),
                    Num::Dec(Dec { mant: i + 1, scale: 2 }),
                )
            })
            .collect();
        group.throughput(Throughput::Elements(vals.len() as u64));
        group.bench_function("dec_mul", |b| {
            b.iter(|| {
                let mut found = 0usize;
                for &(a, bv) in &vals {
                    if a.binop(bv, ArithOp::Mul).is_some() {
                        found += 1;
                    }
                }
                found
            });
        });
    }

    // Double addition: `f64 + f64` through the `Num` tower
    {
        let vals: Vec<(Num, Num)> = (0..1_000)
            .map(|i| (Num::Double(i as f64 * 0.1), Num::Double(i as f64 * 0.01)))
            .collect();
        group.throughput(Throughput::Elements(vals.len() as u64));
        group.bench_function("double_add", |b| {
            b.iter(|| {
                let mut acc = 0.0f64;
                for &(a, bv) in &vals {
                    if let Some(Num::Double(v)) = a.binop(bv, ArithOp::Add) {
                        acc += v;
                    }
                }
                acc
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

criterion_group!(join_benches, bench_merge_join, bench_hash_build, bench_hash_probe);
criterion_group!(numeric_benches, bench_num_arithmetic);
criterion_main!(join_benches, numeric_benches);
