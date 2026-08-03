//! Deletion-heavy / re-derivation-adversarial differential tests for incremental
//! maintenance — the FBF-grade retraction correctness invariant of bead `sq-6tykl.4`
//! (parent epic `sq-6tykl`; measurement half in `bench/reason-deletion`).
//!
//! The general incremental oracle tests (`incremental_prop`, `incremental_owl_prop`,
//! `incremental_n3_prop`) run *balanced* random insert/delete schedules. RDFox's FBF /
//! DRed / B-F and GraphDB's smooth-delete exist specifically for the two cases those
//! balanced schedules under-sample:
//!
//!   1. **Over-deletion + re-derivation** — a derived fact supported by *several
//!      independent* derivations. Removing a proper subset of its supports must NOT retract
//!      it (it is still derivable); removing the last support must. sparq takes the
//!      derivation-**counting** route (a bounded per-fact support count) rather than DRed's
//!      delete-everything-then-re-derive, so this is the load-bearing invariant that the
//!      count decrements to exactly the right multiplicity — never under-deleting (a
//!      dangling derived fact) nor over-deleting (a fact whose alternate support survives).
//!
//!   2. **High deletion ratios** — retracting a large fraction of the ABox in one batch,
//!      where under/over-derivation bugs compound.
//!
//! Every assertion is DIFFERENTIAL: after each mutation the incrementally-maintained
//! closure must be set-equal to a from-scratch re-materialization of the current base.

use rustc_hash::FxHashSet;
use sparq_core::dict::{Dict, Id};
use sparq_reason::{materialize_owl_rl, materialize_rdfs, MaterializedGraph, MaterializedOwlGraph};

/// Deterministic xorshift64* RNG — reproducible failures, no dev-dependency.
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

fn rdfs_oracle(dict: &mut Dict, base: &FxHashSet<[Id; 3]>) -> FxHashSet<[Id; 3]> {
    let mut v: Vec<[Id; 3]> = base.iter().copied().collect();
    materialize_rdfs(dict, &mut v);
    v.into_iter().collect()
}

fn owl_oracle(dict: &mut Dict, base: &FxHashSet<[Id; 3]>) -> FxHashSet<[Id; 3]> {
    let mut v: Vec<[Id; 3]> = base.iter().copied().collect();
    materialize_owl_rl(dict, &mut v);
    v.into_iter().collect()
}

fn iri(dict: &mut Dict, s: &str) -> Id {
    dict.intern_iri(s)
}

// ───────────────────────────────────────────────────────────────────────────────────────
// 1a. RDFS: a derived fact with TWO independent subclass supports.
// ───────────────────────────────────────────────────────────────────────────────────────

/// `x rdf:type Super` is derivable both via `x rdf:type A` (A ⊑ Super) and via
/// `x rdf:type B` (B ⊑ Super). Deleting one support must leave it in the closure (still
/// re-derivable through the other); deleting the second must retract it. Checked against a
/// from-scratch re-materialization at every step.
#[test]
fn rdfs_multi_support_survives_partial_retraction() {
    let mut dict = Dict::default();
    let ty = iri(&mut dict, "http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
    let sc = iri(&mut dict, "http://www.w3.org/2000/01/rdf-schema#subClassOf");
    let a = iri(&mut dict, "http://ex/A");
    let b = iri(&mut dict, "http://ex/B");
    let super_c = iri(&mut dict, "http://ex/Super");
    let x = iri(&mut dict, "http://ex/x");

    let mut base: FxHashSet<[Id; 3]> = [[a, sc, super_c], [b, sc, super_c], [x, ty, a], [x, ty, b]]
        .into_iter()
        .collect();

    let mut g = MaterializedGraph::new(&mut dict, &base.iter().copied().collect::<Vec<_>>());
    assert!(
        g.contains(&[x, ty, super_c]),
        "Super membership must be derived"
    );
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        rdfs_oracle(&mut dict, &base)
    );

    // Retract the first support: still derivable via B ⊑ Super.
    g.delete(&[[x, ty, a]]);
    base.remove(&[x, ty, a]);
    assert!(
        g.contains(&[x, ty, super_c]),
        "one surviving support must keep the derived type (over-deletion guard)"
    );
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        rdfs_oracle(&mut dict, &base)
    );
    assert_eq!(g.full_rebuilds(), 0, "ABox retraction must not rebuild");

    // Retract the last support: now unsupported, must leave the closure.
    g.delete(&[[x, ty, b]]);
    base.remove(&[x, ty, b]);
    assert!(
        !g.contains(&[x, ty, super_c]),
        "removing the final support must retract the derived type (under-deletion guard)"
    );
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        rdfs_oracle(&mut dict, &base)
    );
    assert_eq!(g.full_rebuilds(), 0, "ABox retraction must not rebuild");
}

// ───────────────────────────────────────────────────────────────────────────────────────
// 1b. OWL 2 RL fixpoint: a transitive edge derivable via TWO independent paths.
// ───────────────────────────────────────────────────────────────────────────────────────

/// `near` is transitive. Two disjoint 2-hop paths x⇝y⇝z and x⇝w⇝z both derive `x near z`.
/// Deleting one intermediate edge must leave `x near z` (the other path survives); deleting
/// the second must retract it. This drives the recursive-layer retraction (`recompute_layer`)
/// — the closest sparq analogue of the DRed over-deletion scenario. Oracle-checked each step.
#[test]
fn owl_transitive_multipath_retraction_matches_from_scratch() {
    let mut dict = Dict::default();
    let ty = iri(&mut dict, "http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
    let transitive = iri(
        &mut dict,
        "http://www.w3.org/2002/07/owl#TransitiveProperty",
    );
    let near = iri(&mut dict, "http://ex/near");
    let x = iri(&mut dict, "http://ex/x");
    let y = iri(&mut dict, "http://ex/y");
    let z = iri(&mut dict, "http://ex/z");
    let w = iri(&mut dict, "http://ex/w");

    let mut base: FxHashSet<[Id; 3]> = [
        [near, ty, transitive],
        [x, near, y],
        [y, near, z],
        [x, near, w],
        [w, near, z],
    ]
    .into_iter()
    .collect();

    let mut g = MaterializedOwlGraph::new(&mut dict, &base.iter().copied().collect::<Vec<_>>());
    assert!(
        g.contains(&[x, near, z]),
        "transitive edge must be derived via both paths"
    );
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        owl_oracle(&mut dict, &base)
    );

    // Drop the first path's intermediate edge: x near z survives via x⇝w⇝z.
    g.delete(&mut dict, &[[x, near, y]]);
    base.remove(&[x, near, y]);
    assert!(
        g.contains(&[x, near, z]),
        "alternate transitive path must keep the derived edge (over-deletion guard)"
    );
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        owl_oracle(&mut dict, &base)
    );

    // Drop the second path too: no support remains, edge must leave.
    g.delete(&mut dict, &[[x, near, w]]);
    base.remove(&[x, near, w]);
    assert!(
        !g.contains(&[x, near, z]),
        "removing the last transitive path must retract the derived edge (under-deletion guard)"
    );
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        owl_oracle(&mut dict, &base)
    );
}

// ───────────────────────────────────────────────────────────────────────────────────────
// 2. High deletion-ratio stratified schedule (RDFS).
// ───────────────────────────────────────────────────────────────────────────────────────

/// An instance-heavy RDFS ontology: the fixed TBox plus the vocabulary the schedule draws
/// random ABox triples from.
struct RdfsWorld {
    ty: Id,
    classes: Vec<Id>,
    props: Vec<Id>,
    individuals: Vec<Id>,
    tbox: Vec<[Id; 3]>,
}

/// Build an instance-heavy RDFS ontology (subclass + subproperty + domain/range structure)
/// and the vocabulary a schedule draws ABox triples from.
fn build_rdfs(dict: &mut Dict, rng: &mut Rng) -> RdfsWorld {
    let ty = iri(dict, "http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
    let sc = iri(dict, "http://www.w3.org/2000/01/rdf-schema#subClassOf");
    let sp = iri(dict, "http://www.w3.org/2000/01/rdf-schema#subPropertyOf");
    let dom = iri(dict, "http://www.w3.org/2000/01/rdf-schema#domain");
    let rng_p = iri(dict, "http://www.w3.org/2000/01/rdf-schema#range");

    let mut base: Vec<[Id; 3]> = Vec::new();
    let mut classes = Vec::new();
    for i in 0..4 {
        let chain: Vec<Id> = (0..5)
            .map(|j| iri(dict, &format!("http://ex/C{i}_{j}")))
            .collect();
        for pair in chain.windows(2) {
            base.push([pair[0], sc, pair[1]]);
        }
        classes.extend(chain);
    }
    let mut props = Vec::new();
    for i in 0..3 {
        let chain: Vec<Id> = (0..3)
            .map(|j| iri(dict, &format!("http://ex/p{i}_{j}")))
            .collect();
        for pair in chain.windows(2) {
            base.push([pair[0], sp, pair[1]]);
        }
        for &p in &chain {
            base.push([p, dom, classes[rng.below(classes.len())]]);
            base.push([p, rng_p, classes[rng.below(classes.len())]]);
        }
        props.extend(chain);
    }
    let individuals: Vec<Id> = (0..400)
        .map(|i| iri(dict, &format!("http://ex/ind{i}")))
        .collect();
    RdfsWorld {
        ty,
        classes,
        props,
        individuals,
        tbox: base,
    }
}

fn random_abox(ty: Id, classes: &[Id], props: &[Id], individuals: &[Id], rng: &mut Rng) -> [Id; 3] {
    let s = individuals[rng.below(individuals.len())];
    if rng.below(3) == 0 {
        [s, ty, classes[rng.below(classes.len())]]
    } else {
        [
            s,
            props[rng.below(props.len())],
            individuals[rng.below(individuals.len())],
        ]
    }
}

/// Retract a large, escalating fraction of the current ABox each batch (25% → 50% → 75% →
/// 90% → 95%), with only a trickle of re-inserts, so the schedule is dominated by deletion.
/// After every batch the incremental closure must equal the from-scratch closure and the
/// ABox must never have taken the rebuild path.
#[test]
fn deletion_heavy_stratified_ratios_match_from_scratch() {
    let mut dict = Dict::default();
    let mut rng = Rng(0x5EED_D001);
    let RdfsWorld {
        ty,
        classes,
        props,
        individuals,
        tbox,
    } = build_rdfs(&mut dict, &mut rng);

    // Seed a large ABox pool (the TBox is fixed for the whole schedule).
    let mut abox: FxHashSet<[Id; 3]> = FxHashSet::default();
    while abox.len() < 1200 {
        abox.insert(random_abox(ty, &classes, &props, &individuals, &mut rng));
    }
    let mut base: FxHashSet<[Id; 3]> = tbox.iter().copied().collect();
    base.extend(abox.iter().copied());

    let mut g = MaterializedGraph::new(&mut dict, &base.iter().copied().collect::<Vec<_>>());
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        rdfs_oracle(&mut dict, &base),
        "initial closure disagrees with from-scratch"
    );

    for (batch, &ratio_pct) in [25usize, 50, 75, 90, 95].iter().enumerate() {
        // Delete `ratio_pct`% of the CURRENT ABox (rounded down), sampled without replacement.
        let abox_vec: Vec<[Id; 3]> = abox.iter().copied().collect();
        let want = abox_vec.len() * ratio_pct / 100;
        let mut del: Vec<[Id; 3]> = Vec::with_capacity(want);
        let mut seen: FxHashSet<[Id; 3]> = FxHashSet::default();
        while del.len() < want && seen.len() < abox_vec.len() {
            let t = abox_vec[rng.below(abox_vec.len())];
            if seen.insert(t) {
                del.push(t);
            }
        }
        g.delete(&del);
        for t in &del {
            abox.remove(t);
            base.remove(t);
        }

        // A small trickle of fresh inserts so the pool never fully drains.
        let mut ins: Vec<[Id; 3]> = Vec::new();
        for _ in 0..20 {
            let t = random_abox(ty, &classes, &props, &individuals, &mut rng);
            if base.insert(t) {
                abox.insert(t);
                ins.push(t);
            }
        }
        g.insert(&ins);

        assert_eq!(
            g.closure().into_iter().collect::<FxHashSet<_>>(),
            rdfs_oracle(&mut dict, &base),
            "closure diverged after {ratio_pct}% deletion (batch {batch})"
        );
        assert_eq!(g.base_len(), base.len(), "base drifted (batch {batch})");
        assert_eq!(
            g.full_rebuilds(),
            0,
            "pure-ABox deletion must never rebuild (batch {batch})"
        );
    }
}
