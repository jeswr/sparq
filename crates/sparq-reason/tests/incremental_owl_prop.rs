//! Property tests for incremental OWL 2 RL maintenance: after EVERY random mutation batch,
//! the incrementally maintained closure must equal the from-scratch batch closure
//! (`materialize_owl_rl`) of the current base — the correctness oracle. Same discipline as
//! tests/incremental_prop.rs (the RDFS version), with an ontology that exercises the OWL
//! counting modes: subclass/subproperty chains with domain/range, `owl:inverseOf`,
//! `owl:SymmetricProperty`, `owl:TransitiveProperty` (the transitive layer),
//! `owl:equivalentClass`/`owl:equivalentProperty`, and `owl:differentFrom`; plus a separate
//! schedule for the documented fallback profile (a `someValuesFrom` restriction).

use rustc_hash::FxHashSet;
use sparq_core::dict::{Dict, Id};
use sparq_reason::{materialize_owl_rl, MaterializedOwlGraph, OwlMode};

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

struct V {
    ty: Id,
    sc: Id,
    sp: Id,
    dom: Id,
    rng: Id,
    inv: Id,
    symmetric: Id,
    transitive: Id,
    eqc: Id,
    eqp: Id,
    diff: Id,
}

fn vocab(dict: &mut Dict) -> V {
    use oxrdf::vocab::{rdf, rdfs};
    let owl = "http://www.w3.org/2002/07/owl#";
    V {
        ty: dict.intern_iri(rdf::TYPE.as_str()),
        sc: dict.intern_iri(rdfs::SUB_CLASS_OF.as_str()),
        sp: dict.intern_iri(rdfs::SUB_PROPERTY_OF.as_str()),
        dom: dict.intern_iri(rdfs::DOMAIN.as_str()),
        rng: dict.intern_iri(rdfs::RANGE.as_str()),
        inv: dict.intern_iri(&format!("{owl}inverseOf")),
        symmetric: dict.intern_iri(&format!("{owl}SymmetricProperty")),
        transitive: dict.intern_iri(&format!("{owl}TransitiveProperty")),
        eqc: dict.intern_iri(&format!("{owl}equivalentClass")),
        eqp: dict.intern_iri(&format!("{owl}equivalentProperty")),
        diff: dict.intern_iri(&format!("{owl}differentFrom")),
    }
}

struct Onto {
    v: V,
    classes: Vec<Id>,
    props: Vec<Id>, // includes plain, sub-chained, inverse, symmetric, transitive props
    individuals: Vec<Id>,
}

impl Onto {
    fn is_tbox(&self, t: &[Id; 3]) -> bool {
        let v = &self.v;
        t[1] == v.sc
            || t[1] == v.sp
            || t[1] == v.dom
            || t[1] == v.rng
            || t[1] == v.inv
            || t[1] == v.eqc
            || t[1] == v.eqp
            || (t[1] == v.ty && (t[2] == v.symmetric || t[2] == v.transitive))
    }
}

/// Build a TBox exercising every counting-mode feature + an instance-heavy ABox.
/// `with_transitive` toggles the CountingFixpoint vs CountingMono regime.
fn build(dict: &mut Dict, rng: &mut Rng, with_transitive: bool) -> (Onto, Vec<[Id; 3]>) {
    let v = vocab(dict);
    let mut base: Vec<[Id; 3]> = Vec::new();
    // 4 class chains of depth 5, with one equivalentClass bridge between chains.
    let mut classes = Vec::new();
    for i in 0..4 {
        let chain: Vec<Id> = (0..5)
            .map(|j| dict.intern_iri(&format!("http://ex/C{i}_{j}")))
            .collect();
        for w in chain.windows(2) {
            base.push([w[0], v.sc, w[1]]);
        }
        classes.extend(chain);
    }
    base.push([classes[2], v.eqc, classes[7]]); // C0_2 ≡ C1_2
                                                // 3 subproperty chains of depth 3, each property with random domain/range.
    let mut props = Vec::new();
    for i in 0..3 {
        let chain: Vec<Id> = (0..3)
            .map(|j| dict.intern_iri(&format!("http://ex/p{i}_{j}")))
            .collect();
        for w in chain.windows(2) {
            base.push([w[0], v.sp, w[1]]);
        }
        for &p in &chain {
            base.push([p, v.dom, classes[rng.below(classes.len())]]);
            base.push([p, v.rng, classes[rng.below(classes.len())]]);
        }
        props.extend(chain);
    }
    // equivalentProperty bridge.
    base.push([props[1], v.eqp, props[4]]);
    // An inverse pair (with domain/range so the orientation matters), a symmetric property,
    // and — optionally — two transitive properties, one of them a SUPERproperty of a plain
    // property (so non-transitive assertions feed the transitive layer) and one symmetric+
    // transitive (orientation-flipping closure).
    let has_part = dict.intern_iri("http://ex/hasPart");
    let part_of = dict.intern_iri("http://ex/partOf");
    base.push([has_part, v.inv, part_of]);
    base.push([has_part, v.dom, classes[0]]);
    base.push([part_of, v.rng, classes[5]]);
    props.push(has_part);
    props.push(part_of);
    let touches = dict.intern_iri("http://ex/touches");
    base.push([touches, v.ty, v.symmetric]);
    props.push(touches);
    if with_transitive {
        let ancestor = dict.intern_iri("http://ex/ancestorOf");
        let parent = dict.intern_iri("http://ex/parentOf");
        base.push([ancestor, v.ty, v.transitive]);
        base.push([parent, v.sp, ancestor]); // plain property feeding the transitive layer
        base.push([ancestor, v.dom, classes[10]]);
        let connected = dict.intern_iri("http://ex/connectedTo");
        base.push([connected, v.ty, v.transitive]);
        base.push([connected, v.ty, v.symmetric]); // symmetric + transitive
        props.push(ancestor);
        props.push(parent);
        props.push(connected);
    }
    // 150 individuals with random types and property assertions (+ some differentFrom).
    let individuals: Vec<Id> = (0..150)
        .map(|i| dict.intern_iri(&format!("http://ex/ind{i}")))
        .collect();
    for &s in &individuals {
        base.push([s, v.ty, classes[rng.below(classes.len())]]);
        for _ in 0..2 {
            let p = props[rng.below(props.len())];
            let o = individuals[rng.below(individuals.len())];
            base.push([s, p, o]);
        }
    }
    for _ in 0..10 {
        let a = individuals[rng.below(individuals.len())];
        let b = individuals[rng.below(individuals.len())];
        base.push([a, v.diff, b]);
    }
    (
        Onto {
            v,
            classes,
            props,
            individuals,
        },
        base,
    )
}

/// A random ABox triple: type assertion, property assertion, or differentFrom.
fn random_abox(o: &Onto, rng: &mut Rng) -> [Id; 3] {
    let s = o.individuals[rng.below(o.individuals.len())];
    match rng.below(6) {
        0 | 1 => [s, o.v.ty, o.classes[rng.below(o.classes.len())]],
        5 => [s, o.v.diff, o.individuals[rng.below(o.individuals.len())]],
        _ => {
            let p = o.props[rng.below(o.props.len())];
            [s, p, o.individuals[rng.below(o.individuals.len())]]
        }
    }
}

fn oracle(dict: &mut Dict, mirror: &FxHashSet<[Id; 3]>) -> FxHashSet<[Id; 3]> {
    let mut v: Vec<[Id; 3]> = mirror.iter().copied().collect();
    materialize_owl_rl(dict, &mut v);
    v.into_iter().collect()
}

fn run_schedule(seed: u64, with_transitive: bool, batches: usize) {
    let mut rng = Rng(seed);
    let mut dict = Dict::new();
    let (onto, base) = build(&mut dict, &mut rng, with_transitive);

    let mut g = MaterializedOwlGraph::new(&mut dict, &base);
    let mut mirror: FxHashSet<[Id; 3]> = base.into_iter().collect();

    let expected_mode = if with_transitive {
        OwlMode::CountingFixpoint
    } else {
        OwlMode::CountingMono
    };
    assert_eq!(g.mode(), expected_mode, "profile detection");

    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        oracle(&mut dict, &mirror),
        "initial materialization disagrees with batch"
    );

    let mut tbox_batches = 0usize;
    for batch in 0..batches {
        let op = rng.below(100);
        if op < 6 {
            // TBox mutation (~6%): toggle a subClassOf edge between random classes —
            // exercises the full-rematerialization fallback.
            tbox_batches += 1;
            let a = onto.classes[rng.below(onto.classes.len())];
            let b = onto.classes[rng.below(onto.classes.len())];
            let t = [a, onto.v.sc, b];
            let rebuilds_before = g.full_rebuilds();
            if mirror.contains(&t) {
                g.delete(&mut dict, &[t]);
                mirror.remove(&t);
            } else {
                g.insert(&mut dict, &[t]);
                mirror.insert(t);
            }
            assert_eq!(
                g.full_rebuilds(),
                rebuilds_before + 1,
                "TBox mutation must trigger exactly one rebuild (batch {batch})"
            );
        } else if op < 53 {
            // ABox insert batch: 1-8 random triples.
            let n = 1 + rng.below(8);
            let delta: Vec<[Id; 3]> = (0..n).map(|_| random_abox(&onto, &mut rng)).collect();
            g.insert(&mut dict, &delta);
            mirror.extend(delta);
        } else {
            // ABox delete batch: mostly currently-asserted ABox triples + some randoms.
            let current: Vec<[Id; 3]> = mirror
                .iter()
                .copied()
                .filter(|t| !onto.is_tbox(t))
                .collect();
            let n = 1 + rng.below(8);
            let delta: Vec<[Id; 3]> = (0..n)
                .map(|_| {
                    if rng.below(4) == 0 || current.is_empty() {
                        random_abox(&onto, &mut rng)
                    } else {
                        current[rng.below(current.len())]
                    }
                })
                .collect();
            g.delete(&mut dict, &delta);
            for t in &delta {
                mirror.remove(t);
            }
        }

        // THE ORACLE: incrementally maintained closure == from-scratch closure, every batch.
        let inc: FxHashSet<[Id; 3]> = g.closure().into_iter().collect();
        let full = oracle(&mut dict, &mirror);
        if inc != full {
            let missing: Vec<_> = full.difference(&inc).take(5).collect();
            let extra: Vec<_> = inc.difference(&full).take(5).collect();
            panic!(
                "closure diverged at batch {batch} (inc {} vs full {}):\n missing: {missing:?}\n extra: {extra:?}",
                inc.len(),
                full.len()
            );
        }
        assert_eq!(
            g.len(),
            inc.len(),
            "len() inconsistent with closure() at batch {batch}"
        );
        assert_eq!(g.base_len(), mirror.len(), "base drifted at batch {batch}");
    }
    assert!(
        tbox_batches > 0,
        "schedule should have exercised the TBox fallback"
    );
}

#[test]
fn counting_mono_matches_from_scratch_over_random_mutations() {
    run_schedule(0x5EED_2026_0612_0001, false, 90);
}

#[test]
fn counting_fixpoint_matches_from_scratch_over_random_mutations() {
    run_schedule(0x5EED_2026_0612_0002, true, 90);
}

#[test]
fn fallback_profile_stays_correct_and_counts_rebuilds() {
    // An ontology with a someValuesFrom restriction (a fallback feature): every mutation
    // re-materializes; correctness must still hold against the batch oracle.
    let mut rng = Rng(0x5EED_2026_0612_0003);
    let mut dict = Dict::new();
    let (onto, mut base) = build(&mut dict, &mut rng, false);
    let owl = "http://www.w3.org/2002/07/owl#";
    let restriction = dict.intern_iri("http://ex/R");
    let on_property = dict.intern_iri(&format!("{owl}onProperty"));
    let svf = dict.intern_iri(&format!("{owl}someValuesFrom"));
    base.push([restriction, on_property, onto.props[0]]);
    base.push([restriction, svf, onto.classes[0]]);

    let mut g = MaterializedOwlGraph::new(&mut dict, &base);
    assert_eq!(
        g.mode(),
        OwlMode::Fallback,
        "restriction must force fallback mode"
    );
    let mut mirror: FxHashSet<[Id; 3]> = base.into_iter().collect();
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        oracle(&mut dict, &mirror)
    );

    for batch in 0..12 {
        let n = 1 + rng.below(5);
        let delta: Vec<[Id; 3]> = (0..n).map(|_| random_abox(&onto, &mut rng)).collect();
        let rebuilds_before = g.full_rebuilds();
        if batch % 2 == 0 {
            g.insert(&mut dict, &delta);
            mirror.extend(delta);
        } else {
            g.delete(&mut dict, &delta);
            for t in &delta {
                mirror.remove(t);
            }
        }
        if g.base_len() != mirror.len() {
            // delete of absent triples is a no-op in both — keep mirrors aligned
            panic!("base drifted at batch {batch}");
        }
        assert!(
            g.full_rebuilds() > rebuilds_before || g.base_len() == mirror.len(),
            "fallback mode re-materializes on every effective mutation"
        );
        assert_eq!(
            g.closure().into_iter().collect::<FxHashSet<_>>(),
            oracle(&mut dict, &mirror),
            "fallback closure diverged at batch {batch}"
        );
    }
}

#[test]
fn mode_flips_when_fallback_feature_added_and_removed() {
    let mut rng = Rng(0x5EED_2026_0612_0004);
    let mut dict = Dict::new();
    let (onto, base) = build(&mut dict, &mut rng, true);
    let owl = "http://www.w3.org/2002/07/owl#";
    let same_as = dict.intern_iri(&format!("{owl}sameAs"));

    let mut g = MaterializedOwlGraph::new(&mut dict, &base);
    let mut mirror: FxHashSet<[Id; 3]> = base.into_iter().collect();
    assert_eq!(g.mode(), OwlMode::CountingFixpoint);

    // Adding a sameAs assertion flips to fallback (and stays correct).
    let t = [onto.individuals[0], same_as, onto.individuals[1]];
    g.insert(&mut dict, &[t]);
    mirror.insert(t);
    assert_eq!(g.mode(), OwlMode::Fallback, "sameAs must flip to fallback");
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        oracle(&mut dict, &mirror)
    );

    // Removing it flips back to counting.
    g.delete(&mut dict, &[t]);
    mirror.remove(&t);
    assert_eq!(
        g.mode(),
        OwlMode::CountingFixpoint,
        "fallback must lift when sameAs goes"
    );
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        oracle(&mut dict, &mirror)
    );

    // And ABox updates are incremental again.
    let rebuilds = g.full_rebuilds();
    let d = random_abox(&onto, &mut rng);
    g.insert(&mut dict, &[d]);
    mirror.insert(d);
    assert_eq!(
        g.full_rebuilds(),
        rebuilds,
        "counting mode must not rebuild on ABox insert"
    );
    assert_eq!(
        g.closure().into_iter().collect::<FxHashSet<_>>(),
        oracle(&mut dict, &mirror)
    );
}
