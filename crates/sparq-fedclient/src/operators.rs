//! Physical federation operators (design §4.4) — **Phase 3 (materialised single-source
//! interpreter)**.
//!
//! This module walks a [`JoinTree`](sparq_fedplan::JoinTree) and *interprets* it: for each
//! leaf it lowers the pattern to a [`SubQuery`](crate::source::SubQuery)
//! ([`crate::planner::lower_leaf`]), runs it through the Phase-2
//! [`FederatedSource`](crate::FederatedSource) adapter (`execute` → an SRJ body), parses
//! that body into a [`Relation`], and joins the leaf relations **in the plan's join order**
//! with a materialised left-deep hash join. The output is a [`Relation`] — the federated
//! answer.
//!
//! # The load-bearing correctness property
//!
//! The Phase-3 deliverable is the **result-equivalence invariant**, tested against the real
//! engine:
//!
//! > For a single source holding a graph `G`, materialising a `sparq-fedplan` plan of a BGP
//! > `Q` through this interpreter yields **the same solution multiset** as evaluating `Q`
//! > locally with `sparq-engine` over `G`.
//!
//! [`solutions_equal`] is the multiset comparison (order-independent, bag semantics) and
//! the integration test `planner_result_equals_local_eval.rs` drives it end-to-end: it
//! loads a graph into a `sparq-core::Graph`, serves each leaf sub-query from that graph via
//! a [`FederatedSource`] whose transport answers from the **local engine** (so the
//! "remote" SRJ is exactly what a conformant endpoint over `G` would return), runs this
//! interpreter, and asserts [`solutions_equal`] against `sparq_engine::query(&G, Q)`.
//! That closes the loop the bead asks for: the materialised result EQUALS local eval.
//!
//! # Honest work-vs-stub split (Phase 3)
//!
//! REAL here: the SRJ parser, the index-resolved leaf lowering, the materialised left-deep
//! join, the projection over the BGP's variables, and the result-equivalence test against
//! the engine. The join is a **blocking** materialised hash join — every leaf relation is
//! fetched and held in full before joining. STUBBED for Phase 5: the non-blocking
//! `StreamJoin` feeder, concurrent leaf fan-out, the per-block bind-join operator, and
//! multi-source UNION-per-leaf (this interpreter answers a plan whose every leaf resolves
//! to ONE source; a leaf with two retained sources is rejected with
//! [`InterpError::MultiSource`] rather than silently dropping a source). The interpreter
//! also requires the plan's join algorithm to be a hash-class step — it does not yet
//! special-case [`JoinAlgo::Bind`](sparq_fedplan::JoinAlgo::Bind); a bind join is executed
//! with the same materialised hash join (semantically identical result, just not the
//! pushed-down execution discipline), which is sound for the equivalence property.
//
// [OPUS-4.8] sq-j27p (epic sq-dnko): Phase-3 materialised single-source interpreter +
// SRJ parser + result-equivalence invariant. Flagged for Fable re-review when available.

use crate::planner::{lower_leaf, pattern_vars, SourceResolver};
use crate::source::{FedError, FederatedSource};
use oxrdf::{BlankNode, Literal, NamedNode, NamedOrBlankNode, Term, Triple};
use sparq_fedplan::{JoinNode, JoinTree, PatternSources};
use std::collections::HashMap;

// ─── The materialised relation (the interpreter's intermediate value) ────────────────

/// A materialised solution table: named variables + rows, each cell `Some(term)` bound or
/// `None` unbound — the same shape as `sparq-engine`'s `QueryResult`. The interpreter
/// produces and joins these. [OPUS-4.8] sq-j27p.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Relation {
    /// Variable names (without the leading `?`), in column order.
    pub vars: Vec<String>,
    /// One entry per `vars` column; `None` is unbound.
    pub rows: Vec<Vec<Option<Term>>>,
}

impl Relation {
    /// Project this relation onto `keep` (variable names), dropping every other column and
    /// reordering to `keep`'s order. A name absent from `self.vars` becomes an all-`None`
    /// (unbound) column — the SPARQL semantics of projecting a variable not in scope.
    pub fn project(&self, keep: &[String]) -> Relation {
        let idx: Vec<Option<usize>> = keep
            .iter()
            .map(|k| self.vars.iter().position(|v| v == k))
            .collect();
        let rows = self
            .rows
            .iter()
            .map(|row| {
                idx.iter()
                    .map(|i| i.and_then(|i| row.get(i).cloned().flatten()))
                    .collect()
            })
            .collect();
        Relation {
            vars: keep.to_vec(),
            rows,
        }
    }
}

// ─── Interpreter errors ──────────────────────────────────────────────────────────────

/// A failure walking / materialising a plan. [OPUS-4.8] sq-j27p.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpError {
    /// A plan index could not be resolved (see [`crate::planner::ResolveError`]).
    Resolve(crate::planner::ResolveError),
    /// A source adapter's `execute` failed (SSRF refusal, transport error, unsupported).
    Source(FedError),
    /// The SRJ body the adapter returned could not be parsed as SPARQL-Results-JSON.
    BadSrj(String),
    /// A leaf retained more than one source — Phase-3's interpreter is single-source; the
    /// multi-source UNION-per-leaf fan-out is Phase 5. Fail closed rather than drop a source.
    MultiSource { pattern: usize, sources: usize },
}

impl std::fmt::Display for InterpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpError::Resolve(e) => write!(f, "interpreter: {}", e),
            InterpError::Source(e) => write!(f, "interpreter: source error: {}", e),
            InterpError::BadSrj(m) => write!(f, "interpreter: malformed SRJ: {}", m),
            InterpError::MultiSource { pattern, sources } => write!(
                f,
                "interpreter: pattern {} has {} retained sources; the Phase-3 single-source \
                 interpreter answers one source per leaf (multi-source UNION is Phase 5)",
                pattern, sources
            ),
        }
    }
}

impl std::error::Error for InterpError {}

impl From<crate::planner::ResolveError> for InterpError {
    fn from(e: crate::planner::ResolveError) -> Self {
        InterpError::Resolve(e)
    }
}

impl From<FedError> for InterpError {
    fn from(e: FedError) -> Self {
        InterpError::Source(e)
    }
}

// ─── The materialised single-source interpreter ──────────────────────────────────────

/// Materialise a `sparq-fedplan` [`JoinTree`] against a **single source** and return the
/// federated solution table.
///
/// `resolver` maps the plan's pattern/source indices to patterns/adapters (the Phase-0
/// finding's resolution layer); `selection` is the planner's per-pattern source selection
/// ([`select_sources`](sparq_fedplan::select_sources)) — the interpreter consults it to
/// **enforce the single-source contract**: a leaf that retained more than one source is
/// rejected with [`InterpError::MultiSource`] (Phase 5 fans those out as a UNION). `source`
/// is the one [`FederatedSource`] every leaf is answered by. For each leaf the interpreter
/// lowers the pattern ([`lower_leaf`]), fetches the SRJ via `source.execute`, parses it into
/// a [`Relation`], and folds the leaves together in the plan's join order with a
/// materialised left-deep natural (hash) join. The final relation is projected onto the
/// BGP's variables (the union of every pattern's variables, in first-seen order).
///
/// The join algorithm chosen by the planner ([`JoinAlgo`](sparq_fedplan::JoinAlgo)) is
/// honoured *semantically* but not *physically* in this phase: bind and hash both execute
/// as the same materialised natural join (identical result multiset). The non-blocking
/// `StreamJoin` feeder + the pushed-down bind-join operator are Phase 5. [OPUS-4.8] sq-j27p.
pub fn materialize_single_source(
    resolver: &SourceResolver<'_>,
    selection: &[PatternSources],
    source: &dyn FederatedSource,
    tree: &JoinTree,
) -> Result<Relation, InterpError> {
    let mut rel = materialize_node(resolver, selection, source, &tree.root)?;
    // Project onto the whole-BGP variable set (first-seen position order across patterns),
    // matching what a `SELECT *` over the BGP would scope.
    let project = bgp_vars(resolver);
    rel = rel.project(&project);
    Ok(rel)
}

/// All distinct variables of the resolver's BGP, in first-seen order (subject, predicate,
/// object within a pattern; patterns in BGP order) — the `SELECT *` scope. [OPUS-4.8] sq-j27p.
fn bgp_vars(resolver: &SourceResolver<'_>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tp in &resolver.bgp().patterns {
        for v in pattern_vars(tp) {
            if !out.contains(&v) {
                out.push(v);
            }
        }
    }
    out
}

/// Recursively materialise one plan node into a [`Relation`]. A leaf fetches + parses its
/// source sub-result; a join materialises both sides and natural-joins them. [OPUS-4.8].
fn materialize_node(
    resolver: &SourceResolver<'_>,
    selection: &[PatternSources],
    source: &dyn FederatedSource,
    node: &JoinNode,
) -> Result<Relation, InterpError> {
    match node {
        JoinNode::Leaf { pattern, .. } => materialize_leaf(resolver, selection, source, *pattern),
        JoinNode::Join { left, right, .. } => {
            let l = materialize_node(resolver, selection, source, left)?;
            let r = materialize_leaf(resolver, selection, source, *right)?;
            Ok(natural_join(&l, &r))
        }
    }
}

/// Fetch + parse one leaf pattern's sub-result from the single source. Enforces the
/// single-source contract first: a leaf the planner retained more than one source for is
/// rejected (Phase-3 is single-source). [OPUS-4.8] sq-j27p.
fn materialize_leaf(
    resolver: &SourceResolver<'_>,
    selection: &[PatternSources],
    source: &dyn FederatedSource,
    pattern: usize,
) -> Result<Relation, InterpError> {
    // Single-source guard: reject (never silently drop) a leaf with >1 retained source.
    if let Some(ps) = selection.iter().find(|ps| ps.pattern == pattern) {
        if ps.candidates.len() > 1 {
            return Err(InterpError::MultiSource {
                pattern,
                sources: ps.candidates.len(),
            });
        }
    }
    let tp = resolver.pattern(pattern)?;
    let sub = lower_leaf(tp);
    let body = source.execute(&sub)?;
    parse_srj(&body).map_err(InterpError::BadSrj)
}

/// A materialised **natural (inner) join** of two relations on their shared variables,
/// using a hash index on the smaller side. Shared columns must be equal (same `Some(term)`)
/// for two rows to combine; the result carries `left.vars` followed by the right-only vars.
/// This is the materialised analogue of the `StreamJoin` Phase 5 will run incrementally —
/// it produces the identical result multiset (bag semantics, no de-duplication).
/// [OPUS-4.8] sq-j27p.
fn natural_join(left: &Relation, right: &Relation) -> Relation {
    // Shared variables (join key) and the right-only variables (appended columns).
    let shared: Vec<(usize, usize)> = left
        .vars
        .iter()
        .enumerate()
        .filter_map(|(li, lv)| right.vars.iter().position(|rv| rv == lv).map(|ri| (li, ri)))
        .collect();
    let right_only: Vec<usize> = right
        .vars
        .iter()
        .enumerate()
        .filter(|(_, rv)| !left.vars.contains(rv))
        .map(|(ri, _)| ri)
        .collect();

    let mut out_vars = left.vars.clone();
    for &ri in &right_only {
        out_vars.push(right.vars[ri].clone());
    }

    // Build a hash index on the LEFT side keyed by the shared-column values, then probe with
    // the right. The key is the vector of bound terms at the shared columns; an UNBOUND
    // shared cell never equi-joins (it cannot equal a bound value), matching SPARQL's
    // compatible-mapping rule for the variables both sides bind.
    let mut index: HashMap<Vec<Term>, Vec<usize>> = HashMap::new();
    for (i, lrow) in left.rows.iter().enumerate() {
        if let Some(key) = join_key(lrow, &shared, true) {
            index.entry(key).or_default().push(i);
        }
    }

    let mut rows: Vec<Vec<Option<Term>>> = Vec::new();
    for rrow in &right.rows {
        let Some(key) = join_key(rrow, &shared, false) else {
            continue;
        };
        if let Some(matches) = index.get(&key) {
            for &li in matches {
                let lrow = &left.rows[li];
                let mut out = lrow.clone();
                for &ri in &right_only {
                    out.push(rrow.get(ri).cloned().flatten());
                }
                rows.push(out);
            }
        }
    }

    Relation {
        vars: out_vars,
        rows,
    }
}

/// The join key for a row at the shared columns: the bound terms in column order, or `None`
/// if any shared cell is unbound (an unbound join variable cannot equi-join). `from_left`
/// selects the left or right index of each `(li, ri)` shared pair. [OPUS-4.8] sq-j27p.
fn join_key(row: &[Option<Term>], shared: &[(usize, usize)], from_left: bool) -> Option<Vec<Term>> {
    let mut key = Vec::with_capacity(shared.len());
    for &(li, ri) in shared {
        let col = if from_left { li } else { ri };
        match row.get(col).cloned().flatten() {
            Some(t) => key.push(t),
            None => return None,
        }
    }
    Some(key)
}

// ─── SRJ parser (SPARQL-Results-JSON → Relation) ─────────────────────────────────────

/// Parse a SPARQL-Results-JSON SELECT body into a [`Relation`].
///
/// This mirrors `sparq-engine`'s `service::parse_srj` (which is `pub(crate)` there, so it
/// cannot be imported) over the SAME `oxrdf::Term` model: `uri`/`bnode`/`literal`
/// (+`xml:lang`/`its:dir`/`datatype`) and the SPARQL-1.2 `triple` term. A variable absent
/// from a solution object is UNBOUND (`None`). An ASK `boolean` body is rejected (this
/// interpreter only joins SELECT relations). [OPUS-4.8] sq-j27p.
pub fn parse_srj(text: &str) -> Result<Relation, String> {
    let v: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("invalid results JSON: {}", e))?;
    if v.get("boolean").is_some() {
        return Err("endpoint returned an ASK boolean, expected SELECT bindings".to_string());
    }
    let vars: Vec<String> = v
        .pointer("/head/vars")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                .collect()
        })
        .ok_or_else(|| "results JSON missing head.vars".to_string())?;

    let mut rows: Vec<Vec<Option<Term>>> = Vec::new();
    for sol in v
        .pointer("/results/bindings")
        .and_then(|a| a.as_array())
        .ok_or_else(|| "results JSON missing results.bindings".to_string())?
    {
        let obj = sol
            .as_object()
            .ok_or_else(|| "a solution binding is not a JSON object".to_string())?;
        let mut row: Vec<Option<Term>> = Vec::with_capacity(vars.len());
        for var in &vars {
            match obj.get(var) {
                Some(cell) => row.push(Some(srj_term(cell)?)),
                None => row.push(None),
            }
        }
        rows.push(row);
    }
    Ok(Relation { vars, rows })
}

/// Reconstruct one `oxrdf::Term` from an SRJ binding value object — the inbound counterpart
/// of `sparq-engine`'s `json::term_to_json` writer (so a relation parsed here is identical,
/// term-for-term, to what the engine emitted). [OPUS-4.8] sq-j27p.
fn srj_term(val: &serde_json::Value) -> Result<Term, String> {
    let get = |k: &str| val.get(k).and_then(|s| s.as_str());
    match get("type") {
        Some("uri") => {
            let iri = get("value").unwrap_or_default();
            Ok(Term::NamedNode(
                NamedNode::new(iri).map_err(|e| format!("bad IRI {:?}: {}", iri, e))?,
            ))
        }
        Some("bnode") => {
            let id = get("value").unwrap_or_default();
            Ok(Term::BlankNode(
                BlankNode::new(id).map_err(|e| format!("bad bnode {:?}: {}", id, e))?,
            ))
        }
        Some("literal") | Some("typed-literal") | None => {
            let value = get("value")
                .ok_or_else(|| "literal binding without value".to_string())?
                .to_string();
            if let Some(lang) = get("xml:lang") {
                // RDF 1.2 base direction (its:dir) is carried separately from xml:lang —
                // reconstruct a directional literal when present + valid, else degrade to a
                // plain language-tagged literal (the same decision the engine's parser makes).
                match get("its:dir").and_then(parse_base_direction) {
                    Some(dir) => Ok(Term::Literal(
                        Literal::new_directional_language_tagged_literal(value, lang, dir)
                            .map_err(|e| format!("bad language tag {:?}: {}", lang, e))?,
                    )),
                    None => Ok(Term::Literal(
                        Literal::new_language_tagged_literal(value, lang)
                            .map_err(|e| format!("bad language tag {:?}: {}", lang, e))?,
                    )),
                }
            } else if let Some(dt) = get("datatype") {
                let dt =
                    NamedNode::new(dt).map_err(|e| format!("bad datatype {:?}: {}", dt, e))?;
                Ok(Term::Literal(Literal::new_typed_literal(value, dt)))
            } else {
                Ok(Term::Literal(Literal::new_simple_literal(value)))
            }
        }
        Some("triple") => {
            let inner = val
                .get("value")
                .ok_or_else(|| "triple term without value".to_string())?;
            let part = |k: &str| -> Result<Term, String> {
                srj_term(
                    inner
                        .get(k)
                        .ok_or_else(|| format!("triple term without {}", k))?,
                )
            };
            let subject = match part("subject")? {
                Term::NamedNode(n) => NamedOrBlankNode::NamedNode(n),
                Term::BlankNode(b) => NamedOrBlankNode::BlankNode(b),
                other => return Err(format!("invalid triple-term subject: {}", other)),
            };
            let predicate = match part("predicate")? {
                Term::NamedNode(n) => n,
                other => return Err(format!("invalid triple-term predicate: {}", other)),
            };
            Ok(Term::Triple(Box::new(Triple {
                subject,
                predicate,
                object: part("object")?,
            })))
        }
        Some(other) => Err(format!("unknown binding type {:?}", other)),
    }
}

/// Parse the SPARQL-1.2 `its:dir` base-direction token. Mirrors
/// `sparq_core::dict::parse_base_direction` (not re-exported through `sparq-engine`, and
/// `sparq-core` is not a fedclient dependency), so the two-token grammar is inlined here.
/// [OPUS-4.8] sq-j27p.
fn parse_base_direction(s: &str) -> Option<oxrdf::BaseDirection> {
    match s {
        "ltr" => Some(oxrdf::BaseDirection::Ltr),
        "rtl" => Some(oxrdf::BaseDirection::Rtl),
        _ => None,
    }
}

// ─── Result-equivalence (the load-bearing invariant) ─────────────────────────────────

/// One solution as a binding map (variable → bound term **rendered to its canonical
/// string**), dropping unbound variables — the normal form for **bag-semantics,
/// order-independent** comparison.
///
/// `oxrdf::Term` is `Eq`/`Hash` but not `Ord`, so the bag is sorted on the term's `Display`
/// form, which is the canonical N-Triples serialisation (`<iri>`, `_:b`, `"v"^^<dt>`,
/// `"v"@lang`, `<< s p o >>`). That serialisation is injective — two terms are equal iff
/// their Display strings are equal — so comparing on it preserves the term-equality the
/// invariant needs while giving a total order to sort by. [OPUS-4.8] sq-j27p.
fn solution_map(vars: &[String], row: &[Option<Term>]) -> Vec<(String, String)> {
    let mut m: Vec<(String, String)> = vars
        .iter()
        .zip(row.iter())
        .filter_map(|(v, cell)| cell.as_ref().map(|t| (v.clone(), t.to_string())))
        .collect();
    m.sort();
    m
}

/// Multiset (bag) of solution maps for a relation — the comparison normal form. A relation
/// and the engine's `QueryResult` are equal iff these bags are equal (same maps with the
/// same multiplicities), independent of row order and column order. [OPUS-4.8] sq-j27p.
fn solution_bag(vars: &[String], rows: &[Vec<Option<Term>>]) -> Vec<Vec<(String, String)>> {
    let mut bag: Vec<Vec<(String, String)>> =
        rows.iter().map(|row| solution_map(vars, row)).collect();
    bag.sort();
    bag
}

/// The load-bearing **result-equivalence** check: do `a` and `b` carry the same solution
/// multiset (bag semantics, order-independent in both rows and columns)?
///
/// This is the predicate the Phase-3 correctness test asserts between this interpreter's
/// materialised [`Relation`] and `sparq-engine`'s local `QueryResult` for the same BGP over
/// the same graph. It compares **solutions**, not table layout: SPARQL solution sequences
/// are unordered (absent ORDER BY) and a binding map is column-order-independent, so two
/// results that differ only in row/column order are equal — but a different multiplicity or
/// a different bound term is NOT. [OPUS-4.8] sq-j27p.
pub fn solutions_equal(
    a_vars: &[String],
    a_rows: &[Vec<Option<Term>>],
    b_vars: &[String],
    b_rows: &[Vec<Option<Term>>],
) -> bool {
    solution_bag(a_vars, a_rows) == solution_bag(b_vars, b_rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::SourceResolver;
    use crate::source::{Endpoint, SubQuery, Transport};
    use sparq_fedplan::{
        plan_bgp, select_sources, Bgp, PlanOptions, SourceDescriptor, SourceId, Term as FpTerm,
        TriplePattern, Var,
    };
    use std::collections::HashMap as StdHashMap;
    use std::sync::Mutex;

    fn iri(s: &str) -> FpTerm {
        FpTerm::Iri(s.to_string())
    }
    fn var(s: &str) -> FpTerm {
        FpTerm::Var(Var::new(s))
    }

    /// A transport that answers a sub-query from a fixed map (sub-query SPARQL → SRJ body),
    /// recording what it was asked. Lets the interpreter run with NO network and NO engine,
    /// over hand-built SRJ. [OPUS-4.8] sq-j27p.
    struct MapTransport {
        answers: StdHashMap<String, String>,
        seen: Mutex<Vec<String>>,
    }
    impl MapTransport {
        fn new(answers: StdHashMap<String, String>) -> Self {
            MapTransport {
                answers,
                seen: Mutex::new(Vec::new()),
            }
        }
    }
    impl Transport for MapTransport {
        fn fetch(&self, _endpoint: &str, query: &str) -> Result<String, String> {
            self.seen.lock().unwrap().push(query.to_string());
            self.answers
                .get(query)
                .cloned()
                .ok_or_else(|| format!("no canned answer for {:?}", query))
        }
    }

    fn nn(s: &str) -> Term {
        Term::NamedNode(NamedNode::new(s).unwrap())
    }

    // A small SRJ builder for the canned-answer tests.
    fn srj(vars: &[&str], rows: &[&[(&str, &str)]]) -> String {
        let mut s = String::from("{\"head\":{\"vars\":[");
        for (i, v) in vars.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&format!("\"{}\"", v));
        }
        s.push_str("]},\"results\":{\"bindings\":[");
        for (ri, row) in rows.iter().enumerate() {
            if ri > 0 {
                s.push(',');
            }
            s.push('{');
            for (ci, (var, uri)) in row.iter().enumerate() {
                if ci > 0 {
                    s.push(',');
                }
                s.push_str(&format!("\"{}\":{{\"type\":\"uri\",\"value\":\"{}\"}}", var, uri));
            }
            s.push('}');
        }
        s.push_str("]}}");
        s
    }

    #[test]
    fn parse_srj_round_trips_terms() {
        let body = r#"{"head":{"vars":["s","o"]},"results":{"bindings":[
            {"s":{"type":"uri","value":"http://ex/a"},"o":{"type":"literal","value":"hi"}},
            {"s":{"type":"uri","value":"http://ex/b"}}
        ]}}"#;
        let rel = parse_srj(body).unwrap();
        assert_eq!(rel.vars, vec!["s", "o"]);
        assert_eq!(rel.rows.len(), 2);
        assert_eq!(rel.rows[0][0], Some(nn("http://ex/a")));
        assert_eq!(
            rel.rows[0][1],
            Some(Term::Literal(Literal::new_simple_literal("hi")))
        );
        // ?o absent in the second solution ⇒ unbound.
        assert_eq!(rel.rows[1][1], None);
    }

    #[test]
    fn parse_srj_rejects_ask_boolean() {
        assert!(parse_srj(r#"{"head":{},"boolean":true}"#).is_err());
        assert!(parse_srj("not json").is_err());
    }

    #[test]
    fn natural_join_matches_on_shared_var() {
        // L(?s,?o) join R(?o,?z): combine where ?o equal; bag semantics.
        let l = Relation {
            vars: vec!["s".into(), "o".into()],
            rows: vec![
                vec![Some(nn("http://ex/a")), Some(nn("http://ex/x"))],
                vec![Some(nn("http://ex/b")), Some(nn("http://ex/y"))],
            ],
        };
        let r = Relation {
            vars: vec!["o".into(), "z".into()],
            rows: vec![
                vec![Some(nn("http://ex/x")), Some(nn("http://ex/p"))],
                vec![Some(nn("http://ex/x")), Some(nn("http://ex/q"))],
                vec![Some(nn("http://ex/w")), Some(nn("http://ex/r"))],
            ],
        };
        let j = natural_join(&l, &r);
        assert_eq!(j.vars, vec!["s", "o", "z"]);
        // ?o=x matches a-x against {p,q}; ?o=y has no match; w has no left.
        assert_eq!(j.rows.len(), 2);
        let bag = solution_bag(&j.vars, &j.rows);
        let expect = Relation {
            vars: vec!["s".into(), "o".into(), "z".into()],
            rows: vec![
                vec![
                    Some(nn("http://ex/a")),
                    Some(nn("http://ex/x")),
                    Some(nn("http://ex/p")),
                ],
                vec![
                    Some(nn("http://ex/a")),
                    Some(nn("http://ex/x")),
                    Some(nn("http://ex/q")),
                ],
            ],
        };
        assert_eq!(bag, solution_bag(&expect.vars, &expect.rows));
    }

    #[test]
    fn solutions_equal_is_order_independent() {
        let a_vars = vec!["s".to_string(), "o".to_string()];
        let a_rows = vec![
            vec![Some(nn("http://ex/a")), Some(nn("http://ex/1"))],
            vec![Some(nn("http://ex/b")), Some(nn("http://ex/2"))],
        ];
        // Same solutions, reversed rows and swapped columns.
        let b_vars = vec!["o".to_string(), "s".to_string()];
        let b_rows = vec![
            vec![Some(nn("http://ex/2")), Some(nn("http://ex/b"))],
            vec![Some(nn("http://ex/1")), Some(nn("http://ex/a"))],
        ];
        assert!(solutions_equal(&a_vars, &a_rows, &b_vars, &b_rows));
        // A different multiplicity is NOT equal.
        let mut dup = a_rows.clone();
        dup.push(a_rows[0].clone());
        assert!(!solutions_equal(&a_vars, &dup, &b_vars, &b_rows));
    }

    /// The interpreter end-to-end over canned SRJ (no engine): a 2-pattern star
    /// ?s :p ?o . ?s :q ?z answered by ONE endpoint, joined on ?s. This drives the real
    /// lowering + resolver + materialised join path. [OPUS-4.8] sq-j27p.
    #[test]
    fn interpreter_materialises_two_pattern_join() {
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/p"), var("o")),
            TriplePattern::new(var("s"), iri("http://ex/q"), var("z")),
        ]);
        // One source declaring both predicates.
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(100)
            .predicate(sparq_fedplan::PredPartition {
                predicate: "http://ex/p".into(),
                triples: 10,
                distinct_subjects: 10,
                distinct_objects: 10,
            })
            .predicate(sparq_fedplan::PredPartition {
                predicate: "http://ex/q".into(),
                triples: 10,
                distinct_subjects: 10,
                distinct_objects: 10,
            })
            .build();
        let descriptors = [src];
        let sel = select_sources(&bgp, &descriptors);
        let tree = plan_bgp(&bgp, &sel, &descriptors, &PlanOptions::default()).unwrap();

        // Canned SRJ answers keyed by the EXACT sub-query the lowering emits.
        let mut answers = StdHashMap::new();
        answers.insert(
            "SELECT ?s ?o WHERE { ?s <http://ex/p> ?o }".to_string(),
            srj(
                &["s", "o"],
                &[
                    &[("s", "http://ex/a"), ("o", "http://ex/o1")],
                    &[("s", "http://ex/b"), ("o", "http://ex/o2")],
                ],
            ),
        );
        answers.insert(
            "SELECT ?s ?z WHERE { ?s <http://ex/q> ?z }".to_string(),
            srj(
                &["s", "z"],
                &[
                    &[("s", "http://ex/a"), ("z", "http://ex/z1")],
                    // no row for ex/b ⇒ it drops out of the join.
                ],
            ),
        );
        let ep = Endpoint::new("http://8.8.8.8/sparql", Box::new(MapTransport::new(answers)));
        let adapters: Vec<&dyn FederatedSource> = vec![&ep];
        let resolver = SourceResolver::new(&bgp, &adapters);

        let rel = materialize_single_source(&resolver, &sel, &ep, &tree).unwrap();
        // Only ex/a joins across both arms ⇒ one solution {s=a, o=o1, z=z1}.
        let expect_vars = vec!["s".to_string(), "o".to_string(), "z".to_string()];
        let expect_rows = vec![vec![
            Some(nn("http://ex/a")),
            Some(nn("http://ex/o1")),
            Some(nn("http://ex/z1")),
        ]];
        assert!(
            solutions_equal(&rel.vars, &rel.rows, &expect_vars, &expect_rows),
            "got {:?}",
            rel
        );
    }

    /// A transport that PANICS if reached — the multi-source guard must reject BEFORE any
    /// `execute`/`fetch`, so a leaf with >1 retained source never hits the network.
    struct PanicTransport;
    impl Transport for PanicTransport {
        fn fetch(&self, _e: &str, _q: &str) -> Result<String, String> {
            panic!("multi-source guard must fail closed before any fetch");
        }
    }

    #[test]
    fn interpreter_rejects_multi_source_leaf_phase3() {
        // A leaf with two retained sources must fail closed (single-source Phase 3) — and do
        // so without ever calling the transport (PanicTransport proves the early return).
        let bgp = Bgp::new(vec![TriplePattern::new(
            var("s"),
            iri("http://ex/p"),
            var("o"),
        )]);
        // Two descriptors BOTH holding :p ⇒ the leaf retains two sources.
        let mk = |id: &str| {
            SourceDescriptor::builder(SourceId::new(id))
                .total_triples(10)
                .predicate(sparq_fedplan::PredPartition {
                    predicate: "http://ex/p".into(),
                    triples: 10,
                    distinct_subjects: 10,
                    distinct_objects: 10,
                })
                .build()
        };
        let descriptors = [mk("A"), mk("B")];
        let sel = select_sources(&bgp, &descriptors);
        assert_eq!(sel[0].candidates.len(), 2, "both sources retained");
        let tree = plan_bgp(&bgp, &sel, &descriptors, &PlanOptions::default()).unwrap();
        let ep = Endpoint::new("http://8.8.8.8/sparql", Box::new(PanicTransport));
        let adapters: Vec<&dyn FederatedSource> = vec![&ep];
        let resolver = SourceResolver::new(&bgp, &adapters);
        let err = materialize_single_source(&resolver, &sel, &ep, &tree).unwrap_err();
        assert_eq!(
            err,
            InterpError::MultiSource {
                pattern: 0,
                sources: 2,
            }
        );
    }

    /// Unused-import guard: `SubQuery` is the lowering's output type; reference it so the
    /// test module's import set is honest. (Kept minimal — the real lowering is exercised by
    /// `interpreter_materialises_two_pattern_join` through `lower_leaf`.) [OPUS-4.8] sq-j27p.
    #[test]
    fn subquery_type_is_in_scope() {
        let _ = SubQuery::new("ASK {}");
    }
}
