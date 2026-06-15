//! The `vec:` magic predicates: vector k-NN search inside plain SPARQL, with
//! ZERO engine changes. [OPUS-4.8] (sq-k6ex, epic sq-3183)
//!
//! ```
//! use sparq_core::Graph;
//! use sparq_vectors::VectorStore;
//! use oxrdf::NamedNode;
//!
//! # fn doit() -> Result<(), String> {
//! let g = Graph::load_str(r#"
//!     <http://ex/a> <http://ex/p> "a" .
//!     <http://ex/b> <http://ex/p> "b" .
//!     <http://ex/c> <http://ex/p> "c" .
//! "#, "ntriples").unwrap();
//! let id = |s: &str| g.id_of(&NamedNode::new(s).unwrap().into()).unwrap();
//! let path = std::env::temp_dir().join("sparq_vec_doctest.spqv");
//! let mut store = VectorStore::create(&path, 2).unwrap();
//! store.put(id("http://ex/a"), &[1.0, 0.0]).unwrap();
//! store.put(id("http://ex/b"), &[0.0, 1.0]).unwrap();
//! store.put(id("http://ex/c"), &[0.9, 0.1]).unwrap();
//! // The two entities most aligned with the x-axis query vector "1,0".
//! let r = sparq_vectors::query_vec(&g,
//!     "PREFIX vec: <http://sparq.dev/vec#>
//!      SELECT ?node WHERE { ?node vec:nearest ( \"1,0\" 2 ) }",
//!     &store)?;
//! assert_eq!(r.len(), 2); // <http://ex/a> and <http://ex/c>
//! # Ok(()) }
//! # doit().unwrap();
//! ```
//!
//! ## The rewrite
//!
//! [`rewrite_query`] walks the parsed spargebra algebra; in every basic graph
//! pattern, each magic triple pattern
//!
//! - `?node vec:nearest ( <query> <k> )` — bind `?node` to the `<k>` nearest
//!   neighbours (best first) of the query;
//! - `( ?node ?score ) vec:search ( <query> <k> )` — the same, additionally
//!   binding `?score` to each neighbour's cosine similarity (`xsd:double`).
//!
//! is REMOVED and replaced by an inline [`Values`](GraphPattern::Values) table
//! of the search hits — the neighbour graph nodes (and, for `vec:search`, their
//! scores), resolved through the graph's dictionary. The surrounding query then
//! joins those nodes to triples through the store's ordinary permutation
//! indexes; the rewritten algebra runs through sparq-engine's prepared-query
//! seam ([`PreparedQuery`]`: From<spargebra::Query>`), so the engine — planner,
//! executor, wasm bundle — is completely unaware of vector search.
//!
//! The argument lists `( … )` are ordinary SPARQL RDF collections — spargebra
//! lowers them to `rdf:first`/`rdf:rest` blank-node chains in the same BGP, and
//! the rewrite walks those chains back into the argument tuple, so the surface
//! is plain SPARQL with no custom grammar.
//!
//! ## The query argument
//!
//! `<query>` is a **constant**, either of:
//!
//! - a **node IRI** (`<http://ex/seed>`) whose stored vector is the query — the
//!   "neighbours of this entity" form (the seed is itself excluded from its
//!   neighbours); empty if the IRI is absent from the graph or unembedded;
//! - a **vector literal** — a comma-separated list of `dim` floats
//!   (`"0.1,0.9,..."`) — a query-by-vector. The dimension must match the store.
//!
//! `<k>` is a **constant** non-negative integer literal.
//!
//! ## Constraints (each a hard query error, not a silent mismatch)
//!
//! the neighbour position(s) must be variables; the query/`k` arguments must be
//! constants (bind-time rewriting has no per-row values); the object argument
//! list must be exactly `( query k )`, and the `vec:search` subject list exactly
//! `( ?node ?score )` with both positions variables. Any other IRI in the `vec:`
//! namespace is unknown.
//!
//! Result ordering: `VALUES` rows carry no order through joins — sort with
//! `ORDER BY DESC(?score)` over a `vec:search` score variable to recover the
//! best-first order in the output.
//!
//! The store is per-graph (dictionary-local ids), so hits come from the store
//! you pass — typically the one built against the default graph.

use crate::ann::nearest_exact;
use crate::store::VectorStore;
use crate::vocab;
use oxrdf::{Literal, NamedNode, Term};
use rustc_hash::FxHashMap;
use spargebra::algebra::GraphPattern;
use spargebra::term::{GroundTerm, NamedNodePattern, TermPattern, TriplePattern, Variable};
use spargebra::Query;
use sparq_core::dict::Id;
use sparq_core::Graph;
use sparq_engine::{PreparedQuery, QueryBudget, QueryResult};

/// `rdf:first`, `rdf:rest`, `rdf:nil` — the collection vocabulary spargebra
/// lowers `( … )` argument lists into.
const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";

/// Executes a SPARQL query that may use the `vec:` magic predicates: parse,
/// [`rewrite_query`], then evaluate with the standard engine.
pub fn query_vec(graph: &Graph, sparql: &str, store: &VectorStore) -> Result<QueryResult, String> {
    sparq_engine::query_prepared(graph, &prepare_vec(graph, sparql, store)?)
}

/// [`query_vec`] under a cooperative [`QueryBudget`].
pub fn query_vec_with_budget(
    graph: &Graph,
    sparql: &str,
    store: &VectorStore,
    budget: &QueryBudget,
) -> Result<QueryResult, String> {
    sparq_engine::query_prepared_with_budget(graph, &prepare_vec(graph, sparql, store)?, budget)
}

/// Parses and rewrites into a [`PreparedQuery`] — compose with any of the
/// engine's `*_prepared` entry points (`ask_prepared`, `construct_prepared`,
/// …). Note the hits are frozen at rewrite time: re-prepare after the graph
/// (and store) change.
pub fn prepare_vec(
    graph: &Graph,
    sparql: &str,
    store: &VectorStore,
) -> Result<PreparedQuery, String> {
    let query = spargebra::SparqlParser::new()
        .parse_query(sparql)
        .map_err(|e| e.to_string())?;
    Ok(PreparedQuery::from(rewrite_query(query, graph, store)?))
}

/// Rewrites every `vec:` magic pattern in the query into inline `VALUES` over
/// the search hits (see the module docs). A query without `vec:` patterns
/// passes through unchanged.
pub fn rewrite_query(
    mut query: Query,
    graph: &Graph,
    store: &VectorStore,
) -> Result<Query, String> {
    let pattern = match &mut query {
        Query::Select { pattern, .. }
        | Query::Construct { pattern, .. }
        | Query::Describe { pattern, .. }
        | Query::Ask { pattern, .. } => pattern,
    };
    rewrite_pattern(pattern, graph, store)?;
    Ok(query)
}

/// Recursively rewrites the magic patterns inside `p`.
fn rewrite_pattern(p: &mut GraphPattern, graph: &Graph, store: &VectorStore) -> Result<(), String> {
    match p {
        GraphPattern::Bgp { patterns } => {
            let patterns = std::mem::take(patterns);
            *p = rewrite_bgp(patterns, graph, store)?;
        }
        GraphPattern::Join { left, right }
        | GraphPattern::LeftJoin { left, right, .. }
        | GraphPattern::Union { left, right }
        | GraphPattern::Minus { left, right }
        | GraphPattern::Lateral { left, right } => {
            rewrite_pattern(left, graph, store)?;
            rewrite_pattern(right, graph, store)?;
        }
        GraphPattern::Filter { inner, .. }
        | GraphPattern::Graph { inner, .. }
        | GraphPattern::Extend { inner, .. }
        | GraphPattern::OrderBy { inner, .. }
        | GraphPattern::Project { inner, .. }
        | GraphPattern::Distinct { inner }
        | GraphPattern::Reduced { inner }
        | GraphPattern::Slice { inner, .. }
        | GraphPattern::Group { inner, .. }
        | GraphPattern::Service { inner, .. } => rewrite_pattern(inner, graph, store)?,
        // No BGPs inside: property paths and inline VALUES pass through.
        GraphPattern::Path { .. } | GraphPattern::Values { .. } => {}
    }
    Ok(())
}

/// The query a `vec:` request searches by.
enum QueryArg {
    /// A node IRI whose stored vector is the query (and which is excluded from
    /// its own neighbours).
    Node(NamedNode),
    /// An explicit query vector parsed from a comma-separated literal.
    Vector(Vec<f32>),
}

/// One `vec:nearest`/`vec:search` request found in a BGP. The `score` variable
/// is `Some` only for the `vec:search` form. [OPUS-4.8]
struct KnnReq {
    node: Variable,
    score: Option<Variable>,
    query: QueryArg,
    k: usize,
}

/// Splits a BGP's `vec:` magic patterns out, runs the k-NN search, and joins
/// the `VALUES` hit tables onto the remaining ordinary patterns. The collection
/// (`rdf:first`/`rdf:rest`) triples that spargebra emitted for the argument
/// lists are consumed by the rewrite and removed from the surviving BGP.
fn rewrite_bgp(
    patterns: Vec<TriplePattern>,
    graph: &Graph,
    store: &VectorStore,
) -> Result<GraphPattern, String> {
    // Index the rdf:first/rdf:rest triples by their (blank-node) list-cell
    // subject so the magic-predicate handlers can walk each `( … )` chain.
    let lists = ListCells::collect(&patterns);

    let mut rest: Vec<TriplePattern> = Vec::with_capacity(patterns.len());
    let mut reqs: Vec<KnnReq> = Vec::new();

    for tp in &patterns {
        // The collection-cell triples are an implementation detail of the
        // argument lists; once consumed they must not survive into the engine.
        if is_list_triple(tp) {
            continue;
        }
        let NamedNodePattern::NamedNode(pred) = &tp.predicate else {
            rest.push(tp.clone());
            continue;
        };
        let iri = pred.as_str();
        if !iri.starts_with(vocab::VEC_NS) {
            rest.push(tp.clone());
            continue;
        }
        match iri {
            vocab::NEAREST => reqs.push(parse_nearest(tp, &lists)?),
            vocab::SEARCH => reqs.push(parse_search(tp, &lists)?),
            _ => {
                return Err(format!(
                    "vec: unknown magic predicate <{iri}> (supported: vec:nearest, vec:search)"
                ))
            }
        }
    }

    // Join the hit tables onto the remaining ordinary patterns.
    let mut out = GraphPattern::Bgp { patterns: rest };
    for req in reqs {
        let hits = run_knn(&req, graph, store)?;
        let mut variables = vec![req.node];
        if let Some(s) = &req.score {
            variables.push(s.clone());
        }
        let score_wanted = req.score.is_some();
        let bindings = hits
            .into_iter()
            .filter_map(|(id, score)| {
                // Neighbour ids resolve to graph nodes (IRIs/literals); a blank
                // node cannot appear in a VALUES row, so skip it (entities embed
                // as IRIs in practice).
                let node = term_to_ground(graph.dict.term(id))?;
                let mut row = vec![Some(node)];
                if score_wanted {
                    row.push(Some(GroundTerm::Literal(Literal::from(f64::from(score)))));
                }
                Some(row)
            })
            .collect();
        let values = GraphPattern::Values {
            variables,
            bindings,
        };
        out = match out {
            // An all-magic BGP leaves an empty Bgp behind: drop the unit table.
            GraphPattern::Bgp { ref patterns } if patterns.is_empty() => values,
            other => GraphPattern::Join {
                left: Box::new(values),
                right: Box::new(other),
            },
        };
    }
    Ok(out)
}

/// True for the `rdf:first`/`rdf:rest` triples spargebra emits for a `( … )`
/// collection — these are consumed by the rewrite, not passed to the engine.
fn is_list_triple(tp: &TriplePattern) -> bool {
    matches!(&tp.predicate, NamedNodePattern::NamedNode(p) if p.as_str() == RDF_FIRST || p.as_str() == RDF_REST)
}

/// The `rdf:first`/`rdf:rest` cells of every collection in a BGP, keyed by the
/// list-cell blank node, so an argument list can be walked from its head.
struct ListCells<'a> {
    /// blank-node id → (first element, rest cell)
    cells: FxHashMap<&'a str, (&'a TermPattern, &'a TermPattern)>,
}

impl<'a> ListCells<'a> {
    fn collect(patterns: &'a [TriplePattern]) -> ListCells<'a> {
        let mut firsts: FxHashMap<&str, &TermPattern> = FxHashMap::default();
        let mut rests: FxHashMap<&str, &TermPattern> = FxHashMap::default();
        for tp in patterns {
            let TermPattern::BlankNode(b) = &tp.subject else {
                continue;
            };
            let NamedNodePattern::NamedNode(p) = &tp.predicate else {
                continue;
            };
            match p.as_str() {
                RDF_FIRST => {
                    firsts.insert(b.as_str(), &tp.object);
                }
                RDF_REST => {
                    rests.insert(b.as_str(), &tp.object);
                }
                _ => {}
            }
        }
        let cells = firsts
            .iter()
            .filter_map(|(&b, &first)| rests.get(b).map(|&rest| (b, (first, rest))))
            .collect();
        ListCells { cells }
    }

    /// Walks the collection whose head is `head` into its element [`TermPattern`]s.
    /// `head` is the object/subject of a `vec:` predicate — a blank-node list
    /// head, or `rdf:nil` for the empty list. Errors if the chain is malformed
    /// (dangling cell, or a non-collection term where a list was required).
    fn elements(&self, head: &'a TermPattern) -> Result<Vec<&'a TermPattern>, String> {
        let mut out = Vec::new();
        let mut cur = head;
        let mut guard = 0usize;
        loop {
            match cur {
                TermPattern::NamedNode(n) if n.as_str() == RDF_NIL => return Ok(out),
                TermPattern::BlankNode(b) => {
                    let Some(&(first, rest)) = self.cells.get(b.as_str()) else {
                        return Err(
                            "vec: malformed argument list (dangling rdf:rest cell)".to_string()
                        );
                    };
                    out.push(first);
                    cur = rest;
                }
                other => {
                    return Err(format!(
                        "vec: a vec: predicate requires a `( … )` argument list, got {other}"
                    ))
                }
            }
            guard += 1;
            if guard > 1 << 20 {
                return Err("vec: argument list is cyclic".to_string());
            }
        }
    }
}

/// Parses `?node vec:nearest ( <query> <k> )`.
fn parse_nearest(tp: &TriplePattern, lists: &ListCells) -> Result<KnnReq, String> {
    let node = require_var(&tp.subject, "the subject of vec:nearest")?;
    let (query, k) = parse_obj_args(&tp.object, lists)?;
    Ok(KnnReq {
        node,
        score: None,
        query,
        k,
    })
}

/// Parses `( ?node ?score ) vec:search ( <query> <k> )`.
fn parse_search(tp: &TriplePattern, lists: &ListCells) -> Result<KnnReq, String> {
    let subj = lists.elements(&tp.subject)?;
    let [node, score] = subj.as_slice() else {
        return Err(format!(
            "vec: the subject of vec:search must be a 2-element list ( ?node ?score ), got {} \
             element(s)",
            subj.len()
        ));
    };
    let node = require_var(node, "the first vec:search subject element")?;
    let score = require_var(score, "the second vec:search subject element")?;
    let (query, k) = parse_obj_args(&tp.object, lists)?;
    Ok(KnnReq {
        node,
        score: Some(score),
        query,
        k,
    })
}

/// Decodes the `( <query> <k> )` object argument list shared by both predicates.
fn parse_obj_args(o: &TermPattern, lists: &ListCells) -> Result<(QueryArg, usize), String> {
    let args = lists.elements(o)?;
    let [query, k] = args.as_slice() else {
        return Err(format!(
            "vec: the argument list must be ( <query> <k> ) — exactly two elements, got {}",
            args.len()
        ));
    };
    Ok((parse_query_arg(query)?, parse_k(k)?))
}

/// `<query>` → an entity IRI or a parsed comma-separated query vector.
fn parse_query_arg(t: &TermPattern) -> Result<QueryArg, String> {
    match t {
        TermPattern::NamedNode(n) => Ok(QueryArg::Node(n.clone())),
        TermPattern::Literal(l) => {
            let v: Result<Vec<f32>, _> = l
                .value()
                .split(',')
                .map(|s| s.trim().parse::<f32>())
                .collect();
            let v = v.map_err(|_| {
                format!(
                    "vec: the query literal must be a comma-separated list of floats, got \"{}\"",
                    l.value()
                )
            })?;
            if v.is_empty() {
                return Err("vec: the query vector literal is empty".to_string());
            }
            Ok(QueryArg::Vector(v))
        }
        other => Err(format!(
            "vec: the query argument must be a node IRI or a vector literal, got {other}"
        )),
    }
}

/// `<k>` → a non-negative integer.
fn parse_k(t: &TermPattern) -> Result<usize, String> {
    let TermPattern::Literal(l) = t else {
        return Err(format!(
            "vec: k must be a non-negative integer literal, got {t}"
        ));
    };
    l.value().parse::<usize>().map_err(|_| {
        format!(
            "vec: k must be a non-negative integer, got \"{}\"",
            l.value()
        )
    })
}

/// Requires `t` to be a bare variable; `what` names the position for errors.
fn require_var(t: &TermPattern, what: &str) -> Result<Variable, String> {
    match t {
        TermPattern::Variable(v) => Ok(v.clone()),
        other => Err(format!("vec: {what} must be a variable, got {other}")),
    }
}

/// Maps a dictionary [`Term`] to a [`GroundTerm`] for a VALUES row, or `None`
/// for a blank node / quoted triple (not expressible in a VALUES neighbour slot).
fn term_to_ground(t: Term) -> Option<GroundTerm> {
    match t {
        Term::NamedNode(n) => Some(GroundTerm::NamedNode(n)),
        Term::Literal(l) => Some(GroundTerm::Literal(l)),
        _ => None,
    }
}

/// Runs the k-NN search for `req` and returns `(neighbour id, cosine score)`
/// pairs, best first.
fn run_knn(req: &KnnReq, graph: &Graph, store: &VectorStore) -> Result<Vec<(Id, f32)>, String> {
    match &req.query {
        QueryArg::Node(iri) => {
            let term = Term::NamedNode(iri.clone());
            let Some(id) = graph.id_of(&term) else {
                return Ok(Vec::new());
            };
            let Some(query) = store.get(id) else {
                return Ok(Vec::new());
            };
            // Over-fetch by one and drop the seed itself.
            Ok(nearest_exact(store, query, req.k + 1)
                .into_iter()
                .filter(|&(n, _)| n != id)
                .take(req.k)
                .collect())
        }
        QueryArg::Vector(v) => {
            if v.len() != store.dim() {
                return Err(format!(
                    "vec: query vector has {} dims but the store has {}",
                    v.len(),
                    store.dim()
                ));
            }
            Ok(nearest_exact(store, v, req.k))
        }
    }
}
