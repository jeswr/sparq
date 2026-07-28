//! L3 (part 1) — negation normal form + the `sub(O)` subexpression closure.
//!
//! 🤖 SPARQ agent [FABLE-5]. Bead sq-pbz04.4.3 (epic sq-pbz04.4, design record
//! `research/owl2-direct-semantics-scoping.md` §3). This module lowers a
//! [`crate::model::ClassExpression`] into **negation normal form** (NNF) and builds the
//! finite **subexpression closure** the ALCH tableau (`crate::tableau`) expands over.
//!
//! # Negation normal form
//! A class expression is in NNF when negation ([`ClassExpression::ObjectComplementOf`])
//! applies **only to named classes** ([`ClassExpression::Class`]). Every ALCH class
//! expression has a logically **equivalent** NNF, obtained by the standard rewrites
//! (see Baader & Sattler, *An Overview of Tableau Algorithms for Description Logics*,
//! Studia Logica 69, 2001, §2):
//!
//! ```text
//! ¬⊤ ⇝ ⊥                      ¬⊥ ⇝ ⊤                      ¬¬C ⇝ C
//! ¬(C₁ ⊓ … ⊓ Cₙ) ⇝ ¬C₁ ⊔ … ⊔ ¬Cₙ      (De Morgan)
//! ¬(C₁ ⊔ … ⊔ Cₙ) ⇝ ¬C₁ ⊓ … ⊓ ¬Cₙ      (De Morgan)
//! ¬(∃R.C) ⇝ ∀R.¬C             ¬(∀R.C) ⇝ ∃R.¬C             (quantifier duality)
//! ```
//!
//! Each rewrite is a classical logical equivalence under the OWL 2 Direct Semantics
//! (which interprets the ALCH constructors exactly as classical ALC does), so
//! `nnf(C) ≡ C` and `nnf_complement(C) ≡ ¬C` in every interpretation. NNF does not
//! duplicate subterms — every input operator is visited once and mapped to exactly one
//! output operator — so the output size is linear in the input size.
//!
//! Why the tableau wants NNF: with negation confined to named classes, a **clash** is
//! detectable purely locally as `{A, ¬A}` in one node label (or `⊥` in a label), and the
//! completion rules need no rule for pushing negations around at expansion time.
//!
//! # The subexpression closure `sub(O)`
//! [`subexpression_closure`] computes, for a set of (NNF) seed expressions, the set of
//! **all** their subexpressions — deduplicated, in deterministic first-visit preorder.
//! This set is finite and its size is linear in the total size of the seeds. It is the
//! load-bearing finiteness witness of the tableau's termination argument
//! (`crate::tableau` module docs, step 1): every node label the tableau ever builds is a
//! subset of the closure of its preprocessed input, because the completion rules only
//! ever add (a) subexpressions of concepts already present or (b) the fixed internalised
//! GCI constraints, which are themselves seeds.
//!
//! # Data ranges (opt-in `dl_datatypes`)
//! [SONNET-4.6] sq-pbz04.4.19. The concrete-domain extension adds two class-expression
//! constructors, `∃T.dr` and `∀T.dr` over a data property `T` and a
//! `crate::model::DataRange` `dr`. Their NNF rewrites are the quantifier duality again,
//! pushed into the CONCRETE domain:
//!
//! ```text
//! ¬(∃T.dr) ⇝ ∀T.¬dr            ¬(∀T.dr) ⇝ ∃T.¬dr            ¬¬dr ⇝ dr
//! ```
//!
//! Because the data-range grammar is just `Datatype(d) | ¬dr`, an NNF data range is always a
//! LITERAL — `d` or `¬d` — which is exactly the shape the concrete-domain oracle
//! (`crate::cdomain::satisfiable`) consumes. Data ranges are NOT class expressions, so they
//! contribute nothing to [`subexpression_closure`]; the tableau interns them in a separate
//! table, leaving the concept closure (and hence the termination argument) untouched.
//!
//! Fragment note: these functions are total over the L1 [`crate::model`] types, which by
//! construction admit ONLY the ALCH fragment (no inverses, cardinality, nominals) — extended
//! by the concrete domain when, and only when, `dl_datatypes` is enabled. Nothing here
//! extends the fragment.

#[cfg(feature = "dl_datatypes")]
use crate::model::DataRange;
use crate::model::ClassExpression;
use rustc_hash::FxHashSet;

/// Rewrite a class expression into an equivalent negation normal form.
///
/// The result contains [`ClassExpression::ObjectComplementOf`] only directly around a
/// [`ClassExpression::Class`] (see the module docs for the rewrite system and the
/// equivalence argument). Idempotent: `nnf(&nnf(c)) == nnf(c)`.
#[must_use]
pub fn nnf(ce: &ClassExpression) -> ClassExpression {
    match ce {
        ClassExpression::Class(_) | ClassExpression::Thing | ClassExpression::Nothing => {
            ce.clone()
        }
        ClassExpression::ObjectIntersectionOf(members) => {
            ClassExpression::ObjectIntersectionOf(members.iter().map(nnf).collect())
        }
        ClassExpression::ObjectUnionOf(members) => {
            ClassExpression::ObjectUnionOf(members.iter().map(nnf).collect())
        }
        // ¬C: push the negation inward over C.
        ClassExpression::ObjectComplementOf(inner) => nnf_complement(inner),
        ClassExpression::ObjectSomeValuesFrom(p, filler) => {
            ClassExpression::ObjectSomeValuesFrom(p.clone(), Box::new(nnf(filler)))
        }
        ClassExpression::ObjectAllValuesFrom(p, filler) => {
            ClassExpression::ObjectAllValuesFrom(p.clone(), Box::new(nnf(filler)))
        }
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataSomeValuesFrom(t, dr) => {
            ClassExpression::DataSomeValuesFrom(t.clone(), dr_nnf(dr))
        }
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataAllValuesFrom(t, dr) => {
            ClassExpression::DataAllValuesFrom(t.clone(), dr_nnf(dr))
        }
    }
}

/// Rewrite a data range into an equivalent NNF data-range LITERAL (`d` or `¬d`).
///
/// Total over [`DataRange`]: the only non-literal input is a nested complement, which
/// collapses by `¬¬dr ⇝ dr`. Idempotent.
#[cfg(feature = "dl_datatypes")]
#[must_use]
pub fn dr_nnf(dr: &DataRange) -> DataRange {
    match dr {
        DataRange::Datatype(_) => dr.clone(),
        DataRange::DataComplementOf(inner) => dr_complement(inner),
    }
}

/// The NNF of the **complement** of a data range: `dr_complement(dr) == dr_nnf(¬dr)`.
#[cfg(feature = "dl_datatypes")]
#[must_use]
pub fn dr_complement(dr: &DataRange) -> DataRange {
    match dr {
        DataRange::Datatype(_) => DataRange::DataComplementOf(Box::new(dr.clone())),
        // ¬¬dr ⇝ dr (then normalise dr itself).
        DataRange::DataComplementOf(inner) => dr_nnf(inner),
    }
}

/// `true` iff the data range is an NNF literal — a bare datatype or a complement applied
/// directly to one.
#[cfg(feature = "dl_datatypes")]
#[must_use]
pub fn is_dr_nnf(dr: &DataRange) -> bool {
    match dr {
        DataRange::Datatype(_) => true,
        DataRange::DataComplementOf(inner) => matches!(inner.as_ref(), DataRange::Datatype(_)),
    }
}

/// The negation normal form of the **complement** of a class expression:
/// `nnf_complement(c) == nnf(¬c)`, without materialising the intermediate `¬c`.
///
/// Used by the tableau's GCI internalisation (`C ⊑ D` becomes the universal constraint
/// `nnf(¬C) ⊔ nnf(D)`) and by the class-satisfiability entry point.
#[must_use]
pub fn nnf_complement(ce: &ClassExpression) -> ClassExpression {
    match ce {
        // ¬A for a named class A: already NNF.
        ClassExpression::Class(_) => ClassExpression::ObjectComplementOf(Box::new(ce.clone())),
        // ¬⊤ ⇝ ⊥ and ¬⊥ ⇝ ⊤.
        ClassExpression::Thing => ClassExpression::Nothing,
        ClassExpression::Nothing => ClassExpression::Thing,
        // De Morgan.
        ClassExpression::ObjectIntersectionOf(members) => {
            ClassExpression::ObjectUnionOf(members.iter().map(nnf_complement).collect())
        }
        ClassExpression::ObjectUnionOf(members) => {
            ClassExpression::ObjectIntersectionOf(members.iter().map(nnf_complement).collect())
        }
        // ¬¬C ⇝ C (then normalise C itself).
        ClassExpression::ObjectComplementOf(inner) => nnf(inner),
        // Quantifier duality.
        ClassExpression::ObjectSomeValuesFrom(p, filler) => {
            ClassExpression::ObjectAllValuesFrom(p.clone(), Box::new(nnf_complement(filler)))
        }
        ClassExpression::ObjectAllValuesFrom(p, filler) => {
            ClassExpression::ObjectSomeValuesFrom(p.clone(), Box::new(nnf_complement(filler)))
        }
        // Quantifier duality in the concrete domain.
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataSomeValuesFrom(t, dr) => {
            ClassExpression::DataAllValuesFrom(t.clone(), dr_complement(dr))
        }
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataAllValuesFrom(t, dr) => {
            ClassExpression::DataSomeValuesFrom(t.clone(), dr_complement(dr))
        }
    }
}

/// `true` iff the expression is in negation normal form — every
/// [`ClassExpression::ObjectComplementOf`] applies directly to a named
/// [`ClassExpression::Class`] (so `¬⊤`, `¬⊥`, `¬¬C`, `¬(C ⊓ D)`, `¬∃R.C`, … all fail).
///
/// This is the structural invariant [`nnf`] and [`nnf_complement`] establish; the tableau
/// debug-asserts it on its preprocessed input.
#[must_use]
pub fn is_nnf(ce: &ClassExpression) -> bool {
    match ce {
        ClassExpression::Class(_) | ClassExpression::Thing | ClassExpression::Nothing => true,
        ClassExpression::ObjectComplementOf(inner) => {
            matches!(inner.as_ref(), ClassExpression::Class(_))
        }
        ClassExpression::ObjectIntersectionOf(members)
        | ClassExpression::ObjectUnionOf(members) => members.iter().all(is_nnf),
        ClassExpression::ObjectSomeValuesFrom(_, filler)
        | ClassExpression::ObjectAllValuesFrom(_, filler) => is_nnf(filler),
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataSomeValuesFrom(_, dr)
        | ClassExpression::DataAllValuesFrom(_, dr) => is_dr_nnf(dr),
    }
}

/// The subexpression closure of a set of seed class expressions: every seed plus every
/// (transitive) subexpression of a seed, deduplicated, in deterministic first-visit
/// preorder over the seed slice.
///
/// For NNF seeds this is the finite `sub(O)` universe of the tableau's termination
/// argument (module docs above; `crate::tableau` module docs step 1). Its length is
/// linear in the total size of the seeds. Subexpressions of an NNF expression are
/// themselves NNF, so the closure of NNF seeds is entirely NNF.
#[must_use]
pub fn subexpression_closure(seeds: &[ClassExpression]) -> Vec<ClassExpression> {
    let mut seen: FxHashSet<ClassExpression> = FxHashSet::default();
    let mut out: Vec<ClassExpression> = Vec::new();
    for seed in seeds {
        visit(seed, &mut seen, &mut out);
    }
    out
}

/// Preorder DFS collecting `ce` and all its subexpressions into `out` (first visit wins).
fn visit(
    ce: &ClassExpression,
    seen: &mut FxHashSet<ClassExpression>,
    out: &mut Vec<ClassExpression>,
) {
    if !seen.insert(ce.clone()) {
        return;
    }
    out.push(ce.clone());
    match ce {
        ClassExpression::Class(_) | ClassExpression::Thing | ClassExpression::Nothing => {}
        ClassExpression::ObjectIntersectionOf(members)
        | ClassExpression::ObjectUnionOf(members) => {
            for m in members {
                visit(m, seen, out);
            }
        }
        ClassExpression::ObjectComplementOf(inner) => visit(inner, seen, out),
        ClassExpression::ObjectSomeValuesFrom(_, filler)
        | ClassExpression::ObjectAllValuesFrom(_, filler) => visit(filler, seen, out),
        // A data quantifier is a LEAF of the concept closure: its filler is a data range,
        // not a class expression, so it contributes no further concepts (module docs).
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataSomeValuesFrom(_, _) | ClassExpression::DataAllValuesFrom(_, _) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ClassExpression as CE;

    fn a() -> CE {
        CE::Class(1)
    }
    fn b() -> CE {
        CE::Class(2)
    }
    fn not(c: CE) -> CE {
        CE::ObjectComplementOf(Box::new(c))
    }

    #[test]
    fn nnf_pushes_negation_to_named_classes() {
        // ¬(A ⊓ ∃R.B) ⇝ ¬A ⊔ ∀R.¬B  (De Morgan + quantifier duality)
        let input = not(CE::ObjectIntersectionOf(vec![a(), CE::some(10, b())]));
        let expected = CE::ObjectUnionOf(vec![not(a()), CE::only(10, not(b()))]);
        assert_eq!(nnf(&input), expected);
        // ¬¬A ⇝ A; ¬⊤ ⇝ ⊥; ¬⊥ ⇝ ⊤; leaves are fixed points.
        assert_eq!(nnf(&not(not(a()))), a());
        assert_eq!(nnf(&not(CE::Thing)), CE::Nothing);
        assert_eq!(nnf(&not(CE::Nothing)), CE::Thing);
        assert_eq!(nnf(&a()), a());
        // Idempotent, and the result satisfies is_nnf.
        let once = nnf(&input);
        assert_eq!(nnf(&once), once);
        assert!(is_nnf(&once));
    }

    #[test]
    fn nnf_complement_is_nnf_of_negation() {
        // nnf_complement(C) must equal nnf(¬C) for every constructor.
        let cases = vec![
            a(),
            CE::Thing,
            CE::Nothing,
            not(a()),
            CE::ObjectIntersectionOf(vec![a(), b()]),
            CE::ObjectUnionOf(vec![a(), not(b())]),
            CE::some(10, CE::ObjectUnionOf(vec![a(), b()])),
            CE::only(10, not(a())),
        ];
        for c in cases {
            assert_eq!(nnf_complement(&c), nnf(&not(c.clone())), "case {:?}", c);
            assert!(is_nnf(&nnf_complement(&c)), "case {:?}", c);
        }
    }

    #[test]
    fn is_nnf_accepts_nnf_and_rejects_buried_negation() {
        assert!(is_nnf(&a()));
        assert!(is_nnf(&not(a())));
        assert!(is_nnf(&CE::only(10, CE::ObjectUnionOf(vec![not(a()), b()]))));
        // Negation over anything but a named class is NOT NNF.
        assert!(!is_nnf(&not(CE::Thing)));
        assert!(!is_nnf(&not(not(a()))));
        assert!(!is_nnf(&not(CE::ObjectIntersectionOf(vec![a(), b()]))));
        assert!(!is_nnf(&CE::some(10, not(CE::ObjectUnionOf(vec![a(), b()])))));
    }

    #[test]
    fn subexpression_closure_dedups_in_first_visit_order() {
        // seeds: [∃R.(A ⊓ B), A] — A occurs both nested and as a seed; counted once.
        let seed0 = CE::some(10, CE::ObjectIntersectionOf(vec![a(), b()]));
        let closure = subexpression_closure(&[seed0.clone(), a()]);
        assert_eq!(
            closure,
            vec![seed0, CE::ObjectIntersectionOf(vec![a(), b()]), a(), b()]
        );
        // Empty seed set: empty closure.
        assert!(subexpression_closure(&[]).is_empty());
        // Closure of NNF seeds is entirely NNF.
        let seeds = vec![CE::ObjectUnionOf(vec![not(a()), CE::only(10, b())])];
        assert!(subexpression_closure(&seeds).iter().all(is_nnf));
    }

    // [SONNET-4.6] sq-pbz04.4.19 — the concrete-domain NNF arm (opt-in `dl_datatypes`).
    #[cfg(feature = "dl_datatypes")]
    mod datatypes {
        use super::*;
        use crate::cdomain::Datatype;
        use crate::model::{DataPropertyExpression, DataRange};

        fn t() -> DataPropertyExpression {
            DataPropertyExpression::DataProperty(99)
        }
        fn int() -> DataRange {
            DataRange::Datatype(Datatype::XsdInteger)
        }
        fn not_dr(dr: DataRange) -> DataRange {
            DataRange::DataComplementOf(Box::new(dr))
        }

        #[test]
        fn data_range_nnf_collapses_double_negation() {
            assert_eq!(dr_nnf(&int()), int());
            assert_eq!(dr_nnf(&not_dr(int())), not_dr(int()));
            assert_eq!(dr_nnf(&not_dr(not_dr(int()))), int());
            assert_eq!(dr_complement(&int()), not_dr(int()));
            assert_eq!(dr_complement(&not_dr(int())), int());
            assert!(is_dr_nnf(&int()));
            assert!(is_dr_nnf(&not_dr(int())));
            assert!(!is_dr_nnf(&not_dr(not_dr(int()))));
        }

        #[test]
        fn data_quantifier_duality_and_closure_leafness() {
            let some = CE::DataSomeValuesFrom(t(), int());
            let all = CE::DataAllValuesFrom(t(), int());
            // ¬∃T.d ⇝ ∀T.¬d and ¬∀T.d ⇝ ∃T.¬d, matching nnf(¬·) exactly.
            assert_eq!(
                nnf_complement(&some),
                CE::DataAllValuesFrom(t(), not_dr(int()))
            );
            assert_eq!(
                nnf_complement(&all),
                CE::DataSomeValuesFrom(t(), not_dr(int()))
            );
            assert_eq!(nnf_complement(&some), nnf(&not(some.clone())));
            assert_eq!(nnf_complement(&all), nnf(&not(all.clone())));
            // Both forms are NNF, and NNF is idempotent on them.
            assert!(is_nnf(&some) && is_nnf(&all));
            assert!(is_nnf(&nnf_complement(&some)));
            assert_eq!(nnf(&nnf(&some)), nnf(&some));
            // A data quantifier is a closure LEAF — it adds itself and nothing else.
            assert_eq!(subexpression_closure(&[some.clone()]), vec![some]);
            // …and nesting one under an object quantifier still terminates the walk there.
            let nested = CE::some(10, all.clone());
            assert_eq!(subexpression_closure(&[nested.clone()]), vec![nested, all]);
        }
    }
}
