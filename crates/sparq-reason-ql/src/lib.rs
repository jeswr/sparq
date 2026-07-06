#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-t5bne: crate has zero `unsafe`.

//! See the crate README (rendered above) for the capability overview and the honest scope.
//!
//! The load-bearing entry points are [`as_conjunctive_query`] and [`as_ucq`] (the always-present,
//! fail-closed **CQ/UCQ-shape gates**) and, behind the off-by-default `experimental` feature,
//! `rewrite` (the baseline PerfectRef DL-Lite_R query rewriter) and `rewrite_production` (the
//! production path: PerfectRef + tree-witness folding + UCQ-containment minimisation). Both are
//! plain code spans here, not intra-doc links, since those items are absent from the
//! default-feature doc surface. The algorithm lives in five modules: `cq` (the gate), `dllite`
//! (TBox extraction), `perfectref` (the rewrite/reduce saturation), `treewitness` (bounded
//! existential-witness folding), and `minimise` (UCQ-containment minimisation by homomorphism).
//!
//! The rewriter is validated against a hand-checked DL-Lite_R oracle. On the FORMAL DL-Lite_R
//! suite (the hand-derived certain-answer oracle from sq-g19x0) the rewrite is sound AND complete
//! case by case — that has GRADUATED to a pinned floor (sq-qo1a9): `sparq-conformance`'s
//! `ql_dllite_suite` runner rewrites each case and asserts its UCQ, evaluated over the unmodified
//! ABox, returns exactly the hand-derived certain answers; it is a `sparq extension` row in the
//! central scoreboard (tallied separately, NOT a full-OWL-2-QL-conformance claim, since no runnable
//! normative W3C QL certain-answer suite exists). The BROADER `pr:QL` `sparql11/entailment` arm
//! (sq-kuvu3, opt-in `sparq-conformance/ql-experimental`) stays experimental / OutOfScope — it
//! mixes intensional cases outside sound rewriting, so the harness reports honestly what it
//! computes (fail-closed abstain / computed-equivalent evidence / computed-divergent gap), never a
//! graduated conformance pass — see the README.

mod cq;
mod dllite;
#[cfg(feature = "experimental")]
mod emit;
#[cfg(feature = "experimental")]
mod minimise;
#[cfg(feature = "experimental")]
mod perfectref;
#[cfg(feature = "experimental")]
mod treewitness;

pub use cq::{as_conjunctive_query, as_ucq, ConjunctiveQuery, CqError, Ucq, ValuesBlock};
pub use dllite::{Basic, ConceptInclusion, Role, TBox};

#[cfg(feature = "experimental")]
use spargebra::algebra::GraphPattern;
#[cfg(feature = "experimental")]
use spargebra::Query;

/// The outcome of [`rewrite`]: either the rewritten UCQ query plus a [`RewriteReport`], or an
/// honest reason the input was out of QL rewriting scope.
#[cfg(feature = "experimental")]
#[derive(Clone, Debug)]
pub struct Rewritten {
    /// The rewritten SPARQL query — a union of conjunctive queries under the original
    /// projection/distinct — ready to run UNCHANGED through the engine's query path.
    pub query: Query,
    /// What the TBox extraction and rewrite produced (disjunct count, skipped non-QL axioms).
    pub report: RewriteReport,
}

/// A tally of the rewrite, so a caller can honestly surface what happened.
#[cfg(feature = "experimental")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RewriteReport {
    /// Number of conjunctive queries in the emitted UCQ. For [`rewrite`] this is the raw
    /// PerfectRef UCQ size (1 = the TBox added nothing); for [`rewrite_production`] this is the
    /// MINIMISED UCQ size (the count actually emitted into the query).
    pub disjuncts: usize,
    /// TBox axioms outside the DL-Lite_R (OWL 2 QL) fragment that were IGNORED for rewriting
    /// (never silently applied). See [`TBox::skipped`].
    pub skipped_axioms: usize,
    /// The UCQ size BEFORE containment minimisation (PerfectRef + tree-witness foldings). For
    /// [`rewrite`] this equals `disjuncts` (no minimisation is applied); for [`rewrite_production`]
    /// it is the pre-minimisation count, so `disjuncts_before_minimisation - disjuncts` is the
    /// number of redundant disjuncts dropped. Never below `disjuncts` (minimisation only removes).
    pub disjuncts_before_minimisation: usize,
}

/// Rewrite a conjunctive SPARQL query into the **union of conjunctive queries** whose evaluation
/// over the **unmodified data** returns the **certain answers** under the DL-Lite_R (OWL 2 QL)
/// TBox extracted from `tbox` (a slice of RDF triples carrying the schema). EXPERIMENTAL.
///
/// **UCQ input (B1):** a top-level `UNION` of CQ branches is also accepted; each branch is
/// rewritten independently and the results unioned (certain answers distribute over union in
/// DL-Lite_R). Use `rewrite` directly; the gate classifies the input via `as_ucq`.
///
/// **Literal-object atoms (B2):** role atoms with literal constants in the object position are
/// now accepted (e.g. `?x foaf:name "Alice"@en`). A literal is never an unbound position, so
/// the applicability condition is untouched; role inclusions carry it unchanged.
///
/// **FILTER on distinguished-only variables (B3):** a `FILTER(expr)` that mentions only
/// projected (answer) variables is passed through unmodified and re-applied after rewriting.
///
/// **Constant-only VALUES over distinguished variables (B4):** a `VALUES` block whose
/// variables are all distinguished and whose rows are fully bound constants is passed through
/// and re-applied after rewriting. `UNDEF` cells or non-distinguished variables → rejected.
///
/// **Intensional-atom guard (B6):** a role atom whose predicate is semantics-bearing schema
/// vocabulary (`rdfs:subClassOf`, `rdfs:subPropertyOf`, `rdfs:domain`, `rdfs:range`, any
/// `owl:` predicate) is **rejected** as `OutOfScope` — the rewriter evaluates over ABox data
/// only and would silently under-answer such a query. Annotation predicates (`rdfs:label`,
/// `rdfs:comment`, `rdfs:seeAlso`, `rdfs:isDefinedBy`) remain admitted as plain role atoms.
///
/// FAIL-CLOSED: if `query` is not a (union of) conjunctive quer(y/ies) the rewriter is sound
/// for, this returns [`CqError::OutOfScope`] naming the construct — it NEVER silently mis-answers
/// a non-CQ/UCQ query. The gate ([`as_ucq`]) runs first and unconditionally.
///
/// The returned [`Rewritten::query`] runs unchanged through the engine (the spargebra
/// rewrite seam): a UCQ folds to a `Union` tree of `Bgp`s under the original projection.
///
/// Scope (honest): the rewriter handles the POSITIVE DL-Lite_R inclusions
/// (`rdfs:subClassOf`/`subPropertyOf`/`domain`/`range`, `owl:inverseOf`, unqualified `∃R`
/// restrictions). It does NOT minimise the UCQ (no containment check — the deferred production
/// path) and does NOT check consistency. See the crate README for the full boundary.
///
/// ```
/// # #[cfg(feature = "experimental")] {
/// use oxrdf::Triple;
/// use spargebra::SparqlParser;
/// use sparq_reason_ql::rewrite;
/// use std::str::FromStr;
///
/// // TBox: Manager rdfs:subClassOf Employee.
/// let tbox = vec![Triple::from_str(
///     "<http://ex/Manager> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://ex/Employee> ."
/// ).unwrap()];
/// let q = SparqlParser::new().parse_query(
///     "SELECT ?x WHERE { ?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex/Employee> }"
/// ).unwrap();
/// let r = rewrite(&q, &tbox).unwrap();
/// // The UCQ now also matches Managers, who are certainly Employees.
/// assert_eq!(r.report.disjuncts, 2);
/// # }
/// ```
#[cfg(feature = "experimental")]
pub fn rewrite(query: &Query, tbox: &[oxrdf::Triple]) -> Result<Rewritten, CqError> {
    // 1. FAIL-CLOSED UCQ-shape gate (the soundness keystone — always first).
    let ucq = as_ucq(query)?;
    // 2. Extract the DL-Lite_R TBox once (shared across branches).
    let tbox_model = TBox::extract(tbox);
    let mut all_disjuncts: Vec<perfectref::Cq> = Vec::new();
    // 3. Rewrite each UCQ branch independently and collect all disjuncts.
    for branch in &ucq.branches {
        let (atoms, answer) = emit::cq_to_atoms(branch)?;
        let branch_ucq = perfectref::perfect_ref(atoms, answer, &tbox_model);
        all_disjuncts.extend(branch_ucq);
    }
    let disjuncts = all_disjuncts.len();
    // 4. Fold the combined UCQ back into a spargebra body. For a multi-branch UCQ where ANY
    //    branch carries per-branch FILTER or VALUES, the single-passthrough emitter would
    //    hoist branch[0]'s filter/values over the ENTIRE union — dropping real answers (branch-0
    //    filter applied to branches that do not own it) and/or silently applying no filter to
    //    later branches (invents answers). That is an unsoundness. THE SAFE FIX (fail-closed,
    //    deferred to sq-pbz04.3.2): REJECT any multi-branch UCQ where ANY branch carries
    //    per-branch filter_exprs or values_blocks. [SONNET-4.6] sq-pbz04.3.1
    //
    //    Single-branch UCQ (the common case): the one branch IS the whole query, so its
    //    filter_exprs/values_blocks are re-applied soundly by ucq_to_pattern. Still accepted.
    if ucq.branches.len() > 1 {
        let has_per_branch_modifier = ucq.branches.iter().any(|b| {
            !b.filter_exprs.is_empty() || !b.values_blocks.is_empty()
        });
        if has_per_branch_modifier {
            return Err(CqError::OutOfScope(
                "multi-branch UCQ with per-branch FILTER or VALUES is not yet \
                 supported (branch-aware emitter deferred to sq-pbz04.3.2 — \
                 fail-closed to prevent hoisting branch-0's filter over the whole union)"
                    .into(),
            ));
        }
    }
    let cq_for_passthrough = &ucq.branches[0];
    let body = emit::ucq_to_pattern(all_disjuncts, cq_for_passthrough);
    let rewritten = rewrap(query, &ucq, body);
    Ok(Rewritten {
        query: rewritten,
        report: RewriteReport {
            disjuncts,
            skipped_axioms: tbox_model.skipped,
            disjuncts_before_minimisation: disjuncts,
        },
    })
}

/// Rewrite a conjunctive SPARQL query into the **minimised** union of conjunctive queries via the
/// PRODUCTION path: baseline PerfectRef saturation, **augmented** with bounded **tree-witness**
/// folding (existential witnesses captured without an unbounded chase), then **UCQ-containment
/// minimisation** (redundant disjuncts dropped by the homomorphism containment test). EXPERIMENTAL.
///
/// All broadened shapes from [`rewrite`] (B1 UCQ, B2 literal atoms, B3 FILTER, B4 VALUES, B6
/// intensional-atom guard) apply here as well.
///
/// SOUNDNESS — the certain-answer set is IDENTICAL to [`rewrite`]'s. PerfectRef is sound +
/// complete, so the augmented pre-minimisation UCQ has at least PerfectRef's answers; tree-witness
/// folding only adds disjuncts that are themselves PerfectRef-derivable (so it adds no NEW
/// answers); and minimisation drops only a disjunct **contained** in a retained one (so it removes
/// no answers). The net result returns exactly the certain answers, in a SMALLER UCQ. The
/// containment check is NP-complete and **fail-closed**: when containment cannot be decided within
/// the search budget the disjunct is KEPT (never dropped on uncertainty) — an over-aggressive
/// minimisation that dropped a non-contained disjunct would be an unsoundness bug.
///
/// FAIL-CLOSED CQ/UCQ-shape gate as in [`rewrite`]: a non-CQ/UCQ query is rejected as
/// [`CqError::OutOfScope`], never mis-answered.
///
/// ```
/// # #[cfg(feature = "experimental")] {
/// use oxrdf::Triple;
/// use spargebra::SparqlParser;
/// use sparq_reason_ql::rewrite_production;
/// use std::str::FromStr;
///
/// // TBox: Manager rdfs:subClassOf Employee.
/// let tbox = vec![Triple::from_str(
///     "<http://ex/Manager> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://ex/Employee> ."
/// ).unwrap()];
/// let q = SparqlParser::new().parse_query(
///     "SELECT ?x WHERE { ?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex/Employee> }"
/// ).unwrap();
/// let r = rewrite_production(&q, &tbox).unwrap();
/// // Employee + Manager: two incomparable disjuncts; nothing is redundant, so none is dropped.
/// assert_eq!(r.report.disjuncts, 2);
/// assert_eq!(r.report.disjuncts_before_minimisation, 2);
/// # }
/// ```
#[cfg(feature = "experimental")]
pub fn rewrite_production(query: &Query, tbox: &[oxrdf::Triple]) -> Result<Rewritten, CqError> {
    // 1. FAIL-CLOSED UCQ-shape gate (the soundness keystone — always first).
    let ucq = as_ucq(query)?;
    // 2. Extract the DL-Lite_R TBox once (shared across branches).
    let tbox_model = TBox::extract(tbox);
    let mut augmented: Vec<perfectref::Cq> = Vec::new();
    // 3. For each UCQ branch: PerfectRef → tree-witness augmentation.
    for branch in &ucq.branches {
        let (atoms, answer) = emit::cq_to_atoms(branch)?;
        let perfect = perfectref::perfect_ref(atoms, answer.clone(), &tbox_model);
        for disjunct in perfect {
            for folded in treewitness::tree_witness_ucq(disjunct, &tbox_model) {
                augmented.push(folded);
            }
        }
    }
    let answer_for_minimise: Vec<String> = ucq.distinguished.iter()
        .map(|v| v.as_str().to_string())
        .collect();
    let before = {
        // De-duplicate structurally for the honest "before minimisation" tally.
        let mut seen = rustc_hash::FxHashSet::default();
        augmented.retain(|c| seen.insert(c.clone()));
        augmented.len()
    };
    // 4. UCQ-containment minimisation.
    let minimal = minimise::minimise_ucq(augmented, &answer_for_minimise);
    let disjuncts = minimal.len();
    // 5. Fold the minimised UCQ back. Same multi-branch-modifier guard as `rewrite`: a
    //    multi-branch UCQ where ANY branch carries per-branch FILTER or VALUES is rejected
    //    fail-closed (sq-pbz04.3.2 defers the branch-aware emitter). [SONNET-4.6] sq-pbz04.3.1
    if ucq.branches.len() > 1 {
        let has_per_branch_modifier = ucq.branches.iter().any(|b| {
            !b.filter_exprs.is_empty() || !b.values_blocks.is_empty()
        });
        if has_per_branch_modifier {
            return Err(CqError::OutOfScope(
                "multi-branch UCQ with per-branch FILTER or VALUES is not yet \
                 supported (branch-aware emitter deferred to sq-pbz04.3.2 — \
                 fail-closed to prevent hoisting branch-0's filter over the whole union)"
                    .into(),
            ));
        }
    }
    let cq_for_passthrough = &ucq.branches[0];
    let body = emit::ucq_to_pattern(minimal, cq_for_passthrough);
    let rewritten = rewrap(query, &ucq, body);
    Ok(Rewritten {
        query: rewritten,
        report: RewriteReport {
            disjuncts,
            skipped_axioms: tbox_model.skipped,
            disjuncts_before_minimisation: before,
        },
    })
}

/// Re-wrap a rewritten UCQ `body` in the original query's projection + DISTINCT, preserving
/// SELECT-vs-ASK and the projected variables. The dataset/base-IRI are dropped (a rewritten UCQ
/// runs against the same default dataset; carrying a custom dataset through is out of MVP scope
/// — the gate accepts only default-dataset CQs implicitly, since GRAPH is rejected).
#[cfg(feature = "experimental")]
fn rewrap(original: &Query, ucq: &Ucq, body: GraphPattern) -> Query {
    use spargebra::term::Variable;
    let projected: Vec<Variable> = ucq.distinguished.clone();
    // Apply DISTINCT first (innermost), then Project — matching spargebra's nesting.
    let inner = if ucq.distinct {
        GraphPattern::Distinct {
            inner: Box::new(body),
        }
    } else {
        body
    };
    match original {
        Query::Ask { .. } => Query::Ask {
            dataset: None,
            pattern: inner,
            base_iri: None,
        },
        _ => Query::Select {
            dataset: None,
            pattern: GraphPattern::Project {
                inner: Box::new(inner),
                variables: projected,
            },
            base_iri: None,
        },
    }
}

#[cfg(all(test, feature = "experimental"))]
mod tests {
    use super::*;
    use oxrdf::Triple;
    use spargebra::SparqlParser;
    use std::str::FromStr;

    fn tbox(nt: &[&str]) -> Vec<Triple> {
        nt.iter().map(|l| Triple::from_str(l).unwrap()).collect()
    }

    fn q(s: &str) -> Query {
        SparqlParser::new().parse_query(s).unwrap()
    }

    const RDFS_SUB: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
    const TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

    #[test]
    fn subclass_doubles_the_ucq() {
        let t = tbox(&[&format!("<http://ex/A> <{RDFS_SUB}> <http://ex/B> .")]);
        let query = q(&format!(
            "SELECT ?x WHERE {{ ?x <{TYPE}> <http://ex/B> }}"
        ));
        let r = rewrite(&query, &t).unwrap();
        assert_eq!(r.report.disjuncts, 2);
        // The rewritten query must serialise as a UNION (sanity that the fold happened).
        assert!(r.query.to_string().to_uppercase().contains("UNION"));
    }

    #[test]
    fn non_cq_is_rejected_not_rewritten() {
        let query = q(&format!(
            "SELECT ?x WHERE {{ ?x <{TYPE}> <http://ex/B> OPTIONAL {{ ?x <http://ex/r> ?y }} }}"
        ));
        let err = rewrite(&query, &[]).unwrap_err();
        assert!(matches!(err, CqError::OutOfScope(_)));
    }

    #[test]
    fn empty_tbox_is_identity_ucq() {
        let query = q(&format!("SELECT ?x WHERE {{ ?x <{TYPE}> <http://ex/B> }}"));
        let r = rewrite(&query, &[]).unwrap();
        assert_eq!(r.report.disjuncts, 1, "no TBox => UCQ is just the input CQ");
    }

    // [SONNET-4.6] sq-pbz04.3.1 — UCQ rewrite (B1): a top-level UNION rewrites each branch.
    #[test]
    fn ucq_input_rewrites_both_branches() {
        let t = tbox(&[&format!("<http://ex/Manager> <{RDFS_SUB}> <http://ex/Employee> .")]);
        let query = q(&format!(
            "SELECT ?x WHERE {{ {{ ?x <{TYPE}> <http://ex/Employee> }} UNION {{ ?x <{TYPE}> <http://ex/Contractor> }} }}"
        ));
        let r = rewrite(&query, &t).unwrap();
        // Employee branch rewrites to {Employee, Manager}; Contractor branch stays {Contractor}.
        // Total: 3 disjuncts.
        assert_eq!(
            r.report.disjuncts, 3,
            "UCQ input: Employee->2 + Contractor->1 = 3; report = {:?}",
            r.report
        );
    }

    // [SONNET-4.6] sq-pbz04.3.1 — intensional-atom guard (B6): rdfs:subClassOf as atom predicate
    // must be rejected even though it is a valid IRI constant.
    #[test]
    fn intensional_atom_rejected() {
        let query = q(
            "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#> \
             PREFIX : <http://ex/> \
             SELECT ?c WHERE { ?c rdfs:subClassOf :A }",
        );
        let err = rewrite(&query, &[]).unwrap_err();
        assert!(
            matches!(err, CqError::OutOfScope(ref r) if r.contains("intensional")),
            "rdfs:subClassOf as atom predicate must be rejected as intensional; got {:?}",
            err
        );
    }

    // ---- FIX (a) differential tests: multi-branch UCQ with per-branch FILTER or VALUES ----
    // These test the FAIL-CLOSED guard added to close the B1×B3/B4 soundness bug.
    // Reviewer's exact repro: UNION × FILTER must be rejected (OutOfScope), not mis-answered.
    // [SONNET-4.6] sq-pbz04.3.1 fix

    /// `SELECT ?x { { ?x a :A FILTER(?x != :Bad) } UNION { ?x a :B } }` — the
    /// branch[0] FILTER would be hoisted over the entire union body, dropping real
    /// answers from branch[1] and potentially inventing wrong answers. Rejected as
    /// OutOfScope (fail-closed, deferred to sq-pbz04.3.2). [SONNET-4.6]
    #[test]
    fn union_with_branch_filter_is_rejected() {
        let query = q(&format!(
            "PREFIX : <http://ex/> \
             SELECT ?x WHERE {{ \
               {{ ?x <{TYPE}> :A FILTER(?x != :Bad) }} \
               UNION \
               {{ ?x <{TYPE}> :B }} \
             }}"
        ));
        let err = rewrite(&query, &[]).unwrap_err();
        assert!(
            matches!(err, CqError::OutOfScope(ref r) if
                r.contains("multi-branch") && r.contains("FILTER")),
            "UNION with per-branch FILTER must be rejected OutOfScope (fail-closed B1×B3); got {:?}",
            err
        );
        // Same rejection via rewrite_production.
        let err2 = rewrite_production(&query, &[]).unwrap_err();
        assert!(
            matches!(err2, CqError::OutOfScope(_)),
            "rewrite_production must also reject UNION×FILTER; got {:?}",
            err2
        );
    }

    /// Branch[1]-filter variant: `SELECT ?x { { ?x a :A } UNION { ?x a :B FILTER(?x != :Bad) } }`.
    /// Branch[1]'s FILTER would be silently dropped (never applied) because the emitter only
    /// uses branch[0] for pass-through. Rejected as OutOfScope. [SONNET-4.6]
    #[test]
    fn union_with_branch1_filter_is_rejected() {
        let query = q(&format!(
            "PREFIX : <http://ex/> \
             SELECT ?x WHERE {{ \
               {{ ?x <{TYPE}> :A }} \
               UNION \
               {{ ?x <{TYPE}> :B FILTER(?x != :Bad) }} \
             }}"
        ));
        let err = rewrite(&query, &[]).unwrap_err();
        assert!(
            matches!(err, CqError::OutOfScope(ref r) if
                r.contains("multi-branch") && r.contains("FILTER")),
            "UNION with per-branch FILTER on branch[1] must be rejected OutOfScope; got {:?}",
            err
        );
    }

    /// `SELECT ?x { { ?x a :A } UNION { ?x a :B VALUES ?x { :C } } }` — the
    /// branch[1] VALUES would be silently dropped by the single-passthrough emitter.
    /// Rejected as OutOfScope (fail-closed, deferred to sq-pbz04.3.2). [SONNET-4.6]
    #[test]
    fn union_with_branch_values_is_rejected() {
        let query = q(&format!(
            "PREFIX : <http://ex/> \
             SELECT ?x WHERE {{ \
               {{ ?x <{TYPE}> :A }} \
               UNION \
               {{ ?x <{TYPE}> :B VALUES ?x {{ :C }} }} \
             }}"
        ));
        let err = rewrite(&query, &[]).unwrap_err();
        assert!(
            matches!(err, CqError::OutOfScope(ref r) if
                r.contains("multi-branch") && r.contains("VALUES")),
            "UNION with per-branch VALUES must be rejected OutOfScope (fail-closed B1×B4); got {:?}",
            err
        );
        let err2 = rewrite_production(&query, &[]).unwrap_err();
        assert!(
            matches!(err2, CqError::OutOfScope(_)),
            "rewrite_production must also reject UNION×VALUES; got {:?}",
            err2
        );
    }

    /// The SOUND case: a single-branch CQ with a FILTER on the distinguished variable
    /// must still be accepted, and the rewritten query must return the original answers.
    /// (branch[0] IS the whole query, so hoisting its filter is correct.) [SONNET-4.6]
    #[test]
    fn single_branch_filter_still_accepted_and_sound() {
        // No TBox — identity UCQ (1 disjunct). The FILTER is passed through.
        let query = q(&format!(
            "PREFIX : <http://ex/> \
             SELECT ?x WHERE {{ ?x <{TYPE}> :A FILTER(?x != :Bad) }}"
        ));
        let r = rewrite(&query, &[]).unwrap();
        // 1 disjunct (identity with no TBox), and the rewritten query must contain FILTER.
        assert_eq!(
            r.report.disjuncts, 1,
            "single-branch FILTER: identity UCQ (no TBox) = 1 disjunct; report = {:?}",
            r.report
        );
        assert!(
            r.query.to_string().to_uppercase().contains("FILTER"),
            "FILTER must be re-applied in the rewritten query; got: {}",
            r.query
        );
        // rewrite_production must also accept the single-branch case.
        let r2 = rewrite_production(&query, &[]).unwrap();
        assert_eq!(r2.report.disjuncts, 1, "rewrite_production: same single-branch result");
    }
}
