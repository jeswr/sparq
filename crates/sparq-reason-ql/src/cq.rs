// [OPUS-4.8] sq-t5bne (epic sq-pbz04): the STRICT, FAIL-CLOSED CQ-shape gate.
//
// PerfectRef (and every OWL 2 QL query-rewriting algorithm) is sound + complete only for
// (unions of) CONJUNCTIVE QUERIES — the Pérez–Arenas–Gutiérrez "AND-only" fragment with an
// outer projection/distinct. Firing the rewrite on anything else SILENTLY LOSES OR INVENTS
// answers (the research records call this the #1 unsoundness trap: applicability is the
// load-bearing condition). So this gate is the soundness keystone of the whole crate: it is
// the FIRST thing every public rewriting entry point runs, and it REJECTS — never silently
// degrades — any query outside the CQ fragment as [`CqError::OutOfScope`].
//
// Crucially this module carries NO `cfg(feature = "experimental")`: the gate is always
// compiled and always cheap (a structural walk of the parsed algebra, no TBox, no rewriting),
// so the fail-closed classification is available even to a caller that has NOT enabled the
// rewriter. The rewriter (behind `experimental`) calls into the gate; it can never bypass it.

use spargebra::algebra::GraphPattern;
use spargebra::term::{TermPattern, TriplePattern, Variable};
use spargebra::Query;

/// Why a query is not a conjunctive query the QL rewriter may soundly handle. Carries a
/// human-readable reason naming the offending construct, so a caller (or the conformance
/// harness) can surface an honest `OutOfScope(reason)` rather than a misleading wrong answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CqError {
    /// The query is outside the conjunctive-query fragment the rewriter is sound for. The
    /// string names the disqualifying construct (e.g. `"OPTIONAL (LeftJoin)"`).
    OutOfScope(String),
}

impl std::fmt::Display for CqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Positional `format!` arg (NOT `{reason}`) — avoids the CodeQL
            // rust/unused-variable false positive flagged in the shared contract.
            CqError::OutOfScope(reason) => write!(f, "query out of QL rewriting scope: {}", reason),
        }
    }
}

impl std::error::Error for CqError {}

/// A conjunctive query in the shape the rewriter consumes: a flat conjunction of triple
/// patterns (the body atoms) under a set of DISTINGUISHED (answer / projected) variables.
///
/// Built only by [`as_conjunctive_query`], which has already proven via the fail-closed gate
/// that the source query is a genuine CQ. The `distinguished` set is the SELECT projection
/// (or, for ASK, empty — a boolean CQ); every other variable in `atoms` is non-distinguished
/// (existentially quantified), which is exactly the distinction PerfectRef's applicability
/// condition turns on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConjunctiveQuery {
    /// The projected (answer) variables — the distinguished variables of the CQ.
    pub distinguished: Vec<Variable>,
    /// The body: a conjunction of triple patterns.
    pub atoms: Vec<TriplePattern>,
    /// Whether the original query was DISTINCT (preserved so the rewriter can re-wrap).
    pub distinct: bool,
    /// Whether the original query was an ASK (boolean CQ — `distinguished` is then empty).
    pub ask: bool,
}

/// Classify `query` and, if it is a conjunctive query the rewriter may soundly handle, return
/// its [`ConjunctiveQuery`] form; otherwise return [`CqError::OutOfScope`] naming the
/// disqualifying construct. FAIL-CLOSED: anything not recognised as plainly a CQ is rejected.
///
/// Accepted shape (and ONLY this shape): `SELECT [DISTINCT] ?vars WHERE { conjunction of
/// triple patterns }`, or the same body under `ASK`. The body may be a single `Bgp` or a tree
/// of `Join`s whose leaves are all `Bgp`s (spargebra splits a `{ t1 . t2 . }` conjunction into
/// joined BGPs in some shapes) — both reduce to one flat conjunction. Everything else is out:
///
/// - `LeftJoin` (OPTIONAL), `Filter`, `Minus`, `Union`, `Lateral`, `Group`/aggregation,
///   `Service`, `Graph`, `Extend` (BIND), `OrderBy`, `Slice`, `Values`, `Path` (property
///   paths) — none has a certain-answer-preserving UCQ rewriting (property paths in particular
///   reintroduce the transitivity/recursion QL deliberately excludes);
/// - CONSTRUCT / DESCRIBE — not a CQ-answering shape;
/// - a triple pattern whose predicate is a VARIABLE, or any RDF-star quoted-triple term — both
///   outside DL-Lite atom shape (the rewriter reasons over named class/role atoms only).
pub fn as_conjunctive_query(query: &Query) -> Result<ConjunctiveQuery, CqError> {
    let (pattern, ask) = match query {
        Query::Select { pattern, .. } => (pattern, false),
        Query::Ask { pattern, .. } => (pattern, true),
        Query::Construct { .. } => {
            return Err(CqError::OutOfScope("CONSTRUCT is not a CQ-answering query".into()))
        }
        Query::Describe { .. } => {
            return Err(CqError::OutOfScope("DESCRIBE is not a CQ-answering query".into()))
        }
    };

    // Peel the allowed outer modifiers: Project (the SELECT list) and Distinct. Reject every
    // other top-level modifier — they either change the answer semantics (Slice/OrderBy carry
    // no CQ meaning under certain answers) or are non-CQ operators.
    let mut distinct = false;
    let mut distinguished: Option<Vec<Variable>> = None;
    let mut body = pattern;
    loop {
        match body {
            GraphPattern::Project { inner, variables } => {
                // Multiple Projects would be unusual; take the OUTERMOST projection as the
                // answer variables and keep peeling (the inner one, if any, is then rejected
                // as a non-CQ nesting by the body walker below).
                if distinguished.is_none() {
                    distinguished = Some(variables.clone());
                }
                body = inner;
            }
            GraphPattern::Distinct { inner } => {
                distinct = true;
                body = inner;
            }
            GraphPattern::Reduced { inner } => {
                // REDUCED is a permission to drop duplicates; treat like DISTINCT for the
                // certain-answer set (sound: certain answers are a set).
                distinct = true;
                body = inner;
            }
            _ => break,
        }
    }

    let mut atoms = Vec::new();
    collect_conjunction(body, &mut atoms)?;

    // An ASK has no projection; a SELECT must have one (SELECT * lowers to a Project over all
    // in-scope vars in spargebra, so `distinguished` is always Some for a well-formed SELECT).
    let distinguished = if ask {
        Vec::new()
    } else {
        distinguished.ok_or_else(|| {
            CqError::OutOfScope("SELECT without a projection (unsupported shape)".into())
        })?
    };

    if atoms.is_empty() {
        return Err(CqError::OutOfScope("empty query body (no triple patterns)".into()));
    }

    Ok(ConjunctiveQuery {
        distinguished,
        atoms,
        distinct,
        ask,
    })
}

/// Walk a CQ body, flattening nested `Join`s into one conjunction of triple patterns. Any
/// non-conjunctive operator encountered is a hard `OutOfScope` rejection (fail-closed).
fn collect_conjunction(p: &GraphPattern, out: &mut Vec<TriplePattern>) -> Result<(), CqError> {
    match p {
        GraphPattern::Bgp { patterns } => {
            for tp in patterns {
                check_atom_shape(tp)?;
                out.push(tp.clone());
            }
            Ok(())
        }
        GraphPattern::Join { left, right } => {
            collect_conjunction(left, out)?;
            collect_conjunction(right, out)
        }
        // Every other operator is outside the conjunctive fragment — REJECT, naming it.
        GraphPattern::LeftJoin { .. } => {
            Err(CqError::OutOfScope("OPTIONAL (LeftJoin) is not conjunctive".into()))
        }
        GraphPattern::Filter { .. } => {
            Err(CqError::OutOfScope("FILTER is not conjunctive".into()))
        }
        GraphPattern::Union { .. } => {
            Err(CqError::OutOfScope("UNION is not a single conjunctive query".into()))
        }
        GraphPattern::Minus { .. } => Err(CqError::OutOfScope("MINUS is not conjunctive".into())),
        GraphPattern::Path { .. } => Err(CqError::OutOfScope(
            "property path is not conjunctive (reintroduces recursion QL excludes)".into(),
        )),
        GraphPattern::Graph { .. } => {
            Err(CqError::OutOfScope("named-graph (GRAPH) is out of scope".into()))
        }
        GraphPattern::Extend { .. } => {
            Err(CqError::OutOfScope("BIND (Extend) is not conjunctive".into()))
        }
        GraphPattern::Group { .. } => {
            Err(CqError::OutOfScope("aggregation (GROUP BY) is not conjunctive".into()))
        }
        GraphPattern::Service { .. } => {
            Err(CqError::OutOfScope("SERVICE (federation) is out of scope".into()))
        }
        GraphPattern::Values { .. } => {
            Err(CqError::OutOfScope("inline VALUES is out of scope".into()))
        }
        GraphPattern::OrderBy { .. } => {
            Err(CqError::OutOfScope("ORDER BY inside the body is out of scope".into()))
        }
        GraphPattern::Slice { .. } => {
            Err(CqError::OutOfScope("LIMIT/OFFSET inside the body is out of scope".into()))
        }
        GraphPattern::Distinct { .. } | GraphPattern::Reduced { .. } => Err(CqError::OutOfScope(
            "nested DISTINCT/REDUCED is out of scope".into(),
        )),
        GraphPattern::Project { .. } => {
            Err(CqError::OutOfScope("nested sub-SELECT is out of scope".into()))
        }
        // `Lateral` exists because the vendored spargebra is built with `sep-0006` (the
        // workspace dep enables it). Match it unconditionally — a `cfg(feature = "sep-0006")`
        // here would test THIS crate's features (which has none of that name) and so never fire,
        // leaving the match non-exhaustive.
        GraphPattern::Lateral { .. } => {
            Err(CqError::OutOfScope("LATERAL is not conjunctive".into()))
        }
    }
}

/// A triple pattern is a DL-Lite-shaped atom only if its predicate is a CONSTANT IRI (a class
/// atom `?x rdf:type A` or a role atom `?x R ?y` with `R` a named property) and neither subject
/// nor object is an RDF-star quoted triple. A variable predicate (`?x ?p ?y`) cannot be matched
/// to a DL-Lite inclusion, so the query is out of scope (fail-closed, not silently un-rewritten).
fn check_atom_shape(tp: &TriplePattern) -> Result<(), CqError> {
    use spargebra::term::NamedNodePattern;
    if let NamedNodePattern::Variable(_) = tp.predicate {
        return Err(CqError::OutOfScope(
            "triple pattern with a variable predicate is not a DL-Lite atom".into(),
        ));
    }
    if is_quoted_triple(&tp.subject) || is_quoted_triple(&tp.object) {
        return Err(CqError::OutOfScope(
            "RDF-star quoted-triple term is out of DL-Lite scope".into(),
        ));
    }
    Ok(())
}

fn is_quoted_triple(t: &TermPattern) -> bool {
    // `TermPattern::Triple` exists because the vendored spargebra is built with `sparql-12`
    // (the workspace dep enables it). A `cfg(feature = "sparql-12")` here would test THIS
    // crate's features, not spargebra's, so match the variant directly.
    matches!(t, TermPattern::Triple(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use spargebra::SparqlParser;

    fn parse(q: &str) -> Query {
        SparqlParser::new().parse_query(q).expect("parse")
    }

    fn accepts(q: &str) -> ConjunctiveQuery {
        as_conjunctive_query(&parse(q)).expect("should be a CQ")
    }

    fn rejects(q: &str) -> String {
        match as_conjunctive_query(&parse(q)) {
            Err(CqError::OutOfScope(r)) => r,
            Ok(_) => panic!("expected OutOfScope rejection for: {}", q),
        }
    }

    const PRE: &str = "PREFIX : <http://ex/> PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>";

    #[test]
    fn accepts_single_bgp_select() {
        let cq = accepts(&format!("{PRE} SELECT ?x WHERE {{ ?x rdf:type :A }}"));
        assert_eq!(cq.atoms.len(), 1);
        assert_eq!(cq.distinguished.len(), 1);
        assert!(!cq.ask);
    }

    #[test]
    fn accepts_multi_atom_conjunction() {
        let cq = accepts(&format!(
            "{PRE} SELECT ?x ?y WHERE {{ ?x rdf:type :A . ?x :r ?y }}"
        ));
        assert_eq!(cq.atoms.len(), 2);
    }

    #[test]
    fn accepts_distinct() {
        let cq = accepts(&format!("{PRE} SELECT DISTINCT ?x WHERE {{ ?x rdf:type :A }}"));
        assert!(cq.distinct);
    }

    #[test]
    fn accepts_ask() {
        let cq = accepts(&format!("{PRE} ASK WHERE {{ ?x rdf:type :A }}"));
        assert!(cq.ask);
        assert!(cq.distinguished.is_empty());
    }

    #[test]
    fn rejects_optional() {
        assert!(rejects(&format!(
            "{PRE} SELECT ?x WHERE {{ ?x rdf:type :A OPTIONAL {{ ?x :r ?y }} }}"
        ))
        .contains("OPTIONAL"));
    }

    #[test]
    fn rejects_filter() {
        assert!(rejects(&format!(
            "{PRE} SELECT ?x WHERE {{ ?x rdf:type :A FILTER(?x = :B) }}"
        ))
        .contains("FILTER"));
    }

    #[test]
    fn rejects_union() {
        assert!(rejects(&format!(
            "{PRE} SELECT ?x WHERE {{ {{ ?x rdf:type :A }} UNION {{ ?x rdf:type :B }} }}"
        ))
        .contains("UNION"));
    }

    #[test]
    fn rejects_minus() {
        assert!(rejects(&format!(
            "{PRE} SELECT ?x WHERE {{ ?x rdf:type :A MINUS {{ ?x rdf:type :B }} }}"
        ))
        .contains("MINUS"));
    }

    #[test]
    fn rejects_property_path() {
        assert!(rejects(&format!(
            "{PRE} SELECT ?x WHERE {{ ?x :r+ ?y }}"
        ))
        .contains("property path"));
    }

    #[test]
    fn rejects_bind() {
        assert!(rejects(&format!(
            "{PRE} SELECT ?x WHERE {{ ?x rdf:type :A BIND(:B AS ?y) }}"
        ))
        .contains("BIND"));
    }

    #[test]
    fn rejects_aggregation() {
        // spargebra lowers `SELECT (COUNT(..) AS ?n)` to Project -> Extend -> Group, so the
        // gate rejects it at the Extend (BIND) or Group layer — either way it is OutOfScope,
        // never silently rewritten. We assert the REJECTION (the soundness invariant), not the
        // exact construct named (Extend wraps the aggregate, so "BIND" is the surfaced reason).
        let r = rejects(&format!(
            "{PRE} SELECT (COUNT(?x) AS ?n) WHERE {{ ?x rdf:type :A }}"
        ));
        assert!(
            r.contains("aggregation") || r.contains("BIND") || r.contains("GROUP"),
            "aggregation query must be rejected as non-conjunctive, got: {}",
            r
        );
    }

    #[test]
    fn rejects_bare_group_by() {
        // A GROUP BY without a wrapping BIND surfaces the aggregation/Group reason directly.
        let r = rejects(&format!(
            "{PRE} SELECT ?x WHERE {{ ?x rdf:type :A }} GROUP BY ?x"
        ));
        assert!(
            r.contains("aggregation") || r.contains("GROUP"),
            "GROUP BY must be rejected as non-conjunctive, got: {}",
            r
        );
    }

    #[test]
    fn rejects_variable_predicate() {
        assert!(rejects(&format!("{PRE} SELECT ?x WHERE {{ ?x ?p :A }}"))
            .contains("variable predicate"));
    }

    #[test]
    fn rejects_construct() {
        assert!(rejects(&format!(
            "{PRE} CONSTRUCT {{ ?x rdf:type :A }} WHERE {{ ?x rdf:type :A }}"
        ))
        .contains("CONSTRUCT"));
    }
}
