//! T18 headline measurement: incremental RDFS closure maintenance vs full re-materialization.
//!
//! Builds a 200k-individual ontology (25 subClassOf chains of depth 8, 10 subPropertyOf
//! chains of depth 3 with domain/range, ~600k base triples), materializes the closure, then
//! measures — interleaved, best-of-3 — (a) inserting 100 ABox triples and (b) deleting 100
//! ABox triples: `MaterializedGraph::insert/delete` vs from-scratch `materialize_rdfs` on the
//! same mutated base. Each round also asserts the incremental closure size matches the
//! from-scratch one (the oracle rides along with the benchmark).
//!
//! Run: `cargo run -p sparq-reason --example incremental_bench --release`

use std::time::{Duration, Instant};

use sparq_core::dict::{Dict, Id};
use sparq_reason::{materialize_rdfs, MaterializedGraph};

/// Deterministic xorshift64* RNG.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const N_INDIVIDUALS: usize = 200_000;
const BATCH: usize = 100;

fn main() {
    use oxrdf::vocab::{rdf, rdfs};
    let mut rng = Rng(0x5EED_2026_0610_0001);
    let mut dict = Dict::new();
    let ty = dict.intern_iri(rdf::TYPE.as_str());
    let sc = dict.intern_iri(rdfs::SUB_CLASS_OF.as_str());
    let sp = dict.intern_iri(rdfs::SUB_PROPERTY_OF.as_str());
    let dom = dict.intern_iri(rdfs::DOMAIN.as_str());
    let rng_p = dict.intern_iri(rdfs::RANGE.as_str());

    // TBox: 25 class chains of depth 8; 10 property chains of depth 3 with domain+range.
    let mut base: Vec<[Id; 3]> = Vec::new();
    let mut classes: Vec<Id> = Vec::new();
    for i in 0..25 {
        let chain: Vec<Id> = (0..8)
            .map(|j| dict.intern_iri(&format!("http://ex/C{i}_{j}")))
            .collect();
        for w in chain.windows(2) {
            base.push([w[0], sc, w[1]]);
        }
        classes.extend(chain);
    }
    let mut props: Vec<Id> = Vec::new();
    for i in 0..10 {
        let chain: Vec<Id> = (0..3)
            .map(|j| dict.intern_iri(&format!("http://ex/p{i}_{j}")))
            .collect();
        for w in chain.windows(2) {
            base.push([w[0], sp, w[1]]);
        }
        for &p in &chain {
            base.push([p, dom, classes[rng.below(classes.len())]]);
            base.push([p, rng_p, classes[rng.below(classes.len())]]);
        }
        props.extend(chain);
    }
    // ABox: 200k individuals, each with one type + two property assertions (~600k triples).
    let individuals: Vec<Id> = (0..N_INDIVIDUALS)
        .map(|i| dict.intern_iri(&format!("http://ex/ind{i}")))
        .collect();
    for &s in &individuals {
        base.push([s, ty, classes[rng.below(classes.len())]]);
        for _ in 0..2 {
            let p = props[rng.below(props.len())];
            base.push([s, p, individuals[rng.below(individuals.len())]]);
        }
    }
    println!(
        "ontology: {} individuals, {} base triples ({} classes, {} properties)",
        N_INDIVIDUALS,
        base.len(),
        classes.len(),
        props.len()
    );

    // Initial materialization.
    let t0 = Instant::now();
    let mut g = MaterializedGraph::new(&mut dict, &base);
    let init = t0.elapsed();
    println!(
        "initial materialization: {} closure triples in {init:?}",
        g.len()
    );

    let random_abox = |rng: &mut Rng| -> [Id; 3] {
        let s = individuals[rng.below(individuals.len())];
        if rng.below(3) == 0 {
            [s, ty, classes[rng.below(classes.len())]]
        } else {
            let p = props[rng.below(props.len())];
            [s, p, individuals[rng.below(individuals.len())]]
        }
    };

    // -------- (a) INSERT 100 ABox triples: incremental vs full, interleaved best-of-3 ------
    let mut best_inc_ins = Duration::MAX;
    let mut best_full_ins = Duration::MAX;
    for round in 0..3 {
        // 100 fresh, currently-absent ABox triples (deduplicated so undo restores the base).
        let mut delta: Vec<[Id; 3]> = Vec::with_capacity(BATCH);
        let mut seen = std::collections::HashSet::new();
        while delta.len() < BATCH {
            let t = random_abox(&mut rng);
            if !g.contains(&t) && seen.insert(t) {
                delta.push(t);
            }
        }

        // Incremental insert.
        let t = Instant::now();
        g.insert(&delta);
        let inc = t.elapsed();
        best_inc_ins = best_inc_ins.min(inc);

        // Full re-materialization on the same mutated base — interleaved with the above.
        let mut full_base: Vec<[Id; 3]> = g.base_triples().collect();
        let t = Instant::now();
        materialize_rdfs(&mut dict, &mut full_base);
        let full = t.elapsed();
        best_full_ins = best_full_ins.min(full);

        assert_eq!(
            g.len(),
            full_base.len(),
            "closure size mismatch after insert round {round}"
        );
        g.delete(&delta); // undo: every round starts from the same base
    }
    println!(
        "insert {BATCH} ABox triples:  incremental {best_inc_ins:?}  vs  full {best_full_ins:?}  ({:.0}x faster)",
        best_full_ins.as_secs_f64() / best_inc_ins.as_secs_f64().max(1e-9)
    );

    // -------- (b) DELETE 100 ABox triples: incremental vs full, interleaved best-of-3 ------
    let abox: Vec<[Id; 3]> = g
        .base_triples()
        .filter(|t| t[1] != sc && t[1] != sp && t[1] != dom && t[1] != rng_p)
        .collect();
    let mut best_inc_del = Duration::MAX;
    let mut best_full_del = Duration::MAX;
    for round in 0..3 {
        let mut delta: Vec<[Id; 3]> = Vec::with_capacity(BATCH);
        let mut seen = std::collections::HashSet::new();
        while delta.len() < BATCH {
            let t = abox[rng.below(abox.len())];
            if seen.insert(t) {
                delta.push(t);
            }
        }

        // Incremental delete.
        let t = Instant::now();
        g.delete(&delta);
        let inc = t.elapsed();
        best_inc_del = best_inc_del.min(inc);

        // Full re-materialization on the same mutated base.
        let mut full_base: Vec<[Id; 3]> = g.base_triples().collect();
        let t = Instant::now();
        materialize_rdfs(&mut dict, &mut full_base);
        let full = t.elapsed();
        best_full_del = best_full_del.min(full);

        assert_eq!(
            g.len(),
            full_base.len(),
            "closure size mismatch after delete round {round}"
        );
        g.insert(&delta); // restore
    }
    println!(
        "delete {BATCH} ABox triples:  incremental {best_inc_del:?}  vs  full {best_full_del:?}  ({:.0}x faster)",
        best_full_del.as_secs_f64() / best_inc_del.as_secs_f64().max(1e-9)
    );
    assert_eq!(
        g.full_rebuilds(),
        0,
        "ABox-only benchmark must never trigger the TBox fallback"
    );
}
