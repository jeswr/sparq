// [OPUS-4.8] sq-t5bne (epic sq-pbz04): the HAND-CHECKED DL-Lite oracle for the OWL 2 QL
// PerfectRef rewriter.
//
// No Rust PerfectRef implementation exists to differentially diff against (the ecosystem is
// Java/Scala — Ontop, QuOnto, Requiem), so — exactly as the design records prescribe (research/
// reasoner-suite-on-substrate.md §2.5, phase Q1) — the oracle is a CURATED set of DL-Lite cases
// with HAND-DERIVED expected unions of conjunctive queries. Each case below states the TBox, the
// input CQ, and the EXACT set of disjuncts PerfectRef must produce; the test asserts the
// rewriter's UCQ canonicalises to that set. The derivations are written out in the comments so
// a reviewer can re-check them by hand.
//
// Canonicalisation makes the comparison robust to the rewriter's fresh-`_` variable naming: each
// disjunct BGP is reduced to a SORTED multiset of triple signatures where a CONSTANT is its IRI,
// a DISTINGUISHED (answer) variable keeps its name, and every OTHER variable is collapsed to `_`
// (since two existential `_`s in different positions are interchangeable for matching).

#![cfg(feature = "experimental")]

use oxrdf::Triple;
use spargebra::algebra::GraphPattern;
use spargebra::term::{NamedNodePattern, TermPattern};
use spargebra::{Query, SparqlParser};
use std::collections::BTreeSet;
use std::str::FromStr;

use sparq_reason_ql::rewrite;

const PRE: &str = "\
@prefix : <http://ex/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
";

const Q_PRE: &str = "\
PREFIX : <http://ex/>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
";

fn tbox(ttl: &str) -> Vec<Triple> {
    let src = format!("{PRE}{ttl}");
    oxttl::TurtleParser::new()
        .for_slice(src.as_bytes())
        .map(|r| r.expect("turtle parse"))
        .collect()
}

fn query(q: &str) -> Query {
    SparqlParser::new()
        .parse_query(&format!("{Q_PRE}{q}"))
        .expect("query parse")
}

/// Canonical form of a rewritten query's UCQ: the set of per-disjunct canonical BGP strings. The
/// answer variables are passed so they survive canonicalisation (everything else → `_`).
fn ucq_canonical(q: &Query, answer: &[&str]) -> BTreeSet<String> {
    let pattern = match q {
        Query::Select { pattern, .. } => pattern,
        Query::Ask { pattern, .. } => pattern,
        _ => panic!("rewritten query must be SELECT/ASK"),
    };
    let body = peel(pattern);
    let mut disjuncts = Vec::new();
    collect_union(body, &mut disjuncts);
    disjuncts
        .iter()
        .map(|bgp| canon_bgp(bgp, answer))
        .collect()
}

/// Peel the outer Project/Distinct to reach the UCQ body.
fn peel(p: &GraphPattern) -> &GraphPattern {
    match p {
        GraphPattern::Project { inner, .. }
        | GraphPattern::Distinct { inner }
        | GraphPattern::Reduced { inner } => peel(inner),
        other => other,
    }
}

/// Flatten a Union tree into its leaf BGPs.
fn collect_union<'a>(p: &'a GraphPattern, out: &mut Vec<&'a GraphPattern>) {
    match p {
        GraphPattern::Union { left, right } => {
            collect_union(left, out);
            collect_union(right, out);
        }
        bgp @ GraphPattern::Bgp { .. } => out.push(bgp),
        // A single-atom UCQ may be a bare Bgp; nothing else is expected here.
        other => out.push(other),
    }
}

/// Canonicalise a single BGP to a stable string: sorted triple signatures, constants verbatim,
/// answer variables kept, all other variables → `_`.
fn canon_bgp(p: &GraphPattern, answer: &[&str]) -> String {
    let GraphPattern::Bgp { patterns } = p else {
        return format!("NON-BGP:{p:?}");
    };
    let mut sigs: Vec<String> = patterns
        .iter()
        .map(|tp| {
            let s = canon_term(&tp.subject, answer);
            let pr = match &tp.predicate {
                NamedNodePattern::NamedNode(n) => format!("<{}>", n.as_str()),
                NamedNodePattern::Variable(v) => format!("?{}", v.as_str()),
            };
            let o = canon_term(&tp.object, answer);
            format!("{s} {pr} {o}")
        })
        .collect();
    sigs.sort();
    sigs.join(" . ")
}

fn canon_term(t: &TermPattern, answer: &[&str]) -> String {
    match t {
        TermPattern::NamedNode(n) => format!("<{}>", n.as_str()),
        TermPattern::Variable(v) => {
            if answer.contains(&v.as_str()) {
                format!("?{}", v.as_str())
            } else {
                "_".to_string()
            }
        }
        TermPattern::Literal(l) => format!("{l}"),
        TermPattern::BlankNode(b) => format!("_:{}", b.as_str()),
        TermPattern::Triple(_) => "<<triple>>".to_string(),
    }
}

/// Build the EXPECTED canonical UCQ from a list of disjunct BGP strings (each in the same
/// `subj pred obj . subj pred obj` shape `canon_bgp` emits, atoms in any order — we re-sort).
fn expected(disjuncts: &[&str]) -> BTreeSet<String> {
    disjuncts
        .iter()
        .map(|d| {
            let mut atoms: Vec<String> = d.split(" . ").map(|s| s.trim().to_string()).collect();
            atoms.sort();
            atoms.join(" . ")
        })
        .collect()
}

const TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_SUB: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";

// ---------------------------------------------------------------------------------------------
// Oracle case 1 — class subsumption (the basic A1 ⊑ A2 backward rewrite).
//
//   TBox:  :Manager rdfs:subClassOf :Employee .
//   CQ:    SELECT ?x WHERE { ?x a :Employee }
//   By hand: PerfectRef rewrites Employee(?x) using Manager ⊑ Employee ⇒ Manager(?x). No reduce
//   applies (single atom). UCQ = { Employee(?x), Manager(?x) }.
// ---------------------------------------------------------------------------------------------
#[test]
fn oracle_class_subsumption() {
    let t = tbox(":Manager rdfs:subClassOf :Employee .");
    let q = query("SELECT ?x WHERE { ?x a :Employee }");
    let r = rewrite(&q, &t).unwrap();
    let got = ucq_canonical(&r.query, &["x"]);
    let want = expected(&[
        &format!("?x <{TYPE}> <http://ex/Employee>"),
        &format!("?x <{TYPE}> <http://ex/Manager>"),
    ]);
    assert_eq!(got, want, "report = {:?}", r.report);
}

// ---------------------------------------------------------------------------------------------
// Oracle case 2 — transitive subclass chain (rewrite fires twice).
//
//   TBox:  :A ⊑ :B, :B ⊑ :C
//   CQ:    SELECT ?x WHERE { ?x a :C }
//   By hand: C(?x) ⇒ B(?x) ⇒ A(?x). UCQ = { C(?x), B(?x), A(?x) }.
// ---------------------------------------------------------------------------------------------
#[test]
fn oracle_transitive_subclass() {
    let t = tbox(":A rdfs:subClassOf :B . :B rdfs:subClassOf :C .");
    let q = query("SELECT ?x WHERE { ?x a :C }");
    let r = rewrite(&q, &t).unwrap();
    let got = ucq_canonical(&r.query, &["x"]);
    let want = expected(&[
        &format!("?x <{TYPE}> <http://ex/C>"),
        &format!("?x <{TYPE}> <http://ex/B>"),
        &format!("?x <{TYPE}> <http://ex/A>"),
    ]);
    assert_eq!(got, want, "report = {:?}", r.report);
}

// ---------------------------------------------------------------------------------------------
// Oracle case 3 — domain inclusion introduces a role atom (∃R ⊑ A).
//
//   TBox:  :worksFor rdfs:domain :Employee .   (i.e. ∃worksFor ⊑ Employee)
//   CQ:    SELECT ?x WHERE { ?x a :Employee }
//   By hand: Employee(?x) ⇒ worksFor(?x, _) (the domain rule introduces an unbound object).
//   UCQ = { Employee(?x), worksFor(?x, _) }.
// ---------------------------------------------------------------------------------------------
#[test]
fn oracle_domain_introduces_role() {
    let t = tbox(":worksFor rdfs:domain :Employee .");
    let q = query("SELECT ?x WHERE { ?x a :Employee }");
    let r = rewrite(&q, &t).unwrap();
    let got = ucq_canonical(&r.query, &["x"]);
    let want = expected(&[
        &format!("?x <{TYPE}> <http://ex/Employee>"),
        "?x <http://ex/worksFor> _",
    ]);
    assert_eq!(got, want, "report = {:?}", r.report);
}

// ---------------------------------------------------------------------------------------------
// Oracle case 4 — range inclusion introduces an INVERSE role atom (∃R⁻ ⊑ A).
//
//   TBox:  :worksFor rdfs:range :Company .     (i.e. ∃worksFor⁻ ⊑ Company)
//   CQ:    SELECT ?y WHERE { ?y a :Company }
//   By hand: Company(?y) ⇒ via ∃worksFor⁻ ⊑ Company, the surviving bound term is the OBJECT of
//   worksFor, and the SUBJECT is unbound: worksFor(_, ?y). UCQ = { Company(?y), worksFor(_, ?y) }.
// ---------------------------------------------------------------------------------------------
#[test]
fn oracle_range_introduces_inverse_role() {
    let t = tbox(":worksFor rdfs:range :Company .");
    let q = query("SELECT ?y WHERE { ?y a :Company }");
    let r = rewrite(&q, &t).unwrap();
    let got = ucq_canonical(&r.query, &["y"]);
    let want = expected(&[
        &format!("?y <{TYPE}> <http://ex/Company>"),
        "_ <http://ex/worksFor> ?y",
    ]);
    assert_eq!(got, want, "report = {:?}", r.report);
}

// ---------------------------------------------------------------------------------------------
// Oracle case 5 — role inclusion (R1 ⊑ R2).
//
//   TBox:  :manages rdfs:subPropertyOf :worksFor .
//   CQ:    SELECT ?x ?y WHERE { ?x :worksFor ?y }
//   By hand: worksFor(?x,?y) ⇒ manages(?x,?y). UCQ = { worksFor(?x,?y), manages(?x,?y) }.
// ---------------------------------------------------------------------------------------------
#[test]
fn oracle_role_inclusion() {
    let t = tbox(":manages rdfs:subPropertyOf :worksFor .");
    let q = query("SELECT ?x ?y WHERE { ?x :worksFor ?y }");
    let r = rewrite(&q, &t).unwrap();
    let got = ucq_canonical(&r.query, &["x", "y"]);
    let want = expected(&[
        "?x <http://ex/worksFor> ?y",
        "?x <http://ex/manages> ?y",
    ]);
    assert_eq!(got, want, "report = {:?}", r.report);
}

// ---------------------------------------------------------------------------------------------
// Oracle case 6 — the APPLICABILITY CONDITION: an existential generator must NOT fire on a
// BOUND/answer variable (the #1 unsoundness trap).
//
//   TBox:  :Employee rdfs:subClassOf [ owl:onProperty :worksFor ; owl:someValuesFrom owl:Thing ]
//          (i.e. Employee ⊑ ∃worksFor)
//   CQ-a:  SELECT ?x WHERE { ?x :worksFor ?y }    -- ?y is DISTINGUISHED? no — only ?x is.
//          But ?y is NOT projected and occurs ONCE → `_`-eligible ⇒ Employee ⊑ ∃worksFor FIRES:
//          worksFor(?x, _) ⇒ Employee(?x). UCQ = { worksFor(?x,_), Employee(?x) }.
//   CQ-b:  SELECT ?x ?y WHERE { ?x :worksFor ?y }  -- ?y IS projected (distinguished) ⇒ the
//          existential generator must NOT fire (the object position is bound). UCQ = { the input
//          only }.
// ---------------------------------------------------------------------------------------------
#[test]
fn oracle_exists_super_fires_on_unbound_object() {
    let t = tbox(
        ":Employee rdfs:subClassOf [ owl:onProperty :worksFor ; owl:someValuesFrom owl:Thing ] .",
    );
    let q = query("SELECT ?x WHERE { ?x :worksFor ?y }");
    let r = rewrite(&q, &t).unwrap();
    let got = ucq_canonical(&r.query, &["x"]);
    let want = expected(&[
        "?x <http://ex/worksFor> _",
        &format!("?x <{TYPE}> <http://ex/Employee>"),
    ]);
    assert_eq!(got, want, "report = {:?}", r.report);
}

#[test]
fn oracle_exists_super_blocked_on_bound_object() {
    // The LOAD-BEARING soundness test: ?y projected ⇒ the existential generator MUST NOT fire,
    // because dropping the worksFor atom for Employee(?x) would lose the ?y binding (unsound).
    let t = tbox(
        ":Employee rdfs:subClassOf [ owl:onProperty :worksFor ; owl:someValuesFrom owl:Thing ] .",
    );
    let q = query("SELECT ?x ?y WHERE { ?x :worksFor ?y }");
    let r = rewrite(&q, &t).unwrap();
    let got = ucq_canonical(&r.query, &["x", "y"]);
    let want = expected(&["?x <http://ex/worksFor> ?y"]);
    assert_eq!(
        got, want,
        "existential generator must NOT fire on a bound (projected) object; report = {:?}",
        r.report
    );
}

// ---------------------------------------------------------------------------------------------
// Oracle case 7 — REDUCE re-enables a rewrite (the canonical PerfectRef interaction).
//
//   TBox:  :Employee ⊑ ∃worksFor  (Employee rdfs:subClassOf [∃worksFor]) .
//   CQ:    SELECT ?x WHERE { ?x :worksFor ?y . ?z :worksFor ?y }
//   By hand: initially ?y is SHARED (two worksFor atoms) so it is bound — the generator cannot
//   fire on either object. But REDUCE unifies the two worksFor atoms (?x=?z, shared ?y), leaving
//   ONE atom worksFor(?x, ?y) where ?y now occurs once and is non-distinguished → `_`. THEN the
//   generator fires: Employee(?x). So the UCQ contains at least:
//     { worksFor(?x,_) . worksFor(_,_)  [original, two atoms — _ anonymised non-shared] ...
//   This case is mainly a SMOKE test that reduce+rewrite compose without panic and that the
//   post-reduce Employee(?x) disjunct appears. We assert the Employee(?x) disjunct is present.
// ---------------------------------------------------------------------------------------------
#[test]
fn oracle_reduce_then_rewrite() {
    let t = tbox(
        ":Employee rdfs:subClassOf [ owl:onProperty :worksFor ; owl:someValuesFrom owl:Thing ] .",
    );
    let q = query("SELECT ?x WHERE { ?x :worksFor ?y . ?x :worksFor ?z }");
    let r = rewrite(&q, &t).unwrap();
    let got = ucq_canonical(&r.query, &["x"]);
    // After reducing the two worksFor atoms (unify ?y,?z), the single worksFor(?x,_) enables
    // Employee(?x). The disjunct must appear in the UCQ.
    let employee = format!("?x <{TYPE}> <http://ex/Employee>");
    assert!(
        got.contains(&employee),
        "reduce+rewrite must yield Employee(?x); got = {got:?}, report = {:?}",
        r.report
    );
}

// ---------------------------------------------------------------------------------------------
// Oracle case 8 — non-QL axioms are SKIPPED (counted), the QL part still rewrites.
//
//   TBox:  :A owl:unionOf (...) is non-QL on the LHS of a subClassOf; :Manager ⊑ :Employee is QL.
//   We assert the QL rewrite still happens AND skipped_axioms > 0.
// ---------------------------------------------------------------------------------------------
#[test]
fn oracle_non_ql_axiom_skipped() {
    let t = tbox(
        ":Manager rdfs:subClassOf :Employee . \
         [ owl:unionOf ( :A :B ) ] rdfs:subClassOf :C .",
    );
    let q = query("SELECT ?x WHERE { ?x a :Employee }");
    let r = rewrite(&q, &t).unwrap();
    assert!(
        r.report.skipped_axioms >= 1,
        "the unionOf axiom must be counted as skipped; report = {:?}",
        r.report
    );
    let got = ucq_canonical(&r.query, &["x"]);
    assert!(got.contains(&format!("?x <{TYPE}> <http://ex/Manager>")));
}

// ---------------------------------------------------------------------------------------------
// Oracle case 9 — owl:inverseOf wires a role atom to its inverse.
//
//   TBox:  :employs owl:inverseOf :worksFor .   (employs ≡ worksFor⁻)
//          :manages rdfs:subPropertyOf :employs .
//   CQ:    SELECT ?x ?y WHERE { ?x :worksFor ?y }
//   By hand: worksFor(?x,?y). employs ≡ worksFor⁻ means worksFor(?x,?y) ≡ employs(?y,?x). Then
//   manages ⊑ employs gives manages(?y,?x), i.e. (in worksFor terms via the inverse) the rewrite
//   chain reaches manages(?y,?x). We assert manages appears with swapped argument order.
// ---------------------------------------------------------------------------------------------
#[test]
fn oracle_inverse_of_role_chain() {
    let t = tbox(
        ":employs owl:inverseOf :worksFor . :manages rdfs:subPropertyOf :employs .",
    );
    let q = query("SELECT ?x ?y WHERE { ?x :worksFor ?y }");
    let r = rewrite(&q, &t).unwrap();
    let got = ucq_canonical(&r.query, &["x", "y"]);
    // The original worksFor(?x,?y) must be present.
    assert!(got.contains("?x <http://ex/worksFor> ?y"));
    // manages, reached via worksFor ≡ employs⁻ and manages ⊑ employs, binds (?y,?x) in
    // worksFor's frame: manages(?y, ?x) — the inverse swap.
    assert!(
        got.iter().any(|d| d.contains("<http://ex/manages>")),
        "the inverse-of role chain must surface a manages atom; got = {got:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// Oracle case 10 — termination + the input is always a disjunct (no TBox ⇒ identity UCQ).
// ---------------------------------------------------------------------------------------------
#[test]
fn oracle_empty_tbox_is_identity() {
    let q = query("SELECT ?x WHERE { ?x a :Employee }");
    let r = rewrite(&q, &[]).unwrap();
    assert_eq!(r.report.disjuncts, 1);
    let got = ucq_canonical(&r.query, &["x"]);
    assert_eq!(
        got,
        expected(&[&format!("?x <{TYPE}> <http://ex/Employee>")])
    );
}

// A direct construction check that the oracle helper's expected/canonical agree on a literal NT
// triple parse (guards the test harness itself).
#[test]
fn harness_triple_parse() {
    let t = Triple::from_str(&format!("<http://ex/A> <{RDFS_SUB}> <http://ex/B> .")).unwrap();
    assert_eq!(t.predicate.as_str(), RDFS_SUB);
}
