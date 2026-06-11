//! Query rewriting for the two session-query paths (design doc §4.4 / §5):
//!
//! 1. **GRAPH wrapping** ([`wrap_for_view`], used by BOTH paths): every default-graph
//!    triple/path pattern is wrapped in `GRAPH ?__sgN { … }` (fresh variable per
//!    pattern, joined above — cross-document joins keep working; this is the standard
//!    union-default-graph emulation, modulo duplicate rows when the same triple is
//!    asserted in several accessible graphs);
//! 2. **dataset-clause injection** ([`rewrite_for`] = step 1 + this, the v1/portability
//!    path): the dataset clause is replaced by `FROM NAMED <g>` for exactly the
//!    authorized graphs (intersected with any pre-existing FROM NAMED), so `GRAPH`
//!    patterns range over the authorized set only — enforced by the engine's
//!    `build_active` semantics (the store's own graphs do not leak in; absent graphs
//!    are empty).
//!
//! The honest cost of step 2: `build_active` decodes + rebuilds every listed graph PER
//! QUERY. The default `DatasetView` path (engine L1, design doc §5) deletes that copy:
//! it needs only step 1, because graph visibility is enforced by the view itself
//! (O(1) hash check, zero copy) — see [`crate::PodStore::query_as`].

use oxrdf::{NamedNode, Variable};
use spargebra::algebra::{Expression, GraphPattern, QueryDataset};
use spargebra::term::NamedNodePattern;
use spargebra::{Query, SparqlParser};

/// Rewrite `sparql` so it can only see `allowed` named graphs. Returns the rewritten
/// query string (parse → transform → serialize round-trip).
///
/// Two transformations (see the module docs for the why):
///
/// 1. default-graph triple/path patterns get wrapped in `GRAPH ?fresh { … }`
///    (per-pattern fresh variables — union-default emulation, cross-document joins
///    keep working);
/// 2. the dataset clause becomes `FROM NAMED <g>` for exactly the `allowed` graphs,
///    intersected with any `FROM NAMED` the query already carried (a query can
///    restrict further, never widen). `FROM` (default-graph) clauses are dropped —
///    pod data never lives in the store default graph.
///
/// Fail-closed invariant: an empty `allowed` set yields a query over a single
/// reserved **sentinel** graph (`<urn:sparq:nothing>`, guaranteed absent — the loader
/// strips reserved graphs), because an empty `FROM NAMED` list would not survive the
/// serialize→reparse round-trip and the store's graphs would leak back in.
///
/// Callers normally go through [`crate::PodStore::query_as`], which feeds this the
/// session's authorized graph set.
///
/// # Errors
///
/// Returns `Err` if `sparql` is not a valid SPARQL query (SELECT / ASK / CONSTRUCT /
/// DESCRIBE). The rewrite itself cannot fail.
///
/// # Examples
///
/// ```
/// use oxrdf::NamedNode;
/// use sparq_solid::rewrite_for;
///
/// let allowed = [NamedNode::new("https://pod.ex/notes/n1").unwrap()];
/// let q = rewrite_for("SELECT ?t WHERE { ?s <https://ex.dev/ns#title> ?t }", &allowed)?;
/// assert!(q.contains("FROM NAMED <https://pod.ex/notes/n1>"));
/// assert!(q.contains("GRAPH ?__sg0")); // the pattern now ranges over authorized graphs
///
/// // empty set -> the absent sentinel graph keeps the dataset clause fail-closed
/// let none = rewrite_for("SELECT ?t WHERE { ?s ?p ?t }", &[])?;
/// assert!(none.contains("FROM NAMED <urn:sparq:nothing>"));
/// # Ok::<(), String>(())
/// ```
pub fn rewrite_for(sparql: &str, allowed: &[NamedNode]) -> Result<String, String> {
    let mut q = SparqlParser::new().parse_query(sparql).map_err(|e| e.to_string())?;
    let dataset = match &mut q {
        Query::Select { dataset, .. }
        | Query::Construct { dataset, .. }
        | Query::Describe { dataset, .. }
        | Query::Ask { dataset, .. } => dataset,
    };
    // restrict the dataset: FROM NAMED = allowed (∩ pre-existing), FROM = nothing
    let mut named: Vec<NamedNode> = match dataset.as_ref().and_then(|d| d.named.clone()) {
        Some(existing) => existing.into_iter().filter(|g| allowed.contains(g)).collect(),
        None => allowed.to_vec(),
    };
    if named.is_empty() {
        // an EMPTY FROM NAMED list would vanish in the serialize→reparse round-trip and
        // the store's own graphs would leak back in; a sentinel absent graph keeps the
        // dataset clause present (build_active: absent graph = empty graph)
        named.push(NamedNode::new_unchecked("urn:sparq:nothing"));
    }
    *dataset = Some(QueryDataset { default: Vec::new(), named: Some(named) });
    wrap_query(&mut q, sparql);
    Ok(q.to_string())
}

/// Rewrite step 1 ONLY — wrap every default-graph triple/path pattern in
/// `GRAPH ?fresh { … }` (union-default emulation), leaving any dataset clause alone.
///
/// This is the rewrite the **default** `DatasetView` query path needs
/// ([`crate::PodStore::query_as`]): under the view, graph visibility is already
/// enforced by the engine (`GRAPH ?g` enumerates only visible graphs, a dataset
/// clause is intersected with the view), and the view's default graph is empty —
/// so the only transformation left is making default-graph patterns range over the
/// (visible) named graphs.
///
/// # Errors
///
/// Returns `Err` if `sparql` is not a valid SPARQL query. The rewrite itself cannot
/// fail.
///
/// # Examples
///
/// ```
/// let q = sparq_solid::wrap_for_view("SELECT ?t WHERE { ?s <https://ex.dev/ns#title> ?t }")?;
/// assert!(q.contains("GRAPH ?__sg0")); // pattern ranges over (visible) named graphs
/// assert!(!q.contains("FROM"));        // no dataset clause: the view enforces visibility
/// # Ok::<(), String>(())
/// ```
pub fn wrap_for_view(sparql: &str) -> Result<String, String> {
    let mut q = SparqlParser::new().parse_query(sparql).map_err(|e| e.to_string())?;
    wrap_query(&mut q, sparql);
    Ok(q.to_string())
}

/// Apply the per-pattern GRAPH wrap to a parsed query, with a graph-variable prefix
/// that cannot collide with any user variable: lengthen until it appears nowhere in
/// the original query text.
fn wrap_query(q: &mut Query, original: &str) {
    let pattern = match q {
        Query::Select { pattern, .. }
        | Query::Construct { pattern, .. }
        | Query::Describe { pattern, .. }
        | Query::Ask { pattern, .. } => pattern,
    };
    let mut prefix = "__sg".to_owned();
    while original.contains(&prefix) {
        prefix.push('x');
    }
    let mut fresh = Fresh { prefix, n: 0 };
    wrap_in_graph(pattern, &mut fresh, false);
}

struct Fresh {
    prefix: String,
    n: usize,
}

fn fresh_graph_var(fresh: &mut Fresh) -> NamedNodePattern {
    let v = Variable::new_unchecked(format!("{}{}", fresh.prefix, fresh.n));
    fresh.n += 1;
    NamedNodePattern::Variable(v)
}

/// Recursively wrap default-graph-scoped triple/path patterns in `GRAPH ?fresh { … }`.
/// `in_graph` = already inside a GRAPH scope (leave those patterns alone).
fn wrap_in_graph(p: &mut GraphPattern, fresh: &mut Fresh, in_graph: bool) {
    match p {
        GraphPattern::Bgp { patterns } => {
            if in_graph || patterns.is_empty() {
                return;
            }
            // one GRAPH scope PER TRIPLE pattern: a join above the graph operator, so a
            // BGP may join triples from different documents (union-default semantics)
            let triples = std::mem::take(patterns);
            let mut acc: Option<GraphPattern> = None;
            for t in triples {
                let wrapped = GraphPattern::Graph {
                    name: fresh_graph_var(fresh),
                    inner: Box::new(GraphPattern::Bgp { patterns: vec![t] }),
                };
                acc = Some(match acc {
                    None => wrapped,
                    Some(left) => {
                        GraphPattern::Join { left: Box::new(left), right: Box::new(wrapped) }
                    }
                });
            }
            *p = acc.expect("non-empty");
        }
        GraphPattern::Path { .. } => {
            if in_graph {
                return;
            }
            let inner = std::mem::replace(p, GraphPattern::Bgp { patterns: Vec::new() });
            *p = GraphPattern::Graph { name: fresh_graph_var(fresh), inner: Box::new(inner) };
        }
        GraphPattern::Graph { inner, .. } => wrap_in_graph(inner, fresh, true),
        GraphPattern::Join { left, right }
        | GraphPattern::Union { left, right }
        | GraphPattern::Minus { left, right } => {
            wrap_in_graph(left, fresh, in_graph);
            wrap_in_graph(right, fresh, in_graph);
        }
        GraphPattern::Lateral { left, right } => {
            wrap_in_graph(left, fresh, in_graph);
            wrap_in_graph(right, fresh, in_graph);
        }
        GraphPattern::LeftJoin { left, right, expression } => {
            wrap_in_graph(left, fresh, in_graph);
            wrap_in_graph(right, fresh, in_graph);
            if let Some(e) = expression {
                wrap_expr(e, fresh, in_graph);
            }
        }
        GraphPattern::Filter { expr, inner } => {
            wrap_expr(expr, fresh, in_graph);
            wrap_in_graph(inner, fresh, in_graph);
        }
        GraphPattern::Extend { inner, .. }
        | GraphPattern::OrderBy { inner, .. }
        | GraphPattern::Project { inner, .. }
        | GraphPattern::Distinct { inner }
        | GraphPattern::Reduced { inner }
        | GraphPattern::Slice { inner, .. }
        | GraphPattern::Group { inner, .. } => wrap_in_graph(inner, fresh, in_graph),
        GraphPattern::Service { inner, .. } => wrap_in_graph(inner, fresh, true),
        GraphPattern::Values { .. } => {}
    }
}

/// EXISTS / NOT EXISTS carry nested patterns; wrap those too.
fn wrap_expr(e: &mut Expression, fresh: &mut Fresh, in_graph: bool) {
    match e {
        Expression::Exists(inner) => wrap_in_graph(inner, fresh, in_graph),
        Expression::Or(a, b)
        | Expression::And(a, b)
        | Expression::Equal(a, b)
        | Expression::SameTerm(a, b)
        | Expression::Greater(a, b)
        | Expression::GreaterOrEqual(a, b)
        | Expression::Less(a, b)
        | Expression::LessOrEqual(a, b)
        | Expression::Add(a, b)
        | Expression::Subtract(a, b)
        | Expression::Multiply(a, b)
        | Expression::Divide(a, b) => {
            wrap_expr(a, fresh, in_graph);
            wrap_expr(b, fresh, in_graph);
        }
        Expression::In(a, list) => {
            wrap_expr(a, fresh, in_graph);
            for x in list {
                wrap_expr(x, fresh, in_graph);
            }
        }
        Expression::UnaryPlus(a) | Expression::UnaryMinus(a) | Expression::Not(a) => {
            wrap_expr(a, fresh, in_graph);
        }
        Expression::If(a, b, c) => {
            wrap_expr(a, fresh, in_graph);
            wrap_expr(b, fresh, in_graph);
            wrap_expr(c, fresh, in_graph);
        }
        Expression::Coalesce(list) | Expression::FunctionCall(_, list) => {
            for x in list {
                wrap_expr(x, fresh, in_graph);
            }
        }
        Expression::Bound(_) | Expression::NamedNode(_) | Expression::Literal(_) | Expression::Variable(_) => {}
    }
}
