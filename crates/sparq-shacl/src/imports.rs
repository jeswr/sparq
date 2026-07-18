//! [SONNET-4.6] (sq-uz0) Shapes-graph ASSEMBLY (W3C SHACL §1.4): `sh:shapesGraph`
//! discovery in the data graph plus the transitive, cycle-guarded `owl:imports`
//! closure of every referenced shapes document, unioned into one shapes
//! [`Graph`].
//!
//! Dereferencing an IRI to a document stays a CALLER concern — this engine never
//! touches the network — via the loader callback (`FnMut(&str) ->
//! Result<Option<Graph>, String>`): `Ok(Some(g))` is the fetched document,
//! `Ok(None)` an honest "I cannot resolve this IRI" (recorded, not fatal —
//! SHACL leaves imports support optional), `Err` a hard failure that aborts
//! resolution. What the library owns is everything callers previously
//! hand-rolled: the traversal order, the dedupe/cycle guard (each IRI is
//! dereferenced at most once), the RDF-merge discipline (blank nodes of each
//! loaded document are standardised apart so labels reused across documents
//! never collapse into one node), and the final union.

use crate::view::GraphView;
use oxrdf::{BlankNode, NamedOrBlankNode, Term, Triple};
use rustc_hash::FxHashSet;
use sparq_core::dict::{Dict, Id};
use sparq_core::Graph;

const SH_SHAPES_GRAPH: &str = "http://www.w3.org/ns/shacl#shapesGraph";
const OWL_IMPORTS: &str = "http://www.w3.org/2002/07/owl#imports";

/// The outcome of assembling a shapes graph: the union graph plus an honest
/// record of which referenced IRIs were (and were not) dereferenced.
pub struct ShapesGraphResolution {
    /// The assembled shapes graph (deduplicated union of every loaded
    /// document — and, for [`resolve_imports`], the seed graph).
    pub shapes: Graph,
    /// The IRIs the loader dereferenced, in breadth-first traversal order.
    pub resolved: Vec<String>,
    /// The IRIs the loader declined (`Ok(None)`), in traversal order. SHACL
    /// keeps imports support optional, so these are recorded rather than fatal.
    pub unresolved: Vec<String>,
}

// Manual: `Graph` is not `Debug`, so the assembled graph is elided.
impl std::fmt::Debug for ShapesGraphResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShapesGraphResolution")
            .field("resolved", &self.resolved)
            .field("unresolved", &self.unresolved)
            .finish_non_exhaustive()
    }
}

/// Assembles the shapes graph a data graph points at (W3C SHACL §1.4): the
/// union of every graph an `sh:shapesGraph` triple in `data` references, plus
/// the transitive `owl:imports` closure of each loaded document. The data
/// graph's own triples are NOT included. Pass the result's
/// [`shapes`](ShapesGraphResolution::shapes) to [`crate::validate`].
pub fn resolve_shapes_graph<F>(data: &Graph, loader: F) -> Result<ShapesGraphResolution, String>
where
    F: FnMut(&str) -> Result<Option<Graph>, String>,
{
    assemble(None, iri_objects(data, SH_SHAPES_GRAPH), loader)
}

/// Assembles the `owl:imports` closure of a shapes graph the caller already
/// holds: `seed`'s own triples (blank-node labels intact) unioned with every
/// document its `owl:imports` references, transitively and cycle-guarded.
pub fn resolve_imports<F>(seed: &Graph, loader: F) -> Result<ShapesGraphResolution, String>
where
    F: FnMut(&str) -> Result<Option<Graph>, String>,
{
    assemble(Some(seed), iri_objects(seed, OWL_IMPORTS), loader)
}

/// The distinct IRI objects of `(?, pred, ?)` in `g`, in scan order.
fn iri_objects(g: &Graph, pred: &str) -> Vec<String> {
    let mut seen: FxHashSet<String> = FxHashSet::default();
    let mut out = Vec::new();
    for [_, _, o] in GraphView::new(g).triples(None, Some(pred), None) {
        if let Term::NamedNode(n) = o {
            let iri = n.into_string();
            if seen.insert(iri.clone()) {
                out.push(iri);
            }
        }
    }
    out
}

/// Breadth-first assembly: dequeue an IRI (at most once — the cycle guard),
/// load it, enqueue its `owl:imports`, and union its triples with each loaded
/// document's blank nodes standardised apart (ordinal-prefixed labels).
fn assemble<F>(
    seed: Option<&Graph>,
    queue: Vec<String>,
    mut loader: F,
) -> Result<ShapesGraphResolution, String>
where
    F: FnMut(&str) -> Result<Option<Graph>, String>,
{
    let mut dict = Dict::new();
    let mut triples: Vec<[Id; 3]> = Vec::new();
    let mut seen_rows: FxHashSet<[Id; 3]> = FxHashSet::default();
    if let Some(g) = seed {
        add_graph(g, None, &mut dict, &mut triples, &mut seen_rows);
    }

    let mut queue = queue;
    let mut visited: FxHashSet<String> = FxHashSet::default();
    let mut resolved = Vec::new();
    let mut unresolved = Vec::new();
    let mut i = 0;
    while i < queue.len() {
        let iri = queue[i].clone();
        i += 1;
        if !visited.insert(iri.clone()) {
            continue;
        }
        match loader(&iri) {
            Err(e) => return Err(format!("loading shapes graph <{}>: {}", iri, e)),
            Ok(None) => unresolved.push(iri),
            Ok(Some(g)) => {
                queue.extend(iri_objects(&g, OWL_IMPORTS));
                add_graph(&g, Some(resolved.len()), &mut dict, &mut triples, &mut seen_rows);
                resolved.push(iri);
            }
        }
    }
    Ok(ShapesGraphResolution {
        shapes: Graph::from_parts(dict, triples),
        resolved,
        unresolved,
    })
}

/// Interns every triple of `g` into the union under construction, dropping
/// exact duplicates. `ord: Some(n)` standardises the document's blank nodes
/// apart by prefixing labels with the document ordinal; `None` (the
/// [`resolve_imports`] seed) keeps labels intact.
fn add_graph(
    g: &Graph,
    ord: Option<usize>,
    dict: &mut Dict,
    out: &mut Vec<[Id; 3]>,
    seen_rows: &mut FxHashSet<[Id; 3]>,
) {
    for [s, p, o] in GraphView::new(g).triples(None, None, None) {
        let row = [
            intern(dict, s, ord),
            intern(dict, p, ord),
            intern(dict, o, ord),
        ];
        if seen_rows.insert(row) {
            out.push(row);
        }
    }
}

fn intern(dict: &mut Dict, t: Term, ord: Option<usize>) -> Id {
    match ord {
        None => dict.intern(&t),
        Some(n) => dict.intern(&rename_blanks(t, n)),
    }
}

/// Rewrites every blank node in `t` (including inside RDF-1.2 triple terms) to
/// an ordinal-prefixed label, so blank nodes from different documents can never
/// collide in the union (an RDF *merge*, not a plain union).
fn rename_blanks(t: Term, ord: usize) -> Term {
    match t {
        Term::BlankNode(b) => Term::BlankNode(renamed(&b, ord)),
        Term::Triple(tr) => {
            let subject = match tr.subject {
                NamedOrBlankNode::BlankNode(b) => NamedOrBlankNode::BlankNode(renamed(&b, ord)),
                s => s,
            };
            Term::Triple(Box::new(Triple::new(
                subject,
                tr.predicate.clone(),
                rename_blanks(tr.object.clone(), ord),
            )))
        }
        other => other,
    }
}

fn renamed(b: &BlankNode, ord: usize) -> BlankNode {
    // A valid blank-node label prefixed with `i<ord>_` stays a valid label
    // (ASCII alphanumerics + underscore), so unchecked construction is safe.
    BlankNode::new_unchecked(format!("i{}_{}", ord, b.as_str()))
}
