//! Trace-overhead benches (plan §4.E, module-B deliverable 2): the cost of
//! the zk-trace recorder, traced (armed) vs untraced (default execution),
//! across plan shapes on a deterministic synthetic social graph.
//!
//! The honest measurement the brief asks for: per-operator capture is NOT
//! free when armed (it materializes the consumed input set — ids deduped,
//! terms once per unique triple). These benches quantify that, and the "off"
//! arm confirms the disarmed cost is the one-thread-local-read floor.
//!
//! [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sparq_core::Graph;
use sparq_engine::{query, zk};

/// Deterministic synthetic social graph (WatDiv-flavoured): each of `n`
/// persons has type/name/age/city plus `follows` edges — star joins, chains,
/// triangles. Mirrors sparq-bench's generator so the shapes are comparable.
fn generate(n: u32) -> String {
    let n = n.max(1);
    let follows_per = 4u32;
    let n_cities = (n / 10).max(1);
    let mut s = String::with_capacity(n as usize * 9 * 40);
    s.push_str("@prefix ex: <http://ex/> .\n");
    let mut seed = 0x9E3779B97F4A7C15u64;
    let mut rng = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as u32
    };
    for i in 0..n {
        s.push_str(&format!(
            "ex:n{i} a ex:Person ;\n  ex:name \"name{i}\" ;\n  ex:age {} ;\n  ex:city ex:c{} .\n",
            20 + i % 80,
            i % n_cities
        ));
        for _ in 0..follows_per {
            s.push_str(&format!("ex:n{i} ex:follows ex:n{} .\n", rng() % n));
        }
    }
    s
}

/// The query workloads (every plan shape the trace covers).
const QUERIES: &[(&str, &str)] = &[
    ("bgp-star", "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:age ?a . ?s ex:name ?n . ?s ex:city ?c }"),
    ("bgp-chain", "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:follows ?o . ?o ex:age ?a }"),
    ("bgp-triangle", "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:follows ?o . ?o ex:follows ?t . ?t ex:follows ?s }"),
    ("filter", "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:age ?a . ?s ex:name ?n FILTER(?a > 50) }"),
    ("optional", "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:name ?n OPTIONAL { ?s ex:follows ?o } }"),
    ("union", "PREFIX ex: <http://ex/> SELECT * WHERE { { ?s ex:age ?a } UNION { ?s ex:name ?a } }"),
    ("distinct", "PREFIX ex: <http://ex/> SELECT DISTINCT ?c WHERE { ?s ex:city ?c }"),
    ("count", "PREFIX ex: <http://ex/> SELECT (COUNT(?s) AS ?c) WHERE { ?s ex:age ?a }"),
];

fn bench_trace_overhead(c: &mut Criterion) {
    // Credential-to-mid scale: tens to a few thousand entities (the stage-1
    // target plus a larger point). 1k entities ~= 8k triples.
    for &n in &[100u32, 1000] {
        let g = Graph::load_str(&generate(n), "turtle").unwrap();
        let mut grp = c.benchmark_group(format!("trace/{n}entities"));
        for (name, q) in QUERIES {
            // Untraced: default execution (no recorder).
            grp.bench_with_input(BenchmarkId::new(format!("{name}/untraced"), n), q, |b, q| {
                b.iter(|| query(&g, q).unwrap());
            });
            // Traced: recorder armed; drained each iter (the proving path).
            grp.bench_with_input(BenchmarkId::new(format!("{name}/traced"), n), q, |b, q| {
                b.iter(|| {
                    let _guard = zk::install();
                    let r = query(&g, q).unwrap();
                    let t = zk::take();
                    (r, t)
                });
            });
        }
        grp.finish();
    }
}

criterion_group!(benches, bench_trace_overhead);
criterion_main!(benches);
