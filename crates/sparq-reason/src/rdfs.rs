//! RDFS forward-chaining materialization.
//!
//! Implements the *useful, non-explosive* RDFS entailment rules (RDF 1.1 Semantics §9.2.1)
//! as a fixpoint over dictionary-encoded triples:
//!
//! | rule | premise | conclusion |
//! |------|---------|------------|
//! | rdfs2  | `(p domain c)`, `(s p o)`         | `(s type c)` |
//! | rdfs3  | `(p range c)`, `(s p o)`          | `(o type c)` |
//! | rdfs5  | `(p subPropertyOf q)`, `(q subPropertyOf r)` | `(p subPropertyOf r)` |
//! | rdfs7  | `(p subPropertyOf q)`, `(s p o)`  | `(s q o)` |
//! | rdfs9  | `(c subClassOf d)`, `(s type c)`  | `(s type d)` |
//! | rdfs11 | `(c subClassOf d)`, `(d subClassOf e)` | `(c subClassOf e)` |
//!
//! Deliberately OMITTED: the axiomatic triples and the reflexive/`rdfs:Resource` rules
//! (rdfs4a/4b, rdfs6, rdfs8, rdfs10, rdfs13). They entail that every resource is an
//! `rdfs:Resource` and every class a subclass of itself/`Resource` — true but useless, and
//! they blow up the store by O(terms). This matches the "RDFS" rule set materialized by
//! production engines (GraphDB/RDF4J `rdfs` minus the axiomatic closure).
//!
//! The fixpoint is **naive** (re-derive from the full set each round until stable) — chosen
//! for obvious correctness; RDFS closures converge in a handful of rounds (≈ hierarchy
//! depth + 1). Semi-naive (delta-only) evaluation is a future optimization; materialization
//! is an opt-in build-time step, never on the query hot path.

use crate::{Schema, Vocab};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};

/// Incremental indexes for SEMI-NAIVE RDFS materialization: a new triple joins only against
/// the relevant index (not a full scan), and each rule is fired once per newly-derived fact
/// (the `delta`). Both directions of every join are covered so transitivity (rdfs5/11)
/// closes correctly. Without this the naive fixpoint is O(N³) on deep hierarchies (the O(N²)
/// subclass closure re-derived O(N) rounds).
#[derive(Default)]
struct RdfsIndex {
    sc_super: FxHashMap<Id, Vec<Id>>, // c -> super-classes d (c subClassOf d)
    sc_sub: FxHashMap<Id, Vec<Id>>,   // d -> sub-classes c
    sp_super: FxHashMap<Id, Vec<Id>>, // p -> super-properties q
    sp_sub: FxHashMap<Id, Vec<Id>>,   // q -> sub-properties p
    domain: FxHashMap<Id, Vec<Id>>,   // p -> domain classes
    range: FxHashMap<Id, Vec<Id>>,    // p -> range classes
    type_sub: FxHashMap<Id, Vec<Id>>, // c -> subjects typed c
    po: FxHashMap<Id, Vec<(Id, Id)>>, // predicate -> (subject, object) assertions
}

impl RdfsIndex {
    fn insert(&mut self, [s, p, o]: [Id; 3], v: &Vocab) {
        if p == v.sub_class {
            self.sc_super.entry(s).or_default().push(o);
            self.sc_sub.entry(o).or_default().push(s);
        } else if p == v.sub_prop {
            self.sp_super.entry(s).or_default().push(o);
            self.sp_sub.entry(o).or_default().push(s);
        } else if p == v.domain {
            self.domain.entry(s).or_default().push(o);
        } else if p == v.range {
            self.range.entry(s).or_default().push(o);
        } else if p == v.ty {
            self.type_sub.entry(o).or_default().push(s);
        }
        self.po.entry(p).or_default().push((s, o));
    }

    /// All immediate RDFS consequences of `[s,p,o]` joining against the current index, pushed
    /// into `out`. Each rule appears in BOTH delta directions (this triple as either premise).
    fn derive(&self, [s, p, o]: [Id; 3], v: &Vocab, out: &mut Vec<[Id; 3]>) {
        if p == v.sub_class {
            // rdfs11 (s sc o)(o sc x)⊢(s sc x) and (c sc s)(s sc o)⊢(c sc o)
            if let Some(xs) = self.sc_super.get(&o) {
                out.extend(xs.iter().map(|&x| [s, v.sub_class, x]));
            }
            if let Some(cs) = self.sc_sub.get(&s) {
                out.extend(cs.iter().map(|&c| [c, v.sub_class, o]));
            }
            // rdfs9 (y type s)(s sc o)⊢(y type o)
            if let Some(ys) = self.type_sub.get(&s) {
                out.extend(ys.iter().map(|&y| [y, v.ty, o]));
            }
        } else if p == v.sub_prop {
            // rdfs5 transitivity (both directions)
            if let Some(xs) = self.sp_super.get(&o) {
                out.extend(xs.iter().map(|&x| [s, v.sub_prop, x]));
            }
            if let Some(cs) = self.sp_sub.get(&s) {
                out.extend(cs.iter().map(|&c| [c, v.sub_prop, o]));
            }
            // rdfs7 (x s y)(s sp o)⊢(x o y)
            if let Some(pairs) = self.po.get(&s) {
                out.extend(pairs.iter().map(|&(x, y)| [x, o, y]));
            }
        } else if p == v.ty {
            // rdfs9 (s type o)(o sc d)⊢(s type d)
            if let Some(ds) = self.sc_super.get(&o) {
                out.extend(ds.iter().map(|&d| [s, v.ty, d]));
            }
        } else if p == v.domain {
            // rdfs2 (x s y)(s domain o)⊢(x type o)
            if let Some(pairs) = self.po.get(&s) {
                out.extend(pairs.iter().map(|&(x, _)| [x, v.ty, o]));
            }
        } else if p == v.range {
            // rdfs3 (x s y)(s range o)⊢(y type o)
            if let Some(pairs) = self.po.get(&s) {
                out.extend(pairs.iter().map(|&(_, y)| [y, v.ty, o]));
            }
        }
        // For the OTHER delta direction of rdfs7/2/3: this (s p o) as the data triple.
        if let Some(qs) = self.sp_super.get(&p) {
            out.extend(qs.iter().map(|&q| [s, q, o]));
        }
        if let Some(cs) = self.domain.get(&p) {
            out.extend(cs.iter().map(|&c| [s, v.ty, c]));
        }
        if let Some(cs) = self.range.get(&p) {
            out.extend(cs.iter().map(|&c| [o, v.ty, c]));
        }
    }
}

/// Expand `triples` in place with the RDFS closure. Returns the number of NEW triples added.
pub fn materialize_rdfs(dict: &mut Dict, triples: &mut Vec<[Id; 3]>) -> usize {
    let v = Vocab::intern(dict);
    let before = triples.len();

    // Dedup set seeded with the input (the input may itself contain duplicates; we return a
    // duplicate-free closure either way).
    let mut all: FxHashSet<[Id; 3]> = triples.iter().copied().collect();

    // SEMI-NAIVE fixpoint with incremental indexes: each round derives only from the previous
    // round's NEW facts (`delta`), joining against the index — never re-scanning the whole
    // closure. Linear in the number of derivations instead of the naive O(rounds × closure).
    let mut idx = RdfsIndex::default();
    for &t in &all {
        idx.insert(t, &v);
    }
    let mut delta: Vec<[Id; 3]> = all.iter().copied().collect();
    let mut cand: Vec<[Id; 3]> = Vec::new();
    while !delta.is_empty() {
        cand.clear();
        for &t in &delta {
            idx.derive(t, &v, &mut cand);
        }
        let mut next: Vec<[Id; 3]> = Vec::new();
        for &t in &cand {
            if all.insert(t) {
                idx.insert(t, &v);
                next.push(t);
            }
        }
        delta = next;
    }

    // Rebuild `triples` as the (duplicate-free) closure, preserving the original triples'
    // relative order at the front for determinism, then the newly-derived ones sorted (so
    // the materialized output is deterministic regardless of HashSet iteration order).
    let original: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
    let mut derived: Vec<[Id; 3]> = all.iter().copied().filter(|t| !original.contains(t)).collect();
    derived.sort_unstable();
    triples.clear();
    triples.extend(original_in_order(&original));
    triples.extend(derived);
    triples.len() - before
}

/// Applies one round of the RDFS rules (rdfs2/3/5/7/9/11) to `all` using `schema`, pushing
/// every immediate consequence into `cand`. Shared by the RDFS and OWL 2 RL materializers
/// (RL subsumes RDFS). Deduplication / fixpoint control is the caller's.
pub(crate) fn rdfs_round(all: &FxHashSet<[Id; 3]>, v: &Vocab, schema: &Schema, cand: &mut Vec<[Id; 3]>) {
    for &[s, p, o] in all {
        // rdfs11 — subClassOf transitivity: (s sc o), (o sc x) ⊢ (s sc x)
        if p == v.sub_class {
            if let Some(sup) = schema.sub_class.get(&o) {
                cand.extend(sup.iter().map(|&x| [s, v.sub_class, x]));
            }
        }
        // rdfs5 — subPropertyOf transitivity
        if p == v.sub_prop {
            if let Some(sup) = schema.sub_prop.get(&o) {
                cand.extend(sup.iter().map(|&x| [s, v.sub_prop, x]));
            }
        }
        // rdfs9 — type propagation up the class hierarchy: (s type o), (o sc d) ⊢ (s type d)
        if p == v.ty {
            if let Some(sup) = schema.sub_class.get(&o) {
                cand.extend(sup.iter().map(|&d| [s, v.ty, d]));
            }
        }
        // rdfs7 — subproperty entailment: (s p o), (p sp q) ⊢ (s q o)
        if let Some(sup) = schema.sub_prop.get(&p) {
            cand.extend(sup.iter().map(|&q| [s, q, o]));
        }
        // rdfs2 — domain typing: (s p o), (p domain c) ⊢ (s type c)
        if let Some(cs) = schema.domain.get(&p) {
            cand.extend(cs.iter().map(|&c| [s, v.ty, c]));
        }
        // rdfs3 — range typing: (s p o), (p range c) ⊢ (o type c)
        if let Some(cs) = schema.range.get(&p) {
            cand.extend(cs.iter().map(|&c| [o, v.ty, c]));
        }
    }
}

/// The original triples, deduplicated but in a deterministic (sorted) order. (We do not have
/// the caller's original Vec here after clearing; sorting gives a stable, reproducible base.)
fn original_in_order(original: &FxHashSet<[Id; 3]>) -> Vec<[Id; 3]> {
    let mut v: Vec<[Id; 3]> = original.iter().copied().collect();
    v.sort_unstable();
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::vocab::{rdf, rdfs};
    use oxrdf::{NamedNode, Term};

    fn iri(dict: &mut Dict, s: &str) -> Id {
        dict.intern_iri(s)
    }
    fn ex(dict: &mut Dict, local: &str) -> Id {
        dict.intern_iri(&format!("http://ex/{local}"))
    }
    /// Is the triple (by IRI strings) in the materialized set?
    fn has(dict: &Dict, set: &FxHashSet<[Id; 3]>, s: &str, p: &str, o: &str) -> bool {
        let g = |iri: &str| dict.lookup(&Term::NamedNode(NamedNode::new_unchecked(iri.to_string())));
        let (si, pi, oi) = (g(s), g(p), g(o));
        si != 0 && pi != 0 && oi != 0 && set.contains(&[si, pi, oi])
    }

    #[test]
    fn subclass_transitivity_and_type_propagation() {
        // ex:Dog sc ex:Mammal sc ex:Animal ; ex:rex a ex:Dog.  Expect ex:rex a Mammal, Animal.
        let mut dict = Dict::new();
        let (dog, mammal, animal, rex) =
            (ex(&mut dict, "Dog"), ex(&mut dict, "Mammal"), ex(&mut dict, "Animal"), ex(&mut dict, "rex"));
        let (sc, ty) = (iri(&mut dict, rdfs::SUB_CLASS_OF.as_str()), iri(&mut dict, rdf::TYPE.as_str()));
        let mut triples = vec![[dog, sc, mammal], [mammal, sc, animal], [rex, ty, dog]];
        let added = materialize_rdfs(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(has(&dict, &set, "http://ex/rex", rdf::TYPE.as_str(), "http://ex/Mammal"), "rdfs9 one hop");
        assert!(has(&dict, &set, "http://ex/rex", rdf::TYPE.as_str(), "http://ex/Animal"), "rdfs9 transitive");
        assert!(has(&dict, &set, "http://ex/Dog", rdfs::SUB_CLASS_OF.as_str(), "http://ex/Animal"), "rdfs11");
        assert!(added >= 3);
    }

    #[test]
    fn domain_range_and_subproperty() {
        // ex:hasParent sp ex:relatedTo ; ex:hasParent domain ex:Person ; ex:hasParent range ex:Person.
        // ex:a ex:hasParent ex:b.  Expect: a relatedTo b; a type Person; b type Person.
        let mut dict = Dict::new();
        let (hp, rel, person, a, b) = (
            ex(&mut dict, "hasParent"), ex(&mut dict, "relatedTo"), ex(&mut dict, "Person"),
            ex(&mut dict, "a"), ex(&mut dict, "b"),
        );
        let (sp, dom, rng) = (
            iri(&mut dict, rdfs::SUB_PROPERTY_OF.as_str()),
            iri(&mut dict, rdfs::DOMAIN.as_str()),
            iri(&mut dict, rdfs::RANGE.as_str()),
        );
        let mut triples = vec![[hp, sp, rel], [hp, dom, person], [hp, rng, person], [a, hp, b]];
        materialize_rdfs(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(has(&dict, &set, "http://ex/a", "http://ex/relatedTo", "http://ex/b"), "rdfs7");
        assert!(has(&dict, &set, "http://ex/a", rdf::TYPE.as_str(), "http://ex/Person"), "rdfs2 domain");
        assert!(has(&dict, &set, "http://ex/b", rdf::TYPE.as_str(), "http://ex/Person"), "rdfs3 range");
    }

    #[test]
    fn subproperty_domain_interaction() {
        // ex:p sp ex:q ; ex:q domain ex:C ; ex:s ex:p ex:o.  rdfs7 gives (s q o), then rdfs2
        // on q's domain gives (s type C). Tests the rule-interaction the fixpoint must catch.
        let mut dict = Dict::new();
        let (p, q, c, s, o) = (
            ex(&mut dict, "p"), ex(&mut dict, "q"), ex(&mut dict, "C"), ex(&mut dict, "s"), ex(&mut dict, "o"),
        );
        let (sp, dom) = (iri(&mut dict, rdfs::SUB_PROPERTY_OF.as_str()), iri(&mut dict, rdfs::DOMAIN.as_str()));
        let mut triples = vec![[p, sp, q], [q, dom, c], [s, p, o]];
        materialize_rdfs(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(has(&dict, &set, "http://ex/s", "http://ex/q", "http://ex/o"), "rdfs7");
        assert!(has(&dict, &set, "http://ex/s", rdf::TYPE.as_str(), "http://ex/C"), "rdfs7->rdfs2 interaction");
    }

    #[test]
    fn idempotent() {
        let mut dict = Dict::new();
        let (dog, animal, rex) = (ex(&mut dict, "Dog"), ex(&mut dict, "Animal"), ex(&mut dict, "rex"));
        let (sc, ty) = (iri(&mut dict, rdfs::SUB_CLASS_OF.as_str()), iri(&mut dict, rdf::TYPE.as_str()));
        let mut triples = vec![[dog, sc, animal], [rex, ty, dog]];
        materialize_rdfs(&mut dict, &mut triples);
        let n = triples.len();
        let added2 = materialize_rdfs(&mut dict, &mut triples);
        assert_eq!(added2, 0, "second materialization adds nothing");
        assert_eq!(triples.len(), n, "idempotent");
    }
}
