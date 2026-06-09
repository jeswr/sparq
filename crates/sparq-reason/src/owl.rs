//! OWL 2 RL materialization (the forward-chainable OWL profile).
//!
//! Implements a correct, useful subset of the W3C OWL 2 RL/RDF rules
//! (<https://www.w3.org/TR/owl2-profiles/#Reasoning_in_OWL_2_RL_and_RDF_Graphs>) over the
//! same dictionary-encoded forward-chaining fixpoint as RDFS — RL rules are datalog-style
//! joins over integer ids. The RDFS rules (rdfs2/3/5/7/9/11) are included (RL subsumes them).
//!
//! | group | rule | premise ⊢ conclusion |
//! |---|---|---|
//! | equality | eq-sym   | `(x sameAs y)` ⊢ `(y sameAs x)` |
//! |          | eq-trans | `(x sameAs y),(y sameAs z)` ⊢ `(x sameAs z)` |
//! |          | eq-rep-s/p/o | `(s sameAs s'),(s p o)` ⊢ `(s' p o)` (and for p, o) |
//! | property | prp-inv1/2 | `(p inverseOf q),(x p y)` ⊢ `(y q x)` (and converse) |
//! |          | prp-symp | `(p a SymmetricProperty),(x p y)` ⊢ `(y p x)` |
//! |          | prp-trp  | `(p a TransitiveProperty),(x p y),(y p z)` ⊢ `(x p z)` |
//! |          | prp-eqp1/2 | `(p equivalentProperty q),(x p y)` ⊢ `(x q y)` (and converse) |
//! |          | prp-fp   | `(p a FunctionalProperty),(x p y1),(x p y2)` ⊢ `(y1 sameAs y2)` |
//! |          | prp-ifp  | `(p a InverseFunctionalProperty),(x1 p y),(x2 p y)` ⊢ `(x1 sameAs x2)` |
//! | class    | cax-eqc1/2 | `(c equivalentClass d),(x a c)` ⊢ `(x a d)` (and converse) |
//!
//! Not yet covered (roadmap): class-expression rules (cls-* for someValuesFrom/allValuesFrom/
//! hasValue/intersectionOf — need `owl:Restriction` / RDF-list decoding), `prp-spo2`
//! (propertyChainAxiom), and the consistency rules (cax-dw, eq-diff, prp-pdw → clashes). The
//! `sameAs` rules use the spec's eq-rep substitution (RL semantics); for large `sameAs`
//! cliques a union-find canonicalization is a future optimization.

use crate::{rdfs_round, Schema, Vocab};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};

const OWL: &str = "http://www.w3.org/2002/07/owl#";

struct Owl {
    same_as: Id,
    inverse_of: Id,
    symmetric: Id,        // owl:SymmetricProperty
    transitive: Id,       // owl:TransitiveProperty
    equiv_prop: Id,       // owl:equivalentProperty
    equiv_class: Id,      // owl:equivalentClass
    functional: Id,       // owl:FunctionalProperty
    inv_functional: Id,   // owl:InverseFunctionalProperty
}

impl Owl {
    fn intern(dict: &mut Dict) -> Owl {
        let mut i = |frag: &str| dict.intern_iri(&format!("{OWL}{frag}"));
        Owl {
            same_as: i("sameAs"),
            inverse_of: i("inverseOf"),
            symmetric: i("SymmetricProperty"),
            transitive: i("TransitiveProperty"),
            equiv_prop: i("equivalentProperty"),
            equiv_class: i("equivalentClass"),
            functional: i("FunctionalProperty"),
            inv_functional: i("InverseFunctionalProperty"),
        }
    }
}

/// Expand `triples` in place with the OWL 2 RL (+ RDFS) closure. Returns NEW triple count.
pub fn materialize_owl_rl(dict: &mut Dict, triples: &mut Vec<[Id; 3]>) -> usize {
    let v = Vocab::intern(dict);
    let o = Owl::intern(dict);
    let before = triples.len();
    let mut all: FxHashSet<[Id; 3]> = triples.iter().copied().collect();

    loop {
        let schema = Schema::build(&all, &v);
        let ax = Axioms::build(&all, &v, &o);
        let mut cand: Vec<[Id; 3]> = Vec::new();

        // RDFS rules (RL includes them) — shared with the RDFS materializer.
        rdfs_round(&all, &v, &schema, &mut cand);

        for &[s, p, obj] in &all {
            // --- equality (eq-sym, eq-rep-s/p/o) -------------------------------------
            if p == o.same_as {
                cand.push([obj, o.same_as, s]); // eq-sym
            }
            // eq-rep: substitute any sameAs-equal term in each position.
            if let Some(eqs) = ax.same.get(&s) {
                cand.extend(eqs.iter().map(|&s2| [s2, p, obj]));
            }
            if let Some(eqp) = ax.same.get(&p) {
                cand.extend(eqp.iter().map(|&p2| [s, p2, obj]));
            }
            if let Some(eqo) = ax.same.get(&obj) {
                cand.extend(eqo.iter().map(|&o2| [s, p, o2]));
            }
            // --- property axioms on assertion (s p obj) ------------------------------
            if let Some(invs) = ax.inverse.get(&p) {
                cand.extend(invs.iter().map(|&q| [obj, q, s])); // prp-inv1/2
            }
            if ax.symmetric.contains(&p) {
                cand.push([obj, p, s]); // prp-symp
            }
            if let Some(eqp) = ax.equiv_prop.get(&p) {
                cand.extend(eqp.iter().map(|&q| [s, q, obj])); // prp-eqp1/2
            }
            // --- class equivalence on type assertion ---------------------------------
            if p == v.ty {
                if let Some(eqc) = ax.equiv_class.get(&obj) {
                    cand.extend(eqc.iter().map(|&d| [s, v.ty, d])); // cax-eqc1/2
                }
            }
        }

        // --- joins that need an adjacency index (prp-trp / prp-fp / prp-ifp) ----------
        if !ax.transitive.is_empty() || !ax.functional.is_empty() || !ax.inv_functional.is_empty() {
            // by_pred: p -> (s -> [o]) and inverse o -> [s], built once per round.
            let mut out: FxHashMap<Id, FxHashMap<Id, Vec<Id>>> = FxHashMap::default();
            let mut inc: FxHashMap<Id, FxHashMap<Id, Vec<Id>>> = FxHashMap::default();
            let need: FxHashSet<Id> = ax
                .transitive
                .iter()
                .chain(ax.functional.iter())
                .chain(ax.inv_functional.iter())
                .copied()
                .collect();
            for &[s, p, obj] in &all {
                if need.contains(&p) {
                    out.entry(p).or_default().entry(s).or_default().push(obj);
                    inc.entry(p).or_default().entry(obj).or_default().push(s);
                }
            }
            // prp-trp: (x p y),(y p z) ⊢ (x p z).
            for &p in &ax.transitive {
                if let Some(adj) = out.get(&p) {
                    for (&x, ys) in adj {
                        for &y in ys {
                            if let Some(zs) = adj.get(&y) {
                                cand.extend(zs.iter().map(|&z| [x, p, z]));
                            }
                        }
                    }
                }
            }
            // prp-fp: functional ⊢ the two objects of one subject are sameAs.
            for &p in &ax.functional {
                if let Some(adj) = out.get(&p) {
                    for ys in adj.values() {
                        for i in 0..ys.len() {
                            for j in (i + 1)..ys.len() {
                                cand.push([ys[i], o.same_as, ys[j]]);
                            }
                        }
                    }
                }
            }
            // prp-ifp: inverse-functional ⊢ the two subjects of one object are sameAs.
            for &p in &ax.inv_functional {
                if let Some(adj) = inc.get(&p) {
                    for xs in adj.values() {
                        for i in 0..xs.len() {
                            for j in (i + 1)..xs.len() {
                                cand.push([xs[i], o.same_as, xs[j]]);
                            }
                        }
                    }
                }
            }
        }

        let mut changed = false;
        for t in cand {
            changed |= all.insert(t);
        }
        if !changed {
            break;
        }
    }

    let original: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
    let mut derived: Vec<[Id; 3]> = all.iter().copied().filter(|t| !original.contains(t)).collect();
    derived.sort_unstable();
    let mut base: Vec<[Id; 3]> = original.into_iter().collect();
    base.sort_unstable();
    triples.clear();
    triples.extend(base);
    triples.extend(derived);
    triples.len() - before
}

/// OWL axiom maps (the TBox), rebuilt each fixpoint round.
#[derive(Default)]
struct Axioms {
    same: FxHashMap<Id, Vec<Id>>,        // x -> sameAs partners (both directions seeded)
    inverse: FxHashMap<Id, Vec<Id>>,     // p -> inverse properties (both directions)
    equiv_prop: FxHashMap<Id, Vec<Id>>,  // p -> equivalent properties (both directions)
    equiv_class: FxHashMap<Id, Vec<Id>>, // c -> equivalent classes (both directions)
    symmetric: FxHashSet<Id>,
    transitive: FxHashSet<Id>,
    functional: FxHashSet<Id>,
    inv_functional: FxHashSet<Id>,
}

impl Axioms {
    fn build(all: &FxHashSet<[Id; 3]>, v: &Vocab, o: &Owl) -> Axioms {
        let mut ax = Axioms::default();
        let mut bi = |m: &mut FxHashMap<Id, Vec<Id>>, a: Id, b: Id| {
            m.entry(a).or_default().push(b);
            m.entry(b).or_default().push(a);
        };
        for &[s, p, obj] in all {
            if p == o.same_as {
                bi(&mut ax.same, s, obj);
            } else if p == o.inverse_of {
                bi(&mut ax.inverse, s, obj);
            } else if p == o.equiv_prop {
                bi(&mut ax.equiv_prop, s, obj);
            } else if p == o.equiv_class {
                bi(&mut ax.equiv_class, s, obj);
            } else if p == v.ty {
                if obj == o.symmetric {
                    ax.symmetric.insert(s);
                } else if obj == o.transitive {
                    ax.transitive.insert(s);
                } else if obj == o.functional {
                    ax.functional.insert(s);
                } else if obj == o.inv_functional {
                    ax.inv_functional.insert(s);
                }
            }
        }
        ax
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::vocab::rdf;
    use oxrdf::{NamedNode, Term};

    fn ex(dict: &mut Dict, local: &str) -> Id {
        dict.intern_iri(&format!("http://ex/{local}"))
    }
    fn owl(dict: &mut Dict, frag: &str) -> Id {
        dict.intern_iri(&format!("{OWL}{frag}"))
    }
    fn has(dict: &Dict, set: &FxHashSet<[Id; 3]>, s: &str, p: &str, o: &str) -> bool {
        let g = |iri: &str| dict.lookup(&Term::NamedNode(NamedNode::new_unchecked(iri.to_string())));
        let (si, pi, oi) = (g(s), g(p), g(o));
        si != 0 && pi != 0 && oi != 0 && set.contains(&[si, pi, oi])
    }

    #[test]
    fn inverse_symmetric_transitive() {
        let mut dict = Dict::new();
        let (parent, child, a, b, c) = (
            ex(&mut dict, "parentOf"), ex(&mut dict, "childOf"),
            ex(&mut dict, "a"), ex(&mut dict, "b"), ex(&mut dict, "c"),
        );
        let (anc, knows) = (ex(&mut dict, "ancestorOf"), ex(&mut dict, "knows"));
        let ty = dict.intern_iri(rdf::TYPE.as_str());
        let inv = owl(&mut dict, "inverseOf");
        let trans = owl(&mut dict, "TransitiveProperty");
        let sym = owl(&mut dict, "SymmetricProperty");
        let mut triples = vec![
            [parent, inv, child],                  // parentOf inverseOf childOf
            [anc, ty, trans],                      // ancestorOf a TransitiveProperty
            [knows, ty, sym],                      // knows a SymmetricProperty
            [a, parent, b],                        // a parentOf b
            [a, anc, b], [b, anc, c],              // a anc b ; b anc c
            [a, knows, b],                         // a knows b
        ];
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(has(&dict, &set, "http://ex/b", "http://ex/childOf", "http://ex/a"), "prp-inv");
        assert!(has(&dict, &set, "http://ex/a", "http://ex/ancestorOf", "http://ex/c"), "prp-trp");
        assert!(has(&dict, &set, "http://ex/b", "http://ex/knows", "http://ex/a"), "prp-symp");
    }

    #[test]
    fn sameas_and_functional() {
        // ex:hasSSN a FunctionalProperty ; a hasSSN s ; a hasSSN s2  ⊢  s sameAs s2.
        // Then eq-rep: (s p o) carries to s2.
        let mut dict = Dict::new();
        let (ssn, a, s1, s2, mark) = (
            ex(&mut dict, "hasSSN"), ex(&mut dict, "a"), ex(&mut dict, "s1"), ex(&mut dict, "s2"),
            ex(&mut dict, "marker"),
        );
        let ty = dict.intern_iri(rdf::TYPE.as_str());
        let func = owl(&mut dict, "FunctionalProperty");
        let p = ex(&mut dict, "p");
        let mut triples = vec![
            [ssn, ty, func],
            [a, ssn, s1], [a, ssn, s2],   // ⊢ s1 sameAs s2
            [s1, p, mark],                // eq-rep ⊢ s2 p marker
        ];
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(
            has(&dict, &set, "http://ex/s1", &format!("{OWL}sameAs"), "http://ex/s2"),
            "prp-fp ⊢ sameAs"
        );
        assert!(has(&dict, &set, "http://ex/s2", "http://ex/p", "http://ex/marker"), "eq-rep substitution");
    }

    #[test]
    fn equivalent_class_and_property() {
        let mut dict = Dict::new();
        let (human, person, x) = (ex(&mut dict, "Human"), ex(&mut dict, "Person"), ex(&mut dict, "x"));
        let (likes, enjoys, y) = (ex(&mut dict, "likes"), ex(&mut dict, "enjoys"), ex(&mut dict, "y"));
        let ty = dict.intern_iri(rdf::TYPE.as_str());
        let eqc = owl(&mut dict, "equivalentClass");
        let eqp = owl(&mut dict, "equivalentProperty");
        let mut triples = vec![
            [human, eqc, person], [x, ty, human],
            [likes, eqp, enjoys], [x, likes, y],
        ];
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(has(&dict, &set, "http://ex/x", rdf::TYPE.as_str(), "http://ex/Person"), "cax-eqc");
        assert!(has(&dict, &set, "http://ex/x", "http://ex/enjoys", "http://ex/y"), "prp-eqp");
    }
}
