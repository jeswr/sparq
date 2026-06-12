//! Throughput benches for the sparq-zk commitment pipeline (plan §2.2):
//! RDFC10 canonicalization, leaf encoding + Poseidon2 fold, and the
//! end-to-end per-graph commitment, at credential scale (the stage-1
//! target: tens-to-hundreds of triples per graph) plus one larger point.
//!
//! Two graph shapes:
//! - `iri`: every subject/object ground (no bnodes) — RDFC10's cheap path.
//! - `bnode`: a bnode chain with per-node attributes — exercises the
//!   canonical labeling (still non-pathological; the poison cases are the
//!   suite's, not a throughput target).

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use oxrdf::{BlankNode, Literal, NamedNode, NamedOrBlankNode, Term, Triple};
use sparq_zk::encode::salt_from_bytes;
use sparq_zk::{canon, commit, encode, poseidon2, Fr};

fn nn(s: &str) -> NamedNode {
    NamedNode::new(s).unwrap()
}

/// `n` ground triples: <http://ex/s{i}> <http://ex/p{i%7}> "v{i}".
fn iri_graph(n: usize) -> Vec<Triple> {
    (0..n)
        .map(|i| {
            Triple::new(
                NamedOrBlankNode::NamedNode(nn(&format!("http://ex/s{i}"))),
                nn(&format!("http://ex/p{}", i % 7)),
                Term::Literal(Literal::new_simple_literal(format!("v{i}"))),
            )
        })
        .collect()
}

/// `n` triples as a bnode chain with literal attributes: for even i,
/// _:b{k} <http://ex/next> _:b{k+1}; for odd i, _:b{k} <http://ex/v> "k".
fn bnode_graph(n: usize) -> Vec<Triple> {
    (0..n)
        .map(|i| {
            let k = i / 2;
            let s = NamedOrBlankNode::BlankNode(BlankNode::new(format!("b{k}")).unwrap());
            if i % 2 == 0 {
                Triple::new(
                    s,
                    nn("http://ex/next"),
                    Term::BlankNode(BlankNode::new(format!("b{}", k + 1)).unwrap()),
                )
            } else {
                Triple::new(
                    s,
                    nn("http://ex/v"),
                    Term::Literal(Literal::new_simple_literal(k.to_string())),
                )
            }
        })
        .collect()
}

fn bench_canon(c: &mut Criterion) {
    let mut g = c.benchmark_group("canon");
    for &n in &[64usize, 256, 1024] {
        for (shape, graph) in [("iri", iri_graph(n)), ("bnode", bnode_graph(n))] {
            g.throughput(Throughput::Elements(n as u64));
            g.bench_with_input(BenchmarkId::new(shape, n), &graph, |b, graph| {
                b.iter(|| canon::canonicalize_triples(graph).unwrap())
            });
        }
    }
    g.finish();
}

fn bench_commit(c: &mut Criterion) {
    let salt = salt_from_bytes(&[7u8; 32]);
    let mut g = c.benchmark_group("commit");
    // End-to-end: RDFC10 + leaf encoding + Poseidon2 fold.
    for &n in &[64usize, 256, 1024] {
        for (shape, graph) in [("iri", iri_graph(n)), ("bnode", bnode_graph(n))] {
            g.throughput(Throughput::Elements(n as u64));
            g.bench_with_input(BenchmarkId::new(shape, n), &graph, |b, graph| {
                b.iter(|| commit::commit_triples(graph, salt).unwrap())
            });
        }
    }
    // Encoding + fold only (canonical form precomputed): the recommit cost
    // when RDFC10 ran at ingest.
    for &n in &[64usize, 256, 1024] {
        let canonical = canon::canonicalize_triples(&iri_graph(n)).unwrap();
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(
            BenchmarkId::new("leaves+fold", n),
            &canonical.triples,
            |b, triples| {
                b.iter(|| {
                    let leaves: Vec<Fr> = triples
                        .iter()
                        .map(|t| encode::encode_triple(t, &salt).unwrap())
                        .collect();
                    poseidon2::hash(&leaves)
                })
            },
        );
    }
    g.finish();
}

fn bench_poseidon2(c: &mut Criterion) {
    let mut g = c.benchmark_group("poseidon2");
    let four = [Fr::from(11u64), Fr::from(22u64), Fr::from(33u64), Fr::from(44u64)];
    g.bench_function("permutation", |b| b.iter(|| poseidon2::permutation(&four)));
    let forty: Vec<Fr> = (1..=40u64).map(Fr::from).collect();
    g.throughput(Throughput::Elements(40));
    g.bench_function("hash40", |b| b.iter(|| poseidon2::hash(&forty)));
    g.finish();
}

criterion_group!(benches, bench_canon, bench_commit, bench_poseidon2);
criterion_main!(benches);
