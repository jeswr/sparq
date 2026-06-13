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

// [OPUS-4.8] FILTER query-binding (audit #5/#6/#10) — extract the query's
// FILTER comparisons and the variable→(pattern,slot) map so the compose
// verifier can require a matching, slot-bound, verdict-gated filter sub-proof.

/// A numeric comparison operator extracted from a query FILTER. Mirrors the
/// `filter_int` circuit's `OP_*` codes; `sparq-zk-compose`'s `FilterOp` maps
/// 1:1 to this (the manifest enum lives downstream, so this neutral enum is
/// what the query parser yields). `code()` is the in-circuit `op` value the
/// reconstructed public-input vector carries — the compose verifier compares
/// it against the bound sub-proof's `op`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterCmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl FilterCmp {
    /// The `op` public-input value the `filter_int` circuit expects (identical
    /// to `sparq_zk_compose::manifest::FilterOp::code`).
    pub fn code(self) -> u32 {
        match self {
            FilterCmp::Lt => 0,
            FilterCmp::Le => 1,
            FilterCmp::Gt => 2,
            FilterCmp::Ge => 3,
            FilterCmp::Eq => 4,
            FilterCmp::Ne => 5,
        }
    }
}

/// One query FILTER comparison the verifier must find a bound `filter_int`
/// sub-proof for: `?variable op bound`, where `bound` is the xsd:integer
/// constant operand parsed from the literal. Only the integer-FILTER fragment
/// the `filter_int` circuit supports is extracted; any other FILTER shape
/// (float, string, arithmetic, `?a op ?b`, `IN`, …) makes the query fall
/// outside the bindable fragment and is rejected by [`collect_filters`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryFilter {
    /// The variable the FILTER constrains (its operand column).
    pub variable: String,
    /// The comparison operator (already normalized to `?var op const` form;
    /// a `const op ?var` FILTER is flipped so the variable is on the left).
    pub op: FilterCmp,
    /// The xsd:integer constant operand, as a non-negative decimal.
    pub bound: u64,
}

/// xsd:integer datatype IRI (the only literal datatype the `filter_int`
/// circuit can bind — see `encode.rs` / `filter_int.nr`).
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

/// Flip a comparison so `const op ?var` becomes the equivalent `?var op' const`.
fn flip_cmp(op: FilterCmp) -> FilterCmp {
    match op {
        FilterCmp::Lt => FilterCmp::Gt,
        FilterCmp::Le => FilterCmp::Ge,
        FilterCmp::Gt => FilterCmp::Lt,
        FilterCmp::Ge => FilterCmp::Le,
        FilterCmp::Eq => FilterCmp::Eq,
        FilterCmp::Ne => FilterCmp::Ne,
    }
}

/// Parse one comparison operand pair `(a, b)` under `op` into a normalized
/// `QueryFilter` (`?var op const`). `None` if it is not a bindable
/// integer-FILTER (e.g. `?a op ?b`, a non-integer / non-canonical literal, or
/// `const op const`). A non-canonical xsd:integer lexical form (leading zero,
/// sign, whitespace) is rejected: `filter_int.nr` can only bind a canonical
/// non-negative decimal token, so a non-canonical bound could never be matched
/// by any honest proof and must fail closed (audit hardening, lexical-form
/// note).
fn comparison_filter(op: FilterCmp, a: &Expression, b: &Expression) -> Option<QueryFilter> {
    let var_of = |e: &Expression| match e {
        Expression::Variable(v) => Some(v.as_str().to_string()),
        _ => None,
    };
    let int_of = |e: &Expression| match e {
        Expression::Literal(l) if l.datatype().as_str() == XSD_INTEGER => {
            canonical_u64(l.value())
        }
        _ => None,
    };
    match (var_of(a), int_of(b)) {
        (Some(variable), Some(bound)) => return Some(QueryFilter { variable, op, bound }),
        _ => {}
    }
    // `const op ?var` — flip so the variable is on the left.
    match (int_of(a), var_of(b)) {
        (Some(bound), Some(variable)) => {
            Some(QueryFilter { variable, op: flip_cmp(op), bound })
        }
        _ => None,
    }
}

/// Parse a canonical non-negative xsd:integer lexical form to `u64`. Rejects
/// leading zeros, signs, and any non-digit byte — only the exact token the
/// circuit's `enc_int_literal` replica can bind round-trips (mirrors
/// `filter_int.nr`'s digit-token constraints).
fn canonical_u64(lex: &str) -> Option<u64> {
    if lex.is_empty() || !lex.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if lex.len() > 1 && lex.starts_with('0') {
        return None; // non-canonical leading zero
    }
    lex.parse::<u64>().ok()
}

/// Walk a FILTER expression and collect every top-level integer comparison it
/// imposes. `AND` chains are flattened (each conjunct is its own obligation).
/// Any FILTER node that is NOT a flat conjunction of bindable integer
/// comparisons (negation, disjunction, arithmetic, `?a op ?b`, function calls,
/// non-integer literals, …) makes the query fall outside the bindable fragment:
/// the function returns `Err(UnsupportedFragment)` so a FILTER the binding layer
/// cannot vouch for fails CLOSED rather than being silently disclosed unproven
/// (audit #10: a FILTER must have a bound sub-proof, so an unbindable FILTER
/// must be rejected, never ignored).
fn collect_filter_expr(e: &Expression, out: &mut Vec<QueryFilter>) -> Result<(), VerifyError> {
    use Expression as E;
    match e {
        E::And(a, b) => {
            collect_filter_expr(a, out)?;
            collect_filter_expr(b, out)
        }
        E::Less(a, b) => push_cmp(FilterCmp::Lt, a, b, out),
        E::LessOrEqual(a, b) => push_cmp(FilterCmp::Le, a, b, out),
        E::Greater(a, b) => push_cmp(FilterCmp::Gt, a, b, out),
        E::GreaterOrEqual(a, b) => push_cmp(FilterCmp::Ge, a, b, out),
        E::Equal(a, b) => push_cmp(FilterCmp::Eq, a, b, out),
        other => Err(VerifyError::UnsupportedFragment(format!(
            "FILTER expression not a bindable integer comparison: {other:?}"
        ))),
    }
}

fn push_cmp(
    op: FilterCmp,
    a: &Expression,
    b: &Expression,
    out: &mut Vec<QueryFilter>,
) -> Result<(), VerifyError> {
    match comparison_filter(op, a, b) {
        Some(qf) => {
            out.push(qf);
            Ok(())
        }
        None => Err(VerifyError::UnsupportedFragment(
            "FILTER comparison is not `?var <op> <xsd:integer>` (the bindable fragment)".into(),
        )),
    }
}

/// Re-derive the query's FILTER obligations from the query text alone, in the
/// same independent `spargebra` parse `fragment_patterns` uses. Returns one
/// [`QueryFilter`] per integer comparison. Rejects any FILTER outside the
/// bindable integer fragment (so it cannot be silently disclosed unproven).
pub fn fragment_filters(sparql: &str) -> Result<Vec<QueryFilter>, VerifyError> {
    let query = spargebra::SparqlParser::new()
        .parse_query(sparql)
        .map_err(|e| VerifyError::Parse(e.to_string()))?;
    let pattern = match &query {
        Query::Select { pattern, .. } | Query::Ask { pattern, .. } => pattern,
        Query::Construct { .. } => return Err(VerifyError::UnsupportedFragment("CONSTRUCT".into())),
        Query::Describe { .. } => return Err(VerifyError::UnsupportedFragment("DESCRIBE".into())),
    };
    let mut out = Vec::new();
    collect_filters_gp(pattern, &mut out)?;
    Ok(out)
}

fn collect_filters_gp(gp: &GraphPattern, out: &mut Vec<QueryFilter>) -> Result<(), VerifyError> {
    match gp {
        GraphPattern::Bgp { .. } => Ok(()),
        GraphPattern::Filter { expr, inner } => {
            if expression_has_exists(expr) {
                return Err(VerifyError::UnsupportedFragment("EXISTS / NOT EXISTS".into()));
            }
            collect_filter_expr(expr, out)?;
            collect_filters_gp(inner, out)
        }
        GraphPattern::Join { left, right } => {
            collect_filters_gp(left, out)?;
            collect_filters_gp(right, out)
        }
        GraphPattern::Project { inner, .. }
        | GraphPattern::Distinct { inner }
        | GraphPattern::Reduced { inner }
        | GraphPattern::Slice { inner, .. } => collect_filters_gp(inner, out),
        // Every other node is already rejected by `collect_patterns`; the
        // verifier always calls `recheck` (which runs `fragment_patterns`)
        // before/with the filter extraction, so a structurally-unsupported
        // query never reaches a point where its FILTERs matter. Fail closed
        // anyway for totality.
        other => Err(VerifyError::UnsupportedFragment(format!("{other:?}"))),
    }
}

/// For each variable, the set of `(pattern_index, slot_index)` positions it
/// occupies across the query-order BGP patterns. The compose verifier uses
/// this to require a FILTER's bound sub-proof to reference exactly the scanned
/// slot the FILTER variable binds to (audit #6).
pub fn variable_slots(patterns: &[PatternKey]) -> Vec<(String, usize, usize)> {
    let mut out = Vec::new();
    for (pi, key) in patterns.iter().enumerate() {
        for (si, slot) in key.slots.iter().enumerate() {
            if let SlotPattern::Var(v) = slot {
                out.push((v.clone(), pi, si));
            }
        }
    }
    out
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

    // [OPUS-4.8] FILTER query-binding extraction tests (audit #5/#6/#10).

    fn xint(v: u64) -> String {
        format!(
            "\"{v}\"^^<http://www.w3.org/2001/XMLSchema#integer>"
        )
    }

    #[test]
    fn extracts_integer_filter_in_both_operand_orders() {
        let q = format!(
            "SELECT ?s ?age WHERE {{ ?s <http://ex/age> ?age FILTER(?age >= {}) }}",
            xint(18)
        );
        let fs = fragment_filters(&q).unwrap();
        assert_eq!(
            fs,
            vec![QueryFilter { variable: "age".into(), op: FilterCmp::Ge, bound: 18 }]
        );
        // `const op ?var` flips to `?var op' const`.
        let q2 = format!(
            "SELECT ?s ?age WHERE {{ ?s <http://ex/age> ?age FILTER({} <= ?age) }}",
            xint(18)
        );
        let fs2 = fragment_filters(&q2).unwrap();
        assert_eq!(
            fs2,
            vec![QueryFilter { variable: "age".into(), op: FilterCmp::Ge, bound: 18 }],
            "18 <= ?age must normalize to ?age >= 18"
        );
    }

    #[test]
    fn flattens_conjoined_filters() {
        let q = format!(
            "SELECT ?s ?age WHERE {{ ?s <http://ex/age> ?age FILTER(?age >= {} && ?age < {}) }}",
            xint(18),
            xint(65)
        );
        let fs = fragment_filters(&q).unwrap();
        assert_eq!(
            fs,
            vec![
                QueryFilter { variable: "age".into(), op: FilterCmp::Ge, bound: 18 },
                QueryFilter { variable: "age".into(), op: FilterCmp::Lt, bound: 65 },
            ]
        );
    }

    #[test]
    fn no_filter_yields_empty() {
        let fs = fragment_filters("SELECT ?s ?o WHERE { ?s <http://ex/age> ?o }").unwrap();
        assert!(fs.is_empty());
    }

    #[test]
    fn unbindable_filters_fail_closed() {
        // Non-integer / arithmetic / var-var / disjunction / non-canonical:
        // each is OUTSIDE the bindable fragment and must REJECT (never be
        // silently disclosed unproven — audit #10).
        for q in [
            // float literal
            "SELECT ?s WHERE { ?s <http://ex/h> ?h FILTER(?h >= \"1.5\"^^<http://www.w3.org/2001/XMLSchema#double>) }",
            // plain string (no xsd:integer datatype)
            "SELECT ?s WHERE { ?s <http://ex/h> ?h FILTER(?h >= \"18\") }",
            // var vs var
            "SELECT ?s WHERE { ?s <http://ex/a> ?a . ?s <http://ex/b> ?b FILTER(?a >= ?b) }",
            // disjunction (not a flat AND of comparisons)
            "SELECT ?s WHERE { ?s <http://ex/age> ?a FILTER(?a >= \"18\"^^<http://www.w3.org/2001/XMLSchema#integer> || ?a < \"0\"^^<http://www.w3.org/2001/XMLSchema#integer>) }",
            // arithmetic operand
            "SELECT ?s WHERE { ?s <http://ex/age> ?a FILTER(?a + \"1\"^^<http://www.w3.org/2001/XMLSchema#integer> >= \"18\"^^<http://www.w3.org/2001/XMLSchema#integer>) }",
            // non-canonical leading-zero integer
            "SELECT ?s WHERE { ?s <http://ex/age> ?a FILTER(?a >= \"018\"^^<http://www.w3.org/2001/XMLSchema#integer>) }",
        ] {
            assert!(
                matches!(fragment_filters(q), Err(VerifyError::UnsupportedFragment(_))),
                "must fail closed: {q}"
            );
        }
    }

    #[test]
    fn variable_slots_maps_var_positions() {
        let keys = fragment_patterns(
            "SELECT ?s ?age WHERE { ?s <http://ex/age> ?age . ?s <http://ex/sal> ?sal }",
        )
        .unwrap();
        let vs = variable_slots(&keys);
        // ?s at (0,0) and (1,0); ?age at (0,2); ?sal at (1,2).
        assert!(vs.contains(&("s".into(), 0, 0)));
        assert!(vs.contains(&("s".into(), 1, 0)));
        assert!(vs.contains(&("age".into(), 0, 2)));
        assert!(vs.contains(&("sal".into(), 1, 2)));
    }
}
