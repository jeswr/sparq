//! Physical execution: BGP via greedy-ordered merge/hash joins over the
//! permutation indexes, plus OPTIONAL / UNION / MINUS / BIND / VALUES,
//! aggregation (GROUP BY / HAVING via Filter), ORDER BY, sub-SELECT and the
//! solution modifiers. All intermediate results are id-level (`Bindings`);
//! values computed at query time (BIND, aggregates) get ids in a per-query
//! `LocalVocab`, mirroring QLever's local vocabulary.

use crate::QueryResult;
use oxrdf::vocab::xsd;
use oxrdf::{Literal, Term, Variable};
use rustc_hash::FxHashMap;
use sparq_core::dict::{Id, NO_ID};
use sparq_core::store::Pattern as IdPattern;
use sparq_core::Graph;
use spargebra::algebra::{
    AggregateExpression, AggregateFunction, Expression, GraphPattern, OrderExpression,
};
use smallvec::SmallVec;
use spargebra::term::{GroundTerm, NamedNodePattern, TermPattern, TriplePattern};
use std::cmp::Ordering;

/// One solution row: the ids bound to each of a [`Bindings`]' variables. Inlined
/// up to 4 columns (the common case) so a join produces no heap allocation per
/// row — the dominant cost on large join results.
type Row = SmallVec<[Id; 4]>;

/// A join / group key (the ids of the shared or grouping columns). Inlined up to
/// 2 columns — most joins are on one or two variables — so building a hash table
/// or probing it allocates nothing per key.
type Key = SmallVec<[Id; 2]>;

/// A hash-table posting list (row indices sharing a key). Inlined up to 2 — many
/// join keys (and almost all OPTIONAL keys) match only one or two rows — so the
/// build allocates nothing per bucket in the common case.
type Posting = SmallVec<[usize; 2]>;

/// Ids at or above this base index into the per-query [`LocalVocab`] instead of
/// the graph dictionary. (M2 uses `u32` ids; the tagged 64-bit ValueId of M4
/// removes this watermark split.)
const LOCAL_BASE: Id = 1 << 31;

#[inline]
fn is_local(id: Id) -> bool {
    id >= LOCAL_BASE
}

/// Per-query vocabulary for terms produced during evaluation (BIND, aggregates)
/// that are not in the graph dictionary.
#[derive(Default)]
pub struct LocalVocab {
    terms: Vec<Term>,
    ids: FxHashMap<Term, Id>,
}

impl LocalVocab {
    /// Interns a term, returning a stable id: equal terms get the same id so
    /// DISTINCT, GROUP BY, joins and equality work on computed values.
    fn intern(&mut self, t: Term) -> Id {
        if let Some(&id) = self.ids.get(&t) {
            return id;
        }
        let id = LOCAL_BASE + self.terms.len() as Id;
        self.terms.push(t.clone());
        self.ids.insert(t, id);
        id
    }
    fn term(&self, id: Id) -> &Term {
        &self.terms[(id - LOCAL_BASE) as usize]
    }
}

/// Resolves an id to its term (graph dictionary or local vocab); `NO_ID` -> None.
fn term_of(graph: &Graph, local: &LocalVocab, id: Id) -> Option<Term> {
    if id == NO_ID {
        None
    } else if is_local(id) {
        Some(local.term(id).clone())
    } else {
        Some(graph.dict.term(id).clone())
    }
}

/// Intermediate result: rows of one id per `vars[i]` (`NO_ID` = unbound).
/// `sorted_by` records the variable the rows are sorted on (if any), enabling a
/// merge join instead of a hash table.
struct Bindings {
    vars: Vec<Variable>,
    rows: Vec<Row>,
    sorted_by: Option<Variable>,
}

impl Bindings {
    fn col(&self, v: &Variable) -> Option<usize> {
        self.vars.iter().position(|x| x == v)
    }
    fn unsorted(vars: Vec<Variable>, rows: Vec<Row>) -> Self {
        Bindings { vars, rows, sorted_by: None }
    }
}

pub fn eval_select(graph: &Graph, pattern: &GraphPattern) -> Result<QueryResult, String> {
    let mut local = LocalVocab::default();
    let bindings = eval_modified(graph, &mut local, pattern)?;

    // SELECT * exposes only real variables, never synthetic blank-node variables.
    let out_vars: Vec<Variable> = bindings
        .vars
        .iter()
        .filter(|v| !v.as_str().starts_with(BNODE_VAR_PREFIX))
        .cloned()
        .collect();

    let col_of: Vec<Option<usize>> = out_vars.iter().map(|v| bindings.col(v)).collect();
    let mut rows: Vec<Vec<Option<Term>>> = Vec::with_capacity(bindings.rows.len());
    for row in &bindings.rows {
        rows.push(col_of.iter().map(|c| c.and_then(|i| term_of(graph, &local, row[i]))).collect());
    }
    Ok(QueryResult { vars: out_vars, rows })
}

/// Evaluates a SELECT but returns only the solution count, skipping the id->term
/// materialisation (the number of id-level rows equals the number of solutions).
pub fn count_select(graph: &Graph, pattern: &GraphPattern) -> Result<usize, String> {
    let mut local = LocalVocab::default();
    let bindings = eval_modified(graph, &mut local, pattern)?;
    Ok(bindings.rows.len())
}

// ---- Algebra dispatch ---------------------------------------------------------

fn eval_modified(graph: &Graph, local: &mut LocalVocab, p: &GraphPattern) -> Result<Bindings, String> {
    match p {
        GraphPattern::Project { inner, variables } => {
            let b = eval_modified(graph, local, inner)?;
            Ok(project_bindings(b, variables))
        }
        GraphPattern::Distinct { inner } => {
            let mut b = eval_modified(graph, local, inner)?;
            distinct_bindings(&mut b);
            Ok(b)
        }
        GraphPattern::Reduced { inner } => eval_modified(graph, local, inner),
        GraphPattern::Slice { inner, start, length } => {
            // LIMIT early-termination: a bare LIMIT over a single-pattern scan
            // (no ORDER BY / DISTINCT / aggregation / join, which all need the full
            // result) can stop after start+length rows instead of materialising the
            // whole relation — true streaming. LIMIT-without-ORDER-BY is
            // order-insensitive so any rows are valid.
            if let Some(len) = length {
                if let Some(cap) = start.checked_add(*len) {
                    if let Some(mut b) = try_capped(graph, inner, cap)? {
                        slice_bindings(&mut b, *start, *length);
                        return Ok(b);
                    }
                }
            }
            let mut b = eval_modified(graph, local, inner)?;
            slice_bindings(&mut b, *start, *length);
            Ok(b)
        }
        GraphPattern::OrderBy { inner, expression } => {
            let mut b = eval_modified(graph, local, inner)?;
            order_bindings(graph, local, &mut b, expression)?;
            Ok(b)
        }
        GraphPattern::Group { inner, variables, aggregates } => {
            // COUNT(*) pushdown: a whole-dataset COUNT over a single pattern is the
            // scan range size — no need to materialise the solutions just to count
            // them (QLever counts lazily too; this is the q02-style win).
            if variables.is_empty() && aggregates.len() == 1 {
                if let (av, AggregateExpression::CountSolutions { distinct: false }) = (&aggregates[0].0, &aggregates[0].1) {
                    if let Some(n) = count_pushdown(graph, inner) {
                        let id = value_to_id(graph, local, &Value::Num(n as f64));
                        let row: Row = std::iter::once(id).collect();
                        return Ok(Bindings { vars: vec![av.clone()], rows: vec![row], sorted_by: None });
                    }
                }
            }
            let b = eval_graph_pattern(graph, local, inner)?;
            group_aggregate(graph, local, b, variables, aggregates)
        }
        other => eval_graph_pattern(graph, local, other),
    }
}

/// Evaluates a pattern producing at most `cap` rows by stopping the scan early,
/// when that is safe: a single-pattern scan (optionally with a pushed-down
/// sargable numeric filter) under projection. Returns `None` for anything that
/// needs the full result first (joins, ORDER BY, DISTINCT, aggregation, residual
/// filters), so the caller falls back to full evaluation.
fn try_capped(graph: &Graph, inner: &GraphPattern, cap: usize) -> Result<Option<Bindings>, String> {
    match inner {
        GraphPattern::Project { inner, variables } => {
            Ok(try_capped(graph, inner, cap)?.map(|b| project_bindings(b, variables)))
        }
        GraphPattern::Reduced { inner } => try_capped(graph, inner, cap),
        p if is_conjunctive(p) => {
            let mut patterns = Vec::new();
            let mut filters = Vec::new();
            flatten_conjunction(p, &mut patterns, &mut filters);
            if patterns.len() != 1 {
                return Ok(None);
            }
            let (pat_filters, residual) = split_sargable(&patterns, &filters);
            if !residual.is_empty() {
                return Ok(None);
            }
            let (id_pat, pos_vars, unsat) = prepare_pattern(graph, &patterns[0])?;
            if unsat {
                return Ok(Some(Bindings::unsorted(collect_vars(&patterns), vec![])));
            }
            let filt = pat_filters[0];
            let sort_col = filt.map(|(c, _)| c);
            Ok(Some(scan_to_bindings(graph, &id_pat, &pos_vars, sort_col, filt, Some(cap))))
        }
        _ => Ok(None),
    }
}

/// COUNT(*) over a single-pattern, filter-free, no-repeated-variable BGP equals
/// the scan range size — returned without materialising any solution. Returns
/// `None` (fall back to full evaluation) for anything more complex.
fn count_pushdown(graph: &Graph, inner: &GraphPattern) -> Option<usize> {
    if !is_conjunctive(inner) {
        return None;
    }
    let mut patterns = Vec::new();
    let mut filters = Vec::new();
    flatten_conjunction(inner, &mut patterns, &mut filters);
    if !filters.is_empty() || patterns.len() != 1 {
        return None;
    }
    let (id_pat, pos_vars, unsat) = prepare_pattern(graph, &patterns[0]).ok()?;
    if unsat {
        return Some(0);
    }
    // A repeated variable (e.g. `?x p ?x`) makes the range size an over-count.
    let vars: Vec<&Variable> = pos_vars.iter().flatten().collect();
    let mut sorted = vars.clone();
    sorted.sort();
    sorted.dedup();
    if sorted.len() != vars.len() {
        return None;
    }
    Some(graph.store.estimate(&id_pat))
}

fn eval_graph_pattern(graph: &Graph, local: &mut LocalVocab, p: &GraphPattern) -> Result<Bindings, String> {
    if is_conjunctive(p) {
        let mut patterns = Vec::new();
        let mut filters = Vec::new();
        flatten_conjunction(p, &mut patterns, &mut filters);
        // Push sargable numeric FILTERs down into the binary-plan scans (the WCOJ
        // path applies them afterwards); apply the rest normally.
        let (mut b, residual) = if bgp_uses_binary(&patterns) {
            let (pat_filters, residual) = split_sargable(&patterns, &filters);
            (eval_bgp_binary(graph, &patterns, &pat_filters)?, residual)
        } else {
            (eval_bgp(graph, &patterns)?, filters)
        };
        for f in &residual {
            apply_filter(graph, local, &mut b, f)?;
        }
        return Ok(b);
    }
    match p {
        GraphPattern::Bgp { patterns } => eval_bgp(graph, patterns),
        GraphPattern::Filter { expr, inner } => {
            let mut b = eval_graph_pattern(graph, local, inner)?;
            apply_filter(graph, local, &mut b, expr)?;
            Ok(b)
        }
        GraphPattern::Join { left, right } => {
            let l = eval_graph_pattern(graph, local, left)?;
            let r = eval_graph_pattern(graph, local, right)?;
            Ok(join_bindings(l, r))
        }
        GraphPattern::LeftJoin { left, right, expression } => {
            let l = eval_graph_pattern(graph, local, left)?;
            let r = eval_graph_pattern(graph, local, right)?;
            left_outer_join(graph, local, l, r, expression.as_ref())
        }
        GraphPattern::Union { left, right } => {
            let l = eval_graph_pattern(graph, local, left)?;
            let r = eval_graph_pattern(graph, local, right)?;
            Ok(union_bindings(l, r))
        }
        GraphPattern::Extend { inner, variable, expression } => {
            let b = eval_graph_pattern(graph, local, inner)?;
            extend_bindings(graph, local, b, variable, expression)
        }
        GraphPattern::Minus { left, right } => {
            let l = eval_graph_pattern(graph, local, left)?;
            let r = eval_graph_pattern(graph, local, right)?;
            Ok(minus_bindings(l, r))
        }
        GraphPattern::Values { variables, bindings } => Ok(values_bindings(graph, local, variables, bindings)),
        GraphPattern::Graph { .. } => {
            // The store is a single default graph; evaluating a named-graph
            // pattern against it would silently return wrong matches.
            Err("named graphs (GRAPH) are not yet supported".into())
        }
        GraphPattern::Project { .. }
        | GraphPattern::Distinct { .. }
        | GraphPattern::Reduced { .. }
        | GraphPattern::Slice { .. }
        | GraphPattern::OrderBy { .. }
        | GraphPattern::Group { .. } => eval_modified(graph, local, p),
        other => Err(format!("unsupported graph pattern: {other:?}")),
    }
}

fn is_conjunctive(p: &GraphPattern) -> bool {
    match p {
        GraphPattern::Bgp { .. } => true,
        GraphPattern::Filter { inner, .. } => is_conjunctive(inner),
        GraphPattern::Join { left, right } => is_conjunctive(left) && is_conjunctive(right),
        _ => false,
    }
}

fn flatten_conjunction(p: &GraphPattern, patterns: &mut Vec<TriplePattern>, filters: &mut Vec<Expression>) {
    match p {
        GraphPattern::Bgp { patterns: tps } => patterns.extend(tps.iter().cloned()),
        GraphPattern::Join { left, right } => {
            flatten_conjunction(left, patterns, filters);
            flatten_conjunction(right, patterns, filters);
        }
        GraphPattern::Filter { expr, inner } => {
            flatten_conjunction(inner, patterns, filters);
            filters.push(expr.clone());
        }
        _ => unreachable!(),
    }
}

// ---- Sargable numeric filters (pushed into the scan) -------------------------

/// A numeric comparison `value OP threshold` that can be pushed down into a
/// pattern scan (FILTER predicate evaluated inline, in the column's sorted order,
/// so the numeric access is sequential rather than a random dictionary gather —
/// the layout fix the hardware research measured as an 8–15× win).
#[derive(Clone, Copy)]
enum NumCmp {
    Gt(f64),
    Ge(f64),
    Lt(f64),
    Le(f64),
    Eq(f64),
}

impl NumCmp {
    #[inline]
    fn test(&self, x: f64) -> bool {
        match *self {
            NumCmp::Gt(t) => x > t,
            NumCmp::Ge(t) => x >= t,
            NumCmp::Lt(t) => x < t,
            NumCmp::Le(t) => x <= t,
            NumCmp::Eq(t) => x == t,
        }
    }
}

/// Recognises a FILTER of the form `?v OP numeric-constant` (or the symmetric
/// `constant OP ?v`), returning the variable and the comparison to push down.
fn extract_sargable(e: &Expression) -> Option<(Variable, NumCmp)> {
    fn lit_num(e: &Expression) -> Option<f64> {
        match e {
            Expression::Literal(l) if is_numeric_dt(l) => l.value().parse().ok(),
            _ => None,
        }
    }
    fn var_of(e: &Expression) -> Option<Variable> {
        match e {
            Expression::Variable(v) => Some(v.clone()),
            _ => None,
        }
    }
    // (left, right, cmp-if-var-on-left, cmp-if-var-on-right)
    let (l, r, on_left, on_right): (&Expression, &Expression, fn(f64) -> NumCmp, fn(f64) -> NumCmp) = match e {
        Expression::Greater(l, r) => (l, r, NumCmp::Gt, NumCmp::Lt),
        Expression::GreaterOrEqual(l, r) => (l, r, NumCmp::Ge, NumCmp::Le),
        Expression::Less(l, r) => (l, r, NumCmp::Lt, NumCmp::Gt),
        Expression::LessOrEqual(l, r) => (l, r, NumCmp::Le, NumCmp::Ge),
        Expression::Equal(l, r) => (l, r, NumCmp::Eq, NumCmp::Eq),
        _ => return None,
    };
    if let (Some(v), Some(c)) = (var_of(l), lit_num(r)) {
        return Some((v, on_left(c)));
    }
    if let (Some(c), Some(v)) = (lit_num(l), var_of(r)) {
        return Some((v, on_right(c)));
    }
    None
}

/// The canonical position (0=subject, 1=predicate, 2=object) of a variable in a
/// triple pattern, if it occurs there.
fn pattern_var_pos(tp: &TriplePattern, var: &Variable) -> Option<usize> {
    if matches!(&tp.subject, TermPattern::Variable(v) if v == var) {
        return Some(0);
    }
    if matches!(&tp.predicate, NamedNodePattern::Variable(v) if v == var) {
        return Some(1);
    }
    if matches!(&tp.object, TermPattern::Variable(v) if v == var) {
        return Some(2);
    }
    None
}

/// Splits FILTERs into per-pattern sargable numeric predicates (pushed into the
/// scan of the first pattern that binds the variable) and the residual filters
/// (applied normally afterwards).
fn split_sargable(patterns: &[TriplePattern], filters: &[Expression]) -> (Vec<Option<(usize, NumCmp)>>, Vec<Expression>) {
    let mut pat_filters: Vec<Option<(usize, NumCmp)>> = vec![None; patterns.len()];
    let mut residual = Vec::new();
    for f in filters {
        if let Some((var, cmp)) = extract_sargable(f) {
            if let Some((i, pos)) = patterns
                .iter()
                .enumerate()
                .find_map(|(i, tp)| pattern_var_pos(tp, &var).filter(|_| pat_filters[i].is_none()).map(|pos| (i, pos)))
            {
                pat_filters[i] = Some((pos, cmp));
                continue;
            }
        }
        residual.push(f.clone());
    }
    (pat_filters, residual)
}

// ---- BGP evaluation ----------------------------------------------------------

/// Evaluates a basic graph pattern, dispatching between a binary join plan and a
/// worst-case-optimal (Leapfrog Triejoin) plan. Cyclic BGPs — where binary plans
/// can blow up to an intermediate result far larger than the final answer (e.g.
/// triangles) — go to WCOJ, which runs in time `Õ(AGM bound)` and so is provably
/// optimal in the worst case. Acyclic (tree-shaped) BGPs use binary joins, which
/// are optimal for them and avoid LFTJ's per-tuple overhead. Both paths are
/// differentially tested to produce identical results.
fn eval_bgp(graph: &Graph, patterns: &[TriplePattern]) -> Result<Bindings, String> {
    if patterns.is_empty() {
        return Ok(Bindings { vars: vec![], rows: vec![Row::new()], sorted_by: None });
    }
    if patterns.len() >= 3 && bgp_is_cyclic(patterns) {
        return eval_bgp_wcoj(graph, patterns);
    }
    eval_bgp_binary(graph, patterns, &[])
}

/// Whether this conjunctive BGP would be routed to the worst-case-optimal plan
/// (so the caller knows whether sargable-filter pushdown into the binary scan
/// applies).
fn bgp_uses_binary(patterns: &[TriplePattern]) -> bool {
    !(patterns.len() >= 3 && bgp_is_cyclic(patterns))
}

/// Binary-join BGP plan: greedy cardinality ordering with sort-merge joins on the
/// current sort variable (falling back to hash, then cross product). `pat_filters`
/// holds an optional pushed-down numeric FILTER per pattern (by original index).
fn eval_bgp_binary(graph: &Graph, patterns: &[TriplePattern], pat_filters: &[Option<(usize, NumCmp)>]) -> Result<Bindings, String> {
    if patterns.is_empty() {
        return Ok(Bindings { vars: vec![], rows: vec![Row::new()], sorted_by: None });
    }
    let pfilter = |i: usize| -> Option<(usize, NumCmp)> { pat_filters.get(i).copied().flatten() };

    struct Prepared {
        id_pat: IdPattern,
        pos_vars: [Option<Variable>; 3],
        est: usize,
        unsatisfiable: bool,
    }
    let mut prepared: Vec<Prepared> = Vec::with_capacity(patterns.len());
    for tp in patterns {
        let (id_pat, pos_vars, unsat) = prepare_pattern(graph, tp)?;
        let est = if unsat { 0 } else { graph.store.estimate(&id_pat) };
        prepared.push(Prepared { id_pat, pos_vars, est, unsatisfiable: unsat });
    }
    if prepared.iter().any(|p| p.unsatisfiable) {
        return Ok(Bindings::unsorted(collect_vars(patterns), vec![]));
    }

    let var_pos = |i: usize, v: &Variable| -> Option<usize> {
        prepared[i].pos_vars.iter().position(|pv| pv.as_ref() == Some(v))
    };

    // Cost-based greedy (GOO): seed with the smallest single-pattern cardinality,
    // then repeatedly add the connected pattern that yields the smallest *estimated
    // join result*, using the per-predicate characteristic stats (distinct
    // subjects/objects) to estimate join selectivity. The join order only affects
    // performance (the result is identical for any order — differentially tested).
    let seed = (0..prepared.len()).min_by_key(|&i| prepared[i].est).unwrap();
    let seed_join_var = prepared[seed]
        .pos_vars
        .iter()
        .flatten()
        .find(|v| (0..prepared.len()).any(|j| j != seed && var_pos(j, v).is_some()))
        .cloned();
    // A pushed-down filter scans in its own column's order (sequential numeric
    // access); otherwise sort by the join variable to enable a merge join.
    let seed_sort_col = pfilter(seed).map(|(c, _)| c).or_else(|| seed_join_var.as_ref().and_then(|v| var_pos(seed, v)));

    let mut result = scan_to_bindings(graph, &prepared[seed].id_pat, &prepared[seed].pos_vars, seed_sort_col, pfilter(seed), None);
    let mut done = vec![false; prepared.len()];
    done[seed] = true;

    // Running estimate of the result cardinality and the per-variable distinct
    // count (ndv), used to score the next join.
    let mut cur_card = prepared[seed].est as f64;
    let mut var_ndv: FxHashMap<Variable, f64> = FxHashMap::default();
    let mut record_vars = |i: usize, cur_card: f64, var_ndv: &mut FxHashMap<Variable, f64>| {
        for (pos, ov) in prepared[i].pos_vars.iter().enumerate() {
            if let Some(v) = ov {
                let ndv = pattern_var_ndv(graph, &prepared[i].id_pat, pos, prepared[i].est).min(cur_card.max(1.0));
                let e = var_ndv.entry(v.clone()).or_insert(ndv);
                *e = e.min(ndv);
            }
        }
    };
    record_vars(seed, cur_card, &mut var_ndv);

    for _ in 1..prepared.len() {
        // Pick the connected candidate with the smallest estimated output.
        let mut best: Option<(usize, f64)> = None;
        for i in 0..prepared.len() {
            if done[i] {
                continue;
            }
            let mut sel = 1.0f64;
            let mut shared = false;
            for (pos, ov) in prepared[i].pos_vars.iter().enumerate() {
                if let Some(v) = ov {
                    if let Some(&rndv) = var_ndv.get(v) {
                        shared = true;
                        let pndv = pattern_var_ndv(graph, &prepared[i].id_pat, pos, prepared[i].est);
                        sel /= rndv.max(pndv).max(1.0);
                    }
                }
            }
            if !shared {
                continue;
            }
            let out = cur_card * prepared[i].est as f64 * sel;
            if best.map_or(true, |(_, bc)| out < bc) {
                best = Some((i, out));
            }
        }
        let i = match best {
            Some((i, out)) => {
                cur_card = out.max(0.0);
                i
            }
            None => {
                // Disconnected: smallest-cardinality remaining (cross product).
                let i = (0..prepared.len()).filter(|&j| !done[j]).min_by_key(|&j| prepared[j].est).unwrap();
                cur_card *= prepared[i].est as f64;
                i
            }
        };
        done[i] = true;

        // Execute: a pushed-down filter forces the scan into its own column order
        // (and filters inline); otherwise sort by the join variable for a merge.
        let filt = pfilter(i);
        let merge_var = result.sorted_by.clone().filter(|sv| var_pos(i, sv).is_some());
        let scan_sort = filt.map(|(c, _)| c).or_else(|| merge_var.as_ref().map(|jv| var_pos(i, jv).unwrap()));
        let rhs = scan_to_bindings(graph, &prepared[i].id_pat, &prepared[i].pos_vars, scan_sort, filt, None);
        let connected = prepared[i].pos_vars.iter().flatten().any(|v| result.vars.contains(v));
        // Merge only when both sides are sorted on the join variable (a filter may
        // have forced the scan into a different order).
        if let Some(jv) = merge_var.filter(|jv| rhs.sorted_by.as_ref() == Some(jv)) {
            result = merge_join(result, rhs, &jv);
        } else if connected {
            result = hash_join(result, rhs);
        } else {
            result = cross_product(result, rhs);
        }
        record_vars(i, cur_card, &mut var_ndv);

        if result.rows.is_empty() {
            break;
        }
    }
    Ok(result)
}

/// Estimated number of distinct values of the variable at canonical position
/// `pos` in a pattern, from the per-predicate characteristic stats. Falls back to
/// the pattern's cardinality (an upper bound) when the predicate is unbound or the
/// other terminal is bound (so the column is effectively keyed).
fn pattern_var_ndv(graph: &Graph, id_pat: &IdPattern, pos: usize, est: usize) -> f64 {
    let est = (est as f64).max(1.0);
    let stat = id_pat[1].and_then(|pid| graph.store.pred_stat(pid));
    match (pos, stat) {
        // subject var, predicate bound, object unbound -> distinct subjects of P
        (0, Some(s)) if id_pat[2].is_none() => (s.ndv_subj as f64).clamp(1.0, est),
        // object var, predicate bound, subject unbound -> distinct objects of P
        (2, Some(s)) if id_pat[0].is_none() => (s.ndv_obj as f64).clamp(1.0, est),
        _ => est,
    }
}

fn collect_vars(patterns: &[TriplePattern]) -> Vec<Variable> {
    let mut vars = Vec::new();
    for tp in patterns {
        for v in [tp_var(&tp.subject), nnp_var(&tp.predicate), tp_var(&tp.object)].into_iter().flatten() {
            if !vars.contains(&v) {
                vars.push(v);
            }
        }
    }
    vars
}

// ---- Worst-case-optimal join: Leapfrog Triejoin ------------------------------
//
// LFTJ (Veldhuizen 2014) evaluates a BGP one *variable* at a time in a fixed
// global order. At each variable it intersects, via the "leapfrog" galloping
// search, the sorted value streams of every pattern that mentions that variable,
// then recurses. Each pattern is a trie of its variable columns (projected from a
// permutation index, sorted in the global variable order). The total work is
// bounded by the AGM fractional-edge-cover bound on the BGP, so for cyclic
// queries it cannot produce the asymptotically-large intermediates a binary plan
// would. See research/ARCHITECTURE.md §4.

/// A pattern's relation projected onto its variables (in global order) as sorted,
/// deduplicated tuples — the trie that LFTJ navigates level by level.
struct Trie {
    tuples: Vec<Vec<Id>>,
}

/// One open level of a [`TrieIter`]: `hi` bounds the current key's subtree (rows
/// sharing all already-fixed columns), `cur` is the cursor within it.
struct Frame {
    hi: usize,
    cur: usize,
}

/// A cursor over a [`Trie`], using Veldhuizen's open-on-entry semantics: it starts
/// *above* the root, and `open()` descends one column, resetting the cursor to the
/// start of that subtree. This reset is what makes non-contiguous variable
/// participation correct — re-entering a level re-opens (rewinds) the iterator.
struct TrieIter<'a> {
    trie: &'a Trie,
    frames: Vec<Frame>,
}

impl<'a> TrieIter<'a> {
    fn new(trie: &'a Trie) -> Self {
        TrieIter { trie, frames: Vec::new() }
    }
    /// The column currently being iterated (valid once at least one `open`).
    #[inline]
    fn col(&self) -> usize {
        self.frames.len() - 1
    }
    #[inline]
    fn at_end(&self) -> bool {
        let f = self.frames.last().unwrap();
        f.cur >= f.hi
    }
    #[inline]
    fn key(&self) -> Id {
        let col = self.col();
        let f = self.frames.last().unwrap();
        self.trie.tuples[f.cur][col]
    }
    /// End (exclusive) of the run of rows in `[start, hi)` whose `col` equals the
    /// value at `start`. The slice is sorted, so this is a binary search — O(log n)
    /// rather than a linear scan of the (possibly large) run.
    #[inline]
    fn run_end(&self, col: usize, start: usize, hi: usize, val: Id) -> usize {
        start + self.trie.tuples[start..hi].partition_point(|row| row[col] <= val)
    }
    /// Advances to the next distinct value in the current column.
    fn next(&mut self) {
        let col = self.col();
        let (cur, hi) = {
            let f = self.frames.last().unwrap();
            (f.cur, f.hi)
        };
        let val = self.trie.tuples[cur][col];
        self.frames.last_mut().unwrap().cur = self.run_end(col, cur, hi, val);
    }
    /// Galloping seek: first value `>= x` in the current column.
    fn seek(&mut self, x: Id) {
        let col = self.col();
        let f = self.frames.last_mut().unwrap();
        let (mut a, mut b) = (f.cur, f.hi);
        while a < b {
            let m = a + (b - a) / 2;
            if self.trie.tuples[m][col] < x {
                a = m + 1;
            } else {
                b = m;
            }
        }
        f.cur = a;
    }
    /// Descends one column: into the subtree of the parent's current key (or, at
    /// the root, the whole relation), with the cursor reset to its start.
    fn open(&mut self) {
        match self.frames.last() {
            None => {
                self.frames.push(Frame { hi: self.trie.tuples.len(), cur: 0 });
            }
            Some(&Frame { cur: plo, hi: phi }) => {
                let pcol = self.frames.len() - 1;
                let val = self.trie.tuples[plo][pcol];
                let end = self.run_end(pcol, plo, phi, val);
                self.frames.push(Frame { hi: end, cur: plo });
            }
        }
    }
    fn up(&mut self) {
        self.frames.pop();
    }
}

/// Leapfrog intersection of the participating iterators at one level.
struct Leapfrog {
    order: Vec<usize>, // participant indices, kept in cyclic key order
    p: usize,
    ended: bool,
    key: Id,
}

impl Leapfrog {
    fn init(iters: &mut [TrieIter], parts: &[usize]) -> Self {
        let mut lf = Leapfrog { order: parts.to_vec(), p: 0, ended: false, key: 0 };
        if parts.iter().any(|&i| iters[i].at_end()) {
            lf.ended = true;
            return lf;
        }
        lf.order.sort_by_key(|&i| iters[i].key());
        lf.search(iters);
        lf
    }
    fn search(&mut self, iters: &mut [TrieIter]) {
        let k = self.order.len();
        loop {
            let max = iters[self.order[(self.p + k - 1) % k]].key();
            let min = iters[self.order[self.p]].key();
            if min == max {
                self.key = min;
                return;
            }
            iters[self.order[self.p]].seek(max);
            if iters[self.order[self.p]].at_end() {
                self.ended = true;
                return;
            }
            self.p = (self.p + 1) % k;
        }
    }
    fn next(&mut self, iters: &mut [TrieIter]) {
        let k = self.order.len();
        iters[self.order[self.p]].next();
        if iters[self.order[self.p]].at_end() {
            self.ended = true;
            return;
        }
        self.p = (self.p + 1) % k;
        self.search(iters);
    }
}

#[allow(clippy::too_many_arguments)]
fn lftj_recurse(
    iters: &mut [TrieIter],
    parts_at_level: &[Vec<usize>],
    level: usize,
    n_levels: usize,
    current: &mut [Id],
    out: &mut Vec<Row>,
) {
    if level == n_levels {
        out.push(Row::from_slice(current));
        return;
    }
    let parts = &parts_at_level[level];
    // Open-on-entry: descend each relevant iterator into this level (rewinding it).
    for &i in parts {
        iters[i].open();
    }
    let mut lf = Leapfrog::init(iters, parts);
    while !lf.ended {
        current[level] = lf.key;
        lftj_recurse(iters, parts_at_level, level + 1, n_levels, current, out);
        lf.next(iters);
    }
    for &i in parts {
        iters[i].up();
    }
}

/// Builds a pattern's trie of projected variable tuples, plus the global levels
/// of those variables (sorted ascending). Repeated-variable patterns keep only
/// rows where all positions of a variable agree.
fn build_trie(
    graph: &Graph,
    id_pat: &IdPattern,
    pos_vars: &[Option<Variable>; 3],
    var_levels: &FxHashMap<Variable, usize>,
) -> (Trie, Vec<usize>) {
    let mut var_positions: Vec<(Variable, Vec<usize>)> = Vec::new();
    for (pos, ov) in pos_vars.iter().enumerate() {
        if let Some(v) = ov {
            if let Some(e) = var_positions.iter_mut().find(|(x, _)| x == v) {
                e.1.push(pos);
            } else {
                var_positions.push((v.clone(), vec![pos]));
            }
        }
    }
    var_positions.sort_by_key(|(v, _)| var_levels[v]);
    let levels: Vec<usize> = var_positions.iter().map(|(v, _)| var_levels[v]).collect();

    let scan = graph.store.scan(id_pat);
    let mut tuples: Vec<Vec<Id>> = Vec::with_capacity(scan.rows.len());
    for row in scan.rows {
        let spo = scan.to_spo(row);
        let mut tup = Vec::with_capacity(var_positions.len());
        let mut ok = true;
        for (_, positions) in &var_positions {
            let v0 = spo[positions[0]];
            if positions.iter().any(|&p| spo[p] != v0) {
                ok = false;
                break;
            }
            tup.push(v0);
        }
        if ok {
            tuples.push(tup);
        }
    }
    tuples.sort_unstable();
    tuples.dedup();
    (Trie { tuples }, levels)
}

fn eval_bgp_wcoj(graph: &Graph, patterns: &[TriplePattern]) -> Result<Bindings, String> {
    // Prepare patterns; an unsatisfiable constant makes the whole BGP empty.
    let mut prepared: Vec<(IdPattern, [Option<Variable>; 3])> = Vec::with_capacity(patterns.len());
    for tp in patterns {
        let (id_pat, pos_vars, unsat) = prepare_pattern(graph, tp)?;
        if unsat {
            return Ok(Bindings::unsorted(collect_vars(patterns), vec![]));
        }
        prepared.push((id_pat, pos_vars));
    }

    // Global variable order: most-constrained first (highest degree, then most
    // selective). Constant-only patterns contribute no variables but must match.
    let all_vars = collect_vars(patterns);
    let degree = |v: &Variable| prepared.iter().filter(|(_, pv)| pv.iter().flatten().any(|x| x == v)).count();
    let min_est = |v: &Variable| {
        prepared
            .iter()
            .filter(|(_, pv)| pv.iter().flatten().any(|x| x == v))
            .map(|(ip, _)| graph.store.estimate(ip))
            .min()
            .unwrap_or(usize::MAX)
    };
    let mut order_vars = all_vars.clone();
    order_vars.sort_by(|a, b| degree(b).cmp(&degree(a)).then(min_est(a).cmp(&min_est(b))));
    let n_levels = order_vars.len();

    let var_levels: FxHashMap<Variable, usize> =
        order_vars.iter().enumerate().map(|(i, v)| (v.clone(), i)).collect();

    // Build a trie per pattern that has variables; check constant-only patterns
    // for existence (empty range => empty BGP).
    let mut tries: Vec<Trie> = Vec::new();
    let mut trie_levels: Vec<Vec<usize>> = Vec::new();
    for (id_pat, pos_vars) in &prepared {
        if pos_vars.iter().all(|v| v.is_none()) {
            if graph.store.estimate(id_pat) == 0 {
                return Ok(Bindings::unsorted(order_vars, vec![]));
            }
            continue;
        }
        let (trie, levels) = build_trie(graph, id_pat, pos_vars, &var_levels);
        if trie.tuples.is_empty() {
            return Ok(Bindings::unsorted(order_vars, vec![]));
        }
        tries.push(trie);
        trie_levels.push(levels);
    }

    // No variables: BGP is a ground check that already succeeded.
    if n_levels == 0 {
        return Ok(Bindings { vars: vec![], rows: vec![Row::new()], sorted_by: None });
    }

    // Participating tries per global level.
    let mut parts_at_level: Vec<Vec<usize>> = vec![Vec::new(); n_levels];
    for (ti, levels) in trie_levels.iter().enumerate() {
        for &lvl in levels {
            parts_at_level[lvl].push(ti);
        }
    }

    let mut iters: Vec<TrieIter> = tries.iter().map(TrieIter::new).collect();
    let mut out: Vec<Row> = Vec::new();
    let mut current = vec![NO_ID; n_levels];
    lftj_recurse(&mut iters, &parts_at_level, 0, n_levels, &mut current, &mut out);

    // Output rows are produced in global-order lexicographic order.
    let sorted_by = order_vars.first().cloned();
    Ok(Bindings { vars: order_vars, rows: out, sorted_by })
}

/// α-acyclicity test via GYO reduction: repeatedly drop variables that occur in
/// only one pattern and patterns whose variable set is contained in another.
/// A BGP is cyclic iff anything remains. Cyclic BGPs benefit from WCOJ.
fn bgp_is_cyclic(patterns: &[TriplePattern]) -> bool {
    let mut next_id = 0u32;
    let mut ids: FxHashMap<Variable, u32> = FxHashMap::default();
    let mut edges: Vec<std::collections::HashSet<u32>> = Vec::new();
    for tp in patterns {
        let mut e = std::collections::HashSet::new();
        for v in [tp_var(&tp.subject), nnp_var(&tp.predicate), tp_var(&tp.object)].into_iter().flatten() {
            let id = *ids.entry(v).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            });
            e.insert(id);
        }
        if !e.is_empty() {
            edges.push(e);
        }
    }

    loop {
        let mut changed = false;

        // Drop variables occurring in exactly one edge.
        let mut count: FxHashMap<u32, usize> = FxHashMap::default();
        for e in &edges {
            for &v in e {
                *count.entry(v).or_default() += 1;
            }
        }
        for e in edges.iter_mut() {
            let before = e.len();
            e.retain(|v| count[v] > 1);
            if e.len() != before {
                changed = true;
            }
        }
        let before = edges.len();
        edges.retain(|e| !e.is_empty());
        if edges.len() != before {
            changed = true;
        }

        // Drop an edge contained in a distinct edge (also removes duplicates).
        let mut removed = None;
        'outer: for i in 0..edges.len() {
            for j in 0..edges.len() {
                if i != j && edges[i].is_subset(&edges[j]) {
                    removed = Some(i);
                    break 'outer;
                }
            }
        }
        if let Some(i) = removed {
            edges.remove(i);
            changed = true;
        }

        if !changed {
            break;
        }
    }

    !edges.is_empty()
}

fn prepare_pattern(graph: &Graph, tp: &TriplePattern) -> Result<(IdPattern, [Option<Variable>; 3], bool), String> {
    let mut id_pat: IdPattern = [None, None, None];
    let mut pos_vars: [Option<Variable>; 3] = [None, None, None];
    let mut unsat = false;

    // Resolves one (subject/object) position: variable -> pos_vars; blank node ->
    // synthetic variable; concrete term -> dictionary id (absent term => unsat).
    let mut bind_term = |slot: usize, tp: &TermPattern| -> Result<(), String> {
        match tp {
            TermPattern::Variable(v) => pos_vars[slot] = Some(v.clone()),
            TermPattern::BlankNode(b) => pos_vars[slot] = Some(bnode_var(b)),
            other => match graph.id_of(&term_pattern_to_term(other)?) {
                Some(id) => id_pat[slot] = Some(id),
                None => unsat = true,
            },
        }
        Ok(())
    };
    bind_term(0, &tp.subject)?;
    bind_term(2, &tp.object)?;

    match &tp.predicate {
        NamedNodePattern::Variable(v) => pos_vars[1] = Some(v.clone()),
        NamedNodePattern::NamedNode(n) => match graph.id_of(&Term::NamedNode(n.clone())) {
            Some(id) => id_pat[1] = Some(id),
            None => unsat = true,
        },
    }
    Ok((id_pat, pos_vars, unsat))
}

fn scan_to_bindings(graph: &Graph, id_pat: &IdPattern, pos_vars: &[Option<Variable>; 3], sort_col: Option<usize>, filter: Option<(usize, NumCmp)>, limit: Option<usize>) -> Bindings {
    let mut vars: Vec<Variable> = Vec::new();
    let mut var_positions: Vec<Vec<usize>> = Vec::new();
    for (pos, v) in pos_vars.iter().enumerate() {
        if let Some(v) = v {
            if let Some(idx) = vars.iter().position(|x| x == v) {
                var_positions[idx].push(pos);
            } else {
                vars.push(v.clone());
                var_positions.push(vec![pos]);
            }
        }
    }
    let scan = match sort_col {
        Some(c) => graph.store.scan_sorted(id_pat, c),
        None => graph.store.scan(id_pat),
    };
    let sorted_by = sort_col.and_then(|c| pos_vars[c].clone());
    // Reserve only up to the LIMIT so a small LIMIT over a huge scan does not
    // allocate for the whole relation (the point of early termination).
    let cap = limit.map_or(scan.rows.len(), |n| n.min(scan.rows.len()));
    let mut rows: Vec<Row> = Vec::with_capacity(cap);
    for row in scan.rows {
        let spo = scan.to_spo(row);
        // Pushed-down numeric FILTER: evaluated inline, before materialising the
        // row. When `sort_col` is the filter column the id stream is monotonic, so
        // `numeric_value` reads sequentially instead of as a random gather.
        if let Some((fpos, cmp)) = filter {
            match graph.numeric_value(spo[fpos]) {
                Some(x) if cmp.test(x) => {}
                _ => continue,
            }
        }
        let mut ok = true;
        let mut out = Row::with_capacity(vars.len());
        for positions in &var_positions {
            let v0 = spo[positions[0]];
            if positions.iter().any(|&p| spo[p] != v0) {
                ok = false;
                break;
            }
            out.push(v0);
        }
        if ok {
            rows.push(out);
            // LIMIT early-termination: stop scanning once we have enough rows.
            if let Some(n) = limit {
                if rows.len() >= n {
                    break;
                }
            }
        }
    }
    Bindings { vars, rows, sorted_by }
}

fn merge_join(left: Bindings, right: Bindings, jv: &Variable) -> Bindings {
    let lk = left.col(jv).unwrap();
    let rk = right.col(jv).unwrap();
    let mut out_vars = left.vars.clone();
    let mut right_only: Vec<usize> = Vec::new();
    let mut extra_shared: Vec<(usize, usize)> = Vec::new();
    for (ri, v) in right.vars.iter().enumerate() {
        match left.col(v) {
            Some(li) if v != jv => extra_shared.push((li, ri)),
            Some(_) => {}
            None => {
                out_vars.push(v.clone());
                right_only.push(ri);
            }
        }
    }
    let (l, r) = (&left.rows, &right.rows);
    let mut rows = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < l.len() && j < r.len() {
        let (lv, rv) = (l[i][lk], r[j][rk]);
        match lv.cmp(&rv) {
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
            Ordering::Equal => {
                let mut i2 = i;
                while i2 < l.len() && l[i2][lk] == lv {
                    i2 += 1;
                }
                let mut j2 = j;
                while j2 < r.len() && r[j2][rk] == rv {
                    j2 += 1;
                }
                for li in i..i2 {
                    for rj in j..j2 {
                        if extra_shared.iter().all(|&(lc, rc)| l[li][lc] == r[rj][rc]) {
                            let mut row = l[li].clone();
                            for &rc in &right_only {
                                row.push(r[rj][rc]);
                            }
                            rows.push(row);
                        }
                    }
                }
                i = i2;
                j = j2;
            }
        }
    }
    Bindings { vars: out_vars, rows, sorted_by: Some(jv.clone()) }
}

/// Layout for combining two bindings: shared (left col, right col) pairs and the
/// right-only columns appended after left's vars.
fn join_layout(left: &Bindings, right: &Bindings) -> (Vec<Variable>, Vec<(usize, usize)>, Vec<usize>) {
    let mut out_vars = left.vars.clone();
    let mut shared = Vec::new();
    let mut right_only = Vec::new();
    for (ri, v) in right.vars.iter().enumerate() {
        match left.col(v) {
            Some(li) => shared.push((li, ri)),
            None => {
                out_vars.push(v.clone());
                right_only.push(ri);
            }
        }
    }
    (out_vars, shared, right_only)
}

/// SPARQL solution compatibility on the shared columns: an unbound (`NO_ID`)
/// value never conflicts; two bound values must be equal.
fn compatible(lrow: &[Id], rrow: &[Id], shared: &[(usize, usize)]) -> bool {
    shared.iter().all(|&(lc, rc)| {
        let (a, b) = (lrow[lc], rrow[rc]);
        a == NO_ID || b == NO_ID || a == b
    })
}

/// Combines two compatible rows: left's row extended with the right-only columns,
/// filling any shared column that was unbound on the left from the right side.
fn merge_rows(lrow: &[Id], rrow: &[Id], shared: &[(usize, usize)], right_only: &[usize]) -> Row {
    let mut row = Row::from_slice(lrow);
    for &(lc, rc) in shared {
        if row[lc] == NO_ID {
            row[lc] = rrow[rc];
        }
    }
    for &rc in right_only {
        row.push(rrow[rc]);
    }
    row
}

/// Whether any row leaves any of the given columns unbound.
fn any_unbound(rows: &[Row], cols: &[usize]) -> bool {
    rows.iter().any(|r| cols.iter().any(|&c| r[c] == NO_ID))
}

fn hash_join(left: Bindings, right: Bindings) -> Bindings {
    // Build the hash table on the smaller side.
    let (build, probe) = if left.rows.len() <= right.rows.len() {
        (left, right)
    } else {
        (right, left)
    };
    // Shared vars relative to (build, probe).
    let shared: Vec<(usize, usize)> = build
        .vars
        .iter()
        .enumerate()
        .filter_map(|(bi, v)| probe.col(v).map(|pi| (bi, pi)))
        .collect();
    let mut out_vars = build.vars.clone();
    let probe_only: Vec<usize> = probe
        .vars
        .iter()
        .enumerate()
        .filter(|(_, v)| !build.vars.contains(v))
        .map(|(i, v)| {
            out_vars.push(v.clone());
            i
        })
        .collect();
    let mut table: FxHashMap<Key, Posting> = FxHashMap::default();
    for (ri, row) in build.rows.iter().enumerate() {
        let key: Key = shared.iter().map(|&(bi, _)| row[bi]).collect();
        table.entry(key).or_default().push(ri);
    }
    let mut rows = Vec::new();
    for prow in &probe.rows {
        let key: Key = shared.iter().map(|&(_, pi)| prow[pi]).collect();
        if let Some(matches) = table.get(&key) {
            for &bi in matches {
                let mut combined = build.rows[bi].clone();
                for &pi in &probe_only {
                    combined.push(prow[pi]);
                }
                rows.push(combined);
            }
        }
    }
    Bindings::unsorted(out_vars, rows)
}

fn cross_product(left: Bindings, right: Bindings) -> Bindings {
    let mut out_vars = left.vars.clone();
    out_vars.extend(right.vars.iter().cloned());
    let mut rows = Vec::with_capacity(left.rows.len() * right.rows.len());
    for l in &left.rows {
        for r in &right.rows {
            let mut row = l.clone();
            row.extend_from_slice(r);
            rows.push(row);
        }
    }
    Bindings::unsorted(out_vars, rows)
}

/// Generic join used for Join of non-conjunctive sub-results. With fully-bound
/// shared columns it takes the fast path (merge if both are sorted on the join
/// var, else hash); when a shared column can be unbound (from OPTIONAL / UNION /
/// VALUES UNDEF), it falls back to a correct solution-compatibility nested loop.
fn join_bindings(left: Bindings, right: Bindings) -> Bindings {
    let (out_vars, shared, right_only) = join_layout(&left, &right);
    if shared.is_empty() {
        return cross_product(left, right);
    }
    let lcols: Vec<usize> = shared.iter().map(|&(lc, _)| lc).collect();
    let rcols: Vec<usize> = shared.iter().map(|&(_, rc)| rc).collect();
    if !any_unbound(&left.rows, &lcols) && !any_unbound(&right.rows, &rcols) {
        if let (Some(lv), Some(rv)) = (&left.sorted_by, &right.sorted_by) {
            if lv == rv && right.col(lv).is_some() {
                let jv = lv.clone();
                return merge_join(left, right, &jv);
            }
        }
        return hash_join(left, right);
    }
    let mut rows = Vec::new();
    for lrow in &left.rows {
        for rrow in &right.rows {
            if compatible(lrow, rrow, &shared) {
                rows.push(merge_rows(lrow, rrow, &shared, &right_only));
            }
        }
    }
    Bindings::unsorted(out_vars, rows)
}

// ---- OPTIONAL / UNION / MINUS / VALUES / BIND --------------------------------

fn left_outer_join(graph: &Graph, local: &mut LocalVocab, left: Bindings, right: Bindings, expr: Option<&Expression>) -> Result<Bindings, String> {
    let (out_vars, shared, right_only) = join_layout(&left, &right);
    let n_right_only = right_only.len();

    // Sort-merge left outer join for the common case — exactly one shared variable,
    // fully bound on both sides (the typical `?s p ?o OPTIONAL { ?s q ?r }`). It
    // avoids building a hash table over the entire right side (which dominates on
    // large OPTIONALs); QLever uses the same sorted-merge strategy here.
    if shared.len() == 1 {
        let (lk, rk) = shared[0];
        if !any_unbound(&left.rows, &[lk]) && !any_unbound(&right.rows, &[rk]) {
            return left_outer_merge(graph, local, left, right, lk, rk, &shared, &right_only, n_right_only, out_vars, expr);
        }
    }

    // Hash the right side by its shared-variable key when those columns are fully
    // bound; otherwise fall back to a compatibility scan (shared vars may be
    // unbound, in which case they act as wildcards).
    let lcols: Vec<usize> = shared.iter().map(|&(lc, _)| lc).collect();
    let rcols: Vec<usize> = shared.iter().map(|&(_, rc)| rc).collect();
    let table: Option<FxHashMap<Key, Posting>> =
        if !shared.is_empty() && !any_unbound(&left.rows, &lcols) && !any_unbound(&right.rows, &rcols) {
            let mut t: FxHashMap<Key, Posting> = FxHashMap::default();
            for (ri, row) in right.rows.iter().enumerate() {
                t.entry(rcols.iter().map(|&c| row[c]).collect()).or_default().push(ri);
            }
            Some(t)
        } else {
            None
        };

    let mut rows = Vec::new();
    for lrow in &left.rows {
        // Inline (no heap alloc per left row): an OPTIONAL key usually has 0–1
        // matches. The hashed branch copies the matching indices by value rather
        // than cloning the table's Vec.
        let candidates: SmallVec<[usize; 4]> = match &table {
            Some(t) => {
                let key: Key = lcols.iter().map(|&c| lrow[c]).collect();
                t.get(&key).map(|v| v.iter().copied().collect()).unwrap_or_default()
            }
            None => (0..right.rows.len()).filter(|&ri| compatible(lrow, &right.rows[ri], &shared)).collect(),
        };
        let mut matched = false;
        for ri in candidates {
            let combined = merge_rows(lrow, &right.rows[ri], &shared, &right_only);
            // OPTIONAL's filter is part of the join condition (evaluated on the
            // combined row); a row that fails it does not count as a match.
            let keep = match expr {
                None => true,
                Some(e) => {
                    let tmp = Bindings { vars: out_vars.clone(), rows: vec![], sorted_by: None };
                    effective_boolean(&eval_expr(graph, local, &tmp, &combined, e)?)
                }
            };
            if keep {
                rows.push(combined);
                matched = true;
            }
        }
        if !matched {
            let mut combined = lrow.clone();
            combined.extend(std::iter::repeat(NO_ID).take(n_right_only));
            rows.push(combined);
        }
    }
    Ok(Bindings::unsorted(out_vars, rows))
}

/// Sort-merge left outer join on a single shared variable (`lk`/`rk` columns).
#[allow(clippy::too_many_arguments)]
fn left_outer_merge(
    graph: &Graph,
    local: &mut LocalVocab,
    mut left: Bindings,
    mut right: Bindings,
    lk: usize,
    rk: usize,
    shared: &[(usize, usize)],
    right_only: &[usize],
    n_right_only: usize,
    out_vars: Vec<Variable>,
    expr: Option<&Expression>,
) -> Result<Bindings, String> {
    // Both sides come from index scans that are usually already key-sorted, so
    // these sorts are near-linear (pattern-defeating quicksort detects sortedness).
    left.rows.sort_unstable_by_key(|r| r[lk]);
    right.rows.sort_unstable_by_key(|r| r[rk]);
    let (l, r) = (&left.rows, &right.rows);
    let mut rows: Vec<Row> = Vec::with_capacity(l.len());
    let mut j = 0usize;
    let mut i = 0usize;
    while i < l.len() {
        let key = l[i][lk];
        let mut i2 = i + 1;
        while i2 < l.len() && l[i2][lk] == key {
            i2 += 1;
        }
        while j < r.len() && r[j][rk] < key {
            j += 1;
        }
        let mut j2 = j;
        while j2 < r.len() && r[j2][rk] == key {
            j2 += 1;
        }
        for li in i..i2 {
            let mut matched = false;
            for rj in j..j2 {
                let combined = merge_rows(&l[li], &r[rj], shared, right_only);
                let keep = match expr {
                    None => true,
                    Some(e) => {
                        let tmp = Bindings { vars: out_vars.clone(), rows: vec![], sorted_by: None };
                        effective_boolean(&eval_expr(graph, local, &tmp, &combined, e)?)
                    }
                };
                if keep {
                    rows.push(combined);
                    matched = true;
                }
            }
            if !matched {
                let mut combined = l[li].clone();
                combined.extend(std::iter::repeat(NO_ID).take(n_right_only));
                rows.push(combined);
            }
        }
        i = i2;
        j = j2;
    }
    // Output is ordered by the join variable (at column lk of out_vars).
    let sorted_by = out_vars.get(lk).cloned();
    Ok(Bindings { vars: out_vars, rows, sorted_by })
}

fn union_bindings(left: Bindings, right: Bindings) -> Bindings {
    let mut out_vars = left.vars.clone();
    for v in &right.vars {
        if !out_vars.contains(v) {
            out_vars.push(v.clone());
        }
    }
    let mut rows: Vec<Row> = Vec::with_capacity(left.rows.len() + right.rows.len());
    let map_row = |src_vars: &[Variable], row: &[Id], out_vars: &[Variable]| -> Row {
        out_vars
            .iter()
            .map(|v| src_vars.iter().position(|x| x == v).map(|i| row[i]).unwrap_or(NO_ID))
            .collect()
    };
    for row in &left.rows {
        rows.push(map_row(&left.vars, row, &out_vars));
    }
    for row in &right.rows {
        rows.push(map_row(&right.vars, row, &out_vars));
    }
    Bindings::unsorted(out_vars, rows)
}

fn minus_bindings(left: Bindings, right: Bindings) -> Bindings {
    // SPARQL MINUS: drop a left row iff some right row is *compatible* with it AND
    // their bound domains overlap on at least one shared variable. (Disjoint
    // domains never remove anything.)
    let shared: Vec<(usize, usize)> = left
        .vars
        .iter()
        .enumerate()
        .filter_map(|(li, v)| right.col(v).map(|ri| (li, ri)))
        .collect();
    if shared.is_empty() {
        return left; // disjoint domains -> MINUS removes nothing
    }
    let lcols: Vec<usize> = shared.iter().map(|&(lc, _)| lc).collect();
    let rcols: Vec<usize> = shared.iter().map(|&(_, rc)| rc).collect();

    // Fast path: shared columns fully bound -> compatibility is exact-key equality
    // and the domains always overlap, so membership in a hash set suffices.
    if !any_unbound(&left.rows, &lcols) && !any_unbound(&right.rows, &rcols) {
        let mut table: FxHashMap<Key, ()> = FxHashMap::default();
        for row in &right.rows {
            table.insert(rcols.iter().map(|&c| row[c]).collect(), ());
        }
        let rows: Vec<Row> = left
            .rows
            .into_iter()
            .filter(|row| !table.contains_key(&lcols.iter().map(|&c| row[c]).collect::<Key>()))
            .collect();
        return Bindings { vars: left.vars, rows, sorted_by: left.sorted_by };
    }

    // General path: per-row compatibility with a bound-domain-overlap check.
    let keep = |lrow: &Row| -> bool {
        !right.rows.iter().any(|rrow| {
            let mut overlap = false;
            for &(lc, rc) in &shared {
                let (a, b) = (lrow[lc], rrow[rc]);
                if a != NO_ID && b != NO_ID {
                    if a != b {
                        return false; // incompatible
                    }
                    overlap = true;
                }
            }
            overlap
        })
    };
    let rows: Vec<Row> = left.rows.iter().filter(|r| keep(r)).cloned().collect();
    Bindings { vars: left.vars, rows, sorted_by: left.sorted_by }
}

fn values_bindings(graph: &Graph, local: &mut LocalVocab, variables: &[Variable], bindings: &[Vec<Option<GroundTerm>>]) -> Bindings {
    let rows = bindings
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| match cell {
                    None => NO_ID,
                    Some(gt) => {
                        // Resolve against the graph dictionary so the id joins with
                        // BGP results; a term absent from the graph gets a local id
                        // (it can only match other VALUES, never the data).
                        let t = ground_to_term(gt);
                        graph.id_of(&t).unwrap_or_else(|| local.intern(t))
                    }
                })
                .collect()
        })
        .collect();
    Bindings::unsorted(variables.to_vec(), rows)
}

fn extend_bindings(graph: &Graph, local: &mut LocalVocab, mut b: Bindings, var: &Variable, expr: &Expression) -> Result<Bindings, String> {
    let mut col = Vec::with_capacity(b.rows.len());
    for row in &b.rows {
        let v = eval_expr(graph, local, &b, row, expr)?;
        col.push(value_to_id(graph, local, &v));
    }
    b.vars.push(var.clone());
    for (row, id) in b.rows.iter_mut().zip(col) {
        row.push(id);
    }
    b.sorted_by = None;
    Ok(b)
}

// ---- Aggregation --------------------------------------------------------------

fn group_aggregate(
    graph: &Graph,
    local: &mut LocalVocab,
    b: Bindings,
    group_vars: &[Variable],
    aggregates: &[(Variable, AggregateExpression)],
) -> Result<Bindings, String> {
    let key_cols: Vec<usize> = group_vars.iter().map(|v| b.col(v).expect("group var present")).collect();

    // Group rows by the group-key id tuple, preserving first-seen order.
    let mut groups: FxHashMap<Key, Vec<usize>> = FxHashMap::default();
    let mut order: Vec<Key> = Vec::new();
    for (ri, row) in b.rows.iter().enumerate() {
        let key: Key = key_cols.iter().map(|&c| row[c]).collect();
        groups.entry(key.clone()).or_insert_with(|| {
            order.push(key.clone());
            Vec::new()
        });
        groups.get_mut(&key).unwrap().push(ri);
    }
    // Whole-dataset aggregate with no GROUP BY: one (empty) group, even if input is empty.
    if group_vars.is_empty() && order.is_empty() {
        order.push(Key::new());
        groups.insert(Key::new(), vec![]);
    }

    let mut out_vars: Vec<Variable> = group_vars.to_vec();
    for (v, _) in aggregates {
        out_vars.push(v.clone());
    }

    let mut rows: Vec<Row> = Vec::with_capacity(order.len());
    for key in &order {
        let members = &groups[key];
        let mut row = Row::from_slice(key);
        for (_, agg) in aggregates {
            let v = eval_aggregate(graph, local, &b, members, agg)?;
            row.push(value_to_id(graph, local, &v));
        }
        rows.push(row);
    }
    Ok(Bindings::unsorted(out_vars, rows))
}

fn eval_aggregate(graph: &Graph, local: &mut LocalVocab, b: &Bindings, members: &[usize], agg: &AggregateExpression) -> Result<Value, String> {
    match agg {
        AggregateExpression::CountSolutions { distinct } => {
            let n = if *distinct {
                let mut seen = std::collections::HashSet::new();
                members.iter().filter(|&&ri| seen.insert(b.rows[ri].clone())).count()
            } else {
                members.len()
            };
            Ok(Value::Num(n as f64))
        }
        AggregateExpression::FunctionCall { name, expr, distinct } => {
            // Collect the per-member values of `expr`.
            let mut vals: Vec<Value> = Vec::with_capacity(members.len());
            for &ri in members {
                let v = eval_expr(graph, local, b, &b.rows[ri], expr)?;
                if !matches!(v, Value::Unbound) {
                    vals.push(v);
                }
            }
            if *distinct {
                dedup_values(&mut vals);
            }
            match name {
                AggregateFunction::Count => Ok(Value::Num(vals.len() as f64)),
                AggregateFunction::Sum => Ok(Value::Num(vals.iter().filter_map(as_num).sum())),
                AggregateFunction::Avg => {
                    let nums: Vec<f64> = vals.iter().filter_map(as_num).collect();
                    Ok(if nums.is_empty() { Value::Num(0.0) } else { Value::Num(nums.iter().sum::<f64>() / nums.len() as f64) })
                }
                AggregateFunction::Min => Ok(vals.into_iter().min_by(|a, c| compare_values(a, c).unwrap_or(Ordering::Equal)).unwrap_or(Value::Unbound)),
                AggregateFunction::Max => Ok(vals.into_iter().max_by(|a, c| compare_values(a, c).unwrap_or(Ordering::Equal)).unwrap_or(Value::Unbound)),
                AggregateFunction::GroupConcat { separator } => {
                    let sep = separator.clone().unwrap_or_else(|| " ".to_string());
                    let joined = vals.iter().filter_map(value_str).collect::<Vec<_>>().join(&sep);
                    Ok(Value::Term(Term::Literal(Literal::new_simple_literal(joined))))
                }
                _ => Err("M2: unsupported aggregate".into()),
            }
        }
    }
}

fn dedup_values(vals: &mut Vec<Value>) {
    let mut seen = std::collections::HashSet::new();
    vals.retain(|v| seen.insert(value_key(v)));
}

fn value_key(v: &Value) -> String {
    match v {
        Value::Term(t) => format!("T{t}"),
        Value::Num(n) => format!("N{n}"),
        Value::Bool(b) => format!("B{b}"),
        Value::Unbound => "U".to_string(),
    }
}

// ---- Modifiers ----------------------------------------------------------------

fn project_bindings(b: Bindings, vars: &[Variable]) -> Bindings {
    let cols: Vec<Option<usize>> = vars.iter().map(|v| b.col(v)).collect();
    let rows = b
        .rows
        .iter()
        .map(|row| cols.iter().map(|c| c.map(|i| row[i]).unwrap_or(NO_ID)).collect())
        .collect();
    let sorted_by = b.sorted_by.filter(|sv| vars.contains(sv));
    Bindings { vars: vars.to_vec(), rows, sorted_by }
}

fn distinct_bindings(b: &mut Bindings) {
    let mut seen = std::collections::HashSet::new();
    b.rows.retain(|r| seen.insert(r.clone()));
}

fn slice_bindings(b: &mut Bindings, start: usize, length: Option<usize>) {
    if start > 0 {
        b.rows.drain(0..start.min(b.rows.len()));
    }
    if let Some(l) = length {
        b.rows.truncate(l);
    }
}

fn order_bindings(graph: &Graph, local: &LocalVocab, b: &mut Bindings, exprs: &[OrderExpression]) -> Result<(), String> {
    // Precompute the sort key (vector of (descending, Value)) for each row.
    let mut keyed: Vec<(Vec<(bool, Value)>, Row)> = Vec::with_capacity(b.rows.len());
    for row in &b.rows {
        let mut key = Vec::with_capacity(exprs.len());
        for oe in exprs {
            let (desc, e) = match oe {
                OrderExpression::Asc(e) => (false, e),
                OrderExpression::Desc(e) => (true, e),
            };
            // Numeric sort keys use the cache (no per-comparison reparse); other
            // expressions fall back to the identity-preserving term evaluation.
            let v = match eval_numeric(graph, local, b, row, e) {
                Some(n) => Value::Num(n),
                None => eval_expr(graph, local, b, row, e)?,
            };
            key.push((desc, v));
        }
        keyed.push((key, row.clone()));
    }
    keyed.sort_by(|a, c| {
        for ((desc, av), (_, cv)) in a.0.iter().zip(c.0.iter()) {
            let ord = compare_values(av, cv).unwrap_or(Ordering::Equal);
            let ord = if *desc { ord.reverse() } else { ord };
            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    });
    b.rows = keyed.into_iter().map(|(_, r)| r).collect();
    b.sorted_by = None;
    Ok(())
}

// ---- FILTER + expression evaluation ------------------------------------------

fn apply_filter(graph: &Graph, local: &LocalVocab, b: &mut Bindings, expr: &Expression) -> Result<(), String> {
    let mut keep = Vec::with_capacity(b.rows.len());
    for row in &b.rows {
        keep.push(effective_boolean(&eval_expr(graph, local, b, row, expr)?));
    }
    let mut i = 0;
    b.rows.retain(|_| {
        let k = keep[i];
        i += 1;
        k
    });
    Ok(())
}

#[derive(Clone, Debug)]
enum Value {
    Bool(bool),
    Num(f64),
    Term(Term),
    Unbound,
}

fn effective_boolean(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Num(n) => *n != 0.0 && !n.is_nan(),
        Value::Unbound => false,
        Value::Term(Term::Literal(l)) => {
            let dt = l.datatype().as_str();
            if dt == xsd::BOOLEAN.as_str() {
                matches!(l.value(), "true" | "1")
            } else if is_numeric_dt(l) {
                l.value().parse::<f64>().map(|n| n != 0.0 && !n.is_nan()).unwrap_or(false)
            } else if dt == xsd::STRING.as_str() || dt == "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString" {
                !l.value().is_empty()
            } else {
                false
            }
        }
        Value::Term(_) => false,
    }
}

fn eval_expr(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], e: &Expression) -> Result<Value, String> {
    use Expression::*;
    match e {
        Variable(v) => match b.col(v) {
            // Always return the original term so term identity is preserved
            // (sameTerm, BIND passthrough, STR, etc.). The numeric fast path that
            // skips this materialisation lives in `eval_numeric`, used only by the
            // arithmetic / comparison operators where only the value matters.
            Some(c) if row[c] != NO_ID => Ok(Value::Term(term_of(graph, local, row[c]).unwrap())),
            _ => Ok(Value::Unbound),
        },
        NamedNode(n) => Ok(Value::Term(Term::NamedNode(n.clone()))),
        Literal(l) => Ok(Value::Term(Term::Literal(l.clone()))),
        And(a, c) => Ok(Value::Bool(eval_bool(graph, local, b, row, a)? && eval_bool(graph, local, b, row, c)?)),
        Or(a, c) => Ok(Value::Bool(eval_bool(graph, local, b, row, a)? || eval_bool(graph, local, b, row, c)?)),
        Not(a) => Ok(Value::Bool(!eval_bool(graph, local, b, row, a)?)),
        Equal(a, c) => cmp_expr(graph, local, b, row, a, c, |o| o == Ordering::Equal),
        SameTerm(a, c) => {
            let (x, y) = (eval_expr(graph, local, b, row, a)?, eval_expr(graph, local, b, row, c)?);
            Ok(Value::Bool(matches!((&x, &y), (Value::Term(p), Value::Term(q)) if p == q)))
        }
        Greater(a, c) => cmp_expr(graph, local, b, row, a, c, |o| o == Ordering::Greater),
        GreaterOrEqual(a, c) => cmp_expr(graph, local, b, row, a, c, |o| o != Ordering::Less),
        Less(a, c) => cmp_expr(graph, local, b, row, a, c, |o| o == Ordering::Less),
        LessOrEqual(a, c) => cmp_expr(graph, local, b, row, a, c, |o| o != Ordering::Greater),
        Add(a, c) => arith(graph, local, b, row, a, c, |x, y| x + y),
        Subtract(a, c) => arith(graph, local, b, row, a, c, |x, y| x - y),
        Multiply(a, c) => arith(graph, local, b, row, a, c, |x, y| x * y),
        Divide(a, c) => arith(graph, local, b, row, a, c, |x, y| x / y),
        UnaryPlus(a) => eval_expr(graph, local, b, row, a),
        UnaryMinus(a) => {
            let v = eval_expr(graph, local, b, row, a)?;
            Ok(as_num(&v).map(|n| Value::Num(-n)).unwrap_or(Value::Unbound))
        }
        Bound(v) => Ok(Value::Bool(b.col(v).map(|c| row[c] != NO_ID).unwrap_or(false))),
        If(cond, t, f) => {
            if eval_bool(graph, local, b, row, cond)? {
                eval_expr(graph, local, b, row, t)
            } else {
                eval_expr(graph, local, b, row, f)
            }
        }
        Coalesce(es) => {
            for e in es {
                let v = eval_expr(graph, local, b, row, e)?;
                if !matches!(v, Value::Unbound) {
                    return Ok(v);
                }
            }
            Ok(Value::Unbound)
        }
        In(a, list) => {
            let x = eval_expr(graph, local, b, row, a)?;
            for c in list {
                let y = eval_expr(graph, local, b, row, c)?;
                if compare_values(&x, &y) == Some(Ordering::Equal) {
                    return Ok(Value::Bool(true));
                }
            }
            Ok(Value::Bool(false))
        }
        other => Err(format!("M2: unsupported expression: {other:?}")),
    }
}

fn eval_bool(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], e: &Expression) -> Result<bool, String> {
    Ok(effective_boolean(&eval_expr(graph, local, b, row, e)?))
}

/// Fast numeric evaluation that never materialises a term: a numeric variable
/// resolves to its value via the dictionary cache, a numeric literal via one
/// parse, and arithmetic recurses. Returns `None` for anything non-numeric, so
/// the caller falls back to the full (identity-preserving) `eval_expr` path.
/// Used only where the value — not the term — matters (comparison / arithmetic).
fn eval_numeric(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], e: &Expression) -> Option<f64> {
    use Expression::*;
    match e {
        Variable(v) => {
            let c = b.col(v)?;
            let id = row[c];
            if id == NO_ID {
                None
            } else if is_local(id) {
                // Computed values are rare; resolve through the local vocab term.
                as_num(&Value::Term(local.term(id).clone()))
            } else {
                graph.numeric_value(id)
            }
        }
        Literal(l) if is_numeric_dt(l) => l.value().parse::<f64>().ok(),
        Add(a, c) => Some(eval_numeric(graph, local, b, row, a)? + eval_numeric(graph, local, b, row, c)?),
        Subtract(a, c) => Some(eval_numeric(graph, local, b, row, a)? - eval_numeric(graph, local, b, row, c)?),
        Multiply(a, c) => Some(eval_numeric(graph, local, b, row, a)? * eval_numeric(graph, local, b, row, c)?),
        Divide(a, c) => Some(eval_numeric(graph, local, b, row, a)? / eval_numeric(graph, local, b, row, c)?),
        UnaryPlus(a) => eval_numeric(graph, local, b, row, a),
        UnaryMinus(a) => Some(-eval_numeric(graph, local, b, row, a)?),
        _ => None,
    }
}

fn cmp_expr(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], a: &Expression, c: &Expression, f: impl Fn(Ordering) -> bool) -> Result<Value, String> {
    // Fast path: both sides numeric -> compare f64 directly, no term materialised.
    if let (Some(x), Some(y)) = (eval_numeric(graph, local, b, row, a), eval_numeric(graph, local, b, row, c)) {
        return Ok(Value::Bool(x.partial_cmp(&y).map(&f).unwrap_or(false)));
    }
    let (x, y) = (eval_expr(graph, local, b, row, a)?, eval_expr(graph, local, b, row, c)?);
    Ok(Value::Bool(compare_values(&x, &y).map(f).unwrap_or(false)))
}

fn arith(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], a: &Expression, c: &Expression, f: impl Fn(f64, f64) -> f64) -> Result<Value, String> {
    if let (Some(x), Some(y)) = (eval_numeric(graph, local, b, row, a), eval_numeric(graph, local, b, row, c)) {
        return Ok(Value::Num(f(x, y)));
    }
    let (x, y) = (eval_expr(graph, local, b, row, a)?, eval_expr(graph, local, b, row, c)?);
    Ok(match (as_num(&x), as_num(&y)) {
        (Some(a), Some(b)) => Value::Num(f(a, b)),
        _ => Value::Unbound,
    })
}

fn as_num(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(*n),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::Term(Term::Literal(l)) if is_numeric_dt(l) => l.value().parse::<f64>().ok(),
        _ => None,
    }
}

fn is_numeric_dt(l: &Literal) -> bool {
    let dt = l.datatype().as_str();
    dt == xsd::INTEGER.as_str()
        || dt == xsd::DECIMAL.as_str()
        || dt == xsd::DOUBLE.as_str()
        || dt == xsd::FLOAT.as_str()
        || dt == xsd::LONG.as_str()
        || dt == xsd::INT.as_str()
        || dt == xsd::SHORT.as_str()
        || dt == xsd::BYTE.as_str()
        || dt == xsd::NON_NEGATIVE_INTEGER.as_str()
        || dt == xsd::POSITIVE_INTEGER.as_str()
        || dt == xsd::UNSIGNED_INT.as_str()
        || dt == xsd::UNSIGNED_LONG.as_str()
}

fn compare_values(x: &Value, y: &Value) -> Option<Ordering> {
    if let (Some(a), Some(b)) = (as_num(x), as_num(y)) {
        return a.partial_cmp(&b);
    }
    Some(value_str(x)?.cmp(&value_str(y)?))
}

fn value_str(v: &Value) -> Option<String> {
    match v {
        Value::Term(Term::Literal(l)) => Some(l.value().to_string()),
        Value::Term(Term::NamedNode(n)) => Some(n.as_str().to_string()),
        Value::Term(Term::BlankNode(b)) => Some(b.as_str().to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Num(n) => Some(fmt_num(*n)),
        Value::Unbound => None,
        Value::Term(_) => None,
    }
}

fn fmt_num(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

/// Converts an evaluated value into an id. Computed terms are resolved against
/// the graph dictionary first (so they join and deduplicate against the data);
/// terms not already present get a per-query local id.
fn value_to_id(graph: &Graph, local: &mut LocalVocab, v: &Value) -> Id {
    let term = match v {
        Value::Unbound => return NO_ID,
        Value::Bool(b) => Term::Literal(Literal::new_typed_literal(b.to_string(), xsd::BOOLEAN)),
        Value::Num(n) => Term::Literal(if n.fract() == 0.0 && n.is_finite() {
            Literal::new_typed_literal((*n as i64).to_string(), xsd::INTEGER)
        } else {
            Literal::new_typed_literal(n.to_string(), xsd::DOUBLE)
        }),
        Value::Term(t) => t.clone(),
    };
    graph.id_of(&term).unwrap_or_else(|| local.intern(term))
}

// ---- spargebra term helpers --------------------------------------------------

fn ground_to_term(g: &GroundTerm) -> Term {
    match g {
        GroundTerm::NamedNode(n) => Term::NamedNode(n.clone()),
        GroundTerm::Literal(l) => Term::Literal(l.clone()),
        other => panic!("unsupported ground term: {other:?}"),
    }
}

fn term_pattern_to_term(tp: &TermPattern) -> Result<Term, String> {
    match tp {
        TermPattern::NamedNode(n) => Ok(Term::NamedNode(n.clone())),
        TermPattern::BlankNode(b) => Ok(Term::BlankNode(b.clone())),
        TermPattern::Literal(l) => Ok(Term::Literal(l.clone())),
        TermPattern::Variable(_) => Err("variable where a term was expected".into()),
        other => Err(format!("unsupported term pattern: {other:?}")),
    }
}

/// Prefix for the synthetic variables that stand in for blank nodes (which are
/// existential variables in a query). `#` cannot appear in a SPARQL `VARNAME`,
/// so these can never collide with a user variable, and the `SELECT *` filter on
/// this prefix can never hide a real one.
const BNODE_VAR_PREFIX: &str = "#bn#";

fn bnode_var(b: &oxrdf::BlankNode) -> Variable {
    Variable::new_unchecked(format!("{BNODE_VAR_PREFIX}{}", b.as_str()))
}

fn tp_var(tp: &TermPattern) -> Option<Variable> {
    match tp {
        TermPattern::Variable(v) => Some(v.clone()),
        TermPattern::BlankNode(b) => Some(bnode_var(b)),
        _ => None,
    }
}

fn nnp_var(p: &NamedNodePattern) -> Option<Variable> {
    match p {
        NamedNodePattern::Variable(v) => Some(v.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod wcoj_tests {
    use super::*;
    use spargebra::SparqlParser;

    /// Extracts the BGP triple patterns from a simple `SELECT * WHERE { ... }`.
    fn bgp(sparql: &str) -> Vec<TriplePattern> {
        let q = SparqlParser::new().parse_query(sparql).unwrap();
        let spargebra::Query::Select { pattern, .. } = q else { panic!() };
        let mut patterns = Vec::new();
        let mut filters = Vec::new();
        // unwrap Project -> inner
        fn inner_of(p: &GraphPattern) -> &GraphPattern {
            match p {
                GraphPattern::Project { inner, .. }
                | GraphPattern::Distinct { inner }
                | GraphPattern::Slice { inner, .. } => inner_of(inner),
                other => other,
            }
        }
        flatten_conjunction(inner_of(&pattern), &mut patterns, &mut filters);
        patterns
    }

    /// Sorted set of result rows from a Bindings, for order-independent equality.
    fn rowset(b: &Bindings) -> Vec<Vec<(String, Id)>> {
        let mut rows: Vec<Vec<(String, Id)>> = b
            .rows
            .iter()
            .map(|r| {
                let mut kv: Vec<(String, Id)> =
                    b.vars.iter().zip(r).map(|(v, &id)| (v.as_str().to_string(), id)).collect();
                kv.sort();
                kv
            })
            .collect();
        rows.sort();
        rows
    }

    fn random_graph(seed0: u64, n_nodes: u32, n_edges: usize) -> Graph {
        let mut seed = seed0;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as u32
        };
        let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
        for _ in 0..n_edges {
            let a = next() % n_nodes;
            let b = next() % n_nodes;
            ttl.push_str(&format!("ex:n{a} ex:e ex:n{b} .\n"));
        }
        Graph::load_str(&ttl, "turtle").unwrap()
    }

    /// The binary and WCOJ BGP plans must return identical result sets.
    fn assert_plans_agree(sparql: &str, graph: &Graph) {
        let patterns = bgp(sparql);
        let binary = eval_bgp_binary(graph, &patterns, &[]).unwrap();
        let wcoj = eval_bgp_wcoj(graph, &patterns).unwrap();
        assert_eq!(
            rowset(&binary),
            rowset(&wcoj),
            "binary vs WCOJ disagree for `{sparql}` (binary {} rows, wcoj {} rows)",
            binary.rows.len(),
            wcoj.rows.len()
        );
    }

    #[test]
    fn cyclicity_classification() {
        assert!(!bgp_is_cyclic(&bgp("PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c }")));
        assert!(bgp_is_cyclic(&bgp(
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?a }"
        )));
        assert!(bgp_is_cyclic(&bgp(
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?d . ?d ex:e ?a }"
        )));
        // star is acyclic
        assert!(!bgp_is_cyclic(&bgp(
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?a ex:e ?c . ?a ex:e ?d }"
        )));
    }

    #[test]
    fn wcoj_matches_binary_over_random_graphs() {
        for seed in [0x1111u64, 0xACE1, 0xDEADBEEF, 0x5EED] {
            let g = random_graph(seed, 25, 120);
            // chain (acyclic)
            assert_plans_agree("PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c }", &g);
            // triangle (cyclic) — the canonical WCOJ win
            assert_plans_agree(
                "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?a }",
                &g,
            );
            // 4-cycle
            assert_plans_agree(
                "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?d . ?d ex:e ?a }",
                &g,
            );
            // square with a diagonal (denser cycle)
            assert_plans_agree(
                "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?a . ?a ex:e ?d . ?d ex:e ?c }",
                &g,
            );
        }
    }

    #[test]
    fn wcoj_repeated_variable_pattern() {
        // self-loops: ?x ex:e ?x  — repeated-variable handling in build_trie.
        let g = Graph::load_str(
            "@prefix ex: <http://ex/> . ex:a ex:e ex:a . ex:a ex:e ex:b . ex:b ex:e ex:b .",
            "turtle",
        )
        .unwrap();
        let patterns = bgp("PREFIX ex: <http://ex/> SELECT * WHERE { ?x ex:e ?x }");
        let wcoj = eval_bgp_wcoj(&g, &patterns).unwrap();
        assert_eq!(wcoj.rows.len(), 2); // a and b
    }
}
