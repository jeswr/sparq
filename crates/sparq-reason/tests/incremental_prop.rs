//! Property test for incremental RDFS maintenance (T18): after EVERY random mutation batch,
//! the incrementally maintained closure must equal the from-scratch batch closure
//! (`materialize_rdfs`) of the current base — the correctness oracle.
//!
//! The ontology is instance-heavy with real TBox structure: several `subClassOf` chains
//! (depth 6), `subPropertyOf` chains (depth 3) with `domain`/`range` declarations pointing
//! into the class chains, and hundreds of individuals with random types and property
//! assertions. Mutation batches are mostly ABox inserts/deletes (the incremental path),
//! with occasional TBox mutations to exercise the v1 full-rematerialization fallback.

use rustc_hash::FxHashSet;
use sparq_core::dict::{Dict, Id};
use sparq_reason::{materialize_rdfs, MaterializedGraph};

/// Deterministic xorshift64* RNG — no dev-dependency needed, reproducible failures.
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

struct Onto {
    ty: Id,
    sc: Id,
    sp: Id,
    dom: Id,
    rng: Id,
    classes: Vec<Id>, // chained via subClassOf
    props: Vec<Id>,   // chained via subPropertyOf, with domains/ranges
    individuals: Vec<Id>,
}

impl Onto {
    fn is_tbox(&self, t: &[Id; 3]) -> bool {
        t[1] == self.sc || t[1] == self.sp || t[1] == self.dom || t[1] == self.rng
    }
}

/// Build the TBox + an instance-heavy ABox; returns the ontology handles and the base triples.
fn build(dict: &mut Dict, rng: &mut Rng) -> (Onto, Vec<[Id; 3]>) {
    use oxrdf::vocab::{rdf, rdfs};
    let ty = dict.intern_iri(rdf::TYPE.as_str());
    let sc = dict.intern_iri(rdfs::SUB_CLASS_OF.as_str());
    let sp = dict.intern_iri(rdfs::SUB_PROPERTY_OF.as_str());
    let dom = dict.intern_iri(rdfs::DOMAIN.as_str());
    let rng_p = dict.intern_iri(rdfs::RANGE.as_str());

    let mut base: Vec<[Id; 3]> = Vec::new();
    // 5 class chains of depth 6: C{i}_0 ⊑ C{i}_1 ⊑ … ⊑ C{i}_5.
    let mut classes = Vec::new();
    for i in 0..5 {
        let chain: Vec<Id> = (0..6)
            .map(|j| dict.intern_iri(&format!("http://ex/C{i}_{j}")))
            .collect();
        for w in chain.windows(2) {
            base.push([w[0], sc, w[1]]);
        }
        classes.extend(chain);
    }
    // 4 property chains of depth 3, each property with a random domain and range class.
    let mut props = Vec::new();
    for i in 0..4 {
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
    // 300 individuals: each gets a random type and two random property assertions.
    let individuals: Vec<Id> = (0..300)
        .map(|i| dict.intern_iri(&format!("http://ex/ind{i}")))
        .collect();
    for &s in &individuals {
        base.push([s, ty, classes[rng.below(classes.len())]]);
        for _ in 0..2 {
            let p = props[rng.below(props.len())];
            let o = individuals[rng.below(individuals.len())];
            base.push([s, p, o]);
        }
    }
    (
        Onto {
            ty,
            sc,
            sp,
            dom,
            rng: rng_p,
            classes,
            props,
            individuals,
        },
        base,
    )
}

/// A random ABox triple: a type assertion or a property assertion between individuals.
fn random_abox(o: &Onto, rng: &mut Rng) -> [Id; 3] {
    let s = o.individuals[rng.below(o.individuals.len())];
    if rng.below(3) == 0 {
        [s, o.ty, o.classes[rng.below(o.classes.len())]]
    } else {
        let p = o.props[rng.below(o.props.len())];
        [s, p, o.individuals[rng.below(o.individuals.len())]]
    }
}

#[test]
fn incremental_matches_from_scratch_over_random_mutations() {
    let mut rng = Rng(0x5EED_CAFE_F00D_2026);
    let mut dict = Dict::new();
    let (onto, base) = build(&mut dict, &mut rng);

    let mut g = MaterializedGraph::new(&mut dict, &base);
    let mut mirror: FxHashSet<[Id; 3]> = base.into_iter().collect();

    // Initial state must already match.
    let oracle = |dict: &mut Dict, mirror: &FxHashSet<[Id; 3]>| -> FxHashSet<[Id; 3]> {
        let mut v: Vec<[Id; 3]> = mirror.iter().copied().collect();
        materialize_rdfs(dict, &mut v);
        v.into_iter().collect()
    };
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        oracle(&mut dict, &mirror),
        "initial materialization disagrees with batch"
    );

    let mut tbox_batches = 0usize;
    for batch in 0..120 {
        let op = rng.below(100);
        if op < 8 {
            // TBox mutation (~8% of batches): add or remove a subClassOf edge between random
            // classes — exercises the full-rematerialization fallback.
            tbox_batches += 1;
            let a = onto.classes[rng.below(onto.classes.len())];
            let b = onto.classes[rng.below(onto.classes.len())];
            let t = [a, onto.sc, b];
            let rebuilds_before = g.full_rebuilds();
            if mirror.contains(&t) {
                g.delete(&[t]);
                mirror.remove(&t);
            } else {
                g.insert(&[t]);
                mirror.insert(t);
            }
            assert_eq!(
                g.full_rebuilds(),
                rebuilds_before + 1,
                "TBox mutation must trigger exactly one full rebuild (batch {batch})"
            );
        } else if op < 54 {
            // ABox insert batch: 1-10 random triples (some may already be present).
            let n = 1 + rng.below(10);
            let delta: Vec<[Id; 3]> = (0..n).map(|_| random_abox(&onto, &mut rng)).collect();
            g.insert(&delta);
            mirror.extend(delta);
        } else {
            // ABox delete batch: 1-10 triples, mixing currently-asserted ABox triples (drawn
            // from the mirror) with random ones (often absent or derived-only — no-ops).
            let current: Vec<[Id; 3]> = mirror
                .iter()
                .copied()
                .filter(|t| !onto.is_tbox(t))
                .collect();
            let n = 1 + rng.below(10);
            let delta: Vec<[Id; 3]> = (0..n)
                .map(|_| {
                    if rng.below(4) == 0 || current.is_empty() {
                        random_abox(&onto, &mut rng)
                    } else {
                        current[rng.below(current.len())]
                    }
                })
                .collect();
            g.delete(&delta);
            for t in &delta {
                mirror.remove(t);
            }
        }

        // THE ORACLE: incrementally maintained closure == from-scratch closure, every batch.
        let inc: FxHashSet<[Id; 3]> = g.closure().into_iter().collect();
        let full = oracle(&mut dict, &mirror);
        assert_eq!(
            inc.len(),
            full.len(),
            "closure size diverged at batch {batch} (inc {} vs full {})",
            inc.len(),
            full.len()
        );
        assert_eq!(inc, full, "closure content diverged at batch {batch}");
        assert_eq!(g.base_len(), mirror.len(), "base drifted at batch {batch}");
    }
    assert!(
        tbox_batches > 0,
        "schedule should have exercised the TBox fallback"
    );
    assert!(
        g.full_rebuilds() >= tbox_batches,
        "every TBox batch rebuilds (got {} rebuilds for {tbox_batches} TBox batches)",
        g.full_rebuilds()
    );
}
