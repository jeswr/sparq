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
#[cfg(feature = "rbox")]
use crate::rbox::RoleBox;
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

/// Runs CR1–CR5 (and, under the `rbox` feature, CR10/CR11 role saturation) to a fixpoint over
/// `axioms` for `n` concepts (the dense count from [`Names`]). Returns the saturated
/// [`Saturation`]; `S[c]` then holds every concept subsuming `c`.
#[cfg(feature = "rbox")]
pub fn saturate(axioms: &[Normal], n: usize, role_box: &RoleBox) -> Saturation {
    saturate_inner(axioms, n, role_box)
}

/// Runs CR1–CR5 to a fixpoint (no role hierarchy — roles are compared for equality only). The
/// E1 entry point; the `rbox` build uses the [`RoleBox`]-aware overload above.
#[cfg(not(feature = "rbox"))]
pub fn saturate(axioms: &[Normal], n: usize) -> Saturation {
    saturate_inner(axioms, n)
}

/// The shared CR1–CR5 (+CR10/CR11 under `rbox`) fixpoint. The `role_box` argument exists only
/// in the `rbox` build; CR3 link insertion routes through it so every asserted existential link
/// is closed under role inclusion + composition before CR4/CR5 fire.
fn saturate_inner(
    axioms: &[Normal],
    n: usize,
    #[cfg(feature = "rbox")] role_box: &RoleBox,
) -> Saturation {
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
        // CR3: every `D ⊑ ∃r.F` axiom adds the link (X, F) to R(r). Under `rbox` the link is
        // closed under CR10 (role inclusion) + CR11 (composition) before CR4/CR5 fire.
        if let Some(links) = ix.exists.get(&d) {
            for &(r, f) in links {
                #[cfg(feature = "rbox")]
                add_link_rbox(&mut sat, r, x, f, &ix, role_box, &mut queue);
                #[cfg(not(feature = "rbox"))]
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
/// pre-existing `⊥ ∈ S(f)` yields `⊥ ∈ S(x)` (CR5). Returns `true` iff the link was new (so the
/// `rbox` caller can decide whether to close it under CR10/CR11).
fn add_link(
    sat: &mut Saturation,
    r: Role,
    x: Concept,
    f: Concept,
    ix: &AxiomIndex,
    queue: &mut Vec<(Concept, Concept)>,
) -> bool {
    let succ = sat.r_succ.entry(r).or_default().entry(x).or_default();
    if !succ.insert(f) {
        return false; // link already present.
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
    true
}

/// Adds the asserted existential link `(x, f) ∈ R(r)` (CR3) and closes it under the RBox role
/// rules to a fixpoint via a link worklist, calling [`add_link`] (which fires CR4/CR5) for every
/// derived link:
///
///   CR10  r ⊑* s          ⇒  (x, f) ∈ R(s)                      for each super-role s of r
///   CR11  r ∘ r2 ⊑ s, (f, z) ∈ R(r2)  ⇒  (x, z) ∈ R(s)         (new link is the FIRST component)
///         r1 ∘ r ⊑ s, (w, x) ∈ R(r1)  ⇒  (w, f) ∈ R(s)         (new link is the SECOND component)
///
/// Newly-derived links are pushed back onto the worklist, so composed links compose again and
/// super-roles of derived roles also propagate — the standard RBox link-saturation. The worklist
/// is bounded by the number of distinct `(role, x, f)` triples (each is stored at most once in
/// `r_succ`), so the fixpoint terminates.
#[cfg(feature = "rbox")]
fn add_link_rbox(
    sat: &mut Saturation,
    r: Role,
    x: Concept,
    f: Concept,
    ix: &AxiomIndex,
    role_box: &RoleBox,
    queue: &mut Vec<(Concept, Concept)>,
) {
    // Each work item is a link to STORE (already at its exact role). Seed with the CR3 link.
    let mut links: Vec<(Role, Concept, Concept)> = vec![(r, x, f)];
    while let Some((lr, lx, lf)) = links.pop() {
        if !add_link(sat, lr, lx, lf, ix, queue) {
            continue; // link already present — its consequences were already enqueued.
        }
        // CR10: propagate to every strict super-role of `lr` (super_roles includes lr itself).
        for &sup in role_box.super_roles(lr) {
            if sup != lr {
                links.push((sup, lx, lf));
            }
        }
        // CR11 (only when the TBox has any composition axiom — a pure role hierarchy skips
        // every probe). New link as the FIRST component: lr ∘ r2 ⊑ s with (lf, z) ∈ R(r2)
        // ⇒ (lx, z) ∈ R(s); and as the SECOND: r1 ∘ lr ⊑ s with (w, lx) ∈ R(r1) ⇒ (w, lf) ∈ R(s).
        if role_box.has_compositions() {
            for &(r2, s) in role_box.compositions_first(lr) {
                if let Some(z_set) = sat.r_succ.get(&r2).and_then(|m| m.get(&lf)) {
                    let zs: Vec<Concept> = z_set.iter().copied().collect();
                    for z in zs {
                        links.push((s, lx, z));
                    }
                }
            }
            for &(r1, s) in role_box.compositions_second(lr) {
                if let Some(w_set) = sat.r_pred.get(&r1).and_then(|m| m.get(&lx)) {
                    let ws: Vec<Concept> = w_set.iter().copied().collect();
                    for w in ws {
                        links.push((s, w, lf));
                    }
                }
            }
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
