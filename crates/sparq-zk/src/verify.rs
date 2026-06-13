//! Verifier-side static re-check (plan §2.4, layer 3).
//!
//! The verifier never trusts prover-supplied plan structures: it re-derives
//! the stage-1 fragment's BGP patterns and join edges from the **query text
//! alone** (an independent `spargebra` parse), takes the manifest's *public*
//! per-pattern graph attributions, and computes which join edges could
//! equate values across graph boundaries. Every such edge carries a
//! **non-bnode obligation**: the joined value must be IRI- or literal-typed
//! (the only terms with cross-credential identity). A manifest that omits
//! one of the required obligations assumes a cross-graph bnode equality —
//! which RDF merge semantics forbid (Q6) — and is rejected.
//!
//! Relation to the prover-side guard ([`crate::trace`]'s rejection at
//! witness-build time): the prover sees concrete matched triples and rejects
//! precisely (only when an actual bnode is equated across actually-disjoint
//! attributions). The verifier cannot see values — bnodes are private — so
//! its check is necessarily **coarser**: an edge is flagged whenever the two
//! patterns' attribution sets admit *any* cross-graph pair (`|A_i ∪ A_j| >
//! 1`). Coarser is the safe direction — the verifier may demand a non-bnode
//! obligation the prover's concrete data never exercised, but can never
//! admit a correlating join the prover would have rejected. The two checks
//! are enforced independently (plan §2.4: a malicious prover cannot smuggle
//! a correlating join past an honest verifier).
//!
//! Fragment honesty (stage 1): SELECT/ASK over BGP + FILTER, with the
//! solution modifiers that do not change pattern input sets (DISTINCT /
//! REDUCED / LIMIT-OFFSET / projection). Everything else — property paths,
//! OPTIONAL, MINUS, UNION, `GRAPH` (contradictory with graph-set privacy,
//! plan §2.4), VALUES, BIND, aggregates, SERVICE — is rejected as outside
//! the verifiable fragment, failing closed.

use sparq_engine::zk::{PatternKey, SlotPattern};
use spargebra::algebra::{Expression, GraphPattern};
use spargebra::term::{NamedNodePattern, TermPattern, TriplePattern};
use spargebra::Query;
use std::collections::BTreeSet;

/// Verifier-side re-check failure. Every variant is a rejection: the
/// manifest/query pair must not be accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyError {
    /// The query text does not parse as SPARQL.
    Parse(String),
    /// The query is outside the stage-1 verifiable fragment.
    UnsupportedFragment(String),
    /// The manifest's attribution list does not match the query's pattern
    /// count (one public graph-set per BGP pattern, in query order).
    ArityMismatch { patterns: usize, attributions: usize },
    /// A pattern with an empty attribution set (a pattern no graph supports
    /// cannot contribute witnesses; the manifest is malformed).
    EmptyAttribution { pattern: usize },
    /// A required non-bnode join obligation is missing from the manifest:
    /// its join edge would otherwise assume a cross-graph bnode equality.
    MissingObligation(JoinEdge),
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::Parse(e) => write!(f, "query does not parse: {e}"),
            VerifyError::UnsupportedFragment(what) => {
                write!(f, "outside the stage-1 verifiable fragment: {what}")
            }
            VerifyError::ArityMismatch { patterns, attributions } => write!(
                f,
                "manifest attribution arity mismatch: query has {patterns} BGP patterns, manifest attributes {attributions}"
            ),
            VerifyError::EmptyAttribution { pattern } => {
                write!(f, "pattern {pattern} has an empty graph attribution set")
            }
            VerifyError::MissingObligation(edge) => write!(
                f,
                "rejected: join on ?{} between patterns {} and {} crosses graph boundaries without a non-bnode obligation — the manifest assumes a cross-graph bnode equality (Q6)",
                edge.variable, edge.patterns.0, edge.patterns.1
            ),
        }
    }
}

impl std::error::Error for VerifyError {}

/// One join edge of the query: `variable` is shared between the BGP patterns
/// at indices `patterns.0 < patterns.1` (indices into the query-order
/// pattern list returned by [`fragment_patterns`]).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JoinEdge {
    pub variable: String,
    pub patterns: (usize, usize),
}

/// Re-derives the stage-1 fragment's BGP triple patterns from the query
/// text, in query (text) order, as the same [`PatternKey`] shape the
/// zk-trace records — so manifest pattern references can be cross-checked
/// against an independent parse. Rejects queries outside the fragment.
pub fn fragment_patterns(sparql: &str) -> Result<Vec<PatternKey>, VerifyError> {
    let query = spargebra::SparqlParser::new()
        .parse_query(sparql)
        .map_err(|e| VerifyError::Parse(e.to_string()))?;
    let pattern = match &query {
        Query::Select { pattern, .. } | Query::Ask { pattern, .. } => pattern,
        Query::Construct { .. } => {
            return Err(VerifyError::UnsupportedFragment("CONSTRUCT".into()))
        }
        Query::Describe { .. } => {
            return Err(VerifyError::UnsupportedFragment("DESCRIBE".into()))
        }
    };
    let mut out = Vec::new();
    collect_patterns(pattern, &mut out)?;
    Ok(out)
}

fn collect_patterns(gp: &GraphPattern, out: &mut Vec<PatternKey>) -> Result<(), VerifyError> {
    match gp {
        GraphPattern::Bgp { patterns } => {
            for tp in patterns {
                out.push(triple_pattern_key(tp)?);
            }
            Ok(())
        }
        GraphPattern::Filter { expr, inner } => {
            // EXISTS/NOT EXISTS embed a graph pattern inside the FILTER
            // expression — non-monotone (NOT EXISTS) and traced unlabelled
            // by the engine seam; outside the stage-1 fragment.
            if expression_has_exists(expr) {
                return Err(VerifyError::UnsupportedFragment("EXISTS / NOT EXISTS".into()));
            }
            collect_patterns(inner, out)
        }
        GraphPattern::Join { left, right } => {
            collect_patterns(left, out)?;
            collect_patterns(right, out)
        }
        GraphPattern::Project { inner, .. }
        | GraphPattern::Distinct { inner }
        | GraphPattern::Reduced { inner }
        | GraphPattern::Slice { inner, .. } => collect_patterns(inner, out),
        GraphPattern::Path { .. } => {
            Err(VerifyError::UnsupportedFragment("property path".into()))
        }
        GraphPattern::LeftJoin { .. } => Err(VerifyError::UnsupportedFragment("OPTIONAL".into())),
        GraphPattern::Minus { .. } => Err(VerifyError::UnsupportedFragment("MINUS".into())),
        GraphPattern::Union { .. } => Err(VerifyError::UnsupportedFragment("UNION".into())),
        GraphPattern::Graph { .. } => Err(VerifyError::UnsupportedFragment(
            "GRAPH (contradictory with graph-set privacy, plan §2.4)".into(),
        )),
        GraphPattern::Extend { .. } => Err(VerifyError::UnsupportedFragment("BIND".into())),
        GraphPattern::Values { .. } => Err(VerifyError::UnsupportedFragment("VALUES".into())),
        GraphPattern::OrderBy { .. } => Err(VerifyError::UnsupportedFragment("ORDER BY".into())),
        GraphPattern::Group { .. } => Err(VerifyError::UnsupportedFragment("aggregates".into())),
        GraphPattern::Service { .. } => Err(VerifyError::UnsupportedFragment("SERVICE".into())),
        // Covers feature-gated variants the workspace enables on spargebra
        // (e.g. sep-0006 LATERAL): everything unknown is outside the fragment.
        other => Err(VerifyError::UnsupportedFragment(format!("{other:?}"))),
    }
}

/// Whether a FILTER expression contains an (NOT) EXISTS subpattern anywhere.
fn expression_has_exists(e: &Expression) -> bool {
    use Expression as E;
    match e {
        E::Exists(_) => true,
        E::NamedNode(_) | E::Literal(_) | E::Variable(_) | E::Bound(_) => false,
        E::Or(a, b)
        | E::And(a, b)
        | E::Equal(a, b)
        | E::SameTerm(a, b)
        | E::Greater(a, b)
        | E::GreaterOrEqual(a, b)
        | E::Less(a, b)
        | E::LessOrEqual(a, b)
        | E::Add(a, b)
        | E::Subtract(a, b)
        | E::Multiply(a, b)
        | E::Divide(a, b) => expression_has_exists(a) || expression_has_exists(b),
        E::UnaryPlus(a) | E::UnaryMinus(a) | E::Not(a) => expression_has_exists(a),
        E::In(a, list) => expression_has_exists(a) || list.iter().any(expression_has_exists),
        E::If(a, b, c) => {
            expression_has_exists(a) || expression_has_exists(b) || expression_has_exists(c)
        }
        E::Coalesce(list) => list.iter().any(expression_has_exists),
        E::FunctionCall(_, args) => args.iter().any(expression_has_exists),
    }
}

fn triple_pattern_key(tp: &TriplePattern) -> Result<PatternKey, VerifyError> {
    let term_slot = |t: &TermPattern| -> Result<SlotPattern, VerifyError> {
        match t {
            TermPattern::NamedNode(n) => Ok(SlotPattern::Term(oxrdf::Term::NamedNode(n.clone()))),
            TermPattern::Literal(l) => Ok(SlotPattern::Term(oxrdf::Term::Literal(l.clone()))),
            TermPattern::Variable(v) => Ok(SlotPattern::Var(v.as_str().to_string())),
            // A bnode in a query pattern is a non-distinguished variable
            // whose scoping rules are easy to get wrong on both sides of the
            // proof; stage 1 requires explicit variables instead.
            TermPattern::BlankNode(_) => Err(VerifyError::UnsupportedFragment(
                "blank node in a triple pattern (use an explicit variable)".into(),
            )),
            other => Err(VerifyError::UnsupportedFragment(format!(
                "triple pattern term {other}"
            ))),
        }
    };
    let predicate = match &tp.predicate {
        NamedNodePattern::NamedNode(n) => SlotPattern::Term(oxrdf::Term::NamedNode(n.clone())),
        NamedNodePattern::Variable(v) => SlotPattern::Var(v.as_str().to_string()),
    };
    Ok(PatternKey {
        slots: [term_slot(&tp.subject)?, predicate, term_slot(&tp.object)?],
    })
}

fn pattern_variables(key: &PatternKey) -> BTreeSet<&str> {
    key.slots
        .iter()
        .filter_map(|s| match s {
            SlotPattern::Var(v) => Some(v.as_str()),
            _ => None,
        })
        .collect()
}

/// Computes the join edges that REQUIRE a non-bnode obligation, given the
/// query-order patterns and the manifest's public per-pattern graph
/// attributions (`attributions[i]` = the set of committed-graph indices
/// pattern `i`'s witnesses may be drawn from).
///
/// An edge is required whenever the two patterns' attributions admit any
/// cross-graph pair — i.e. unless both patterns are supported by the same
/// single graph (`|A_i ∪ A_j| > 1`). See the module docs for why this is
/// deliberately coarser than the prover-side guard.
pub fn cross_graph_join_obligations(
    patterns: &[PatternKey],
    attributions: &[BTreeSet<usize>],
) -> Result<Vec<JoinEdge>, VerifyError> {
    if patterns.len() != attributions.len() {
        return Err(VerifyError::ArityMismatch {
            patterns: patterns.len(),
            attributions: attributions.len(),
        });
    }
    if let Some(i) = attributions.iter().position(BTreeSet::is_empty) {
        return Err(VerifyError::EmptyAttribution { pattern: i });
    }
    let vars: Vec<BTreeSet<&str>> = patterns.iter().map(pattern_variables).collect();
    let mut edges = Vec::new();
    for i in 0..patterns.len() {
        for j in (i + 1)..patterns.len() {
            let cross_possible = attributions[i].union(&attributions[j]).count() > 1;
            if !cross_possible {
                continue;
            }
            for v in vars[i].intersection(&vars[j]) {
                edges.push(JoinEdge {
                    variable: (*v).to_string(),
                    patterns: (i, j),
                });
            }
        }
    }
    Ok(edges)
}

/// The layer-3 verifier gate: re-derives the fragment patterns from the
/// query text, computes the required non-bnode join obligations under the
/// manifest's attributions, and rejects unless every required edge is
/// declared by the manifest (`declared` = the manifest's non-bnode
/// obligations; extra declared edges are harmless — they only constrain the
/// prover further).
pub fn recheck(
    sparql: &str,
    attributions: &[BTreeSet<usize>],
    declared: &[JoinEdge],
) -> Result<Vec<JoinEdge>, VerifyError> {
    let patterns = fragment_patterns(sparql)?;
    let required = cross_graph_join_obligations(&patterns, attributions)?;
    for edge in &required {
        if !declared.contains(edge) {
            return Err(VerifyError::MissingObligation(edge.clone()));
        }
    }
    Ok(required)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(sets: &[&[usize]]) -> Vec<BTreeSet<usize>> {
        sets.iter().map(|s| s.iter().copied().collect()).collect()
    }

    #[test]
    fn patterns_re_derived_in_query_order() {
        let keys = fragment_patterns(
            "SELECT ?n WHERE { ?p <http://ex/worksAt> ?org . ?org <http://ex/name> ?n . FILTER(?n != \"x\") }",
        )
        .unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].slots[2], SlotPattern::Var("org".into()));
        assert_eq!(keys[1].slots[0], SlotPattern::Var("org".into()));
        assert!(matches!(&keys[0].slots[1], SlotPattern::Term(t) if t.to_string().contains("worksAt")));
    }

    #[test]
    fn outside_fragment_fails_closed() {
        for (q, what) in [
            ("SELECT * WHERE { ?s ?p ?o OPTIONAL { ?o ?q ?r } }", "OPTIONAL"),
            ("SELECT * WHERE { GRAPH ?g { ?s ?p ?o } }", "GRAPH"),
            ("SELECT * WHERE { { ?s ?p ?o } UNION { ?o ?p ?s } }", "UNION"),
            // One-or-more survives as an algebra-level Path…
            ("SELECT * WHERE { ?s <http://ex/a>+ ?o }", "property path"),
            // …while a fixed-length sequence path is flattened by the parser
            // into a BGP with an intermediate blank node — caught by the
            // bnode-in-pattern rejection (still fail-closed).
            ("SELECT * WHERE { ?s <http://ex/a>/<http://ex/b> ?o }", "blank node"),
            ("SELECT * WHERE { ?s ?p ?o BIND(1 AS ?x) }", "BIND"),
            (
                "SELECT * WHERE { ?s ?p ?o FILTER NOT EXISTS { ?o ?q ?r } }",
                "EXISTS",
            ),
            ("SELECT * WHERE { _:b ?p ?o }", "blank node"),
        ] {
            let err = fragment_patterns(q).unwrap_err();
            match err {
                VerifyError::UnsupportedFragment(msg) => {
                    assert!(msg.contains(what), "{q}: expected {what}, got {msg}")
                }
                other => panic!("{q}: expected UnsupportedFragment, got {other:?}"),
            }
        }
        assert!(matches!(
            fragment_patterns("SELECT WHERE"),
            Err(VerifyError::Parse(_))
        ));
    }

    #[test]
    fn same_single_graph_join_needs_no_obligation() {
        let keys = fragment_patterns(
            "SELECT * WHERE { ?s <http://ex/p> ?x . ?x <http://ex/q> ?o }",
        )
        .unwrap();
        let edges = cross_graph_join_obligations(&keys, &attrs(&[&[0], &[0]])).unwrap();
        assert!(edges.is_empty(), "within-graph joins may bind bnodes");
    }

    #[test]
    fn cross_graph_join_requires_obligation() {
        let keys = fragment_patterns(
            "SELECT * WHERE { ?s <http://ex/p> ?x . ?x <http://ex/q> ?o }",
        )
        .unwrap();
        // Disjoint attributions.
        let edges = cross_graph_join_obligations(&keys, &attrs(&[&[0], &[1]])).unwrap();
        assert_eq!(
            edges,
            vec![JoinEdge { variable: "x".into(), patterns: (0, 1) }]
        );
        // Overlapping-but-plural attributions are still cross-capable
        // (coarser than the prover guard, by design).
        let edges = cross_graph_join_obligations(&keys, &attrs(&[&[0], &[0, 1]])).unwrap();
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn recheck_rejects_undeclared_edge_and_accepts_declared() {
        let q = "SELECT * WHERE { ?s <http://ex/p> ?x . ?x <http://ex/q> ?o }";
        let a = attrs(&[&[0], &[1]]);
        let err = recheck(q, &a, &[]).unwrap_err();
        assert!(matches!(err, VerifyError::MissingObligation(_)));
        let edge = JoinEdge { variable: "x".into(), patterns: (0, 1) };
        let required = recheck(q, &a, std::slice::from_ref(&edge)).unwrap();
        assert_eq!(required, vec![edge]);
    }

    #[test]
    fn malformed_manifests_rejected() {
        let keys =
            fragment_patterns("SELECT * WHERE { ?s <http://ex/p> ?x . ?x <http://ex/q> ?o }")
                .unwrap();
        assert_eq!(
            cross_graph_join_obligations(&keys, &attrs(&[&[0]])),
            Err(VerifyError::ArityMismatch { patterns: 2, attributions: 1 })
        );
        assert_eq!(
            cross_graph_join_obligations(&keys, &attrs(&[&[0], &[]])),
            Err(VerifyError::EmptyAttribution { pattern: 1 })
        );
    }
}
