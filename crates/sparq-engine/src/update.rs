//! SPARQL 1.1 Update (roadmap T10), v1: the data operations `INSERT DATA`, `DELETE DATA`, and
//! `CLEAR` on the default graph, applied by **rebuild** — collect the current triple set, apply the
//! mutations as set operations, and rebuild the (immutable) store. Correct and simple; O(n) per
//! update batch. Pattern-based `DELETE/INSERT ... WHERE`, `LOAD`, and named-graph operations are
//! follow-ups (they need query evaluation and/or a quad store).

use oxrdf::{NamedOrBlankNode, Term, Variable};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::Dict;
use sparq_core::store::Pattern as IdPattern;
use sparq_core::Graph;
use spargebra::algebra::GraphTarget;
use spargebra::term::{
    GraphName, GraphNamePattern, GroundQuad, GroundTerm, GroundTermPattern, NamedNodePattern, Quad,
    TermPattern,
};
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

// --- template instantiation for DELETE/INSERT … WHERE ------------------------------------------
// Substitute a template term from a WHERE solution. `None` => the position has an unbound variable,
// so the whole quad is skipped (per SPARQL, a template triple with an unbound slot is not produced).
type Subst<'a> = dyn Fn(&Variable) -> Option<Term> + 'a;

fn tp_subst(tp: &TermPattern, get: &Subst) -> Option<Term> {
    match tp {
        TermPattern::Variable(v) => get(v),
        TermPattern::NamedNode(n) => Some(Term::NamedNode(n.clone())),
        TermPattern::BlankNode(b) => Some(Term::BlankNode(b.clone())),
        TermPattern::Literal(l) => Some(Term::Literal(l.clone())),
        _ => None, // RDF-star triple terms not supported
    }
}
fn gtp_subst(tp: &GroundTermPattern, get: &Subst) -> Option<Term> {
    match tp {
        GroundTermPattern::Variable(v) => get(v),
        GroundTermPattern::NamedNode(n) => Some(Term::NamedNode(n.clone())),
        GroundTermPattern::Literal(l) => Some(Term::Literal(l.clone())),
        _ => None,
    }
}
fn nnp_subst(p: &NamedNodePattern, get: &Subst) -> Option<Term> {
    match p {
        NamedNodePattern::NamedNode(n) => Some(Term::NamedNode(n.clone())),
        NamedNodePattern::Variable(v) => get(v),
    }
}
fn gnp_default(g: &GraphNamePattern) -> Result<(), String> {
    match g {
        GraphNamePattern::DefaultGraph => Ok(()),
        _ => Err("named graphs in an UPDATE template are not yet supported".into()),
    }
}

/// Apply a SPARQL Update string to `graph`, returning the updated graph. Errors (leaving the input
/// consumed) on operations not yet supported, so the caller can keep the original on failure by
/// cloning beforehand if needed.
pub fn update(graph: &Graph, sparql: &str) -> Result<Graph, String> {
    let upd = SparqlParser::new().parse_update(sparql).map_err(|e| e.to_string())?;
    let mut triples = current_triples(graph);
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
            // DELETE { d } INSERT { i } WHERE { p } — evaluate the WHERE pattern against the graph
            // state so far, instantiate the delete/insert templates per solution, then apply all
            // deletes and then all inserts (SPARQL semantics).
            GraphUpdateOperation::DeleteInsert { delete, insert, using, pattern } => {
                if using.is_some() {
                    return Err("USING in DELETE/INSERT … WHERE is not yet supported".into());
                }
                let current = build(&triples);
                let (dels, inss) = instantiate_templates(&current, delete, insert, pattern)?;
                for t in dels {
                    triples.remove(&t);
                }
                for t in inss {
                    triples.insert(t);
                }
            }
            other => return Err(format!("UPDATE operation not yet supported: {other:?}")),
        }
    }
    Ok(build(&triples))
}

/// Evaluates a DELETE/INSERT … WHERE pattern against `graph` and instantiates the delete /
/// insert templates per solution (skipping any template triple with an unbound slot, per
/// SPARQL). Shared by the rebuild (`update`) and delta-overlay (`update_in_place`) paths.
fn instantiate_templates(
    graph: &Graph,
    delete: &[spargebra::term::GroundQuadPattern],
    insert: &[spargebra::term::QuadPattern],
    pattern: &spargebra::algebra::GraphPattern,
) -> Result<(Vec<TripleTerms>, Vec<TripleTerms>), String> {
    let result = crate::exec::eval_select(graph, pattern)?;
    let cols: FxHashMap<Variable, usize> =
        result.vars.iter().enumerate().map(|(i, v)| (v.clone(), i)).collect();
    let mut dels: Vec<TripleTerms> = Vec::new();
    let mut inss: Vec<TripleTerms> = Vec::new();
    for row in &result.rows {
        let get = |v: &Variable| -> Option<Term> { cols.get(v).and_then(|&i| row[i].clone()) };
        for dp in delete {
            gnp_default(&dp.graph_name)?;
            if let (Some(s), Some(p), Some(o)) =
                (gtp_subst(&dp.subject, &get), nnp_subst(&dp.predicate, &get), gtp_subst(&dp.object, &get))
            {
                dels.push([s, p, o]);
            }
        }
        for ip in insert {
            gnp_default(&ip.graph_name)?;
            if let (Some(s), Some(p), Some(o)) =
                (tp_subst(&ip.subject, &get), nnp_subst(&ip.predicate, &get), tp_subst(&ip.object, &get))
            {
                inss.push([s, p, o]);
            }
        }
    }
    Ok((dels, inss))
}

/// Applies a SPARQL Update IN PLACE through the store's DELTA-OVERLAY (roadmap T17): the data
/// operations route to [`Graph::apply_delta`] — O(batch) per operation instead of the O(n)
/// decode-everything-and-rebuild that [`update`] performs — and, for a directory-backed graph,
/// each batch is WAL-logged (fsync'd) before it is applied, so updates are durable across a
/// crash. `CLEAR` (DEFAULT/ALL) still REPLACES the graph with an empty rebuild (it discards
/// everything anyway; note this drops a directory-backed graph's WAL/directory association).
/// Fold the accumulated overlay back into the base periodically with [`Graph::compact`].
pub fn update_in_place(graph: &mut Graph, sparql: &str) -> Result<(), String> {
    let upd = SparqlParser::new().parse_update(sparql).map_err(|e| e.to_string())?;
    for op in &upd.operations {
        match op {
            GraphUpdateOperation::InsertData { data } => {
                let inss: Vec<TripleTerms> = data.iter().map(quad_to_triple).collect::<Result<_, _>>()?;
                graph.apply_delta(&inss, &[])?;
            }
            GraphUpdateOperation::DeleteData { data } => {
                let dels: Vec<TripleTerms> = data.iter().map(ground_quad_to_triple).collect::<Result<_, _>>()?;
                graph.apply_delta(&[], &dels)?;
            }
            GraphUpdateOperation::Clear { graph: target, .. } => {
                if clears_default(target) {
                    *graph = build(&FxHashSet::default());
                }
            }
            GraphUpdateOperation::DeleteInsert { delete, insert, using, pattern } => {
                if using.is_some() {
                    return Err("USING in DELETE/INSERT … WHERE is not yet supported".into());
                }
                // The WHERE pattern is evaluated against the graph as updated so far in this
                // request (scans merge the overlay), matching the rebuild path's semantics.
                let (dels, inss) = instantiate_templates(graph, delete, insert, pattern)?;
                graph.apply_delta(&inss, &dels)?;
            }
            other => return Err(format!("UPDATE operation not yet supported: {other:?}")),
        }
    }
    Ok(())
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

    #[test]
    fn named_graph_update_errors() {
        let g = Graph::load_str("@prefix : <http://ex/> . :a :p :b .", "turtle").unwrap();
        assert!(update(&g, "PREFIX : <http://ex/> INSERT DATA { GRAPH :g { :a :p :c } }").is_err());
        let mut g = g;
        assert!(update_in_place(&mut g, "PREFIX : <http://ex/> INSERT DATA { GRAPH :g { :a :p :c } }").is_err());
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
        ];
        let dump = |g: &Graph| -> Vec<(String, String, String)> {
            let scan = g.store.scan(&[None, None, None]);
            let mut v: Vec<_> = scan
                .rows
                .iter()
                .map(|r| {
                    let t = scan.to_spo(r);
                    (g.dict.term(t[0]).to_string(), g.dict.term(t[1]).to_string(), g.dict.term(t[2]).to_string())
                })
                .collect();
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
}
