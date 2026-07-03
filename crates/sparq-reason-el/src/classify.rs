// [OPUS-4.8] sq-evb1: EL+⊥ completion (CR1–CR5) over S(C) and R(r).
// [FABLE-5] sq-pbz04.2.1: + CR6 (safe nominals).
//
// The saturator computes the least fixpoint of the Baader–Brandt–Lutz completion rules with
// bottom (the SAME calculus ELK's core implements, restricted to the recognised fragment):
//
//   init   S(C) := {C, ⊤}                                     for every concept C
//   CR1    C ⊑ D ∈ T,  C ∈ S(X)            ⇒  D ∈ S(X)
//   CR2    C1 ⊓ C2 ⊑ D ∈ T,  C1,C2 ∈ S(X)  ⇒  D ∈ S(X)
//   CR3    C ⊑ ∃r.D ∈ T,  C ∈ S(X)         ⇒  (X,D) ∈ R(r)
//   CR4    ∃r.D ⊑ E ∈ T,  (X,Y) ∈ R(r),  D ∈ S(Y)   ⇒  E ∈ S(X)
//   CR5    (X,Y) ∈ R(r),  ⊥ ∈ S(Y)         ⇒  ⊥ ∈ S(X)
//   CR6    {a} ∈ S(X) ∩ S(Y),  X ⇝_R Y     ⇒  S(X) := S(X) ∪ S(Y)
//
// CR4 is the load-bearing existential-traversal rule that OWL 2 RL lacks (spike §1.2): it is
// the only rule that reasons THROUGH an r-successor, and it is why running `--reason owl` over
// an EL ontology returns a silently-incomplete hierarchy. The implementation is a worklist
// saturation: a per-concept queue of newly-derived subsumers drives rule application, so each
// derived membership is processed once. CR6 (the nominal rule, "Pushing the EL Envelope"
// IJCAI-05) runs as a between-fixpoint pass — see `cr6_pass` for the rule, the ⇝_R
// side-condition, and the soundness argument. Single-threaded (Phase E1; concurrency is E4).

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

/// Runs CR1–CR6 (and, under the `rbox` feature, CR10/CR11 role saturation) to a fixpoint over
/// `axioms` for the concepts of `names`. Returns the saturated [`Saturation`]; `S[c]` then
/// holds every concept subsuming `c`.
#[cfg(feature = "rbox")]
pub fn saturate(axioms: &[Normal], names: &Names, role_box: &RoleBox) -> Saturation {
    saturate_inner(axioms, names, role_box)
}

/// Runs CR1–CR6 to a fixpoint (no role hierarchy — roles are compared for equality only). The
/// E1 entry point; the `rbox` build uses the `RoleBox`-aware overload above.
#[cfg(not(feature = "rbox"))]
pub fn saturate(axioms: &[Normal], names: &Names) -> Saturation {
    saturate_inner(axioms, names)
}

/// The shared CR1–CR5 (+CR10/CR11 under `rbox`) worklist fixpoint, alternated with the CR6
/// nominal pass (sq-pbz04.2.1) until neither derives anything new. The `role_box` argument
/// exists only in the `rbox` build; CR3 link insertion routes through it so every asserted
/// existential link is closed under role inclusion + composition before CR4/CR5 fire.
///
/// All rules are monotone (S-sets and R-links only ever grow, bounded by the concept count),
/// so the alternation terminates and reaches the SAME least fixpoint regardless of rule
/// order — classification stays deterministic. On a nominal-free ontology `cr6_pass` returns
/// `false` on its first O(1) check, so the loop runs the CR1–CR5 worklist exactly once —
/// byte-for-byte the pre-CR6 behaviour and cost.
fn saturate_inner(
    axioms: &[Normal],
    names: &Names,
    #[cfg(feature = "rbox")] role_box: &RoleBox,
) -> Saturation {
    let ix = AxiomIndex::build(axioms);
    let n = names.concept_count();
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

    loop {
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
            // CR3: every `D ⊑ ∃r.F` axiom adds the link (X, F) to R(r). Under `rbox` the link
            // is closed under CR10 (role inclusion) + CR11 (composition) before CR4/CR5 fire.
            if let Some(links) = ix.exists.get(&d) {
                for &(r, f) in links {
                    #[cfg(feature = "rbox")]
                    add_link_rbox(&mut sat, r, x, f, &ix, role_box, &mut queue);
                    #[cfg(not(feature = "rbox"))]
                    add_link(&mut sat, r, x, f, &ix, &mut queue);
                }
            }
            // CR4 / CR5 with the new membership `D ∈ S(X)` as the trigger, where X is the
            // SUCCESSOR `Y` of some link `(P, X) ∈ R(r)`. Collect the (predecessor, concept)
            // work FIRST (releasing the immutable `r_pred` borrow) before mutating `sat.s`.
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
        // CR6 (safe nominals): merges may enqueue new memberships, which can in turn create
        // new links / reachability — re-run the worklist, then re-check, until neither moves.
        if !cr6_pass(&mut sat, names, &mut queue) {
            break;
        }
    }
    sat
}

/// [FABLE-5] sq-pbz04.2.1 — CR6, the safe-nominal completion rule (Baader–Brandt–Lutz,
/// "Pushing the EL Envelope", IJCAI-05):
///
/// ```text
///   CR6   {a} ∈ S(X) ∩ S(Y),  X ⇝_R Y   ⇒   S(X) := S(X) ∪ S(Y)
/// ```
///
/// where the side-condition `X ⇝_R Y` holds iff Y is reachable from X — or from some nominal
/// `{b}` — via the (role-erased) link graph `∪_r R(r)`, taking reachability reflexively (so
/// `Y = X`, or Y itself a nominal, trivially qualify).
///
/// SOUNDNESS (the load-bearing invariant; the side-condition is exactly what makes the merge
/// valid). The saturation maintains: `D ∈ S(C)` ⟹ `T ⊨ C ⊑ D`, and `(C,D) ∈ R(r)` ⟹
/// `T ⊨ C ⊑ ∃r.D`. In any model, an R-path start being NON-EMPTY forces every set on the
/// path non-empty (each element needs a successor), and a nominal `{b}` is ALWAYS non-empty.
/// So for a firing pair (X, Y): if Y is nominal-rooted, Y is non-empty outright; if Y is
/// reached from X, either X is empty (then `X ⊑ E` holds vacuously for every E) or Y is
/// non-empty. A non-empty Y with `Y ⊑ {a}` is EXACTLY `{a}`; with `X ⊑ {a}` that gives
/// `X ⊆ {a} = Y ⊆ E` for every `E ∈ S(Y)` — so every merged membership is entailed. WITHOUT
/// the side-condition the merge would be UNSOUND: from `X ⊑ {a}` and `Y ⊑ {a}` alone, Y may
/// be empty while X is not, and `X ⊑ E` need not hold (pinned by a negative test).
///
/// COMPLETENESS (honest boundary): this is the classic reachability-based rule — complete for
/// the profile's typical "safe" nominal usage (`hasValue` values, singleton `oneOf` classes)
/// but NOT claimed complete for every EL++ nominal interplay: the ELK line of work (Kazakov &
/// Krötzsch, KR 2012) showed unrestricted nominal interaction needs a stronger calculus.
/// Missing consequences are an incompleteness, never an unsound derivation. Likewise a
/// nominal clash (`⊥ ∈ S({a})`) propagates ⊥ only to concepts CR6 relates to `{a}` — a
/// global-inconsistency verdict (everything entailed) is deliberately out of scope here.
///
/// Runs AFTER the CR1–CR5 worklist drains: one pass over the current saturation, applying
/// every firing merge and queueing new memberships. Returns `true` iff anything was added
/// (the caller then re-runs the worklist and re-checks). Cost on nominal-free input: one
/// `has_nominals` check. With nominals: one scan of the S-sets plus per-candidate BFS over
/// the link graph — fine at the scale nominals occur; revisit if a nominal-heavy corpus shows
/// up.
fn cr6_pass(sat: &mut Saturation, names: &Names, queue: &mut Vec<(Concept, Concept)>) -> bool {
    if !names.has_nominals() {
        return false;
    }
    // Group the concepts whose S-set holds a given nominal: candidates[{a}] = every X with
    // {a} ∈ S(X). Only these can pair in CR6.
    let mut candidates: FxHashMap<Concept, Vec<Concept>> = FxHashMap::default();
    for (x, s) in sat.s.iter().enumerate() {
        for &m in s {
            if names.is_nominal(m) {
                candidates.entry(m).or_default().push(x as Concept);
            }
        }
    }
    // Role-erased successor adjacency of ∪_r R(r), for the ⇝_R reachability side-condition.
    let mut adj: FxHashMap<Concept, FxHashSet<Concept>> = FxHashMap::default();
    for succ_by_x in sat.r_succ.values() {
        for (&x, succs) in succ_by_x {
            adj.entry(x).or_default().extend(succs.iter().copied());
        }
    }
    let bfs = |roots: &[Concept]| -> FxHashSet<Concept> {
        let mut seen: FxHashSet<Concept> = roots.iter().copied().collect();
        let mut stack: Vec<Concept> = roots.to_vec();
        while let Some(c) = stack.pop() {
            if let Some(nexts) = adj.get(&c) {
                for &nx in nexts {
                    if seen.insert(nx) {
                        stack.push(nx);
                    }
                }
            }
        }
        seen
    };
    // Concepts provably non-empty regardless of context: reachable from a nominal root
    // (reflexively — every nominal is its own root).
    let nominal_rooted = bfs(names.nominal_concepts());

    // Fire every merge the current saturation justifies. Deterministic iteration is not
    // required for the RESULT (the least fixpoint is order-independent), only termination:
    // each firing merge strictly grows an S-set.
    let mut changed = false;
    let mut reach_from: FxHashMap<Concept, FxHashSet<Concept>> = FxHashMap::default();
    let mut mates: Vec<Concept> = Vec::new();
    for xs in candidates.values() {
        for &x in xs {
            mates.clear();
            mates.extend(xs.iter().copied().filter(|&y| {
                y != x
                    && (nominal_rooted.contains(&y)
                        || reach_from
                            .entry(x)
                            .or_insert_with(|| bfs(&[x]))
                            .contains(&y))
            }));
            for &y in &mates {
                // S(X) := S(X) ∪ S(Y). Collect first: two rows of `sat.s` cannot be borrowed
                // mutably at once, and Y's row is only read.
                let extra: Vec<Concept> = sat.s[y as usize]
                    .difference(&sat.s[x as usize])
                    .copied()
                    .collect();
                for e in extra {
                    add(&mut sat.s[x as usize], x, e, queue);
                    changed = true;
                }
            }
        }
    }
    changed
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
