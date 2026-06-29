#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-t5bne: crate has zero `unsafe`.

//! See the crate README (rendered above) for the capability overview and the honest scope.
//!
//! The load-bearing entry points are [`as_conjunctive_query`] (the always-present, fail-closed
//! **CQ-shape gate**) and, behind the off-by-default `experimental` feature, `rewrite` (the
//! baseline PerfectRef DL-Lite_R query rewriter) and `rewrite_production` (the production path:
//! PerfectRef ∪ tree-witness folding, then UCQ-containment minimisation). Both are plain code
//! spans here, not intra-doc links, since those items are absent from the default-feature doc
//! surface. The algorithm lives in five modules: `cq` (the gate), `dllite` (TBox extraction),
//! `perfectref` (the rewrite/reduce saturation), `treewitness` (bounded existential-witness
//! folding), and `minimise` (UCQ-containment minimisation by homomorphism).
//!
//! EXPERIMENTAL regime: the rewriter is validated against a hand-checked DL-Lite oracle, NOT
//! graduated to a conformance floor. It IS wired into the conformance entailment-regime suite
//! (sq-kuvu3, opt-in `sparq-conformance/ql-experimental`) as experimental / OutOfScope — the
//! harness reports honestly what it computes over the `pr:QL` `sparql11/entailment` cases
//! (fail-closed abstain / computed-equivalent evidence / computed-divergent gap), never a
//! graduated conformance pass. The graduation of that arm to a pinned conformance FLOOR remains a
//! separate, deferred bead (it must sequence through the contended conformance scoreboard) — see
//! the README.

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

pub use cq::{as_conjunctive_query, ConjunctiveQuery, CqError};
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
    /// The UCQ size BEFORE containment minimisation (PerfectRef ∪ tree-witness foldings). For
    /// [`rewrite`] this equals `disjuncts` (no minimisation is applied); for [`rewrite_production`]
    /// it is the pre-minimisation count, so `disjuncts_before_minimisation - disjuncts` is the
    /// number of redundant disjuncts dropped. Never below `disjuncts` (minimisation only removes).
    pub disjuncts_before_minimisation: usize,
}

/// Rewrite a conjunctive SPARQL query into the **union of conjunctive queries** whose evaluation
/// over the **unmodified data** returns the **certain answers** under the DL-Lite_R (OWL 2 QL)
/// TBox extracted from `tbox` (a slice of RDF triples carrying the schema). EXPERIMENTAL.
///
/// FAIL-CLOSED: if `query` is not a conjunctive query the rewriter is sound for (it uses
/// OPTIONAL / FILTER / MINUS / UNION / a property path / aggregation / a variable predicate /
/// …), this returns [`CqError::OutOfScope`] naming the construct — it NEVER silently mis-answers
/// a non-CQ query. The CQ-shape gate ([`as_conjunctive_query`]) runs first and unconditionally.
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
    // 1. FAIL-CLOSED CQ-shape gate (the soundness keystone — always first).
    let cq = as_conjunctive_query(query)?;
    // 2. Map the CQ body to internal DL-Lite atoms.
    let (atoms, answer) = emit::cq_to_atoms(&cq)?;
    // 3. Extract the DL-Lite_R TBox and run PerfectRef to the fixpoint.
    let tbox_model = TBox::extract(tbox);
    let ucq = perfectref::perfect_ref(atoms, answer, &tbox_model);
    let disjuncts = ucq.len();
    // 4. Fold the UCQ back into a spargebra body, re-wrapped in the original projection/distinct.
    let body = emit::ucq_to_pattern(ucq);
    let rewritten = rewrap(query, &cq, body);
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
/// SOUNDNESS — the certain-answer set is IDENTICAL to [`rewrite`]'s. PerfectRef is sound +
/// complete, so the augmented pre-minimisation UCQ has at least PerfectRef's answers; tree-witness
/// folding only adds disjuncts that are themselves PerfectRef-derivable (so it adds no NEW
/// answers); and minimisation drops only a disjunct **contained** in a retained one (so it removes
/// no answers). The net result returns exactly the certain answers, in a SMALLER UCQ. The
/// containment check is NP-complete and **fail-closed**: when containment cannot be decided within
/// the search budget the disjunct is KEPT (never dropped on uncertainty) — an over-aggressive
/// minimisation that dropped a non-contained disjunct would be an unsoundness bug.
///
/// FAIL-CLOSED CQ-shape gate as in [`rewrite`]: a non-CQ query is rejected as
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
    // 1. FAIL-CLOSED CQ-shape gate (the soundness keystone — always first).
    let cq = as_conjunctive_query(query)?;
    // 2. Map the CQ body to internal DL-Lite atoms.
    let (atoms, answer) = emit::cq_to_atoms(&cq)?;
    // 3. Extract the DL-Lite_R TBox and run baseline PerfectRef to the fixpoint (complete).
    let tbox_model = TBox::extract(tbox);
    let perfect = perfectref::perfect_ref(atoms, answer.clone(), &tbox_model);
    // 4. AUGMENT each PerfectRef disjunct with its bounded tree-witness foldings (no new answers —
    //    every folding is PerfectRef-derivable; this only exposes the compact existential form).
    let mut augmented: Vec<perfectref::Cq> = Vec::new();
    for disjunct in perfect {
        for folded in treewitness::tree_witness_ucq(disjunct, &tbox_model) {
            augmented.push(folded);
        }
    }
    let before = {
        // De-duplicate structurally for the honest "before minimisation" tally (the union is a
        // set). Minimisation does this too, but we want the pre-minimisation count to reflect the
        // set, not the multiset, so the dropped-by-containment delta is meaningful.
        let mut seen = rustc_hash::FxHashSet::default();
        augmented.retain(|c| seen.insert(c.clone()));
        augmented.len()
    };
    // 5. UCQ-CONTAINMENT MINIMISATION — drop every disjunct contained in a retained one (sound;
    //    fail-closed on an undecidable-within-budget containment check).
    let minimal = minimise::minimise_ucq(augmented, &answer);
    let disjuncts = minimal.len();
    // 6. Fold the minimised UCQ back into a spargebra body under the original projection/distinct.
    let body = emit::ucq_to_pattern(minimal);
    let rewritten = rewrap(query, &cq, body);
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
fn rewrap(original: &Query, cq: &ConjunctiveQuery, body: GraphPattern) -> Query {
    use spargebra::term::Variable;
    let projected: Vec<Variable> = cq.distinguished.clone();
    // Apply DISTINCT first (innermost), then Project — matching spargebra's nesting.
    let inner = if cq.distinct {
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
        assert_eq!(r.report.disjuncts, 1, "no TBox ⇒ UCQ is just the input CQ");
    }
}
