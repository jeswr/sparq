//! `why()` on [`MaterializedGraph`] (RDFS, `explain` feature): hand-checked proof trees,
//! a differential sweep (every derived triple of a randomized materialization gets a proof
//! that the independent naive checker accepts AND whose asserted leaves alone re-entail
//! the conclusion), and incremental-retraction consistency.
#![cfg(feature = "explain")]

mod explain_common;

use explain_common::{check_proof, proof_leaves};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};
use sparq_reason::{materialize_rdfs, MaterializedGraph};

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

fn ids(dict: &mut Dict) -> (Id, Id, Id, Id, Id) {
    use oxrdf::vocab::{rdf, rdfs};
    (
        dict.intern_iri(rdf::TYPE.as_str()),
        dict.intern_iri(rdfs::SUB_CLASS_OF.as_str()),
        dict.intern_iri(rdfs::SUB_PROPERTY_OF.as_str()),
        dict.intern_iri(rdfs::DOMAIN.as_str()),
        dict.intern_iri(rdfs::RANGE.as_str()),
    )
}

fn ex(dict: &mut Dict, l: &str) -> Id {
    dict.intern_iri(&format!("http://ex/{l}"))
}

fn render(dict: &Dict, t: [Id; 3]) -> [String; 3] {
    [
        dict.term(t[0]).to_string(),
        dict.term(t[1]).to_string(),
        dict.term(t[2]).to_string(),
    ]
}

fn rendered_base(dict: &Dict, g: &MaterializedGraph) -> FxHashSet<[String; 3]> {
    g.base_triples().map(|t| render(dict, t)).collect()
}

/// Strong validity oracle: materializing ONLY the proof's asserted leaves must re-derive
/// the conclusion (the proof really is self-contained evidence).
fn leaves_entail(
    dict: &mut Dict,
    g: &MaterializedGraph,
    tree: &sparq_reason::ProofTree,
    t: [Id; 3],
) {
    let rendered: FxHashMap<[String; 3], [Id; 3]> =
        g.base_triples().map(|b| (render(dict, b), b)).collect();
    let mut leaves: Vec<[Id; 3]> = proof_leaves(tree)
        .iter()
        .map(|l| *rendered.get(l).expect("leaf not in base"))
        .collect();
    materialize_rdfs(dict, &mut leaves);
    assert!(
        leaves.contains(&t),
        "proof leaves do not re-entail the conclusion"
    );
}

#[test]
fn subclass_chain_hand_checked() {
    let mut dict = Dict::new();
    let (ty, sc, _, _, _) = ids(&mut dict);
    let (dog, mammal, animal, rex) = (
        ex(&mut dict, "Dog"),
        ex(&mut dict, "Mammal"),
        ex(&mut dict, "Animal"),
        ex(&mut dict, "rex"),
    );
    let g = MaterializedGraph::new(
        &mut dict,
        &[[dog, sc, mammal], [mammal, sc, animal], [rex, ty, dog]],
    );

    // rex type Animal: rdfs9 over the rdfs11-closed chain.
    let t = [rex, ty, animal];
    assert!(g.contains(&t));
    let tree = g.why(&dict, t).expect("derived triple must have a proof");
    check_proof(&tree, &rendered_base(&dict, &g), Some(&render(&dict, t)));
    leaves_entail(&mut dict, &g, &tree, t);
    // Exact shape: rdfs9( rdfs11( asserted, asserted ), asserted ).
    let root = &tree.nodes()[tree.root() as usize];
    assert_eq!(root.rule, "rdfs9");
    let schema = &tree.nodes()[root.premises[0] as usize];
    assert_eq!(schema.rule, "rdfs11");
    assert_eq!(schema.conclusion, render(&dict, [dog, sc, animal]));
    let data = &tree.nodes()[root.premises[1] as usize];
    assert_eq!(data.rule, "asserted");
    assert_eq!(data.conclusion, render(&dict, [rex, ty, dog]));

    // The closed schema fact itself.
    let t2 = [dog, sc, animal];
    let tree2 = g.why(&dict, t2).expect("schema fact must have a proof");
    check_proof(&tree2, &rendered_base(&dict, &g), Some(&render(&dict, t2)));
    assert_eq!(tree2.nodes()[tree2.root() as usize].rule, "rdfs11");

    // Asserted triples explain as leaves; absent triples have no proof.
    let tree3 = g.why(&dict, [rex, ty, dog]).unwrap();
    assert_eq!(tree3.nodes().len(), 1);
    assert_eq!(tree3.nodes()[0].rule, "asserted");
    assert!(g.why(&dict, [animal, ty, rex]).is_none());

    // Renderings are well-formed.
    assert!(tree.to_json().starts_with("{\"root\":"));
    assert!(tree.to_text().contains("[rdfs9]"));
}

#[test]
fn domain_range_subproperty_hand_checked() {
    // p sp q; q domain C; q range D; C sc E; a p b — the flattened dom_full emission must
    // decompose into rdfs7 → rdfs2 → rdfs9 single steps.
    let mut dict = Dict::new();
    let (ty, sc, sp, dom, rng) = ids(&mut dict);
    let (p, q, c, d, e, a, b) = (
        ex(&mut dict, "p"),
        ex(&mut dict, "q"),
        ex(&mut dict, "C"),
        ex(&mut dict, "D"),
        ex(&mut dict, "E"),
        ex(&mut dict, "a"),
        ex(&mut dict, "b"),
    );
    let g = MaterializedGraph::new(
        &mut dict,
        &[[p, sp, q], [q, dom, c], [q, rng, d], [c, sc, e], [a, p, b]],
    );
    let base = rendered_base(&dict, &g);

    for t in [[a, q, b], [a, ty, c], [a, ty, e], [b, ty, d]] {
        assert!(g.contains(&t), "expected derived triple");
        let tree = g.why(&dict, t).expect("derived triple must have a proof");
        check_proof(&tree, &base, Some(&render(&dict, t)));
        leaves_entail(&mut dict, &g, &tree, t);
    }
    // (a type E) must chain rdfs9 on top of the rdfs2 typing.
    let tree = g.why(&dict, [a, ty, e]).unwrap();
    let root = &tree.nodes()[tree.root() as usize];
    assert_eq!(root.rule, "rdfs9");
    assert_eq!(tree.nodes()[root.premises[1] as usize].rule, "rdfs2");
    // And the rdfs2 step sits on the rdfs7-derived (a q b).
    let rdfs2 = &tree.nodes()[root.premises[1] as usize];
    assert_eq!(tree.nodes()[rdfs2.premises[1] as usize].rule, "rdfs7");
}

#[test]
fn differential_every_derived_triple_has_a_valid_proof() {
    // Randomized instance-heavy ontology (mirrors tests/incremental_prop.rs): EVERY triple
    // of the closure must explain — derived ones with checker-valid, leaf-re-entailing
    // proofs.
    let mut rng = Rng(0xE1A7_2026_0612);
    let mut dict = Dict::new();
    let (ty, sc, sp, dom, rng_p) = ids(&mut dict);

    let mut base: Vec<[Id; 3]> = Vec::new();
    let mut classes = Vec::new();
    for i in 0..4 {
        let chain: Vec<Id> = (0..5)
            .map(|j| dict.intern_iri(&format!("http://ex/C{i}_{j}")))
            .collect();
        for w in chain.windows(2) {
            base.push([w[0], sc, w[1]]);
        }
        classes.extend(chain);
    }
    let mut props = Vec::new();
    for i in 0..3 {
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
    let individuals: Vec<Id> = (0..120)
        .map(|i| dict.intern_iri(&format!("http://ex/ind{i}")))
        .collect();
    for &s in &individuals {
        base.push([s, ty, classes[rng.below(classes.len())]]);
        for _ in 0..2 {
            base.push([
                s,
                props[rng.below(props.len())],
                individuals[rng.below(individuals.len())],
            ]);
        }
    }

    let g = MaterializedGraph::new(&mut dict, &base);
    let rendered = rendered_base(&dict, &g);
    let base_set: FxHashSet<[Id; 3]> = base.iter().copied().collect();
    let mut checked_derived = 0usize;
    for t in g.closure() {
        let tree = g
            .why(&dict, t)
            .unwrap_or_else(|| panic!("no proof for closure triple"));
        check_proof(&tree, &rendered, Some(&render(&dict, t)));
        if !base_set.contains(&t) {
            leaves_entail(&mut dict, &g, &tree, t);
            checked_derived += 1;
        }
    }
    assert!(
        checked_derived > 500,
        "differential sweep too small: {checked_derived}"
    );
    // Determinism: a second run yields byte-identical proofs.
    for t in g.closure().into_iter().take(50) {
        let a = g.why(&dict, t).unwrap().to_json();
        let b = g.why(&dict, t).unwrap().to_json();
        assert_eq!(a, b);
    }
}

#[test]
fn retraction_invalidates_or_finds_alternative_support() {
    // a p b and a q b both type a into C (q via the dom of q; p sp q). Retract one support:
    // why() must keep returning a CURRENTLY-valid proof; retract both: triple gone, no proof.
    let mut dict = Dict::new();
    let (ty, _sc, sp, dom, _r) = ids(&mut dict);
    let (p, q, c, a, b) = (
        ex(&mut dict, "p"),
        ex(&mut dict, "q"),
        ex(&mut dict, "C"),
        ex(&mut dict, "a"),
        ex(&mut dict, "b"),
    );
    let mut g = MaterializedGraph::new(&mut dict, &[[p, sp, q], [q, dom, c], [a, p, b], [a, q, b]]);
    let t = [a, ty, c];
    assert!(g.contains(&t));
    let tree = g.why(&dict, t).unwrap();
    check_proof(&tree, &rendered_base(&dict, &g), Some(&render(&dict, t)));

    // Retract (a p b): (a type C) survives via (a q b); the proof must not lean on (a p b).
    g.delete(&[[a, p, b]]);
    assert!(g.contains(&t), "alternative support keeps the triple");
    let tree = g.why(&dict, t).expect("must find the surviving support");
    check_proof(&tree, &rendered_base(&dict, &g), Some(&render(&dict, t)));
    let retracted = render(&dict, [a, p, b]);
    assert!(
        tree.nodes().iter().all(|n| n.conclusion != retracted),
        "proof references the retracted triple"
    );

    // Retract the second support: the triple leaves the closure and has no proof.
    g.delete(&[[a, q, b]]);
    assert!(!g.contains(&t));
    assert!(g.why(&dict, t).is_none());

    // Re-insert: support (and proof) come back.
    g.insert(&[[a, q, b]]);
    assert!(g.contains(&t));
    let tree = g
        .why(&dict, t)
        .expect("re-inserted support restores the proof");
    check_proof(&tree, &rendered_base(&dict, &g), Some(&render(&dict, t)));
}

#[test]
fn retraction_differential_random_edits() {
    // Random edit batches; after each batch every closure triple must still explain validly
    // and no proof may cite a non-asserted leaf (the checker enforces it).
    let mut rng = Rng(0xBEEF_CAFE);
    let mut dict = Dict::new();
    let (ty, sc, _sp, dom, _r) = ids(&mut dict);
    let classes: Vec<Id> = (0..6).map(|j| ex(&mut dict, &format!("K{j}"))).collect();
    let p = ex(&mut dict, "rel");
    let mut base: Vec<[Id; 3]> = classes.windows(2).map(|w| [w[0], sc, w[1]]).collect();
    base.push([p, dom, classes[0]]);
    let inds: Vec<Id> = (0..40).map(|i| ex(&mut dict, &format!("i{i}"))).collect();
    let mut abox: Vec<[Id; 3]> = Vec::new();
    for &s in &inds {
        abox.push([s, ty, classes[rng.below(classes.len())]]);
        abox.push([s, p, inds[rng.below(inds.len())]]);
    }
    base.extend(&abox);
    let mut g = MaterializedGraph::new(&mut dict, &base);

    for round in 0..10 {
        // Delete a random ABox slice, insert some fresh assertions.
        let dels: Vec<[Id; 3]> = (0..5).map(|_| abox[rng.below(abox.len())]).collect();
        g.delete(&dels);
        let ins: Vec<[Id; 3]> = (0..3)
            .map(|_| {
                [
                    inds[rng.below(inds.len())],
                    ty,
                    classes[rng.below(classes.len())],
                ]
            })
            .collect();
        g.insert(&ins);
        let rendered = rendered_base(&dict, &g);
        for t in g.closure() {
            let tree = g
                .why(&dict, t)
                .unwrap_or_else(|| panic!("round {round}: no proof for closure triple"));
            check_proof(&tree, &rendered, Some(&render(&dict, t)));
        }
        // Deleted triples that left the closure must not explain.
        for d in &dels {
            if !g.contains(d) {
                assert!(
                    g.why(&dict, *d).is_none(),
                    "round {round}: proof for absent triple"
                );
            }
        }
        g.insert(&dels); // restore for the next round
    }
}
