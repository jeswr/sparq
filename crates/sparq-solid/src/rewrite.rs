//! v1 query rewriting on TODAY'S public engine APIs (design doc §4.4):
//!
//! 1. every default-graph triple/path pattern is wrapped in `GRAPH ?__sgN { … }` (fresh
//!    variable per pattern, joined above — cross-document joins keep working; this is
//!    the standard union-default-graph emulation, modulo duplicate rows when the same
//!    triple is asserted in several accessible graphs);
//! 2. the dataset clause is replaced by `FROM NAMED <g>` for exactly the authorized
//!    graphs (intersected with any pre-existing FROM NAMED), so `GRAPH` patterns range
//!    over the authorized set only — enforced by the engine's `build_active` semantics
//!    (the store's own graphs do not leak in; absent graphs are empty).
//!
//! The honest cost: `build_active` decodes + rebuilds every listed graph PER QUERY.
//! That copy is what the L1 engine dataset view (design doc §5) deletes.

use oxrdf::{NamedNode, Variable};
use spargebra::algebra::{Expression, GraphPattern, QueryDataset};
use spargebra::term::NamedNodePattern;
use spargebra::{Query, SparqlParser};

/// Rewrite `sparql` so it can only see `allowed` named graphs. Returns the rewritten
/// query string (parse → transform → serialize round-trip).
pub fn rewrite_for(sparql: &str, allowed: &[NamedNode]) -> Result<String, String> {
    let mut q = SparqlParser::new().parse_query(sparql).map_err(|e| e.to_string())?;
    let (dataset, pattern) = match &mut q {
        Query::Select { dataset, pattern, .. }
        | Query::Construct { dataset, pattern, .. }
        | Query::Describe { dataset, pattern, .. }
        | Query::Ask { dataset, pattern, .. } => (dataset, pattern),
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
    let mut fresh = 0usize;
    wrap_in_graph(pattern, &mut fresh, false);
    Ok(q.to_string())
}

fn fresh_graph_var(fresh: &mut usize) -> NamedNodePattern {
    let v = Variable::new_unchecked(format!("__sg{}", *fresh));
    *fresh += 1;
    NamedNodePattern::Variable(v)
}

/// Recursively wrap default-graph-scoped triple/path patterns in `GRAPH ?fresh { … }`.
/// `in_graph` = already inside a GRAPH scope (leave those patterns alone).
fn wrap_in_graph(p: &mut GraphPattern, fresh: &mut usize, in_graph: bool) {
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
fn wrap_expr(e: &mut Expression, fresh: &mut usize, in_graph: bool) {
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
