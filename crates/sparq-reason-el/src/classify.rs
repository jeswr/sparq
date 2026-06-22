// [OPUS-4.8] sq-evb1: EL+⊥ completion (CR1–CR5) over S(C) and R(r).
//
// The saturator computes the least fixpoint of the Baader–Brandt–Lutz completion rules with
// bottom (the SAME calculus ELK's core implements, restricted to the EL+⊥-minus-RBox MVP):
//
//   init   S(C) := {C, ⊤}                                     for every concept C
//   CR1    C ⊑ D ∈ T,  C ∈ S(X)            ⇒  D ∈ S(X)
//   CR2    C1 ⊓ C2 ⊑ D ∈ T,  C1,C2 ∈ S(X)  ⇒  D ∈ S(X)
//   CR3    C ⊑ ∃r.D ∈ T,  C ∈ S(X)         ⇒  (X,D) ∈ R(r)
//   CR4    ∃r.D ⊑ E ∈ T,  (X,Y) ∈ R(r),  D ∈ S(Y)   ⇒  E ∈ S(X)
//   CR5    (X,Y) ∈ R(r),  ⊥ ∈ S(Y)         ⇒  ⊥ ∈ S(X)
//
// CR4 is the load-bearing existential-traversal rule that OWL 2 RL lacks (spike §1.2): it is
// the only rule that reasons THROUGH an r-successor, and it is why running `--reason owl` over
// an EL ontology returns a silently-incomplete hierarchy. The implementation is a worklist
// saturation: a per-concept queue of newly-derived subsumers drives rule application, so each
// derived membership is processed once. Single-threaded (Phase E1; concurrency is E4).

use crate::normal::{Concept, Names, Normal, Role, BOTTOM, TOP};
use rustc_hash::{FxHashMap, FxHashSet};

/// Indexes the normal-form axioms by the premise shape each completion rule matches on, so a
/// derived membership `C ∈ S(X)` can fire the relevant rules without scanning all axioms.
#[derive(Default)]
struct AxiomIndex {
    /// CR1: `C ⊑ D`, keyed by the subclass `C`.
    sub: FxHashMap<Concept, Vec<Concept>>,
    /// CR2: `C1 ⊓ C2 ⊑ D`, keyed by EACH conjunct (with the partner + head).
    and_by_conjunct: FxHashMap<Concept, Vec<(Concept, Concept)>>,
    /// CR3: `C ⊑ ∃r.D`, keyed by the subclass `C`.
    exists: FxHashMap<Concept, Vec<(Role, Concept)>>,
    /// CR4: `∃r.D ⊑ E`, keyed by `(r, D)` (the filler that must appear in S(Y)).
    exists_sub: FxHashMap<(Role, Concept), Vec<Concept>>,
}

impl AxiomIndex {
    fn build(axioms: &[Normal]) -> AxiomIndex {
        let mut ix = AxiomIndex::default();
        for &ax in axioms {
            match ax {
                Normal::Sub(c, d) => ix.sub.entry(c).or_default().push(d),
                Normal::AndSub(c1, c2, d) => {
                    ix.and_by_conjunct.entry(c1).or_default().push((c2, d));
                    ix.and_by_conjunct.entry(c2).or_default().push((c1, d));
                }
                Normal::SubExists(c, r, d) => ix.exists.entry(c).or_default().push((r, d)),
                Normal::ExistsSub(r, c, d) => ix.exists_sub.entry((r, c)).or_default().push(d),
            }
        }
        ix
    }
}

/// The saturation state: subsumer sets `S(C)` and existential links `R(r)`.
pub struct Saturation {
    /// `S[c]` = the set of basic concepts subsuming `c` (i.e. `c ⊑ each`).
    pub s: Vec<FxHashSet<Concept>>,
    /// For each role, the predecessor index of `R(r)`: `r_pred[r][Y]` = the set of `X` with
    /// `(X, Y) ∈ R(r)`. Indexed by the SUCCESSOR `Y` because CR4/CR5 fire when something new
    /// lands in `S(Y)` and need every predecessor `X`.
    pub r_pred: FxHashMap<Role, FxHashMap<Concept, FxHashSet<Concept>>>,
    /// The forward links `R(r)`: `r_succ[r][X]` = successors `Y`. Kept so CR4 can also fire
    /// when a *new link* (not a new S-membership) is the trigger.
    pub r_succ: FxHashMap<Role, FxHashMap<Concept, FxHashSet<Concept>>>,
}

/// Runs CR1–CR5 to a fixpoint over `axioms` for `n` concepts (the dense count from [`Names`]).
/// Returns the saturated [`Saturation`]; `S[c]` then holds every concept subsuming `c`.
pub fn saturate(axioms: &[Normal], n: usize) -> Saturation {
    let ix = AxiomIndex::build(axioms);
    let mut sat = Saturation {
        s: vec![FxHashSet::default(); n],
        r_pred: FxHashMap::default(),
        r_succ: FxHashMap::default(),
    };

    // Worklist of (concept X, newly-added subsumer D) pairs: each represents "D just entered
    // S(X)". init seeds S(C) = {C, ⊤} for every concept and queues both memberships.
    let mut queue: Vec<(Concept, Concept)> = Vec::new();
    for c in 0..n as Concept {
        if sat.s[c as usize].insert(c) {
            queue.push((c, c));
        }
        if sat.s[c as usize].insert(TOP) {
            queue.push((c, TOP));
        }
    }

    while let Some((x, d)) = queue.pop() {
        // CR1: every `D ⊑ E` axiom adds E to S(X).
        if let Some(es) = ix.sub.get(&d) {
            for &e in es {
                add(&mut sat.s[x as usize], x, e, &mut queue);
            }
        }
        // CR2: every `D ⊓ C2 ⊑ E` axiom fires if C2 also already ∈ S(X).
        if let Some(parts) = ix.and_by_conjunct.get(&d) {
            for &(other, e) in parts {
                if sat.s[x as usize].contains(&other) {
                    add(&mut sat.s[x as usize], x, e, &mut queue);
                }
            }
        }
        // CR3: every `D ⊑ ∃r.F` axiom adds the link (X, F) to R(r).
        if let Some(links) = ix.exists.get(&d) {
            for &(r, f) in links {
                add_link(&mut sat, r, x, f, &ix, &mut queue);
            }
        }
        // CR4 / CR5 with the new membership `D ∈ S(X)` as the trigger, where X is the
        // SUCCESSOR `Y` of some link `(P, X) ∈ R(r)`. Collect the (predecessor, concept) work
        // FIRST (releasing the immutable `r_pred` borrow) before mutating `sat.s`.
        let mut derived: Vec<(Concept, Concept)> = Vec::new();
        for (&r, preds_by_succ) in &sat.r_pred {
            let Some(preds) = preds_by_succ.get(&x) else {
                continue;
            };
            // CR4: `∃r.D ⊑ E` and (P, X) ∈ R(r) and D ∈ S(X)  ⇒  E ∈ S(P).
            if let Some(es) = ix.exists_sub.get(&(r, d)) {
                for &p in preds {
                    for &e in es {
                        derived.push((p, e));
                    }
                }
            }
            // CR5: ⊥ ∈ S(X) and (P, X) ∈ R(r)  ⇒  ⊥ ∈ S(P).
            if d == BOTTOM {
                for &p in preds {
                    derived.push((p, BOTTOM));
                }
            }
        }
        for (p, e) in derived {
            add(&mut sat.s[p as usize], p, e, &mut queue);
        }
    }
    sat
}

/// Inserts `e` into `S(x)`, queueing `(x, e)` if it is new. The set is borrowed mutably by the
/// caller (one row of `sat.s`) so CR rules that touch a different row stay borrow-safe.
#[inline]
fn add(set: &mut FxHashSet<Concept>, x: Concept, e: Concept, queue: &mut Vec<(Concept, Concept)>) {
    if set.insert(e) {
        queue.push((x, e));
    }
}

/// Adds the existential link `(x, f) ∈ R(r)` and fires the link-triggered half of CR4/CR5:
/// for the NEW link, every `D ∈ S(f)` with an axiom `∃r.D ⊑ E` yields `E ∈ S(x)`, and a
/// pre-existing `⊥ ∈ S(f)` yields `⊥ ∈ S(x)` (CR5).
fn add_link(
    sat: &mut Saturation,
    r: Role,
    x: Concept,
    f: Concept,
    ix: &AxiomIndex,
    queue: &mut Vec<(Concept, Concept)>,
) {
    let succ = sat.r_succ.entry(r).or_default().entry(x).or_default();
    if !succ.insert(f) {
        return; // link already present.
    }
    sat.r_pred
        .entry(r)
        .or_default()
        .entry(f)
        .or_default()
        .insert(x);

    // CR4 for the new link: scan S(f) for fillers D with an `∃r.D ⊑ E` axiom.
    let s_f: Vec<Concept> = sat.s[f as usize].iter().copied().collect();
    for d in s_f {
        if let Some(es) = ix.exists_sub.get(&(r, d)) {
            for &e in es {
                add(&mut sat.s[x as usize], x, e, queue);
            }
        }
        // CR5 for the new link.
        if d == BOTTOM {
            add(&mut sat.s[x as usize], x, BOTTOM, queue);
        }
    }
}

/// The classification result: for each NAMED class, the set of NAMED super-classes it is
/// subsumed by (excluding itself, ⊤, and fresh normalization names), plus the unsatisfiable
/// (`⊑ ⊥`) named classes.
pub struct Classification {
    /// `subsumers[c]` = named concepts strictly subsuming the named concept `c`.
    pub subsumers: FxHashMap<Concept, Vec<Concept>>,
    /// Named concepts `c` with `⊥ ∈ S(c)` — i.e. `c ⊑ owl:Nothing` (unsatisfiable).
    pub unsatisfiable: Vec<Concept>,
}

/// Projects the saturation onto NAMED classes only: drops ⊤/⊥/fresh-name subsumers and
/// reflexive self-subsumption, surfacing the queryable subsumption lattice + the
/// unsatisfiable classes. A class is unsatisfiable iff `⊥ ∈ S(c)`.
pub fn classify(sat: &Saturation, names: &Names) -> Classification {
    let mut subsumers: FxHashMap<Concept, Vec<Concept>> = FxHashMap::default();
    let mut unsatisfiable = Vec::new();
    for c in 0..sat.s.len() as Concept {
        if names.dict_of(c).is_none() {
            continue; // not a named class (⊤/⊥/fresh).
        }
        let s = &sat.s[c as usize];
        if s.contains(&BOTTOM) {
            unsatisfiable.push(c);
        }
        let mut sup: Vec<Concept> = s
            .iter()
            .copied()
            .filter(|&d| d != c && d != TOP && d != BOTTOM && names.dict_of(d).is_some())
            .collect();
        sup.sort_unstable();
        if !sup.is_empty() {
            subsumers.insert(c, sup);
        }
    }
    Classification {
        subsumers,
        unsatisfiable,
    }
}
