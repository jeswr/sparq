//! SPARQL 1.1 Update (roadmap T10), v1: the data operations `INSERT DATA`, `DELETE DATA`, and
//! `CLEAR` on the default graph, applied by **rebuild** — collect the current triple set, apply the
//! mutations as set operations, and rebuild the (immutable) store. Correct and simple; O(n) per
//! update batch. Pattern-based `DELETE/INSERT ... WHERE`, `LOAD`, and named-graph operations are
//! follow-ups (they need query evaluation and/or a quad store).

use oxrdf::{NamedOrBlankNode, Term};
use rustc_hash::FxHashSet;
use sparq_core::dict::Dict;
use sparq_core::store::Pattern as IdPattern;
use sparq_core::Graph;
use spargebra::algebra::GraphTarget;
use spargebra::term::{GraphName, GroundQuad, GroundTerm, Quad};
use spargebra::GraphUpdateOperation;
use spargebra::SparqlParser;

type TripleTerms = [Term; 3];

fn nob_to_term(s: &NamedOrBlankNode) -> Term {
    match s {
        NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
        NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
    }
}

fn ground_to_term(t: &GroundTerm) -> Result<Term, String> {
    match t {
        GroundTerm::NamedNode(n) => Ok(Term::NamedNode(n.clone())),
        GroundTerm::Literal(l) => Ok(Term::Literal(l.clone())),
        other => Err(format!("unsupported ground term in DELETE DATA: {other:?}")),
    }
}

fn require_default(g: &GraphName) -> Result<(), String> {
    if *g == GraphName::DefaultGraph {
        Ok(())
    } else {
        Err("named graphs in UPDATE are not yet supported".into())
    }
}

fn quad_to_triple(q: &Quad) -> Result<TripleTerms, String> {
    require_default(&q.graph_name)?;
    Ok([nob_to_term(&q.subject), Term::NamedNode(q.predicate.clone()), q.object.clone()])
}

fn ground_quad_to_triple(q: &GroundQuad) -> Result<TripleTerms, String> {
    require_default(&q.graph_name)?;
    Ok([
        Term::NamedNode(q.subject.clone()),
        Term::NamedNode(q.predicate.clone()),
        ground_to_term(&q.object)?,
    ])
}

/// The current default-graph triples as ground term-triples (decoded from the dictionary).
fn current_triples(g: &Graph) -> FxHashSet<TripleTerms> {
    let pat: IdPattern = [None, None, None];
    let scan = g.store.scan(&pat);
    scan.rows
        .iter()
        .map(|r| {
            let t = scan.to_spo(r);
            [g.dict.term(t[0]), g.dict.term(t[1]), g.dict.term(t[2])]
        })
        .collect()
}

/// Rebuild an immutable graph from a term-triple set (fresh dictionary + permutation indexes).
fn build(triples: &FxHashSet<TripleTerms>) -> Graph {
    let mut dict = Dict::new();
    let mut ids = Vec::with_capacity(triples.len());
    for [s, p, o] in triples {
        ids.push([dict.intern(s), dict.intern(p), dict.intern(o)]);
    }
    Graph::from_parts(dict, ids)
}

/// Apply a SPARQL Update string to `graph`, returning the updated graph. Errors (leaving the input
/// consumed) on operations not yet supported, so the caller can keep the original on failure by
/// cloning beforehand if needed.
pub fn update(graph: Graph, sparql: &str) -> Result<Graph, String> {
    let upd = SparqlParser::new().parse_update(sparql).map_err(|e| e.to_string())?;
    let mut triples = current_triples(&graph);
    for op in &upd.operations {
        match op {
            GraphUpdateOperation::InsertData { data } => {
                for q in data {
                    triples.insert(quad_to_triple(q)?);
                }
            }
            GraphUpdateOperation::DeleteData { data } => {
                for q in data {
                    triples.remove(&ground_quad_to_triple(q)?);
                }
            }
            // Only a default graph exists: CLEAR DEFAULT / ALL empties it; CLEAR of a named graph is
            // a no-op (the graph is absent — `silent` is irrelevant here).
            GraphUpdateOperation::Clear { graph: target, .. } => {
                if clears_default(target) {
                    triples.clear();
                }
            }
            other => return Err(format!("UPDATE operation not yet supported: {other:?}")),
        }
    }
    Ok(build(&triples))
}

fn clears_default(target: &GraphTarget) -> bool {
    matches!(target, GraphTarget::DefaultGraph | GraphTarget::AllGraphs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(g: &Graph) -> usize {
        let pat: IdPattern = [None, None, None];
        g.store.scan(&pat).rows.len()
    }

    #[test]
    fn insert_delete_clear() {
        let g = Graph::load_str("@prefix : <http://ex/> . :a :p :b . :b :p :c .", "turtle").unwrap();
        assert_eq!(count(&g), 2);
        // INSERT DATA adds; set semantics (a re-insert is a no-op).
        let g = update(g, "PREFIX : <http://ex/> INSERT DATA { :c :p :d . :a :q :x }").unwrap();
        assert_eq!(count(&g), 4);
        let g = update(g, "PREFIX : <http://ex/> INSERT DATA { :a :p :b }").unwrap();
        assert_eq!(count(&g), 4);
        // DELETE DATA removes a present triple; deleting an absent one is a no-op.
        let g = update(g, "PREFIX : <http://ex/> DELETE DATA { :a :p :b }").unwrap();
        assert_eq!(count(&g), 3);
        let g = update(g, "PREFIX : <http://ex/> DELETE DATA { :z :z :z }").unwrap();
        assert_eq!(count(&g), 3);
        // The graph is still queryable after a rebuild.
        assert_eq!(crate::count(&g, "PREFIX : <http://ex/> SELECT * WHERE { :c :p ?o }").unwrap(), 1);
        // CLEAR empties the default graph.
        let g = update(g, "CLEAR ALL").unwrap();
        assert_eq!(count(&g), 0);
    }

    #[test]
    fn named_graph_update_errors() {
        let g = Graph::load_str("@prefix : <http://ex/> . :a :p :b .", "turtle").unwrap();
        assert!(update(g, "PREFIX : <http://ex/> INSERT DATA { GRAPH :g { :a :p :c } }").is_err());
    }
}
