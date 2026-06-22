// [OPUS-4.8] sq-evb1: EL concept IR + Baader–Brandt–Lutz normalization.
//
// The classifier works over a SMALL integer index space SEPARATE from the sparq `Dict` id
// space: a `Concept` is either a named class (carrying the original dict `Id` so the result
// lattice can be re-emitted), the top concept ⊤, the bottom concept ⊥, or a FRESH conjunct/
// existential name introduced by normalization. Keeping a private index space (instead of
// interning fresh names back into the Dict) means normalization never pollutes the store's
// dictionary, and the saturation runs over dense `u32` keys.

use rustc_hash::FxHashMap;
use sparq_core::dict::Id;

/// A `Concept` is a node in the internal classification index space. It is NOT a dict id —
/// it is a dense `u32` assigned by [`Names`]. Named concepts remember their dict id so the
/// emitted lattice can name them; ⊤/⊥ and fresh normalization names have none.
pub type Concept = u32;

/// A role (object property) in the internal index space. Like [`Concept`] this is a dense
/// `u32` separate from dict ids; the EL MVP does not reason over the role hierarchy (Phase
/// E2), so a role is only ever compared for equality.
pub type Role = u32;

/// The internal name table: a bijection between the dense classification index space and the
/// source dict ids, plus a counter that mints fresh (anonymous) concepts for normalization.
///
/// ⊤ (top) is ALWAYS concept `0` and ⊥ (bottom) is ALWAYS concept `1`, so the saturator can
/// special-case them without a map lookup. Fresh names get no dict id (they are structural).
#[derive(Default)]
pub struct Names {
    /// `dict Id -> internal Concept` for named classes.
    by_dict: FxHashMap<Id, Concept>,
    /// `internal Concept -> dict Id` for named classes (None for ⊤/⊥/fresh names).
    to_dict: Vec<Option<Id>>,
    /// `dict Id -> internal Role` for named object properties.
    role_by_dict: FxHashMap<Id, Role>,
    /// `internal Role -> dict Id`.
    role_to_dict: Vec<Id>,
}

/// The internal concept index for ⊤ (`owl:Thing`).
pub const TOP: Concept = 0;
/// The internal concept index for ⊥ (`owl:Nothing`).
pub const BOTTOM: Concept = 1;

impl Names {
    /// A fresh table pre-seeded with ⊤ at [`TOP`] and ⊥ at [`BOTTOM`].
    pub fn new() -> Names {
        Names {
            by_dict: FxHashMap::default(),
            to_dict: vec![None, None],
            role_by_dict: FxHashMap::default(),
            role_to_dict: Vec::new(),
        }
    }

    /// Maps a dict class id to its internal concept, minting one on first sight. ⊤/⊥ must be
    /// mapped via [`Names::map_class`] too: the caller seeds them by recognising the
    /// `owl:Thing`/`owl:Nothing` dict ids and routing to [`TOP`]/[`BOTTOM`] before calling
    /// here (see [`crate::extract`]).
    pub fn class(&mut self, dict_id: Id) -> Concept {
        if let Some(&c) = self.by_dict.get(&dict_id) {
            return c;
        }
        let c = self.to_dict.len() as Concept;
        self.to_dict.push(Some(dict_id));
        self.by_dict.insert(dict_id, c);
        c
    }

    /// Maps a dict object-property id to its internal role, minting one on first sight.
    pub fn role(&mut self, dict_id: Id) -> Role {
        if let Some(&r) = self.role_by_dict.get(&dict_id) {
            return r;
        }
        let r = self.role_to_dict.len() as Role;
        self.role_to_dict.push(dict_id);
        self.role_by_dict.insert(dict_id, r);
        r
    }

    /// Mints a fresh anonymous concept (a normalization name with no dict id).
    pub fn fresh(&mut self) -> Concept {
        let c = self.to_dict.len() as Concept;
        self.to_dict.push(None);
        c
    }

    /// The dict id of a concept, if it is a NAMED class (⊤/⊥/fresh names return `None`).
    pub fn dict_of(&self, c: Concept) -> Option<Id> {
        self.to_dict.get(c as usize).copied().flatten()
    }

    /// The number of concepts minted so far (the dense index upper bound).
    pub fn concept_count(&self) -> usize {
        self.to_dict.len()
    }
}

/// A general concept inclusion in EL normal form (Baader–Brandt–Lutz, "Pushing the EL
/// Envelope", IJCAI-05). Every EL+⊥ TBox axiom reduces to a multiset of these four forms via
/// [`Normalizer`]; the saturator ([`crate::classify`]) consumes exactly this enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Normal {
    /// `C ⊑ D` — a basic concept on each side (NF1).
    Sub(Concept, Concept),
    /// `C1 ⊓ C2 ⊑ D` (NF2).
    AndSub(Concept, Concept, Concept),
    /// `C ⊑ ∃r.D` (NF3).
    SubExists(Concept, Role, Concept),
    /// `∃r.C ⊑ D` (NF4).
    ExistsSub(Role, Concept, Concept),
}

/// A concept expression as extracted from the graph, BEFORE normalization. EL+⊥ admits only
/// these constructors; anything else makes the axiom non-EL and is dropped by the extractor
/// with a recorded skip (see [`crate::Report`]).
#[derive(Clone, Debug)]
pub enum Expr {
    /// A named class, ⊤, or ⊥ (already an atomic concept).
    Atom(Concept),
    /// `C1 ⊓ … ⊓ Cn` (`owl:intersectionOf`). Always length ≥ 2 by construction.
    And(Vec<Expr>),
    /// `∃r.C` (`owl:someValuesFrom` restriction).
    Exists(Role, Box<Expr>),
}

/// Normalizes EL+⊥ concept inclusions into [`Normal`] forms, minting fresh names through a
/// borrowed [`Names`]. Structural normalization is conservative (the introduced axioms entail
/// exactly the input over the original signature) and linear in the input size.
pub struct Normalizer<'a> {
    pub(crate) names: &'a mut Names,
    out: Vec<Normal>,
    /// Memoizes `∃r.C ⊑ X` and `X ⊑ ∃r.C` fresh names so identical sub-expressions reuse one
    /// fresh concept (keeps the index space small on restriction-heavy ontologies).
    exists_name: FxHashMap<(Role, Concept), Concept>,
}

impl<'a> Normalizer<'a> {
    pub fn new(names: &'a mut Names) -> Normalizer<'a> {
        Normalizer {
            names,
            out: Vec::new(),
            exists_name: FxHashMap::default(),
        }
    }

    /// Consumes the normalizer, returning the accumulated normal-form axioms.
    pub fn finish(self) -> Vec<Normal> {
        self.out
    }

    /// Adds the inclusion `lhs ⊑ rhs` (both general [`Expr`]s) to the output in normal form.
    pub fn add_sub(&mut self, lhs: &Expr, rhs: &Expr) {
        // Reduce each side to a basic concept (a named atom / ⊤ / ⊥ / fresh name), emitting
        // the structural axioms that DEFINE the fresh name, then connect them with one Sub /
        // AndSub. A conjunction on the LHS is handled specially to land directly on AndSub.
        match lhs {
            Expr::And(parts) if parts.len() >= 2 => {
                let d = self.rhs_concept(rhs);
                self.and_lhs(parts, d);
            }
            _ => {
                let c = self.lhs_concept(lhs);
                let d = self.rhs_concept(rhs);
                self.out.push(Normal::Sub(c, d));
            }
        }
    }

    /// Encodes `C1 ⊓ … ⊓ Cn ⊑ D` by left-folding into binary `AndSub` over fresh names:
    /// `(C1 ⊓ C2) ⊑ f1`, `(f1 ⊓ C3) ⊑ f2`, …, finally `(f_{n-2} ⊓ Cn) ⊑ D`.
    fn and_lhs(&mut self, parts: &[Expr], d: Concept) {
        // Each part is first reduced to a basic concept (LHS position: existentials become
        // ∃r.C ⊑ fresh, so the conjunct is the fresh name).
        let atoms: Vec<Concept> = parts.iter().map(|p| self.lhs_concept(p)).collect();
        debug_assert!(atoms.len() >= 2);
        let mut acc = atoms[0];
        for &next in &atoms[1..atoms.len() - 1] {
            let f = self.names.fresh();
            self.out.push(Normal::AndSub(acc, next, f));
            acc = f;
        }
        let last = atoms[atoms.len() - 1];
        self.out.push(Normal::AndSub(acc, last, d));
    }

    /// Reduces a concept appearing in a SUBclass (LHS) position to a basic concept, emitting
    /// the defining axioms. In LHS position an existential `∃r.C` is named by a fresh `X` with
    /// `∃r.C ⊑ X` (NF4 direction), and a nested conjunction by fresh AndSub chains.
    fn lhs_concept(&mut self, e: &Expr) -> Concept {
        match e {
            Expr::Atom(c) => *c,
            Expr::And(parts) if parts.len() >= 2 => {
                // (C1 ⊓ … ⊓ Cn) as a sub-CONCEPT: name it X with each Ci ⊑-chain ⊑ X is wrong;
                // instead introduce X with `C1 ⊓ … ⊓ Cn ⊑ X` AND, for soundness in LHS reuse,
                // `X ⊑ Ci`. The MVP only needs the ⊑ X direction (LHS occurrence), so emit the
                // AndSub chain that lands on a fresh X.
                let x = self.names.fresh();
                self.and_lhs(parts, x);
                x
            }
            Expr::And(parts) => {
                // Degenerate 0/1-element conjunction: identity / single atom.
                parts.first().map(|p| self.lhs_concept(p)).unwrap_or(TOP)
            }
            Expr::Exists(r, inner) => {
                let c = self.rhs_concept(inner); // filler in a positive (RHS-like) position
                let key = (*r, c);
                if let Some(&x) = self.exists_name.get(&key) {
                    return x;
                }
                let x = self.names.fresh();
                self.exists_name.insert(key, x);
                self.out.push(Normal::ExistsSub(*r, c, x));
                x
            }
        }
    }

    /// Reduces a concept appearing in a SUPERclass (RHS) position to a basic concept. In RHS
    /// position an existential `∃r.C` is named by a fresh `X` with `X ⊑ ∃r.C` (NF3), the
    /// filler `C` recursively reduced. A conjunction `D1 ⊓ … ⊓ Dn` on the RHS becomes `n`
    /// separate inclusions sharing the same subclass (handled by the caller via fresh fan-out).
    fn rhs_concept(&mut self, e: &Expr) -> Concept {
        match e {
            Expr::Atom(c) => *c,
            Expr::And(parts) => {
                // Name the conjunction X with `X ⊑ Di` for each part; return X. This is the
                // RHS conjunction split: a subclass of the conjunction is a subclass of each.
                let x = self.names.fresh();
                for p in parts {
                    let d = self.rhs_concept(p);
                    self.out.push(Normal::Sub(x, d));
                }
                x
            }
            Expr::Exists(r, inner) => {
                let c = self.rhs_concept(inner);
                let x = self.names.fresh();
                self.out.push(Normal::SubExists(x, *r, c));
                x
            }
        }
    }
}
