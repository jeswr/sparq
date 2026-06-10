//! **Incremental maintenance** of the RDFS closure under inserts and deletes (T18).
//!
//! Batch materialization ([`crate::materialize_rdfs`]) recomputes the whole closure; with a
//! delta-overlay store (T17) that means a single triple update invalidates everything. This
//! module keeps the closure up to date in time proportional to the *change*, RDFox-style.
//!
//! # Algorithm: exact **counting** (not DRed) — and why
//!
//! The two standard incremental-materialization algorithms are DRed (overdelete, then
//! rederive survivors) and counting (track how many distinct derivations support each derived
//! fact; a fact dies when its count hits 0). Counting is usually dismissed for recursive
//! datalog because a fact can support its own (re-)derivation, making counts ill-defined or
//! requiring derivation-depth stratification; DRed pays for that generality with a rederivation
//! pass that can touch far more facts than actually change.
//!
//! Our batch RDFS engine, however, is deliberately **non-recursive over the ABox**: it
//! saturates the TBox once (transitive `subClassOf`/`subPropertyOf` closures, domain/range sets
//! closed through both — see [`crate::rdfs`]) and then derives *every* entailed triple in a
//! **single sweep** where each base assertion's consequences (`emit_consequences`) depend
//! only on the closed TBox, never on other derived ABox facts. So with the TBox held fixed:
//!
//! * every derivation is **one step** from a *base* triple — derived facts never feed back;
//! * the multiset of derivations of a derived triple is exactly the multiset of emissions of
//!   `emit_consequences` over the base triples that produce it;
//! * therefore a per-derived-triple **derivation count is exact**: insert a base triple →
//!   `+1` per emission; delete it → `-1` per the *identical* (deterministic, TBox-only)
//!   emission set; a derived fact leaves the closure exactly when its count reaches 0.
//!
//! No overdeletion, no rederivation proofs, no recursion bookkeeping — deletion costs the same
//! as insertion. Memory cost is one `u32` per distinct derived triple (the classic counting
//! objection — one count per *derivation depth* — vanishes because depth is always 1). This is
//! strictly cheaper and simpler than DRed for this rule set, which is why counting was chosen.
//!
//! # TBox mutations (v1)
//!
//! Inserting/deleting a *schema* triple (`rdfs:subClassOf`, `rdfs:subPropertyOf`,
//! `rdfs:domain`, `rdfs:range`) changes the closed TBox that every count was computed against,
//! so v1 falls back to a **full re-materialization** (recomputing closures and all counts).
//! This is documented, deliberate, and cheap to detect (predicate check); ABox-only updates —
//! the overwhelmingly common case — never trigger it. Incremental TBox maintenance (delta
//! TBox closure + re-sweep of affected predicates/classes only) is a follow-up, as is wiring
//! this into the T17 store overlay.
//!
//! # Semantics
//!
//! * `insert`/`delete` operate on the **base** (explicitly asserted) triples with set
//!   semantics: re-inserting a present triple or deleting an absent one is a no-op.
//! * Deleting a base triple that is *also* derivable from the remaining base keeps it in the
//!   closure (its count is still positive) — exactly what from-scratch materialization of the
//!   remainder would produce, and the case DRed needs its rederivation pass for.
//! * Deleting a derived-only triple is a no-op: entailed facts cannot be retracted while
//!   their support exists (standard materialized-view semantics).

use crate::rdfs::{close_dr, sweep, transitive_closure};
use crate::Vocab;
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};

/// An RDFS-materialized triple set maintained **incrementally** under base inserts/deletes.
///
/// Owns the base (asserted) triples, the derived set with exact derivation counts, and the
/// closed TBox the counts were computed against. The full closure (base ∪ derived) is exposed
/// via [`closure`](Self::closure) / [`contains`](Self::contains) and always equals what
/// [`crate::materialize_rdfs`] would produce from scratch on the current base.
pub struct MaterializedGraph {
    v: Vocab,
    /// Explicitly asserted triples (set semantics).
    base: FxHashSet<[Id; 3]>,
    /// Derived triple -> number of one-step derivations currently supporting it (always > 0;
    /// zero-count entries are removed eagerly). May overlap `base`: a fact can be both
    /// asserted and derived, and survives deletion of either support alone.
    counts: FxHashMap<[Id; 3], u32>,
    /// TBox-closure triples (transitive `subClassOf`/`subPropertyOf` facts, rdfs11/rdfs5).
    /// Kept separate from `counts`: they are derived from the TBox alone and are rebuilt
    /// wholesale on any TBox mutation, never adjusted by ABox deltas.
    schema_facts: FxHashSet<[Id; 3]>,
    // The closed TBox (see `crate::rdfs::rdfs_closure` steps 1-2), kept so ABox deltas can be
    // swept against it without recomputation.
    sc_closure: FxHashMap<Id, Vec<Id>>,
    sp_closure: FxHashMap<Id, Vec<Id>>,
    dom_full: FxHashMap<Id, Vec<Id>>,
    rng_full: FxHashMap<Id, Vec<Id>>,
    /// How many times the v1 full-rematerialization fallback ran (TBox mutations). The initial
    /// materialization in [`new`](Self::new) is not counted. Exposed for tests/telemetry.
    rebuilds: usize,
}

impl MaterializedGraph {
    /// Build the graph from `base_triples` and fully materialize the RDFS closure (same rule
    /// set and result as [`crate::materialize_rdfs`]), recording the derivation counts that
    /// make subsequent [`insert`](Self::insert)/[`delete`](Self::delete) incremental.
    ///
    /// `dict` is only needed to intern the RDFS vocabulary ids once; incremental updates are
    /// dictionary-free (callers intern new terms through their own `Dict` as usual).
    pub fn new(dict: &mut Dict, base_triples: &[[Id; 3]]) -> Self {
        let mut g = MaterializedGraph {
            v: Vocab::intern(dict),
            base: base_triples.iter().copied().collect(),
            counts: FxHashMap::default(),
            schema_facts: FxHashSet::default(),
            sc_closure: FxHashMap::default(),
            sp_closure: FxHashMap::default(),
            dom_full: FxHashMap::default(),
            rng_full: FxHashMap::default(),
            rebuilds: 0,
        };
        g.rematerialize();
        g.rebuilds = 0; // the initial materialization is not a fallback
        g
    }

    /// Is `p` one of the four TBox predicates whose triples define the closed schema?
    fn is_tbox_predicate(&self, p: Id) -> bool {
        p == self.v.sub_class || p == self.v.sub_prop || p == self.v.domain || p == self.v.range
    }

    /// Full re-materialization from the current base: recompute the TBox closures, re-sweep
    /// every base triple, and rebuild all derivation counts. Mirrors
    /// `crate::rdfs::rdfs_closure(.., emit_dr_closure = false, MonoOwl::default())` exactly,
    /// except emissions are *counted* instead of deduplicated away.
    fn rematerialize(&mut self) {
        self.rebuilds += 1;
        let v = &self.v;
        // 1. Raw TBox maps from the base.
        let mut sc: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
        let mut sp: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
        let mut dom: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
        let mut rng: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
        for &[s, p, o] in &self.base {
            if p == v.sub_class {
                sc.entry(s).or_default().push(o);
            } else if p == v.sub_prop {
                sp.entry(s).or_default().push(o);
            } else if p == v.domain {
                dom.entry(s).or_default().push(o);
            } else if p == v.range {
                rng.entry(s).or_default().push(o);
            }
        }
        // 2. Saturate the TBox.
        self.sc_closure = transitive_closure(&sc);
        self.sp_closure = transitive_closure(&sp);
        self.dom_full = close_dr(&dom, &self.sp_closure, &self.sc_closure);
        self.rng_full = close_dr(&rng, &self.sp_closure, &self.sc_closure);
        // 3. Sweep every base triple against the closed TBox, counting emissions.
        let asserted: Vec<[Id; 3]> = self.base.iter().copied().collect();
        let emitted = sweep(
            &asserted,
            v,
            &self.sc_closure,
            &self.sp_closure,
            &self.dom_full,
            &self.rng_full,
            None,
        );
        self.counts.clear();
        for t in emitted {
            *self.counts.entry(t).or_insert(0) += 1;
        }
        // 4. TBox-closure facts (rdfs11 / rdfs5), maintained as a set of their own.
        self.schema_facts.clear();
        for (&c, ds) in &self.sc_closure {
            self.schema_facts.extend(ds.iter().map(|&d| [c, v.sub_class, d]));
        }
        for (&p, qs) in &self.sp_closure {
            self.schema_facts.extend(qs.iter().map(|&q| [p, v.sub_prop, q]));
        }
    }

    /// All one-step consequences (the exact emission multiset) of `triples` against the
    /// current closed TBox. Parallel for large deltas (same `sweep` as batch).
    fn consequences(&self, triples: &[[Id; 3]]) -> Vec<[Id; 3]> {
        sweep(
            triples,
            &self.v,
            &self.sc_closure,
            &self.sp_closure,
            &self.dom_full,
            &self.rng_full,
            None,
        )
    }

    /// Insert base triples; the closure is updated incrementally (one sweep over the delta).
    /// Triples already asserted are ignored (set semantics). Returns the number of triples
    /// actually added to the base.
    ///
    /// If any *newly added* triple is a TBox triple (`subClassOf` / `subPropertyOf` /
    /// `domain` / `range`), v1 falls back to full re-materialization (see module docs).
    pub fn insert(&mut self, triples: &[[Id; 3]]) -> usize {
        let mut added: Vec<[Id; 3]> = Vec::with_capacity(triples.len());
        let mut tbox = false;
        for &t in triples {
            if self.base.insert(t) {
                tbox |= self.is_tbox_predicate(t[1]);
                added.push(t);
            }
        }
        if added.is_empty() {
            return 0;
        }
        if tbox {
            self.rematerialize();
            return added.len();
        }
        for t in self.consequences(&added) {
            *self.counts.entry(t).or_insert(0) += 1;
        }
        added.len()
    }

    /// Delete base triples; the closure is updated incrementally by *decrementing* the exact
    /// derivation counts of the delta's consequences — derived facts disappear precisely when
    /// their last support goes (no overdelete/rederive pass needed; see module docs). Triples
    /// not currently asserted — including derived-only facts — are ignored. Returns the number
    /// of triples actually removed from the base.
    ///
    /// TBox deletions fall back to full re-materialization (v1), like [`insert`](Self::insert).
    pub fn delete(&mut self, triples: &[[Id; 3]]) -> usize {
        let mut removed: Vec<[Id; 3]> = Vec::with_capacity(triples.len());
        let mut tbox = false;
        for t in triples {
            if self.base.remove(t) {
                tbox |= self.is_tbox_predicate(t[1]);
                removed.push(*t);
            }
        }
        if removed.is_empty() {
            return 0;
        }
        if tbox {
            self.rematerialize();
            return removed.len();
        }
        for t in self.consequences(&removed) {
            match self.counts.get_mut(&t) {
                Some(c) if *c > 1 => *c -= 1,
                Some(_) => {
                    self.counts.remove(&t);
                }
                None => debug_assert!(
                    false,
                    "derivation-count underflow: delete decremented a consequence that was \
                     never counted (TBox drifted without a rebuild?)"
                ),
            }
        }
        removed.len()
    }

    /// Is `t` in the materialized closure (asserted or derived)?
    pub fn contains(&self, t: &[Id; 3]) -> bool {
        self.base.contains(t) || self.counts.contains_key(t) || self.schema_facts.contains(t)
    }

    /// The full materialized closure (base ∪ derived), deduplicated and sorted — identical as
    /// a set to running [`crate::materialize_rdfs`] from scratch on the current base. The
    /// caller can rebuild a `Graph`/index from this; live integration with the T17 store
    /// overlay is a documented follow-up.
    pub fn closure(&self) -> Vec<[Id; 3]> {
        let mut set: FxHashSet<[Id; 3]> = self.base.clone();
        set.extend(self.counts.keys().copied());
        set.extend(self.schema_facts.iter().copied());
        let mut out: Vec<[Id; 3]> = set.into_iter().collect();
        out.sort_unstable();
        out
    }

    /// Iterate the closure without materializing a `Vec` (unsorted, deduplicated).
    pub fn iter(&self) -> impl Iterator<Item = [Id; 3]> + '_ {
        self.base
            .iter()
            .copied()
            .chain(self.counts.keys().copied().filter(|t| !self.base.contains(t)))
            .chain(
                self.schema_facts
                    .iter()
                    .copied()
                    .filter(|t| !self.base.contains(t) && !self.counts.contains_key(t)),
            )
    }

    /// Number of triples in the closure.
    pub fn len(&self) -> usize {
        self.iter().count()
    }

    /// Is the closure empty?
    pub fn is_empty(&self) -> bool {
        self.base.is_empty() && self.counts.is_empty() && self.schema_facts.is_empty()
    }

    /// Number of *asserted* (base) triples.
    pub fn base_len(&self) -> usize {
        self.base.len()
    }

    /// Iterate the *asserted* (base) triples (unsorted).
    pub fn base_triples(&self) -> impl Iterator<Item = [Id; 3]> + '_ {
        self.base.iter().copied()
    }

    /// How many times a mutation fell back to full re-materialization (TBox changes; v1).
    pub fn full_rebuilds(&self) -> usize {
        self.rebuilds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materialize_rdfs;
    use oxrdf::vocab::{rdf, rdfs};

    struct V {
        ty: Id,
        sc: Id,
        sp: Id,
        dom: Id,
        rng: Id,
    }

    fn vocab(dict: &mut Dict) -> V {
        V {
            ty: dict.intern_iri(rdf::TYPE.as_str()),
            sc: dict.intern_iri(rdfs::SUB_CLASS_OF.as_str()),
            sp: dict.intern_iri(rdfs::SUB_PROPERTY_OF.as_str()),
            dom: dict.intern_iri(rdfs::DOMAIN.as_str()),
            rng: dict.intern_iri(rdfs::RANGE.as_str()),
        }
    }

    fn ex(dict: &mut Dict, local: &str) -> Id {
        dict.intern_iri(&format!("http://ex/{local}"))
    }

    /// Oracle: from-scratch closure of `base` as a set, via the batch API.
    fn oracle(dict: &mut Dict, base: &FxHashSet<[Id; 3]>) -> FxHashSet<[Id; 3]> {
        let mut v: Vec<[Id; 3]> = base.iter().copied().collect();
        materialize_rdfs(dict, &mut v);
        v.into_iter().collect()
    }

    fn assert_matches_oracle(g: &MaterializedGraph, dict: &mut Dict, base: &FxHashSet<[Id; 3]>) {
        let inc: FxHashSet<[Id; 3]> = g.closure().into_iter().collect();
        let full = oracle(dict, base);
        assert_eq!(inc, full, "incremental closure != from-scratch closure");
        assert_eq!(g.len(), inc.len(), "len() inconsistent with closure()");
    }

    /// Small mixed TBox+ABox: Dog ⊑ Mammal ⊑ Animal; hasParent ⊑ relatedTo with
    /// domain/range Person; rex a Dog; a hasParent b.
    fn fixture(dict: &mut Dict) -> (V, Vec<[Id; 3]>) {
        let v = vocab(dict);
        let (dog, mammal, animal) = (ex(dict, "Dog"), ex(dict, "Mammal"), ex(dict, "Animal"));
        let (hp, rel, person) = (ex(dict, "hasParent"), ex(dict, "relatedTo"), ex(dict, "Person"));
        let (rex, a, b) = (ex(dict, "rex"), ex(dict, "a"), ex(dict, "b"));
        let base = vec![
            [dog, v.sc, mammal],
            [mammal, v.sc, animal],
            [hp, v.sp, rel],
            [hp, v.dom, person],
            [hp, v.rng, person],
            [rex, v.ty, dog],
            [a, hp, b],
        ];
        (v, base)
    }

    #[test]
    fn initial_materialization_matches_batch() {
        let mut dict = Dict::new();
        let (_, base) = fixture(&mut dict);
        let g = MaterializedGraph::new(&mut dict, &base);
        let set: FxHashSet<[Id; 3]> = base.iter().copied().collect();
        assert_matches_oracle(&g, &mut dict, &set);
        assert_eq!(g.full_rebuilds(), 0, "new() is not a fallback rebuild");
        assert_eq!(g.base_len(), base.len());
    }

    #[test]
    fn abox_insert_is_incremental_and_correct() {
        let mut dict = Dict::new();
        let (v, base) = fixture(&mut dict);
        let (dog, hp) = (ex(&mut dict, "Dog"), ex(&mut dict, "hasParent"));
        let (fido, c, d) = (ex(&mut dict, "fido"), ex(&mut dict, "c"), ex(&mut dict, "d"));
        let mut g = MaterializedGraph::new(&mut dict, &base);
        let mut set: FxHashSet<[Id; 3]> = base.iter().copied().collect();

        let delta = [[fido, v.ty, dog], [c, hp, d]];
        assert_eq!(g.insert(&delta), 2);
        set.extend(delta);
        assert_eq!(g.full_rebuilds(), 0, "ABox insert must not rebuild");
        assert_matches_oracle(&g, &mut dict, &set);
        // Spot-check entailments: fido is an Animal (rdfs9 chain); c is a Person (rdfs2).
        let (animal, person, ty) = (ex(&mut dict, "Animal"), ex(&mut dict, "Person"), v.ty);
        assert!(g.contains(&[fido, ty, animal]));
        assert!(g.contains(&[c, ty, person]));
    }

    #[test]
    fn abox_delete_is_incremental_and_correct() {
        let mut dict = Dict::new();
        let (v, base) = fixture(&mut dict);
        let (rex, dog, animal) = (ex(&mut dict, "rex"), ex(&mut dict, "Dog"), ex(&mut dict, "Animal"));
        let mut g = MaterializedGraph::new(&mut dict, &base);
        let mut set: FxHashSet<[Id; 3]> = base.iter().copied().collect();

        assert!(g.contains(&[rex, v.ty, animal]));
        assert_eq!(g.delete(&[[rex, v.ty, dog]]), 1);
        set.remove(&[rex, v.ty, dog]);
        assert_eq!(g.full_rebuilds(), 0, "ABox delete must not rebuild");
        assert!(!g.contains(&[rex, v.ty, animal]), "derived types retract with their support");
        assert_matches_oracle(&g, &mut dict, &set);
    }

    #[test]
    fn deleted_base_fact_survives_if_still_derivable() {
        // (a relatedTo b) asserted AND derived from (a hasParent b) via rdfs7. Deleting the
        // assertion must keep it (count > 0); deleting (a hasParent b) too must drop it.
        // This is exactly the case DRed needs its rederivation pass for.
        let mut dict = Dict::new();
        let (v, mut base) = fixture(&mut dict);
        let (a, b, hp, rel) =
            (ex(&mut dict, "a"), ex(&mut dict, "b"), ex(&mut dict, "hasParent"), ex(&mut dict, "relatedTo"));
        base.push([a, rel, b]); // assert what rdfs7 also derives
        let mut g = MaterializedGraph::new(&mut dict, &base);
        let mut set: FxHashSet<[Id; 3]> = base.iter().copied().collect();

        g.delete(&[[a, rel, b]]);
        set.remove(&[a, rel, b]);
        assert!(g.contains(&[a, rel, b]), "still derived from (a hasParent b)");
        assert_matches_oracle(&g, &mut dict, &set);

        g.delete(&[[a, hp, b]]);
        set.remove(&[a, hp, b]);
        assert!(!g.contains(&[a, rel, b]), "last support gone");
        let person = ex(&mut dict, "Person");
        assert!(!g.contains(&[a, v.ty, person]), "domain typing gone too");
        assert_matches_oracle(&g, &mut dict, &set);
        assert_eq!(g.full_rebuilds(), 0);
    }

    #[test]
    fn multi_support_counting() {
        // Two assertions both derive (s type Person) via hasParent's domain; the fact must
        // survive losing one support and die with the second.
        let mut dict = Dict::new();
        let (v, base) = fixture(&mut dict);
        let (s, o1, o2, hp, person) = (
            ex(&mut dict, "s"),
            ex(&mut dict, "o1"),
            ex(&mut dict, "o2"),
            ex(&mut dict, "hasParent"),
            ex(&mut dict, "Person"),
        );
        let mut g = MaterializedGraph::new(&mut dict, &base);
        let mut set: FxHashSet<[Id; 3]> = base.iter().copied().collect();
        g.insert(&[[s, hp, o1], [s, hp, o2]]);
        set.extend([[s, hp, o1], [s, hp, o2]]);
        assert!(g.contains(&[s, v.ty, person]));

        g.delete(&[[s, hp, o1]]);
        set.remove(&[s, hp, o1]);
        assert!(g.contains(&[s, v.ty, person]), "second derivation still supports it");
        assert_matches_oracle(&g, &mut dict, &set);

        g.delete(&[[s, hp, o2]]);
        set.remove(&[s, hp, o2]);
        assert!(!g.contains(&[s, v.ty, person]), "count reached zero");
        assert_matches_oracle(&g, &mut dict, &set);
    }

    #[test]
    fn insert_of_already_derived_fact_then_unwind() {
        // Insert (rex type Mammal), which is already derived. It must remain after the
        // deriving support (rex type Dog) goes, and only vanish when deleted itself.
        let mut dict = Dict::new();
        let (v, base) = fixture(&mut dict);
        let (rex, dog, mammal) = (ex(&mut dict, "rex"), ex(&mut dict, "Dog"), ex(&mut dict, "Mammal"));
        let mut g = MaterializedGraph::new(&mut dict, &base);
        let mut set: FxHashSet<[Id; 3]> = base.iter().copied().collect();

        g.insert(&[[rex, v.ty, mammal]]);
        set.insert([rex, v.ty, mammal]);
        assert_matches_oracle(&g, &mut dict, &set);

        g.delete(&[[rex, v.ty, dog]]);
        set.remove(&[rex, v.ty, dog]);
        assert!(g.contains(&[rex, v.ty, mammal]), "asserted, even though no longer derived");
        assert_matches_oracle(&g, &mut dict, &set);

        g.delete(&[[rex, v.ty, mammal]]);
        set.remove(&[rex, v.ty, mammal]);
        assert!(!g.contains(&[rex, v.ty, mammal]));
        assert_matches_oracle(&g, &mut dict, &set);
    }

    #[test]
    fn noop_mutations() {
        let mut dict = Dict::new();
        let (v, base) = fixture(&mut dict);
        let (rex, dog, animal, ghost) =
            (ex(&mut dict, "rex"), ex(&mut dict, "Dog"), ex(&mut dict, "Animal"), ex(&mut dict, "ghost"));
        let mut g = MaterializedGraph::new(&mut dict, &base);
        let set: FxHashSet<[Id; 3]> = base.iter().copied().collect();

        assert_eq!(g.insert(&[[rex, v.ty, dog]]), 0, "duplicate insert is a no-op");
        assert_eq!(g.delete(&[[ghost, v.ty, dog]]), 0, "deleting absent triple is a no-op");
        assert_eq!(g.delete(&[[rex, v.ty, animal]]), 0, "derived-only facts cannot be deleted");
        assert!(g.contains(&[rex, v.ty, animal]));
        assert_matches_oracle(&g, &mut dict, &set);
        assert_eq!(g.full_rebuilds(), 0);
    }

    #[test]
    fn tbox_insert_triggers_rebuild_and_is_correct() {
        let mut dict = Dict::new();
        let (v, base) = fixture(&mut dict);
        let (animal, thing, rex) = (ex(&mut dict, "Animal"), ex(&mut dict, "Thing"), ex(&mut dict, "rex"));
        let mut g = MaterializedGraph::new(&mut dict, &base);
        let mut set: FxHashSet<[Id; 3]> = base.iter().copied().collect();

        g.insert(&[[animal, v.sc, thing]]);
        set.insert([animal, v.sc, thing]);
        assert_eq!(g.full_rebuilds(), 1, "TBox insert must fall back to full rematerialization");
        assert!(g.contains(&[rex, v.ty, thing]), "existing ABox re-swept against new TBox");
        assert_matches_oracle(&g, &mut dict, &set);
    }

    #[test]
    fn tbox_delete_triggers_rebuild_and_is_correct() {
        let mut dict = Dict::new();
        let (v, base) = fixture(&mut dict);
        let (mammal, animal, rex) = (ex(&mut dict, "Mammal"), ex(&mut dict, "Animal"), ex(&mut dict, "rex"));
        let mut g = MaterializedGraph::new(&mut dict, &base);
        let mut set: FxHashSet<[Id; 3]> = base.iter().copied().collect();

        g.delete(&[[mammal, v.sc, animal]]);
        set.remove(&[mammal, v.sc, animal]);
        assert_eq!(g.full_rebuilds(), 1, "TBox delete must fall back to full rematerialization");
        assert!(!g.contains(&[rex, v.ty, animal]), "entailments through the removed edge gone");
        assert_matches_oracle(&g, &mut dict, &set);

        // Mixed batch (ABox + TBox) also rebuilds, once.
        let (dog, mammal2, fido) = (ex(&mut dict, "Dog"), ex(&mut dict, "Mammal"), ex(&mut dict, "fido"));
        g.insert(&[[fido, v.ty, dog], [mammal2, v.sc, animal]]);
        set.extend([[fido, v.ty, dog], [mammal2, v.sc, animal]]);
        assert_eq!(g.full_rebuilds(), 2);
        assert_matches_oracle(&g, &mut dict, &set);
    }

    #[test]
    fn empty_graph() {
        let mut dict = Dict::new();
        let g = MaterializedGraph::new(&mut dict, &[]);
        assert!(g.is_empty());
        assert_eq!(g.closure(), Vec::<[Id; 3]>::new());
        assert_eq!(g.len(), 0);
    }
}
