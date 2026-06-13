//! SPARQL 1.1 Update (roadmap T10) over the FULL DATASET — default graph + named graphs.
//!
//! v1 applied data operations on the default graph only and silently DROPPED named graphs on
//! rebuild (conformance finding F19, a data-loss class). v2 models the dataset as a set of
//! term-triples per graph and implements every `GraphUpdateOperation`:
//! INSERT DATA / DELETE DATA with `GRAPH` blocks, `DELETE/INSERT … WHERE` with graph templates
//! (including variable graph names) and `USING (NAMED)` dataset re-scoping, CLEAR / DROP /
//! CREATE (ADD / COPY / MOVE arrive from the parser desugared into DROP + DELETE/INSERT),
//! and LOAD for `file://` sources. The rebuild path ([`update`]) is correct-and-simple O(n);
//! the delta-overlay path ([`update_in_place`]) keeps data operations O(batch) per graph.
//!
//! Failure policy: operations whose failure the spec leaves optional (CLEAR/DROP of an absent
//! graph, CREATE of an existing one) are no-ops here — a graph store that auto-creates graphs
//! is explicitly allowed to succeed on them — so `SILENT` only matters for LOAD.

use crate::dataset::{build, decode_triples, empty_graph, TripleSet, TripleTerms};
use oxrdf::{BlankNode, NamedOrBlankNode, Term, Variable};
use rustc_hash::FxHashMap;
use sparq_core::Graph;
use spargebra::algebra::{GraphTarget, QueryDataset};
use spargebra::term::{
    GraphName, GraphNamePattern, GroundQuad, GroundQuadPattern, GroundTerm, GroundTermPattern,
    NamedNodePattern, Quad, QuadPattern, TermPattern,
};
use spargebra::GraphUpdateOperation;
use spargebra::SparqlParser;

/// A graph slot: `None` = the default graph, `Some(term)` = that named graph.
type GraphSlot = Option<Term>;

fn nob_to_term(s: &NamedOrBlankNode) -> Term {
    match s {
        NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
        NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
    }
}

fn ground_to_term(t: &GroundTerm) -> Term {
    match t {
        GroundTerm::NamedNode(n) => Term::NamedNode(n.clone()),
        GroundTerm::Literal(l) => Term::Literal(l.clone()),
        // A ground RDF 1.2 triple term in DELETE DATA: fully concrete.
        GroundTerm::Triple(t) => Term::Triple(Box::new(oxrdf::Triple::new(
            t.subject.clone(),
            t.predicate.clone(),
            ground_to_term(&t.object),
        ))),
    }
}

fn graph_name_slot(g: &GraphName) -> GraphSlot {
    match g {
        GraphName::DefaultGraph => None,
        GraphName::NamedNode(n) => Some(Term::NamedNode(n.clone())),
    }
}

/// [OPUS-4.8] roborev 1646 (Med): per SPARQL 1.1 Update §3.1.1, blank nodes in INSERT DATA are
/// "assumed to be disjoint from the blank nodes in the Graph Store" — they MUST be inserted as
/// FRESH nodes. The old `quad_to_triple` preserved the parsed label, so `INSERT DATA { _:b … }`
/// would conflate with an existing `_:b` in the store. We rewrite every blank-node label
/// (including inside RDF-1.2 triple terms) through `fresh`, so the same label within one INSERT
/// DATA operation stays one node while colliding with no existing graph blank node.
fn freshen_term(t: &Term, fresh: &mut FreshBnodes) -> Term {
    match t {
        Term::BlankNode(b) => fresh.get(b.as_str()),
        Term::Triple(tr) => {
            let subject = match freshen_term(&Term::from(tr.subject.clone()), fresh) {
                Term::NamedNode(n) => NamedOrBlankNode::NamedNode(n),
                Term::BlankNode(b) => NamedOrBlankNode::BlankNode(b),
                // a triple-term subject is always a named/blank node, so this is unreachable;
                // fall back to the original to stay total.
                _ => tr.subject.clone(),
            };
            let object = freshen_term(&tr.object, fresh);
            Term::Triple(Box::new(oxrdf::Triple::new(subject, tr.predicate.clone(), object)))
        }
        other => other.clone(),
    }
}

fn quad_to_triple_fresh(q: &Quad, fresh: &mut FreshBnodes) -> (GraphSlot, TripleTerms) {
    let subject = freshen_term(&nob_to_term(&q.subject), fresh);
    (
        graph_name_slot(&q.graph_name),
        [subject, Term::NamedNode(q.predicate.clone()), freshen_term(&q.object, fresh)],
    )
}

fn ground_quad_to_triple(q: &GroundQuad) -> (GraphSlot, TripleTerms) {
    (
        graph_name_slot(&q.graph_name),
        [Term::NamedNode(q.subject.clone()), Term::NamedNode(q.predicate.clone()), ground_to_term(&q.object)],
    )
}

// --- the mutable dataset model (rebuild path) ---------------------------------------------------
// (Decode/build primitives live in `crate::dataset`, shared with the query side's
// FROM / FROM NAMED active-dataset construction.)

/// The whole dataset as decoded term-triple sets: the working representation of the rebuild
/// path. Named-graph order is preserved; a named graph may be present and EMPTY (CREATE).
struct Dataset {
    default: TripleSet,
    named: Vec<(Term, TripleSet)>,
}

impl Dataset {
    fn decode(graph: &Graph) -> Dataset {
        Dataset {
            default: decode_triples(graph),
            named: graph.named.iter().map(|(name, g)| (name.clone(), decode_triples(g))).collect(),
        }
    }

    fn graph(&self, name: &Term) -> Option<&TripleSet> {
        self.named.iter().find(|(n, _)| n == name).map(|(_, s)| s)
    }

    /// The named graph's set, CREATING an empty one if absent (auto-create store semantics).
    fn graph_mut(&mut self, name: &Term) -> &mut TripleSet {
        if let Some(i) = self.named.iter().position(|(n, _)| n == name) {
            return &mut self.named[i].1;
        }
        self.named.push((name.clone(), TripleSet::default()));
        &mut self.named.last_mut().unwrap().1
    }

    fn slot_mut(&mut self, slot: &GraphSlot) -> &mut TripleSet {
        match slot {
            None => &mut self.default,
            Some(name) => self.graph_mut(name),
        }
    }

    fn insert(&mut self, slot: &GraphSlot, t: TripleTerms) {
        self.slot_mut(slot).insert(t);
    }

    fn remove(&mut self, slot: &GraphSlot, t: &TripleTerms) {
        match slot {
            None => {
                self.default.remove(t);
            }
            // Deleting from an absent graph is a no-op (it has nothing to delete).
            Some(name) => {
                if let Some(i) = self.named.iter().position(|(n, _)| n == name) {
                    self.named[i].1.remove(t);
                }
            }
        }
    }

    /// Materialise the dataset as a queryable [`Graph`] (named graphs included, even empty
    /// ones — `GRAPH <g> {}` over a CREATEd-but-empty graph must yield the unit row).
    fn build(&self) -> Graph {
        let mut g = build(&self.default);
        g.named = self.named.iter().map(|(name, set)| (name.clone(), build(set))).collect();
        g
    }

    /// The active dataset for a `USING (NAMED)` / `WITH` re-scoped WHERE: the default
    /// graph is the UNION of the `USING` graphs (absent store graphs contribute nothing,
    /// like a FROM of an empty document). `named: Some(list)` — explicit `USING NAMED` —
    /// makes exactly those the named graphs; `named: None` (how the parser encodes `WITH`,
    /// which only re-scopes the DEFAULT graph) keeps the store's named graphs untouched.
    fn build_using(&self, u: &QueryDataset) -> Graph {
        let mut default = TripleSet::default();
        for n in &u.default {
            if let Some(s) = self.graph(&Term::NamedNode(n.clone())) {
                default.extend(s.iter().cloned());
            }
        }
        let mut g = build(&default);
        match &u.named {
            Some(named) => {
                for n in named {
                    let name = Term::NamedNode(n.clone());
                    if let Some(s) = self.graph(&name) {
                        g.named.push((name, build(s)));
                    }
                }
            }
            None => g.named = self.named.iter().map(|(name, set)| (name.clone(), build(set))).collect(),
        }
        g
    }
}

// --- template instantiation for DELETE/INSERT … WHERE ------------------------------------------
// Substitute a template term from a WHERE solution. `None` => an unbound variable (or an
// ill-formed substitution), so the whole quad is skipped (per SPARQL, a template quad with an
// unbound/invalid slot is not produced).
type Subst<'a> = dyn Fn(&Variable) -> Option<Term> + 'a;

/// Fresh-blank-node state for ONE solution row: per SPARQL, every blank node in an INSERT
/// template is instantiated FRESH per solution (same label, same row → same fresh node;
/// different rows — and different operations in one request — get DIFFERENT nodes, hence
/// the process-wide counter).
struct FreshBnodes {
    map: FxHashMap<String, Term>,
}

static FRESH_BNODE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl FreshBnodes {
    fn get(&mut self, label: &str) -> Term {
        if let Some(t) = self.map.get(label) {
            return t.clone();
        }
        let n = FRESH_BNODE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let t = Term::BlankNode(BlankNode::new_unchecked(format!("fb{n}")));
        self.map.insert(label.to_string(), t.clone());
        t
    }
}

fn tp_subst(tp: &TermPattern, get: &Subst, fresh: &mut FreshBnodes) -> Option<Term> {
    match tp {
        TermPattern::Variable(v) => get(v),
        TermPattern::NamedNode(n) => Some(Term::NamedNode(n.clone())),
        TermPattern::BlankNode(b) => Some(fresh.get(b.as_str())),
        TermPattern::Literal(l) => Some(Term::Literal(l.clone())),
        // RDF 1.2 triple term in an INSERT template: substitute the components recursively.
        TermPattern::Triple(t) => {
            let subject = match tp_subst(&t.subject, get, fresh)? {
                Term::NamedNode(n) => NamedOrBlankNode::NamedNode(n),
                Term::BlankNode(b) => NamedOrBlankNode::BlankNode(b),
                _ => return None,
            };
            let predicate = match nnp_subst(&t.predicate, get)? {
                Term::NamedNode(n) => n,
                _ => return None,
            };
            let object = tp_subst(&t.object, get, fresh)?;
            Some(Term::Triple(Box::new(oxrdf::Triple::new(subject, predicate, object))))
        }
    }
}

fn gtp_subst(tp: &GroundTermPattern, get: &Subst) -> Option<Term> {
    match tp {
        GroundTermPattern::Variable(v) => get(v),
        GroundTermPattern::NamedNode(n) => Some(Term::NamedNode(n.clone())),
        GroundTermPattern::Literal(l) => Some(Term::Literal(l.clone())),
        GroundTermPattern::Triple(t) => {
            let subject = match gtp_subst(&t.subject, get)? {
                Term::NamedNode(n) => NamedOrBlankNode::NamedNode(n),
                Term::BlankNode(b) => NamedOrBlankNode::BlankNode(b),
                _ => return None,
            };
            let predicate = match nnp_subst(&t.predicate, get)? {
                Term::NamedNode(n) => n,
                _ => return None,
            };
            let object = gtp_subst(&t.object, get)?;
            Some(Term::Triple(Box::new(oxrdf::Triple::new(subject, predicate, object))))
        }
    }
}

fn nnp_subst(p: &NamedNodePattern, get: &Subst) -> Option<Term> {
    match p {
        NamedNodePattern::NamedNode(n) => Some(Term::NamedNode(n.clone())),
        NamedNodePattern::Variable(v) => get(v),
    }
}

/// Resolve a template's graph slot. Outer `None` = skip the quad (unbound variable or a
/// literal where a graph name is required); `Some(None)` = the default graph.
fn gnp_subst(g: &GraphNamePattern, get: &Subst) -> Option<GraphSlot> {
    match g {
        GraphNamePattern::DefaultGraph => Some(None),
        GraphNamePattern::NamedNode(n) => Some(Some(Term::NamedNode(n.clone()))),
        GraphNamePattern::Variable(v) => match get(v)? {
            t @ (Term::NamedNode(_) | Term::BlankNode(_)) => Some(Some(t)),
            _ => None, // a literal / triple term cannot name a graph
        },
    }
}

/// Evaluates a DELETE/INSERT … WHERE pattern against `active` and instantiates the delete /
/// insert templates per solution as (graph-slot, triple) pairs. Shared by the rebuild
/// (`update`) and delta-overlay (`update_in_place`) paths.
fn instantiate_templates(
    active: &Graph,
    delete: &[GroundQuadPattern],
    insert: &[QuadPattern],
    pattern: &spargebra::algebra::GraphPattern,
) -> Result<(Vec<(GraphSlot, TripleTerms)>, Vec<(GraphSlot, TripleTerms)>), String> {
    let result = crate::exec::eval_select(active, pattern)?;
    let cols: FxHashMap<Variable, usize> =
        result.vars.iter().enumerate().map(|(i, v)| (v.clone(), i)).collect();
    let mut dels: Vec<(GraphSlot, TripleTerms)> = Vec::new();
    let mut inss: Vec<(GraphSlot, TripleTerms)> = Vec::new();
    for row in &result.rows {
        let get = |v: &Variable| -> Option<Term> { cols.get(v).and_then(|&i| row[i].clone()) };
        for dp in delete {
            let (Some(slot), Some(s), Some(p), Some(o)) = (
                gnp_subst(&dp.graph_name, &get),
                gtp_subst(&dp.subject, &get),
                nnp_subst(&dp.predicate, &get),
                gtp_subst(&dp.object, &get),
            ) else {
                continue;
            };
            dels.push((slot, [s, p, o]));
        }
        let mut fresh = FreshBnodes { map: FxHashMap::default() };
        for ip in insert {
            let (Some(slot), Some(s), Some(p), Some(o)) = (
                gnp_subst(&ip.graph_name, &get),
                tp_subst(&ip.subject, &get, &mut fresh),
                nnp_subst(&ip.predicate, &get),
                tp_subst(&ip.object, &get, &mut fresh),
            ) else {
                continue;
            };
            inss.push((slot, [s, p, o]));
        }
    }
    Ok((dels, inss))
}

// --- LOAD ----------------------------------------------------------------------------------------

/// LOAD `file://` policy (roborev 1646, High). [OPUS-4.8] `LOAD <file://…>` dereferences a
/// server-LOCAL file, so on the public update path (HTTP `apply_update`) an untrusted client
/// could otherwise import any RDF-readable file the process can read and query it back. The
/// DEFAULT policy is therefore REJECT: `file://` LOAD only works when a caller has explicitly
/// installed an allowlisted base directory via [`with_load_base`] (the conformance runner does
/// this, pointing at the local test-data tree). The requested path is canonicalised and must
/// resolve UNDER that base, so `../` traversal and symlinks cannot escape it.
pub(crate) mod load_policy {
    use std::cell::RefCell;
    use std::path::PathBuf;

    thread_local! {
        static BASE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    }

    /// Uninstalls the allowlisted base when the installing scope returns (also on
    /// error/unwind, so a poisoned thread never leaks a trusted base).
    pub(crate) struct Guard(Option<PathBuf>);
    impl Drop for Guard {
        fn drop(&mut self) {
            BASE.with(|b| *b.borrow_mut() = self.0.take());
        }
    }

    /// Installs `base` (canonicalised) as the allowlisted LOAD directory for the duration of
    /// the returned guard. Restores the previous base on drop (supports nesting).
    pub(crate) fn install(base: PathBuf) -> Guard {
        let canon = std::fs::canonicalize(&base).unwrap_or(base);
        BASE.with(|b| {
            let prev = b.borrow_mut().replace(canon);
            Guard(prev)
        })
    }

    pub(crate) fn base() -> Option<PathBuf> {
        BASE.with(|b| b.borrow().clone())
    }
}

/// Runs `f` with `base` allowlisted as the root for `LOAD <file://…>` (roborev 1646). Without
/// this, every `file://` LOAD is REJECTED — the secure default for public update paths. Trusted
/// local callers (the SPARQL conformance runner) wrap their `update` call in this to permit
/// loading the test-data files that live under `base`.
pub fn with_load_base<R>(base: impl Into<std::path::PathBuf>, f: impl FnOnce() -> R) -> R {
    let _guard = load_policy::install(base.into());
    f()
}

/// Reads and parses a `file://` document for LOAD. `file://` is REJECTED unless an allowlisted
/// base directory is installed (see [`with_load_base`]); when one is, the resolved path is
/// canonicalised and must lie UNDER it. Non-file schemes are an error — `LOAD SILENT` turns any
/// of these into a no-op at the call sites.
fn load_document(source: &str) -> Result<TripleSet, String> {
    let raw = source
        .strip_prefix("file://")
        .ok_or_else(|| format!("LOAD source not supported (only file:// URIs): {source}"))?;
    // `file:///abs` -> "/abs"; keep the literal-strip behaviour for the relative-path form the
    // conformance suites use, but require an installed allowlist either way.
    let Some(base) = load_policy::base() else {
        return Err(format!(
            "LOAD of {source} refused: file:// access is disabled (no allowlisted base directory configured)"
        ));
    };
    let requested = std::path::Path::new(raw);
    // Resolve relative paths against the allowlisted base, then canonicalise to collapse `..`
    // and follow symlinks so the containment check cannot be bypassed.
    let joined = if requested.is_absolute() { requested.to_path_buf() } else { base.join(requested) };
    let path = std::fs::canonicalize(&joined).map_err(|e| format!("LOAD {source}: {e}"))?;
    if !path.starts_with(&base) {
        return Err(format!("LOAD of {source} refused: resolves outside the allowlisted base directory"));
    }
    let text = std::fs::read_to_string(&path).map_err(|e| format!("LOAD {source}: {e}"))?;
    let format = match path.extension().and_then(|e| e.to_str()) {
        Some("nt") => "ntriples",
        Some("nq") => "nquads",
        Some("trig") => "trig",
        _ => "turtle",
    };
    let g = Graph::load_str(&text, format).map_err(|e| format!("LOAD {source}: {e}"))?;
    Ok(decode_triples(&g))
}

// --- the rebuild path ----------------------------------------------------------------------------

/// Applies one parsed operation to the dataset model.
fn apply_op(ds: &mut Dataset, op: &GraphUpdateOperation) -> Result<(), String> {
    match op {
        GraphUpdateOperation::InsertData { data } => {
            // Fresh blank nodes for the whole operation (roborev 1646): same label in this
            // INSERT DATA -> same node; never the existing store's `_:b`.
            let mut fresh = FreshBnodes { map: FxHashMap::default() };
            for q in data {
                let (slot, t) = quad_to_triple_fresh(q, &mut fresh);
                ds.insert(&slot, t);
            }
        }
        GraphUpdateOperation::DeleteData { data } => {
            for q in data {
                let (slot, t) = ground_quad_to_triple(q);
                ds.remove(&slot, &t);
            }
        }
        // CLEAR empties the target graph(s) but keeps named-graph entries; clearing an
        // absent named graph is a no-op (auto-create stores may succeed — we do).
        GraphUpdateOperation::Clear { graph: target, .. } => match target {
            GraphTarget::DefaultGraph => ds.default.clear(),
            GraphTarget::NamedNode(n) => {
                let name = Term::NamedNode(n.clone());
                if let Some(i) = ds.named.iter().position(|(g, _)| *g == name) {
                    ds.named[i].1.clear();
                }
            }
            GraphTarget::NamedGraphs => ds.named.iter_mut().for_each(|(_, s)| s.clear()),
            GraphTarget::AllGraphs => {
                ds.default.clear();
                ds.named.iter_mut().for_each(|(_, s)| s.clear());
            }
        },
        // DROP removes named-graph entries; the default graph always exists, so DROP
        // DEFAULT only empties it.
        GraphUpdateOperation::Drop { graph: target, .. } => match target {
            GraphTarget::DefaultGraph => ds.default.clear(),
            GraphTarget::NamedNode(n) => {
                let name = Term::NamedNode(n.clone());
                ds.named.retain(|(g, _)| *g != name);
            }
            GraphTarget::NamedGraphs => ds.named.clear(),
            GraphTarget::AllGraphs => {
                ds.default.clear();
                ds.named.clear();
            }
        },
        GraphUpdateOperation::Create { graph, .. } => {
            ds.graph_mut(&Term::NamedNode(graph.clone()));
        }
        GraphUpdateOperation::Load { silent, source, destination } => {
            match load_document(source.as_str()) {
                Ok(triples) => ds.slot_mut(&graph_name_slot(destination)).extend(triples),
                Err(e) => {
                    if !silent {
                        return Err(e);
                    }
                }
            }
        }
        // DELETE { d } INSERT { i } WHERE { p } — evaluate the WHERE against the dataset
        // state so far (re-scoped by USING when present), instantiate the templates per
        // solution, then apply all deletes and then all inserts (SPARQL semantics).
        GraphUpdateOperation::DeleteInsert { delete, insert, using, pattern } => {
            let active = match using {
                Some(u) => ds.build_using(u),
                None => ds.build(),
            };
            let (dels, inss) = instantiate_templates(&active, delete, insert, pattern)?;
            for (slot, t) in &dels {
                ds.remove(slot, t);
            }
            for (slot, t) in inss {
                ds.insert(&slot, t);
            }
        }
    }
    Ok(())
}

/// Apply a SPARQL Update string to `graph`, returning the updated graph (named graphs
/// preserved). Errors (leaving the input untouched — it is borrowed) on a parse error or a
/// non-SILENT failing LOAD.
pub fn update(graph: &Graph, sparql: &str) -> Result<Graph, String> {
    let upd = SparqlParser::new().parse_update(sparql).map_err(|e| e.to_string())?;
    let mut ds = Dataset::decode(graph);
    for op in &upd.operations {
        apply_op(&mut ds, op)?;
    }
    Ok(ds.build())
}

// --- the delta-overlay path ------------------------------------------------------------------

/// Groups (graph-slot, triple) pairs per graph slot, preserving first-seen slot order.
fn group_by_slot(items: Vec<(GraphSlot, TripleTerms)>) -> Vec<(GraphSlot, Vec<TripleTerms>)> {
    let mut out: Vec<(GraphSlot, Vec<TripleTerms>)> = Vec::new();
    for (slot, t) in items {
        match out.iter_mut().find(|(s, _)| *s == slot) {
            Some((_, v)) => v.push(t),
            None => out.push((slot, vec![t])),
        }
    }
    out
}

/// Applies one insert/delete batch to the graph slot through the delta overlay,
/// auto-creating an empty named graph for an insert into an absent one.
fn apply_slot_delta(
    graph: &mut Graph,
    slot: &GraphSlot,
    inserts: &[TripleTerms],
    deletes: &[TripleTerms],
) -> Result<(), String> {
    match slot {
        None => graph.apply_delta(inserts, deletes),
        Some(name) => {
            if let Some(i) = graph.named.iter().position(|(n, _)| n == name) {
                return graph.named[i].1.apply_delta(inserts, deletes);
            }
            if inserts.is_empty() {
                return Ok(()); // deleting from an absent graph is a no-op
            }
            let mut g = empty_graph();
            g.apply_delta(inserts, &[])?;
            graph.named.push((name.clone(), g));
            Ok(())
        }
    }
}

/// Applies a SPARQL Update IN PLACE through the store's DELTA-OVERLAY (roadmap T17): the data
/// operations route to [`Graph::apply_delta`] per target graph — O(batch) per operation instead
/// of the O(n) decode-everything-and-rebuild that [`update`] performs — and, for a
/// directory-backed graph, each default-graph batch is WAL-logged (fsync'd) before it is
/// applied, so updates are durable across a crash. `CLEAR`/`DROP` of the default graph still
/// REPLACE it with an empty rebuild (this drops a directory-backed graph's WAL/directory
/// association); named graphs are preserved across every operation. Fold the accumulated
/// overlay back into the base periodically with [`Graph::compact`].
pub fn update_in_place(graph: &mut Graph, sparql: &str) -> Result<(), String> {
    let upd = SparqlParser::new().parse_update(sparql).map_err(|e| e.to_string())?;
    for op in &upd.operations {
        match op {
            GraphUpdateOperation::InsertData { data } => {
                // Fresh blank nodes for the whole operation (roborev 1646) — see apply_op.
                let mut fresh = FreshBnodes { map: FxHashMap::default() };
                let triples: Vec<_> = data.iter().map(|q| quad_to_triple_fresh(q, &mut fresh)).collect();
                for (slot, ins) in group_by_slot(triples) {
                    apply_slot_delta(graph, &slot, &ins, &[])?;
                }
            }
            GraphUpdateOperation::DeleteData { data } => {
                for (slot, del) in group_by_slot(data.iter().map(ground_quad_to_triple).collect()) {
                    apply_slot_delta(graph, &slot, &[], &del)?;
                }
            }
            GraphUpdateOperation::Clear { graph: target, .. } => match target {
                GraphTarget::DefaultGraph => replace_default(graph)?,
                GraphTarget::NamedNode(n) => {
                    let name = Term::NamedNode(n.clone());
                    if let Some(i) = graph.named.iter().position(|(g, _)| *g == name) {
                        graph.named[i].1 = empty_graph();
                    }
                }
                GraphTarget::NamedGraphs => {
                    for entry in &mut graph.named {
                        entry.1 = empty_graph();
                    }
                }
                GraphTarget::AllGraphs => {
                    replace_default(graph)?;
                    for entry in &mut graph.named {
                        entry.1 = empty_graph();
                    }
                }
            },
            GraphUpdateOperation::Drop { graph: target, .. } => match target {
                GraphTarget::DefaultGraph => replace_default(graph)?,
                GraphTarget::NamedNode(n) => {
                    let name = Term::NamedNode(n.clone());
                    graph.named.retain(|(g, _)| *g != name);
                }
                GraphTarget::NamedGraphs => graph.named.clear(),
                GraphTarget::AllGraphs => {
                    replace_default(graph)?;
                    graph.named.clear();
                }
            },
            GraphUpdateOperation::Create { graph: name, .. } => {
                let name = Term::NamedNode(name.clone());
                if !graph.named.iter().any(|(g, _)| *g == name) {
                    graph.named.push((name, empty_graph()));
                }
            }
            GraphUpdateOperation::Load { silent, source, destination } => {
                match load_document(source.as_str()) {
                    Ok(triples) => {
                        let ins: Vec<TripleTerms> = triples.into_iter().collect();
                        apply_slot_delta(graph, &graph_name_slot(destination), &ins, &[])?;
                    }
                    Err(e) => {
                        if !silent {
                            return Err(e);
                        }
                    }
                }
            }
            GraphUpdateOperation::DeleteInsert { delete, insert, using, pattern } => {
                // Without USING the WHERE pattern is evaluated against the graph as updated
                // so far in this request (scans merge the overlay; GRAPH blocks see the live
                // named graphs), matching the rebuild path's semantics. With USING, the
                // re-scoped active dataset is materialised from the current state.
                let (dels, inss) = match using {
                    Some(u) => {
                        let active = Dataset::decode(graph).build_using(u);
                        instantiate_templates(&active, delete, insert, pattern)?
                    }
                    None => instantiate_templates(graph, delete, insert, pattern)?,
                };
                // All deletes first, then all inserts (SPARQL semantics), per graph slot.
                for (slot, del) in group_by_slot(dels) {
                    apply_slot_delta(graph, &slot, &[], &del)?;
                }
                for (slot, ins) in group_by_slot(inss) {
                    apply_slot_delta(graph, &slot, &ins, &[])?;
                }
            }
        }
    }
    Ok(())
}

/// [OPUS-4.8] (review 1593) DURABLY empties the default graph, preserving the named graphs
/// AND (crucially) a directory-backed graph's WAL/directory association — so a CLEAR/DROP of
/// the default graph is persistent across a reopen, instead of being silently lost (the old
/// `*graph = empty_graph()` dropped the WAL, and the on-disk base was untouched, so a reopen
/// restored the cleared data). Falls back to an in-place store clear for in-memory graphs.
fn replace_default(graph: &mut Graph) -> Result<(), String> {
    graph.clear_default_durable()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_core::store::Pattern as IdPattern;

    fn count(g: &Graph) -> usize {
        let pat: IdPattern = [None, None, None];
        g.store.scan(&pat).rows.len()
    }

    #[test]
    fn insert_delete_clear() {
        let g = Graph::load_str("@prefix : <http://ex/> . :a :p :b . :b :p :c .", "turtle").unwrap();
        assert_eq!(count(&g), 2);
        // INSERT DATA adds; set semantics (a re-insert is a no-op).
        let g = update(&g, "PREFIX : <http://ex/> INSERT DATA { :c :p :d . :a :q :x }").unwrap();
        assert_eq!(count(&g), 4);
        let g = update(&g, "PREFIX : <http://ex/> INSERT DATA { :a :p :b }").unwrap();
        assert_eq!(count(&g), 4);
        // DELETE DATA removes a present triple; deleting an absent one is a no-op.
        let g = update(&g, "PREFIX : <http://ex/> DELETE DATA { :a :p :b }").unwrap();
        assert_eq!(count(&g), 3);
        let g = update(&g, "PREFIX : <http://ex/> DELETE DATA { :z :z :z }").unwrap();
        assert_eq!(count(&g), 3);
        // The graph is still queryable after a rebuild.
        assert_eq!(crate::count(&g, "PREFIX : <http://ex/> SELECT * WHERE { :c :p ?o }").unwrap(), 1);
        // CLEAR empties the default graph.
        let g = update(&g, "CLEAR ALL").unwrap();
        assert_eq!(count(&g), 0);
    }

    #[test]
    fn delete_insert_where() {
        let g = Graph::load_str("@prefix : <http://ex/> . :a :age 30 . :b :age 25 .", "turtle").unwrap();
        // Rename predicate :age -> :years for every match.
        let g = update(
            &g,
            "PREFIX : <http://ex/> DELETE { ?s :age ?a } INSERT { ?s :years ?a } WHERE { ?s :age ?a }",
        )
        .unwrap();
        assert_eq!(count(&g), 2);
        assert_eq!(crate::count(&g, "PREFIX : <http://ex/> SELECT * WHERE { ?s :age ?a }").unwrap(), 0);
        assert_eq!(crate::count(&g, "PREFIX : <http://ex/> SELECT * WHERE { ?s :years ?a }").unwrap(), 2);
        // DELETE WHERE shorthand removes all matches.
        let g = update(&g, "PREFIX : <http://ex/> DELETE WHERE { ?s :years ?a }").unwrap();
        assert_eq!(count(&g), 0);
    }

    /// F19 regression: updates must PRESERVE named graphs, route GRAPH-scoped data ops,
    /// and implement the graph-management operations.
    #[test]
    fn named_graph_updates() {
        let src = "<http://ex/a> <http://ex/p> <http://ex/b> .\n\
                   <http://ex/x> <http://ex/q> <http://ex/y> <http://ex/g1> .";
        let g = Graph::load_dataset(src, "nquads").unwrap();
        assert_eq!((count(&g), g.named.len()), (1, 1));

        // A default-graph INSERT DATA must keep :g1 intact.
        let g = update(&g, "PREFIX : <http://ex/> INSERT DATA { :a :p :c }").unwrap();
        assert_eq!((count(&g), g.named.len()), (2, 1));
        assert_eq!(count(&g.named[0].1), 1);

        // GRAPH-scoped INSERT DATA goes to the right graph (auto-creating :g2).
        let g = update(&g, "PREFIX : <http://ex/> INSERT DATA { GRAPH :g1 { :x :q :z } GRAPH :g2 { :n :m :o } }").unwrap();
        assert_eq!(g.named.len(), 2);
        assert_eq!(count(&g.named[0].1), 2);

        // GRAPH-scoped DELETE DATA.
        let g = update(&g, "PREFIX : <http://ex/> DELETE DATA { GRAPH :g1 { :x :q :y } }").unwrap();
        assert_eq!(count(&g.named[0].1), 1);

        // DELETE/INSERT WHERE with GRAPH templates + GRAPH WHERE (the ADD desugaring shape).
        let g = update(
            &g,
            "PREFIX : <http://ex/> INSERT { GRAPH :g3 { ?s ?p ?o } } WHERE { GRAPH :g2 { ?s ?p ?o } }",
        )
        .unwrap();
        let g3 = g.named.iter().find(|(n, _)| n.to_string().contains("g3")).unwrap();
        assert_eq!(count(&g3.1), 1);

        // CLEAR GRAPH empties only that graph; DROP removes it.
        let g = update(&g, "PREFIX : <http://ex/> CLEAR GRAPH :g2").unwrap();
        assert_eq!(g.named.len(), 3); // entry kept, empty
        let g2 = g.named.iter().find(|(n, _)| n.to_string().contains("g2")).unwrap();
        assert_eq!(count(&g2.1), 0);
        let g = update(&g, "PREFIX : <http://ex/> DROP GRAPH :g3").unwrap();
        assert_eq!(g.named.len(), 2);

        // CLEAR DEFAULT keeps named graphs.
        let g = update(&g, "CLEAR DEFAULT").unwrap();
        assert_eq!(count(&g), 0);
        assert_eq!(g.named.len(), 2);
        assert_eq!(count(&g.named[0].1), 1);

        // CREATE makes an (empty) graph that GRAPH <g> {} can see.
        let g = update(&g, "PREFIX : <http://ex/> CREATE GRAPH :fresh").unwrap();
        assert!(g.named.iter().any(|(n, _)| n.to_string().contains("fresh")));

        // DROP ALL empties everything.
        let g = update(&g, "DROP ALL").unwrap();
        assert_eq!((count(&g), g.named.len()), (0, 0));
    }

    /// ADD / COPY / MOVE arrive desugared from the parser; end-to-end they must move the
    /// triples and preserve everything else.
    #[test]
    fn add_copy_move() {
        let src = "<http://ex/d> <http://ex/p> <http://ex/e> .\n\
                   <http://ex/a> <http://ex/p> <http://ex/b> <http://ex/g1> .";
        let g = Graph::load_dataset(src, "nquads").unwrap();
        // ADD the default graph into :g1 (union).
        let g = update(&g, "ADD DEFAULT TO GRAPH <http://ex/g1>").unwrap();
        assert_eq!(count(&g), 1); // source kept
        assert_eq!(count(&g.named[0].1), 2);
        // COPY replaces the destination.
        let g = update(&g, "COPY DEFAULT TO GRAPH <http://ex/g1>").unwrap();
        assert_eq!(count(&g.named.iter().find(|(n, _)| n.to_string().contains("g1")).unwrap().1), 1);
        // MOVE drops the source.
        let g = update(&g, "MOVE GRAPH <http://ex/g1> TO GRAPH <http://ex/g2>").unwrap();
        assert!(!g.named.iter().any(|(n, _)| n.to_string().contains("g1")));
        assert_eq!(count(&g.named.iter().find(|(n, _)| n.to_string().contains("g2")).unwrap().1), 1);
    }

    /// USING re-scopes the WHERE dataset: the named graph becomes the active default graph.
    #[test]
    fn using_rescopes_where() {
        let src = "<http://ex/d> <http://ex/p> <http://ex/e> .\n\
                   <http://ex/a> <http://ex/p> <http://ex/b> <http://ex/g1> .";
        let g = Graph::load_dataset(src, "nquads").unwrap();
        let g = update(
            &g,
            "PREFIX : <http://ex/> INSERT { ?s :copied ?o } USING :g1 WHERE { ?s :p ?o }",
        )
        .unwrap();
        // Only :g1's triple matched (the real default graph was re-scoped away).
        assert_eq!(crate::count(&g, "PREFIX : <http://ex/> SELECT * WHERE { ?s :copied ?o }").unwrap(), 1);
        assert_eq!(crate::count(&g, "PREFIX : <http://ex/> SELECT * WHERE { :a :copied :b }").unwrap(), 1);
    }

    /// Blank nodes in an INSERT template are FRESH per solution.
    #[test]
    fn insert_template_bnodes_fresh_per_solution() {
        let g = Graph::load_str("@prefix : <http://ex/> . :a :age 30 . :b :age 25 .", "turtle").unwrap();
        let g = update(
            &g,
            "PREFIX : <http://ex/> INSERT { ?s :note _:n . _:n :label \"x\" } WHERE { ?s :age ?a }",
        )
        .unwrap();
        // 2 solutions × 2 template triples = 4 inserted (distinct bnodes per solution).
        assert_eq!(count(&g), 2 + 4);
        // Two DISTINCT bnodes -> two :label triples.
        assert_eq!(crate::count(&g, "PREFIX : <http://ex/> SELECT * WHERE { ?n :label ?l }").unwrap(), 2);
    }

    /// The delta-overlay path (`update_in_place`) must be observationally identical to the
    /// rebuild path (`update`) across a sequence of every supported operation — and the
    /// overlay-carrying graph must answer queries identically without compaction.
    #[test]
    fn update_in_place_matches_rebuild() {
        let src = "@prefix : <http://ex/> . :a :p :b . :b :p :c . :a :age 30 . :b :age 25 .";
        let mut inplace = Graph::load_str(src, "turtle").unwrap();
        let mut rebuilt = Graph::load_str(src, "turtle").unwrap();
        let ops = [
            "PREFIX : <http://ex/> INSERT DATA { :c :p :d . :a :q 42 . :a :q 9.5 }",
            "PREFIX : <http://ex/> DELETE DATA { :a :p :b . :z :z :z }",
            "PREFIX : <http://ex/> INSERT DATA { :a :p :b }", // re-insert of a delete
            "PREFIX : <http://ex/> DELETE { ?s :age ?a } INSERT { ?s :years ?a } WHERE { ?s :age ?a }",
            "PREFIX : <http://ex/> DELETE WHERE { ?s :q ?v }",
            // Named-graph operations (F19): both paths must route + preserve identically.
            "PREFIX : <http://ex/> INSERT DATA { GRAPH :g1 { :n :m :o . :n :m :p } }",
            "PREFIX : <http://ex/> DELETE DATA { GRAPH :g1 { :n :m :p } }",
            "PREFIX : <http://ex/> INSERT { GRAPH :g2 { ?s ?p ?o } } WHERE { GRAPH :g1 { ?s ?p ?o } }",
            "PREFIX : <http://ex/> CLEAR GRAPH :g1",
            "PREFIX : <http://ex/> DROP GRAPH :g2",
        ];
        let dump = |g: &Graph| -> Vec<(String, String, String, String)> {
            let mut v: Vec<_> = Vec::new();
            let mut one = |g: &Graph, name: &str| {
                let scan = g.store.scan(&[None, None, None]);
                for r in scan.rows.iter() {
                    let t = scan.to_spo(r);
                    v.push((
                        g.dict.term(t[0]).to_string(),
                        g.dict.term(t[1]).to_string(),
                        g.dict.term(t[2]).to_string(),
                        name.to_string(),
                    ));
                }
            };
            one(g, "");
            for (name, sub) in &g.named {
                one(sub, &name.to_string());
            }
            v.sort();
            v
        };
        for (i, op) in ops.iter().enumerate() {
            update_in_place(&mut inplace, op).unwrap();
            rebuilt = update(&rebuilt, op).unwrap();
            assert_eq!(dump(&inplace), dump(&rebuilt), "states diverged after op {i}");
            // The overlay graph must also answer pattern queries identically (un-compacted).
            for q in [
                "PREFIX : <http://ex/> SELECT * WHERE { ?s :p ?o }",
                "PREFIX : <http://ex/> SELECT * WHERE { ?s ?p ?o . FILTER(?o > 20) }",
                "PREFIX : <http://ex/> SELECT * WHERE { ?s :p ?m . ?m :p ?o }",
                "PREFIX : <http://ex/> SELECT * WHERE { GRAPH ?g { ?s ?p ?o } }",
            ] {
                assert_eq!(crate::count(&inplace, q).unwrap(), crate::count(&rebuilt, q).unwrap(), "query {q} diverged after op {i}");
            }
        }
        assert!(inplace.store.has_overlay(), "the in-place path must have gone through the overlay");
        // CLEAR still empties via replacement.
        update_in_place(&mut inplace, "CLEAR ALL").unwrap();
        assert_eq!(dump(&inplace).len(), 0);
        // And compaction folds without changing anything.
        let mut g = Graph::load_str(src, "turtle").unwrap();
        update_in_place(&mut g, ops[0]).unwrap();
        let before = dump(&g);
        g.compact().unwrap();
        assert!(!g.store.has_overlay());
        assert_eq!(dump(&g), before);
    }

    /// HEADLINE BENCHMARK (T17): per-update latency of a 10-triple INSERT DATA into a
    /// 10M-triple graph — the old decode-everything-and-rebuild path vs the new delta-
    /// overlay path. Run with:
    ///   cargo test -p sparq-engine --release -- --ignored bench_update_latency --nocapture
    /// Override the size with SPARQ_BENCH_TRIPLES.
    #[test]
    #[ignore = "benchmark — run explicitly in --release"]
    fn bench_update_latency() {
        use sparq_core::dict::{Dict as D, Id};
        use std::time::Instant;
        let n: usize =
            std::env::var("SPARQ_BENCH_TRIPLES").ok().and_then(|v| v.parse().ok()).unwrap_or(10_000_000);
        let nsubj = (n / 10).max(1);
        let t0 = Instant::now();
        let mut dict = D::new();
        let preds: Vec<Id> = (0..10).map(|p| dict.intern_iri(&format!("http://ex/p{p}"))).collect();
        let subjs: Vec<Id> = (0..nsubj).map(|s| dict.intern_iri(&format!("http://ex/s{s}"))).collect();
        // NON-LINEAR mix (murmur3 finalizer) for the object index, so the triple set has
        // ~n DISTINCT triples — any linear pattern is periodic mod nsubj and collapses
        // under dedup.
        let mix = |i: usize| -> usize {
            let mut h = i as u64;
            h ^= h >> 33;
            h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
            h ^= h >> 33;
            (h % nsubj as u64) as usize
        };
        let triples: Vec<[Id; 3]> =
            (0..n).map(|i| [subjs[i % nsubj], preds[i % 10], subjs[mix(i)]]).collect();
        let g = Graph::from_parts(dict, triples);
        eprintln!("graph: {} triples, built in {:.2?}", g.len(), t0.elapsed());

        let ins = "PREFIX : <http://ex/> INSERT DATA { :n0 :q :m0 . :n1 :q :m1 . :n2 :q :m2 . \
                   :n3 :q :m3 . :n4 :q :m4 . :n5 :q :m5 . :n6 :q :m6 . :n7 :q :m7 . :n8 :q :m8 . :n9 :q :m9 }";

        // OLD: full rebuild.
        let t = Instant::now();
        let g_old = update(&g, ins).unwrap();
        let old = t.elapsed();
        eprintln!("old (rebuild) 10-triple INSERT DATA: {:.2?}  -> {} triples", old, g_old.len());
        drop(g_old);

        // NEW: delta-overlay, measured over several batches for a stable per-update figure.
        let mut g = g;
        let t = Instant::now();
        update_in_place(&mut g, ins).unwrap();
        let new_first = t.elapsed();
        let t = Instant::now();
        let reps = 100;
        for i in 0..reps {
            let upd = format!("PREFIX : <http://ex/> INSERT DATA {{ {} }}",
                (0..10).map(|j| format!(":x{i}_{j} :q :y{i}_{j} .")).collect::<Vec<_>>().join(" "));
            update_in_place(&mut g, &upd).unwrap();
        }
        let new_avg = t.elapsed() / reps;
        eprintln!(
            "new (overlay) 10-triple INSERT DATA: first {:.2?}, avg over {reps} batches {:.2?}  -> {} triples",
            new_first, new_avg, g.len()
        );
        eprintln!("speedup: {:.0}x (rebuild {:.3}s vs overlay {:.6}s)",
            old.as_secs_f64() / new_avg.as_secs_f64(), old.as_secs_f64(), new_avg.as_secs_f64());
    }

    /// [OPUS-4.8] roborev 1646 (High): `LOAD <file://…>` must be REFUSED by default — an
    /// untrusted update on the public path cannot import server-local files. It only works
    /// inside an allowlisted base, and even then cannot escape it via `..`.
    #[test]
    fn load_file_uri_refused_by_default_and_sandboxed() {
        let dir = std::env::temp_dir().join(format!("sparq_load_1646_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let inside = dir.join("data.ttl");
        std::fs::write(&inside, "@prefix : <http://ex/> . :a :p :b .").unwrap();
        // A secret file OUTSIDE the allowlisted base (one level up).
        let secret = dir.parent().unwrap().join(format!("sparq_secret_1646_{}.ttl", std::process::id()));
        std::fs::write(&secret, "@prefix : <http://ex/> . :secret :leaked :value .").unwrap();

        let g = empty_graph();
        let load_inside = format!("LOAD <file://{}>", inside.display());
        // `Result<Graph, String>::unwrap_err` would need Graph: Debug — extract the error string.
        let err = |r: Result<Graph, String>| -> String { r.err().expect("expected a LOAD error") };

        // (1) DEFAULT: no allowlist installed -> refused (the file is never read).
        let e = err(update(&g, &load_inside));
        assert!(e.contains("file:// access is disabled"), "default LOAD must be refused, got: {e}");
        // SILENT swallows the refusal into a no-op (still loads nothing).
        let g_silent = update(&g, &format!("LOAD SILENT <file://{}>", inside.display())).unwrap();
        assert_eq!(count(&g_silent), 0);

        // (2) Allowlisted base: a file UNDER the base loads.
        let loaded = with_load_base(dir.clone(), || update(&g, &load_inside)).unwrap();
        assert_eq!(count(&loaded), 1, "LOAD under the allowlisted base must succeed");

        // (3) A path that escapes the base (absolute, outside) is refused even with a base set.
        let load_secret = format!("LOAD <file://{}>", secret.display());
        let e = err(with_load_base(dir.clone(), || update(&g, &load_secret)));
        assert!(e.contains("outside the allowlisted base"), "escape must be refused, got: {e}");
        // …and a `..` traversal to the same secret is likewise refused.
        let rel = format!("LOAD <file://../{}>", secret.file_name().unwrap().to_str().unwrap());
        let e = err(with_load_base(dir.clone(), || update(&g, &rel)));
        assert!(e.contains("outside the allowlisted base") || e.contains("LOAD"), "traversal must be refused, got: {e}");

        // (4) The base does not leak past the scope: a subsequent LOAD is refused again.
        let e = err(update(&g, &load_inside));
        assert!(e.contains("file:// access is disabled"), "base must not leak, got: {e}");

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_file(&secret);
    }

    /// [OPUS-4.8] roborev 1646 (Med): blank nodes in INSERT DATA are FRESH per SPARQL — they
    /// must not conflate with an existing store blank node of the same label. The rebuild and
    /// delta-overlay paths must both freshen.
    #[test]
    fn insert_data_blank_nodes_are_fresh() {
        // Graph already holds a blank node labelled `b` (subject of one triple).
        let src = "@prefix : <http://ex/> . _:b :p :existing .";
        let ins = "PREFIX : <http://ex/> INSERT DATA { _:b :p :inserted }";

        // Rebuild path: the inserted `_:b` must be a DIFFERENT node, so :p has two distinct
        // blank-node subjects and the two objects stay on separate subjects.
        let g = Graph::load_str(src, "turtle").unwrap();
        let g = update(&g, ins).unwrap();
        assert_eq!(count(&g), 2, "both triples present");
        let subjects = crate::count(&g, "SELECT DISTINCT ?s WHERE { ?s <http://ex/p> ?o }").unwrap();
        assert_eq!(subjects, 2, "INSERT DATA bnode must be fresh (distinct from existing _:b)");

        // Delta-overlay path: same freshness guarantee.
        let mut g2 = Graph::load_str(src, "turtle").unwrap();
        update_in_place(&mut g2, ins).unwrap();
        assert_eq!(count(&g2), 2);
        let subjects2 = crate::count(&g2, "SELECT DISTINCT ?s WHERE { ?s <http://ex/p> ?o }").unwrap();
        assert_eq!(subjects2, 2, "in-place INSERT DATA bnode must be fresh too");

        // Within ONE INSERT DATA op, the same label is the SAME fresh node.
        let g3 = Graph::load_str("@prefix : <http://ex/> . :x :p :y .", "turtle").unwrap();
        let g3 = update(&g3, "PREFIX : <http://ex/> INSERT DATA { _:s :a :o . _:s :b :o2 }").unwrap();
        let one_subj = crate::count(&g3, "SELECT DISTINCT ?s WHERE { ?s ?p ?o . FILTER(isBlank(?s)) }").unwrap();
        assert_eq!(one_subj, 1, "same label in one INSERT DATA op is one node");
    }
}
