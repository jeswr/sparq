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

use crate::ann::{nearest_exact, nearest_exact_tiebreak};
use crate::spqv_provenance::Metric;
use crate::store::VectorStore;
use crate::vocab;
// [OPUS-4.8] (sq-z589, epic sq-3183) The approximate backend the `*_approx` entry points search
// the UNFILTERED `vec:` path through — the on-disk Vamana graph. `approx-ann` only (the only
// feature that compiles an approximate index); with it off the crate carries only the exact path.
#[cfg(feature = "approx-ann")]
use crate::diskann::DiskAnnIndex;
use oxrdf::{Literal, NamedNode, Term};
use rustc_hash::FxHashMap;
use spargebra::algebra::GraphPattern;
use spargebra::term::{GroundTerm, NamedNodePattern, TermPattern, TriplePattern, Variable};
use spargebra::Query;
use sparq_core::dict::Id;
use sparq_core::Graph;
use sparq_engine::{PreparedQuery, QueryBudget, QueryResult};

// [OPUS-4.8] (sq-bvmd, epic sq-3183) Filtered-ANN BGP→IdMask wiring: when a `vec:`
// request shares its neighbour variable with ordinary patterns in the SAME BGP, those
// patterns carve out the eligible graph nodes (the candidate id-set), and the k-NN
// search is restricted to that set — correct (and, for a selective constraint, faster)
// than post-filtering the unfiltered neighbours. This needs the `filtered-ann` API
// (`IdMask` + the cost-model `nearest_filtered_costed_tiebreak`, sq-7hx6, which picks
// pre-filter vs post-filter by selectivity — identical answer either way), so the whole
// derive-and-filter path is additionally gated on `filtered-ann`; with that feature off
// the `vec:` predicate behaves EXACTLY as before (plain unfiltered `nearest_exact`).
#[cfg(feature = "filtered-ann")]
use crate::cost::{nearest_filtered_costed_tiebreak, CostModel};
#[cfg(feature = "filtered-ann")]
use crate::filter::IdMask;
// [OPUS-4.8] (sq-36ol, epic sq-3183) The derived-mask cache: a `vec:` predicate that shares its
// neighbour variable with constraining patterns re-evaluates that sub-BGP (a full SELECT through
// the engine) on EVERY prepare to build the `IdMask`. Against an unchanged graph the answer is
// identical every time, so we cache it keyed by (constraining sub-BGP, graph fingerprint) — see
// `mask_cache` below. Gated on `filtered-ann` (the only feature with a mask to cache).
#[cfg(feature = "filtered-ann")]
use crate::fingerprint::Fingerprint;

/// `rdf:first`, `rdf:rest`, `rdf:nil` — the collection vocabulary spargebra
/// lowers `( … )` argument lists into.
const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";

/// [OPUS-4.8] (sq-z589, epic sq-3183) The backend the UNFILTERED `vec:` k-NN runs through. The
/// rewrite picks neighbours by this seam, so swapping the exact full scan for an approximate index
/// is a one-line backend swap with NO other change to the rewrite (parse, list-walk, error checks,
/// seed-self-exclusion, VALUES inlining and the join all stay identical). Two impls:
///
/// - [`ExactSearch`] (default; the original `vec:` behaviour) — a full [`nearest_exact`] scan;
///   answer-exact, the deterministic ground truth, a fine default below ~10⁵ vectors.
/// - `ApproxSearch` (`approx-ann` only) — the on-disk [`DiskAnnIndex`] Vamana greedy beam search;
///   APPROXIMATE (recall < 1.0 — see [`crate::ann`]/[`crate::diskann`]), for large `.spqv` stores
///   where the full scan is the bottleneck. NEVER claimed as exact: the index can miss a true
///   neighbour. The FILTERED `vec:` path (when `filtered-ann` composes in) is unchanged — it still
///   goes through the cost-model'd, VG-TIE-1 tie-broken `nearest_filtered_costed_tiebreak`; this
///   seam is only the unfiltered scan.
trait UnfilteredSearch {
    /// Top-`k` `(id, cosine)` by similarity to `query`, best first (same contract as
    /// [`nearest_exact`]: all-zero query → no results).
    fn search(&self, query: &[f32], k: usize) -> Vec<(Id, f32)>;

    /// [SONNET-4.6] (sq-tb9p0) Answer-exact top-`k` with the VG-TIE-1 deterministic boundary
    /// tie-break ([`nearest_exact_tiebreak`]) and the seed already excluded, or `Ok(None)` when
    /// this backend cannot honour the rule — an APPROXIMATE backend has recall < 1 and no true
    /// boundary to break ties at, so it keeps the plain [`search`](Self::search) path. `Err` is
    /// the fail-closed embeddable-domain rejection: a blank-node candidate whose term (not its
    /// score alone) would decide membership is a hard query error, never ranked.
    fn search_tied(
        &self,
        _query: &[f32],
        _k: usize,
        _graph: &Graph,
        _exclude: Option<Id>,
    ) -> Result<Option<Vec<(Id, f32)>>, String> {
        Ok(None)
    }
}

/// The exact full-scan backend — preserves the pre-sq-z589 `vec:` behaviour exactly.
struct ExactSearch<'a> {
    store: &'a VectorStore,
}

impl UnfilteredSearch for ExactSearch<'_> {
    fn search(&self, query: &[f32], k: usize) -> Vec<(Id, f32)> {
        nearest_exact(self.store, query, k)
    }

    fn search_tied(
        &self,
        query: &[f32],
        k: usize,
        graph: &Graph,
        exclude: Option<Id>,
    ) -> Result<Option<Vec<(Id, f32)>>, String> {
        nearest_exact_tiebreak(self.store, graph, query, k, exclude).map(Some)
    }
}

/// [OPUS-4.8] (sq-z589) The approximate on-disk Vamana backend — `approx-ann` only.
#[cfg(feature = "approx-ann")]
struct ApproxSearch<'a> {
    index: &'a DiskAnnIndex,
}

#[cfg(feature = "approx-ann")]
impl UnfilteredSearch for ApproxSearch<'_> {
    fn search(&self, query: &[f32], k: usize) -> Vec<(Id, f32)> {
        self.index.nearest(query, k)
    }
}

/// Executes a SPARQL query that may use the `vec:` magic predicates: parse,
/// [`rewrite_query`], then evaluate with the standard engine. The unfiltered k-NN is the
/// answer-exact full scan — for the **approximate** index path see [`query_vec_approx`].
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
    prepare_with(graph, sparql, store, &ExactSearch { store })
}

/// [OPUS-4.8] (sq-z589, epic sq-3183) [`query_vec`] but with the **unfiltered** k-NN run through an
/// approximate on-disk [`DiskAnnIndex`] (Vamana) instead of the exact full scan — for large `.spqv`
/// stores where the brute-force scan is the bottleneck. Everything else is identical to
/// [`query_vec`]: parse, rewrite, VALUES inlining, joins, error checks, the seed-self-exclusion, and
/// (when `filtered-ann` composes in) the BGP→mask filtered path.
///
/// **Approximate ⇒ recall < 1.0.** The index can miss a true neighbour the greedy beam never
/// visits; this is the *approximate* ranking, NOT the exact top-`k`. Use [`query_vec`] (the exact
/// scan) when answer-exactness matters. `OFF by default` — gated on `approx-ann` (the only feature
/// that pulls an approximate index into the build) on top of `vec-predicate`.
///
/// The `index` must be built against the SAME graph generation as `graph`/`store` (its dictionary
/// ids must match): pass an index built with [`DiskAnnIndex::build_for`] so the fingerprint guard
/// catches a stale index, exactly as the store's own fingerprint guard does.
///
/// Note: the **filtered** `vec:` path (when `filtered-ann` is also on and the neighbour variable is
/// constrained) is unaffected — it still goes through the cost-model'd filtered search. This entry
/// point only swaps the *unfiltered* scan for the approximate index.
#[cfg(feature = "approx-ann")]
pub fn query_vec_approx(
    graph: &Graph,
    sparql: &str,
    store: &VectorStore,
    index: &DiskAnnIndex,
) -> Result<QueryResult, String> {
    sparq_engine::query_prepared(graph, &prepare_vec_approx(graph, sparql, store, index)?)
}

/// [OPUS-4.8] (sq-z589) [`query_vec_approx`] under a cooperative [`QueryBudget`].
#[cfg(feature = "approx-ann")]
pub fn query_vec_approx_with_budget(
    graph: &Graph,
    sparql: &str,
    store: &VectorStore,
    index: &DiskAnnIndex,
    budget: &QueryBudget,
) -> Result<QueryResult, String> {
    sparq_engine::query_prepared_with_budget(
        graph,
        &prepare_vec_approx(graph, sparql, store, index)?,
        budget,
    )
}

/// [OPUS-4.8] (sq-z589) [`prepare_vec`] but resolving the **unfiltered** `vec:` k-NN through the
/// approximate `index` — the prepare-time twin of [`query_vec_approx`]. See that function for the
/// recall / staleness caveats.
#[cfg(feature = "approx-ann")]
pub fn prepare_vec_approx(
    graph: &Graph,
    sparql: &str,
    store: &VectorStore,
    index: &DiskAnnIndex,
) -> Result<PreparedQuery, String> {
    // The index is keyed by `graph`'s dictionary ids too, so verify it alongside the store when it
    // carries a fingerprint (built with `build_for`); a legacy/unbound index is left to work as
    // before, like the store guard below.
    if index.fingerprint().is_some() {
        index.check_graph(store, graph)?;
    }
    prepare_with(graph, sparql, store, &ApproxSearch { index })
}

/// Shared prepare body: staleness-guard the store, parse, then rewrite every `vec:` pattern with
/// `searcher` as the unfiltered backend.
fn prepare_with(
    graph: &Graph,
    sparql: &str,
    store: &VectorStore,
    searcher: &dyn UnfilteredSearch,
) -> Result<PreparedQuery, String> {
    // [OPUS-4.8] Opportunistic staleness guard: the store is keyed by `graph`'s
    // dictionary ids, so a store built against a DIFFERENT graph would silently
    // mis-resolve neighbours. If the store carries a graph fingerprint, verify
    // it now (hard error on mismatch). Unbound / legacy (version-1) stores carry
    // no fingerprint and are left to work as before — we only check when we can.
    if store.fingerprint().is_some() {
        store.check_graph(graph)?;
    }
    // [SONNET-4.6] (sq-tb9p0) VG-MET-1/VG-MET-4 (mainline): the `vec:` query surface evaluates
    // exactly ONE metric — cosine similarity. A store whose persisted v3 provenance declares a
    // DIFFERENT metric must be rejected, not scored: evaluating a cosine ranking over vectors
    // embedded for another metric returns well-formed but meaningless numbers. A legacy v1/v2
    // store (or one built without binding a provenance) declares nothing and is left to work as
    // before — implicit cosine, exactly the pre-provenance behaviour; the strict fail-closed
    // check for those is `VectorStore::check_provenance`. The declared NORMALIZATION regime is
    // deliberately NOT a rejection axis on this path: the exact scan computes cosine with the
    // norms taken at comparison time (scale-invariant), so both declared regimes (`none` / `l2`)
    // are evaluated faithfully under the declared cosine metric and no cross-normalisation
    // mismatch can arise here.
    if let Some(prov) = store.provenance() {
        if prov.metric != Metric::Cosine {
            return Err(format!(
                "vec: this store declares the '{}' distance metric, but the vec: query surface \
                 evaluates cosine similarity only; refusing the cross-metric evaluation (the \
                 scores would be well-formed but meaningless). Rebuild the store under cosine, \
                 or rank it through a '{}'-aware code path",
                prov.metric, prov.metric
            ));
        }
    }
    let query = spargebra::SparqlParser::new()
        .parse_query(sparql)
        .map_err(|e| e.to_string())?;
    Ok(PreparedQuery::from(rewrite_query_with(
        query, graph, store, searcher,
    )?))
}

/// Rewrites every `vec:` magic pattern in the query into inline `VALUES` over
/// the search hits (see the module docs). A query without `vec:` patterns
/// passes through unchanged. The unfiltered k-NN uses the exact full scan — for the approximate
/// index path use [`query_vec_approx`] / [`prepare_vec_approx`].
pub fn rewrite_query(query: Query, graph: &Graph, store: &VectorStore) -> Result<Query, String> {
    rewrite_query_with(query, graph, store, &ExactSearch { store })
}

/// Threaded form of [`rewrite_query`]: `searcher` is the unfiltered k-NN backend (exact scan or an
/// approximate index). [OPUS-4.8] (sq-z589)
fn rewrite_query_with(
    mut query: Query,
    graph: &Graph,
    store: &VectorStore,
    searcher: &dyn UnfilteredSearch,
) -> Result<Query, String> {
    let pattern = match &mut query {
        Query::Select { pattern, .. }
        | Query::Construct { pattern, .. }
        | Query::Describe { pattern, .. }
        | Query::Ask { pattern, .. } => pattern,
    };
    rewrite_pattern(pattern, graph, store, searcher)?;
    Ok(query)
}

/// Recursively rewrites the magic patterns inside `p`.
fn rewrite_pattern(
    p: &mut GraphPattern,
    graph: &Graph,
    store: &VectorStore,
    searcher: &dyn UnfilteredSearch,
) -> Result<(), String> {
    match p {
        GraphPattern::Bgp { patterns } => {
            let patterns = std::mem::take(patterns);
            *p = rewrite_bgp(patterns, graph, store, searcher)?;
        }
        GraphPattern::Join { left, right }
        | GraphPattern::LeftJoin { left, right, .. }
        | GraphPattern::Union { left, right }
        | GraphPattern::Minus { left, right }
        | GraphPattern::Lateral { left, right } => {
            rewrite_pattern(left, graph, store, searcher)?;
            rewrite_pattern(right, graph, store, searcher)?;
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
        | GraphPattern::Service { inner, .. } => rewrite_pattern(inner, graph, store, searcher)?,
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
    searcher: &dyn UnfilteredSearch,
) -> Result<GraphPattern, String> {
    // Index the rdf:first/rdf:rest triples by their (blank-node) list-cell
    // subject so the magic-predicate handlers can walk each `( … )` chain.
    let lists = ListCells::collect(&patterns);

    let mut rest: Vec<TriplePattern> = Vec::with_capacity(patterns.len());
    let mut reqs: Vec<KnnReq> = Vec::new();
    // [OPUS-4.8] Blank-node-subject rdf:first/rdf:rest cells are the lowered
    // form of `( … )` argument lists. They are only an implementation detail of
    // a `vec:` predicate, so we *defer* the decision to drop them: they are
    // consumed (removed) ONLY if this BGP actually contains a `vec:` request.
    // If no `vec:` predicate is present the BGP passes through unchanged —
    // including any legitimate RDF-collection query — and these deferred cells
    // are restored into `rest` below. User-authored `rdf:first`/`rest` patterns
    // with variable/IRI subjects are never deferred (they fall through to
    // `rest` like any other ordinary pattern).
    let mut deferred_lists: Vec<&TriplePattern> = Vec::new();

    for tp in &patterns {
        if is_list_cell(tp) {
            deferred_lists.push(tp);
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
                // [SONNET-4.6] (sq-tb9p0) VG-GOV-3: the VG-VOC-1 hard error states the highest
                // vocabulary revision this build implements, so a query written against a NEWER
                // spec revision's term fails with the version gap named, not just "unknown".
                return Err(format!(
                    "vec: unknown magic predicate <{}> (supported: vec:nearest, vec:search; \
                     this build implements vec: vocabulary revision {})",
                    iri,
                    vocab::VOCAB_REVISION
                ));
            }
        }
    }

    // No `vec:` request in this BGP: it (and any RDF-collection cells in it)
    // must pass through completely unchanged.
    if reqs.is_empty() {
        rest.extend(deferred_lists.into_iter().cloned());
        return Ok(GraphPattern::Bgp { patterns: rest });
    }

    // [OPUS-4.8] (sq-bvmd) The ordinary patterns that survive this BGP are also the
    // patterns that *constrain* each `vec:` neighbour variable: the subjects they admit
    // are the candidate id-set for a filtered ANN search. Derive that set from `rest`
    // BEFORE it is moved into the output (the borrow ends with `mask_constraint`).
    #[cfg(feature = "filtered-ann")]
    let mask_constraint = rest.clone();

    // Join the hit tables onto the remaining ordinary patterns.
    let mut out = GraphPattern::Bgp { patterns: rest };
    for req in reqs {
        #[cfg(feature = "filtered-ann")]
        let hits = run_knn(&req, graph, store, searcher, &mask_constraint)?;
        #[cfg(not(feature = "filtered-ann"))]
        let hits = run_knn(&req, graph, store, searcher)?;
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

/// True for a *synthetic* list-cell triple — a `rdf:first`/`rdf:rest` triple
/// with a **blank-node subject**, the exact shape spargebra emits when it lowers
/// a `( … )` collection. [OPUS-4.8] The blank-node-subject restriction is
/// load-bearing: it means user-authored collection patterns with variable or
/// IRI subjects (e.g. `?x rdf:first ?y`) are NOT treated as argument-list cells
/// and survive into the engine. (Combined with the caller only dropping these
/// when a `vec:` predicate is present, a query with no `vec:` pattern is left
/// entirely unchanged.)
fn is_list_cell(tp: &TriplePattern) -> bool {
    matches!(&tp.subject, TermPattern::BlankNode(_))
        && matches!(&tp.predicate, NamedNodePattern::NamedNode(p) if p.as_str() == RDF_FIRST || p.as_str() == RDF_REST)
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
/// for a blank node (including one nested inside a triple term).
fn term_to_ground(t: Term) -> Option<GroundTerm> {
    match t {
        Term::NamedNode(n) => Some(GroundTerm::NamedNode(n)),
        Term::Literal(l) => Some(GroundTerm::Literal(l)),
        // [GPT-5.6] sq-1ijoc: SPARQL 1.2 VALUES admits ground triple terms;
        // spargebra's conversion recursively rejects any nested blank node.
        Term::Triple(t) => spargebra::term::GroundTriple::try_from(*t)
            .ok()
            .map(|t| GroundTerm::Triple(Box::new(t))),
        Term::BlankNode(_) => None,
    }
}

/// Runs the k-NN search for `req` and returns `(neighbour id, cosine score)`
/// pairs, best first.
///
/// [OPUS-4.8] (sq-z589) The UNFILTERED search runs through `searcher` — the exact full scan
/// ([`ExactSearch`], the default `vec:` behaviour) or an approximate index (`ApproxSearch`, the
/// `*_approx` entry points). The store is still used to look up the seed vector and (for the
/// filtered path) to scan the masked candidates.
///
/// [OPUS-4.8] (sq-bvmd) When the `filtered-ann` feature is on, `constraint` is the
/// BGP's surviving ordinary patterns; if any of them constrain `req.node`, the search
/// is restricted to the candidate id-set those patterns admit (filtered ANN) instead
/// of scanning the whole store and joining afterwards — and that filtered path goes through the
/// cost-model'd `nearest_filtered_costed_tiebreak`, INDEPENDENT of `searcher` (the approximate
/// seam is only the unfiltered scan; the filtered path is always answer-exact and applies the
/// same VG-TIE-1 boundary rule as the unfiltered exact path, with the seed excluded before the
/// boundary is determined — sq-bvmd/sq-7hx6 + sq-tb9p0). When no pattern constrains the
/// neighbour variable — or with `filtered-ann` off — `searcher` runs.
fn run_knn(
    req: &KnnReq,
    graph: &Graph,
    store: &VectorStore,
    searcher: &dyn UnfilteredSearch,
    #[cfg(feature = "filtered-ann")] constraint: &[TriplePattern],
) -> Result<Vec<(Id, f32)>, String> {
    // Derive the candidate id-set (IdMask) from the patterns that constrain `req.node`.
    // `None` => no constraining pattern => fall through to the unfiltered path.
    #[cfg(feature = "filtered-ann")]
    let mask = derive_mask(&req.node, constraint, graph)?;

    match &req.query {
        QueryArg::Node(iri) => {
            let term = Term::NamedNode(iri.clone());
            let Some(id) = graph.id_of(&term) else {
                return Ok(Vec::new());
            };
            let Some(query) = store.get(id) else {
                return Ok(Vec::new());
            };
            #[cfg(feature = "filtered-ann")]
            if let Some(mask) = &mask {
                // Filtered: search only BGP-admitted candidates, with the seed excluded
                // from the pool BEFORE the k boundary is determined and boundary-tie
                // membership decided by the VG-TIE-1 N-Triples rule — so the filtered
                // answer equals post-filtering the unfiltered tie-broken retrieval
                // (VG-FILT-2 in answer-exact mode). [OPUS-4.8] (sq-7hx6) The cost model
                // picks pre-filter (scan the mask) vs post-filter (scan the whole store +
                // drop non-members) by the mask's selectivity; BOTH branches return the
                // identical answer either way.
                let (hits, _est) = nearest_filtered_costed_tiebreak(
                    store,
                    graph,
                    query,
                    mask,
                    req.k,
                    Some(id),
                    &CostModel::default(),
                )?;
                return Ok(hits);
            }
            // [SONNET-4.6] (sq-tb9p0) Answer-exact path: rank with the VG-TIE-1 boundary
            // tie-break (seed excluded before ranking, so the boundary is computed over the
            // true candidate pool). A blank-node candidate whose term would decide
            // membership is a fail-closed hard error (`?`), per the embeddable domain.
            if let Some(hits) = searcher.search_tied(query, req.k, graph, Some(id))? {
                return Ok(hits);
            }
            // Approximate backend: over-fetch by one and drop the seed itself (recall < 1 —
            // no true boundary exists to tie-break at).
            Ok(searcher
                .search(query, req.k + 1)
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
            #[cfg(feature = "filtered-ann")]
            if let Some(mask) = &mask {
                // [OPUS-4.8] (sq-7hx6) Cost-model choice of pre-filter vs post-filter —
                // identical answer either way — with boundary-tie membership decided by
                // the VG-TIE-1 N-Triples rule over the admitted pool (no seed to exclude
                // in the query-by-vector form).
                let (hits, _est) = nearest_filtered_costed_tiebreak(
                    store,
                    graph,
                    v,
                    mask,
                    req.k,
                    None,
                    &CostModel::default(),
                )?;
                return Ok(hits);
            }
            // [SONNET-4.6] (sq-tb9p0) Answer-exact path with the VG-TIE-1 boundary tie-break;
            // an approximate backend falls through to its plain search. A blank-node
            // candidate whose term would decide membership is a fail-closed hard error (`?`).
            if let Some(hits) = searcher.search_tied(v, req.k, graph, None)? {
                return Ok(hits);
            }
            Ok(searcher.search(v, req.k))
        }
    }
}

/// [OPUS-4.8] (sq-3tjd, epic sq-3183) Builds the candidate [`IdMask`] for `node` from the
/// surrounding BGP `constraint` patterns — the dictionary ids of every binding `node` takes
/// when the **join-connected** constraining sub-BGP is evaluated against `graph`, then
/// projected back onto `node`.
///
/// The constraining sub-BGP is the **connected component of the BGP join-graph** that
/// contains `node`: starting from the patterns that mention `node`, follow shared variables
/// transitively into the patterns those reach, and so on. This generalises the original
/// single-variable (#281/sq-bvmd) rule — a *direct-mention-only* component reduces to exactly
/// the old behaviour — to **transitive / multi-variable** constraints: `?node :owns ?x .
/// ?x a :Vehicle` restricts `?node` to subjects that own a `:Vehicle` even though the second
/// pattern never names `?node`.
///
/// Patterns *not* reachable from `node` through shared variables are deliberately excluded:
/// a disconnected pattern joins as a cartesian product, which never *removes* a `node`
/// binding, so pulling it in could only widen (a cross-product can drop a binding only if it
/// is empty — but an empty disconnected BGP would zero the whole join in the outer query too,
/// which is a different concern handled by the engine, not a mask we may narrow with). We
/// only ever **narrow**, never widen: the projected `node` set of the connected component is
/// a superset of (or equal to) the set the full outer BGP would bind, so the filtered top-k
/// stays a subset of the unfiltered top-k and equals post-filtering by the (now transitive)
/// constraint. If `node` is unconstrained (no pattern mentions it) the function returns
/// `Ok(None)` and the caller runs the plain unfiltered search, preserving the existing
/// `vec:` semantics exactly.
///
/// The connected sub-BGP is evaluated through the standard engine (the same seam the outer
/// query uses), so shared join variables are honoured correctly: the mask is exactly the set
/// the engine binds to `node` for that component.
#[cfg(feature = "filtered-ann")]
fn derive_mask(
    node: &Variable,
    constraint: &[TriplePattern],
    graph: &Graph,
) -> Result<Option<IdMask>, String> {
    // The connected component of the BGP join-graph that contains `node`: every pattern
    // join-reachable from `node` through shared variables. A direct-mention-only BGP yields
    // exactly the same patterns the old single-variable rule did (strict no-regression).
    let sub = connected_component(node, constraint);
    if sub.is_empty() {
        return Ok(None);
    }

    // [OPUS-4.8] (sq-36ol) The mask the connected sub-BGP derives is a pure function of (the
    // sub-BGP, the graph) — re-deriving it for the SAME sub-BGP against the SAME graph always
    // yields the same ids. So cache it keyed by (sub-BGP terms, graph fingerprint). Crucially the
    // sub-BGP here is the **connected component** (sq-3tjd) `connected_component` computed above,
    // so a cached entry corresponds to the full TRANSITIVE constraint — caching wraps the
    // transitive derivation, not the old single-variable one. The fingerprint folds dict_len +
    // triple_count + an id-ordered content hash (see `crate::fingerprint`), so ANY mutation that
    // could change which ids the sub-BGP binds — an added/removed triple, a shifted dict id, a
    // changed term — changes the fingerprint and therefore MISSES the cache (recompute). The query
    // variable `?node` is normalised to a fixed name so two queries that differ only in the
    // neighbour variable's name share the entry (the derived id-set is independent of that name).
    // A cache HIT returns an `IdMask` byte-identical to a fresh derivation; a MISS recomputes
    // through the engine as before.
    let fp = Fingerprint::of(graph);
    let key = mask_cache::key(&sub, node);
    if let Some(mask) = mask_cache::get(&key, fp) {
        return Ok(Some(mask));
    }

    // SELECT ?node WHERE { <connected sub-BGP> } — evaluate the constraint through the engine
    // and collect the ids it binds to `node`. Projecting `node` discards the intermediate
    // join variables (e.g. `?x` above), so the mask is precisely the eligible neighbour set.
    let select = Query::Select {
        dataset: None,
        pattern: GraphPattern::Project {
            inner: Box::new(GraphPattern::Bgp { patterns: sub }),
            variables: vec![node.clone()],
        },
        base_iri: None,
    };
    let result = sparq_engine::query_prepared(graph, &PreparedQuery::from(select))?;

    // Map each bound `node` term back to its dictionary id. A row whose `node` is unbound
    // (impossible for a BGP-only projection, but cheap to guard) or whose term is not in
    // the dictionary contributes nothing — it cannot match a stored vector anyway.
    let mask: IdMask = result
        .rows
        .iter()
        .filter_map(|row| row.first().and_then(|c| c.as_ref()))
        .filter_map(|term| graph.id_of(term))
        .collect();
    mask_cache::put(key, fp, mask.clone());
    Ok(Some(mask))
}

/// [OPUS-4.8] (sq-36ol, epic sq-3183) Cross-prepare cache for the filtered-ANN `IdMask` that
/// [`derive_mask`] computes from a constraining sub-BGP. Keyed by **(canonical sub-BGP, graph
/// [`Fingerprint`])**; reused on a hit, invalidated — by construction — on any graph change.
///
/// # Why the invalidation is SOUND (the load-bearing property)
///
/// The cached value is the set of dictionary ids the sub-BGP binds to the neighbour variable
/// against a specific graph. That set is a pure function of (sub-BGP, graph content). The
/// [`Fingerprint`] folds the graph's `dict_len`, `triple_count`, and an **id-ordered content
/// hash** over every dictionary term (see [`crate::fingerprint`]). Any mutation that could
/// change which ids the sub-BGP binds necessarily changes one of those: adding/removing a triple
/// bumps `triple_count`; adding/removing/changing a term changes `dict_len` and/or the content
/// hash; even a pure dict-id *shift* (same terms, different interning order) changes the content
/// hash because each term's id is folded in. A changed fingerprint is a DIFFERENT key, so a
/// post-mutation lookup MISSES and recomputes — a stale mask can never be served. When in doubt
/// we miss: the fingerprint is conservative (it can differ for two graphs that happen to bind the
/// same ids), which only ever costs a recompute, never correctness.
///
/// The cache lives in a `thread_local!` so it needs no change to the `&Graph`/`&VectorStore`
/// shared-reference API and stays internal to the `vec:` rewrite — `sparq-core`/`sparq-engine`
/// are untouched. It is bounded (LRU-ish capacity cap) so a long-lived process running many
/// distinct queries cannot grow it without bound.
#[cfg(feature = "filtered-ann")]
mod mask_cache {
    use super::{Fingerprint, IdMask, TriplePattern, Variable};
    use spargebra::term::TermPattern;
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// Cache key: the canonicalised constraining sub-BGP. The neighbour variable is renamed to a
    /// fixed placeholder so two otherwise-identical constraints that differ only in the neighbour
    /// variable's *name* share an entry — the derived id-set does not depend on that name.
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub(super) struct Key {
        patterns: Vec<TriplePattern>,
    }

    /// Builds a [`Key`] from the constraining `sub` patterns, normalising occurrences of `node`
    /// to a fixed variable name so the entry is shared across neighbour-variable renamings.
    pub(super) fn key(sub: &[TriplePattern], node: &Variable) -> Key {
        let placeholder = Variable::new_unchecked("\u{0}vec_node");
        let rename = |t: &TermPattern| -> TermPattern {
            match t {
                TermPattern::Variable(v) if v == node => TermPattern::Variable(placeholder.clone()),
                other => other.clone(),
            }
        };
        let patterns = sub
            .iter()
            .map(|tp| TriplePattern {
                subject: rename(&tp.subject),
                predicate: match &tp.predicate {
                    spargebra::term::NamedNodePattern::Variable(v) if v == node => {
                        spargebra::term::NamedNodePattern::Variable(placeholder.clone())
                    }
                    other => other.clone(),
                },
                object: rename(&tp.object),
            })
            .collect();
        Key { patterns }
    }

    /// Soft cap on distinct cached entries. A long-lived process issuing many distinct
    /// constraint/graph combinations clears the cache wholesale on overflow rather than growing
    /// unbounded — correctness is unaffected (a cleared entry just re-derives).
    const CAPACITY: usize = 256;

    thread_local! {
        // (Key, Fingerprint) → mask. The fingerprint is PART of the key so an entry for a stale
        // graph can never be read — a changed graph yields a different fingerprint, hence a miss.
        static CACHE: RefCell<HashMap<(Key, Fingerprint), IdMask>> = RefCell::new(HashMap::new());
    }

    /// Returns the cached mask for `(key, fp)`, or `None` on a miss (different sub-BGP OR a
    /// different graph fingerprint — i.e. any graph mutation).
    pub(super) fn get(key: &Key, fp: Fingerprint) -> Option<IdMask> {
        CACHE.with(|c| c.borrow().get(&(key.clone(), fp)).cloned())
    }

    /// Inserts the freshly-derived mask under `(key, fp)`.
    pub(super) fn put(key: Key, fp: Fingerprint, mask: IdMask) {
        CACHE.with(|c| {
            let mut m = c.borrow_mut();
            if m.len() >= CAPACITY {
                m.clear();
            }
            m.insert((key, fp), mask);
        });
    }

    /// Test-only: drop all entries so a test starts from a known-cold cache.
    #[cfg(test)]
    pub(super) fn clear() {
        CACHE.with(|c| c.borrow_mut().clear());
    }

    /// Test-only: number of live entries (to assert a hit reused rather than re-inserted).
    #[cfg(test)]
    pub(super) fn len() -> usize {
        CACHE.with(|c| c.borrow().len())
    }
}

/// [OPUS-4.8] (sq-3tjd) The connected component of the BGP join-graph containing `node`.
///
/// Models the BGP as a graph whose nodes are *variables* and whose edges link the variables
/// that co-occur in a triple pattern; the component is every pattern reachable from `node`
/// through shared variables. Implemented as a worklist over a frontier of "live" variables:
/// seed with `node`, and repeatedly admit any not-yet-taken pattern that mentions a live
/// variable, adding that pattern's own variables to the frontier — a transitive closure that
/// terminates because each pass either consumes a pattern or grows no further.
///
/// Returns the patterns in the component (empty iff no pattern mentions `node`, i.e. `node`
/// is unconstrained). The order mirrors the input BGP for determinism.
#[cfg(feature = "filtered-ann")]
fn connected_component(node: &Variable, patterns: &[TriplePattern]) -> Vec<TriplePattern> {
    // Live variables whose patterns are (transitively) part of the component.
    let mut live: Vec<Variable> = vec![node.clone()];
    // `taken[i]` once pattern `i` has been pulled into the component.
    let mut taken = vec![false; patterns.len()];

    loop {
        let mut grew = false;
        for (i, tp) in patterns.iter().enumerate() {
            if taken[i] {
                continue;
            }
            let vars = pattern_vars(tp);
            if vars.iter().any(|v| live.contains(v)) {
                taken[i] = true;
                grew = true;
                // Promote this pattern's other variables into the live frontier so the
                // closure follows the join transitively.
                for v in vars {
                    if !live.contains(&v) {
                        live.push(v);
                    }
                }
            }
        }
        if !grew {
            break;
        }
    }

    patterns
        .iter()
        .zip(&taken)
        .filter(|(_, &t)| t)
        .map(|(tp, _)| tp.clone())
        .collect()
}

/// [OPUS-4.8] (sq-3tjd) The variables `tp` mentions, in any term position (subject,
/// predicate, object).
#[cfg(feature = "filtered-ann")]
fn pattern_vars(tp: &TriplePattern) -> Vec<Variable> {
    let mut out = Vec::with_capacity(3);
    if let TermPattern::Variable(v) = &tp.subject {
        out.push(v.clone());
    }
    if let NamedNodePattern::Variable(v) = &tp.predicate {
        out.push(v.clone());
    }
    if let TermPattern::Variable(v) = &tp.object {
        out.push(v.clone());
    }
    out
}

// [OPUS-4.8] (sq-36ol, epic sq-3183) Unit tests for the derived-mask cache. These reach the
// PRIVATE `mask_cache` (entry count, clear) so they can assert a HIT *reused* rather than
// re-derived, and that a graph mutation *misses*. The end-to-end public-API behaviour is also
// covered in tests/filtered_bgp.rs. Gated on both features (the cache only exists with them).
#[cfg(all(test, feature = "filtered-ann"))]
mod mask_cache_tests {
    use super::*;
    use sparq_core::dict::Id;

    /// A constraining sub-BGP `?node <kind> :Car` over the given variable name.
    fn car_constraint(node: &str) -> (Variable, Vec<TriplePattern>) {
        let node = Variable::new_unchecked(node);
        let tp = TriplePattern {
            subject: TermPattern::Variable(node.clone()),
            predicate: NamedNodePattern::NamedNode(NamedNode::new("http://ex/kind").unwrap()),
            object: TermPattern::NamedNode(NamedNode::new("http://ex/Car").unwrap()),
        };
        (node, vec![tp])
    }

    /// Sorted ids of a mask, for an order-independent identity comparison (IdMask is a set).
    fn sorted(mask: &IdMask) -> Vec<Id> {
        let mut v: Vec<Id> = mask.iter().collect();
        v.sort_unstable();
        v
    }

    fn cars_graph() -> Graph {
        Graph::load_str(
            "<http://ex/a> <http://ex/kind> <http://ex/Car> .\n\
             <http://ex/b> <http://ex/kind> <http://ex/Car> .\n\
             <http://ex/d> <http://ex/kind> <http://ex/Boat> .\n",
            "ntriples",
        )
        .unwrap()
    }

    /// A HIT returns a mask byte-identical (same id set) to a fresh derivation, and reuses the
    /// SAME entry (no second insertion) — proving the second prepare didn't re-run the engine.
    #[test]
    fn hit_returns_identical_mask_without_reinserting() {
        mask_cache::clear();
        let g = cars_graph();
        let (node, sub) = car_constraint("node");

        let first = derive_mask(&node, &sub, &g).unwrap().unwrap();
        let after_first = mask_cache::len();
        assert_eq!(after_first, 1, "first derivation must populate one entry");

        let second = derive_mask(&node, &sub, &g).unwrap().unwrap();
        assert_eq!(
            mask_cache::len(),
            after_first,
            "a HIT must reuse the entry, not insert another"
        );
        assert_eq!(
            sorted(&first),
            sorted(&second),
            "the cached mask must be identical to the freshly-derived one"
        );
        // {a, b} are the Cars.
        assert_eq!(sorted(&first).len(), 2, "two Cars in the fixture");
    }

    /// ADVERSARIAL invalidation: the same constraining sub-BGP against a MUTATED graph must NOT
    /// serve the stale mask. We construct a second graph that adds a Car (`c`) — the mask must
    /// now include it. Because the fingerprint folds triple_count + an id-ordered content hash,
    /// the post-mutation lookup is a different key → a miss → a recompute → the CHANGED mask.
    ///
    /// The mutation is deliberately the hard case: the new graph keeps every original triple AND
    /// every original (a, b, d) dict id binding, adding only `c`. A naive cache keyed on the
    /// sub-BGP alone (or on triple_count alone, had the change been a pure dict-id permutation)
    /// would wrongly hit. The full fingerprint catches it.
    #[test]
    fn mutation_invalidates_and_recomputes_changed_mask() {
        mask_cache::clear();
        let (node, sub) = car_constraint("node");

        let g1 = cars_graph();
        let mask1 = derive_mask(&node, &sub, &g1).unwrap().unwrap();
        assert_eq!(sorted(&mask1).len(), 2, "g1 has two Cars (a, b)");

        // Mutation: add a third Car `c` (a new triple AND a new term). Anything that could change
        // the bound id-set changes the fingerprint.
        let g2 = Graph::load_str(
            "<http://ex/a> <http://ex/kind> <http://ex/Car> .\n\
             <http://ex/b> <http://ex/kind> <http://ex/Car> .\n\
             <http://ex/c> <http://ex/kind> <http://ex/Car> .\n\
             <http://ex/d> <http://ex/kind> <http://ex/Boat> .\n",
            "ntriples",
        )
        .unwrap();

        let mask2 = derive_mask(&node, &sub, &g2).unwrap().unwrap();
        assert_eq!(
            sorted(&mask2).len(),
            3,
            "after adding Car c the mask must include it — NOT the stale 2-Car set"
        );
        // The stale mask must never be served: the new mask differs from the old one.
        assert_ne!(
            sorted(&mask1),
            sorted(&mask2),
            "the post-mutation mask must differ from the pre-mutation (stale) one"
        );
        // And it resolves to the live graph's Car ids (soundness against g2's dictionary).
        for iri in ["http://ex/a", "http://ex/b", "http://ex/c"] {
            let id = g2
                .id_of(&Term::NamedNode(NamedNode::new(iri).unwrap()))
                .unwrap();
            assert!(mask2.contains(id), "{iri} must be admitted by the g2 mask");
        }
    }

    /// ADVERSARIAL: two graphs that bind the SAME Car IRIs but have DIFFERENT fingerprints must
    /// occupy distinct cache entries — the fingerprint is PART of the key, so the cache can never
    /// serve g1's mask for a query against g2 even when the constraining sub-BGP is byte-identical.
    /// Here g2 adds an unrelated triple (`x p y`): the Car set {a,b} is unchanged but the
    /// fingerprint (triple_count + content hash) differs, which is the exact soundness property —
    /// the cache keys on the WHOLE-graph fingerprint, conservatively missing whenever it changes.
    #[test]
    fn different_fingerprint_same_constraint_misses() {
        mask_cache::clear();
        let (node, sub) = car_constraint("node");

        let g1 = Graph::load_str(
            "<http://ex/a> <http://ex/kind> <http://ex/Car> .\n\
             <http://ex/b> <http://ex/kind> <http://ex/Car> .\n",
            "ntriples",
        )
        .unwrap();
        // g2: same two Cars PLUS an unrelated triple → same Car id-set, DIFFERENT fingerprint.
        let g2 = Graph::load_str(
            "<http://ex/a> <http://ex/kind> <http://ex/Car> .\n\
             <http://ex/b> <http://ex/kind> <http://ex/Car> .\n\
             <http://ex/x> <http://ex/p> <http://ex/y> .\n",
            "ntriples",
        )
        .unwrap();
        // Precondition: the two graphs genuinely differ in fingerprint (the cache key's graph half).
        assert_ne!(
            Fingerprint::of(&g1),
            Fingerprint::of(&g2),
            "the two graphs must have distinct fingerprints for this test to be meaningful"
        );

        let _m1 = derive_mask(&node, &sub, &g1).unwrap().unwrap();
        let entries_after_g1 = mask_cache::len();
        let m2 = derive_mask(&node, &sub, &g2).unwrap().unwrap();
        // A different fingerprint → a second, distinct entry (the g1 mask was not served).
        assert_eq!(
            mask_cache::len(),
            entries_after_g1 + 1,
            "a different graph fingerprint must MISS and insert a fresh entry (never reuse a stale one)"
        );
        // The g2 mask must resolve to g2's own ids for a and b.
        for iri in ["http://ex/a", "http://ex/b"] {
            let id = g2
                .id_of(&Term::NamedNode(NamedNode::new(iri).unwrap()))
                .unwrap();
            assert!(m2.contains(id), "{iri} must be admitted against g2's dict");
        }
    }

    /// The key distinguishes DIFFERENT sub-BGPs against the same graph: a `:Car` constraint and a
    /// `:Boat` constraint must not share an entry, and must yield different masks.
    #[test]
    fn distinct_sub_bgps_do_not_collide() {
        mask_cache::clear();
        let g = cars_graph();

        let (node, car_sub) = car_constraint("node");
        let boat_sub = vec![TriplePattern {
            subject: TermPattern::Variable(node.clone()),
            predicate: NamedNodePattern::NamedNode(NamedNode::new("http://ex/kind").unwrap()),
            object: TermPattern::NamedNode(NamedNode::new("http://ex/Boat").unwrap()),
        }];

        let car_mask = derive_mask(&node, &car_sub, &g).unwrap().unwrap();
        let boat_mask = derive_mask(&node, &boat_sub, &g).unwrap().unwrap();
        assert_eq!(
            mask_cache::len(),
            2,
            "two distinct sub-BGPs must occupy two distinct entries"
        );
        assert_ne!(
            sorted(&car_mask),
            sorted(&boat_mask),
            "the Car and Boat constraints must derive different masks"
        );
        assert_eq!(sorted(&car_mask).len(), 2); // a, b
        assert_eq!(sorted(&boat_mask).len(), 1); // d
    }

    /// Two queries that differ ONLY in the neighbour variable's name share an entry (the derived
    /// id-set is independent of that name) — the key normalises the neighbour variable.
    #[test]
    fn neighbour_variable_rename_shares_entry() {
        mask_cache::clear();
        let g = cars_graph();

        let (node_a, sub_a) = car_constraint("node");
        let (node_b, sub_b) = car_constraint("entity"); // same shape, different var name

        let ma = derive_mask(&node_a, &sub_a, &g).unwrap().unwrap();
        let mb = derive_mask(&node_b, &sub_b, &g).unwrap().unwrap();
        assert_eq!(
            mask_cache::len(),
            1,
            "a neighbour-variable rename must share the cache entry"
        );
        assert_eq!(sorted(&ma), sorted(&mb));
    }
}
