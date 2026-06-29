// [OPUS-4.8] sq-t5bne (epic sq-pbz04): map between spargebra CQ atoms and the rewriter's
// internal [`Atom`]s, and fold the rewritten UCQ back into a spargebra `Query`.
//
// The seam is the same one sparq-vectors proves: produce a rewritten `spargebra::Query` so the
// engine — planner, executor — runs the UCQ unchanged (no store/planner changes). A UCQ folds
// to a left-deep tree of `Union`s of `Bgp`s, wrapped in the ORIGINAL projection (+ Distinct).
//
// This whole module is behind `experimental` (it is only reachable from the gated rewriter).

#![cfg(feature = "experimental")]

use crate::cq::{ConjunctiveQuery, CqError};
use crate::dllite::rdf_type_iri;
use crate::perfectref::{Atom, Cq, Term};
use crate::dllite::Role;
use spargebra::algebra::GraphPattern;
use spargebra::term::{NamedNode, NamedNodePattern, TermPattern, TriplePattern, Variable};

/// Map the gated CQ's triple patterns to internal atoms + the answer-variable names. Returns the
/// (atoms, answer) pair PerfectRef consumes. Errors (OutOfScope) if a triple pattern is not a
/// DL-Lite atom in a way the CQ-shape gate did not already reject (defence in depth: e.g. a
/// blank-node subject/object, or a literal in a position DL-Lite atoms never carry).
pub fn cq_to_atoms(cq: &ConjunctiveQuery) -> Result<(Vec<Atom>, Vec<String>), CqError> {
    let rdf_type = rdf_type_iri();
    let mut atoms = Vec::with_capacity(cq.atoms.len());
    for tp in &cq.atoms {
        let pred = match &tp.predicate {
            NamedNodePattern::NamedNode(n) => n.clone(),
            // The gate already rejects variable predicates; keep the arm total/honest.
            NamedNodePattern::Variable(_) => {
                return Err(CqError::OutOfScope(
                    "variable predicate is not a DL-Lite atom".into(),
                ))
            }
        };
        if pred == rdf_type {
            // Class atom: subject is the arg, object must be a NAMED class.
            let arg = term_pattern_to_term(&tp.subject)?;
            let class = match &tp.object {
                TermPattern::NamedNode(c) => c.as_str().to_string(),
                _ => {
                    return Err(CqError::OutOfScope(
                        "rdf:type object must be a named class for DL-Lite rewriting".into(),
                    ))
                }
            };
            atoms.push(Atom::Class { class, arg });
        } else {
            // Role atom: subject + object are terms, predicate is the role IRI.
            let s = term_pattern_to_term(&tp.subject)?;
            let o = term_pattern_to_term(&tp.object)?;
            atoms.push(Atom::Role {
                role: Role::named(pred.as_str()),
                s,
                o,
            });
        }
    }
    let answer = cq.distinguished.iter().map(|v| v.as_str().to_string()).collect();
    Ok((atoms, answer))
}

/// Map a spargebra `TermPattern` to an internal [`Term`]. Variables → `Var`, named nodes →
/// `Const`. Blank nodes / literals / quoted triples are NOT DL-Lite atom terms → OutOfScope.
fn term_pattern_to_term(t: &TermPattern) -> Result<Term, CqError> {
    match t {
        TermPattern::Variable(v) => Ok(Term::Var(v.as_str().to_string())),
        TermPattern::NamedNode(n) => Ok(Term::Const(n.as_str().to_string())),
        TermPattern::BlankNode(_) => Err(CqError::OutOfScope(
            "blank-node term is out of DL-Lite atom scope".into(),
        )),
        TermPattern::Literal(_) => Err(CqError::OutOfScope(
            "literal in a class/role atom position is out of DL-Lite scope".into(),
        )),
        // `TermPattern::Triple` exists because spargebra is built with `sparql-12` (workspace
        // dep). Match it directly — a `cfg(feature = "sparql-12")` would test THIS crate.
        TermPattern::Triple(_) => Err(CqError::OutOfScope(
            "RDF-star quoted-triple term is out of DL-Lite scope".into(),
        )),
    }
}

/// Fold a UCQ (the rewritten CQ set) into one spargebra `GraphPattern` body: a tree of `Union`s
/// of `Bgp`s, in a DETERMINISTIC order (CQs sorted by their textual atom signature) so the
/// rewrite output is stable run-to-run. An empty UCQ is impossible (the input CQ is always
/// present), but is handled defensively as an empty BGP.
pub fn ucq_to_pattern(mut ucq: Vec<Cq>) -> GraphPattern {
    // Deterministic order: sort by a stable signature of each CQ's normalised atoms.
    ucq.sort_by_cached_key(cq_signature);
    let mut bgps: Vec<GraphPattern> = ucq.iter().map(cq_to_bgp).collect();
    bgps.dedup(); // structurally-identical disjuncts collapse (sound: UCQ is a set).
    match bgps.len() {
        0 => GraphPattern::Bgp { patterns: vec![] },
        1 => bgps.pop().unwrap(),
        _ => {
            // Left-deep Union fold.
            let mut iter = bgps.into_iter();
            let mut acc = iter.next().unwrap();
            for next in iter {
                acc = GraphPattern::Union {
                    left: Box::new(acc),
                    right: Box::new(next),
                };
            }
            acc
        }
    }
}

/// A stable textual signature of a CQ for deterministic UCQ ordering.
fn cq_signature(cq: &Cq) -> String {
    let mut parts: Vec<String> = cq.atoms.iter().map(atom_signature).collect();
    parts.sort();
    parts.join(" ")
}

fn atom_signature(a: &Atom) -> String {
    match a {
        Atom::Class { class, arg } => format!("C|{}|{}", class, term_sig(arg)),
        Atom::Role { role, s, o } => format!(
            "R|{}|{}|{}|{}",
            role.iri,
            role.inverse,
            term_sig(s),
            term_sig(o)
        ),
    }
}

fn term_sig(t: &Term) -> String {
    match t {
        Term::Var(v) => format!("?{}", v),
        Term::Const(c) => format!("<{}>", c),
        Term::Unbound(_) => "_".to_string(),
    }
}

/// Build a `Bgp` from one CQ, mapping each atom to a triple pattern. Unbound `_` terms become
/// fresh, deterministically-named non-distinguished variables (`__ql<n>`) — the standard SPARQL
/// encoding of an existential BGP variable. The naming is per-Bgp so two disjuncts never clash.
fn cq_to_bgp(cq: &Cq) -> GraphPattern {
    let rdf_type = NamedNodePattern::NamedNode(rdf_type_iri());
    let mut next_fresh = 0u32;
    let mut fresh_name = |id: u32| {
        // Map an internal Unbound id to a stable per-Bgp variable name. Use a small local index
        // so the emitted names are compact and order-stable.
        let _ = id;
        let n = next_fresh;
        next_fresh += 1;
        Variable::new_unchecked(format!("__ql{}", n))
    };
    let mut patterns = Vec::with_capacity(cq.atoms.len());
    for atom in &cq.atoms {
        match atom {
            Atom::Class { class, arg } => {
                patterns.push(TriplePattern {
                    subject: term_to_pattern(arg, &mut fresh_name),
                    predicate: rdf_type.clone(),
                    object: TermPattern::NamedNode(NamedNode::new_unchecked(class.clone())),
                });
            }
            Atom::Role { role, s, o } => {
                // Roles are stored forward (the rewriter forward-normalises inverses), so emit
                // the named property directly between subject and object.
                patterns.push(TriplePattern {
                    subject: term_to_pattern(s, &mut fresh_name),
                    predicate: NamedNodePattern::NamedNode(NamedNode::new_unchecked(
                        role.iri.clone(),
                    )),
                    object: term_to_pattern(o, &mut fresh_name),
                });
            }
        }
    }
    GraphPattern::Bgp { patterns }
}

fn term_to_pattern(t: &Term, fresh: &mut impl FnMut(u32) -> Variable) -> TermPattern {
    match t {
        Term::Var(v) => TermPattern::Variable(Variable::new_unchecked(v.clone())),
        Term::Const(c) => TermPattern::NamedNode(NamedNode::new_unchecked(c.clone())),
        Term::Unbound(id) => TermPattern::Variable(fresh(*id)),
    }
}
