//! Physical execution for M1: BGP via greedy-ordered hash joins over the
//! permutation indexes, FILTER, and the solution modifiers.

use crate::QueryResult;
use oxrdf::{NamedNode, Term, Variable};
use rustc_hash::FxHashMap;
use sparq_core::dict::Id;
use sparq_core::store::Pattern as IdPattern;
use sparq_core::Graph;
use spargebra::algebra::{Expression, GraphPattern};
use spargebra::term::{NamedNodePattern, TermPattern, TriplePattern};

/// Intermediate result: a set of rows, each an id per `vars[i]`.
struct Bindings {
    vars: Vec<Variable>,
    rows: Vec<Vec<Id>>,
}

impl Bindings {
    fn col(&self, v: &Variable) -> Option<usize> {
        self.vars.iter().position(|x| x == v)
    }
}

/// Solution row exposed to callers (kept minimal for now).
pub type Solution = Vec<Option<Term>>;

pub fn eval_select(graph: &Graph, pattern: &GraphPattern) -> Result<QueryResult, String> {
    // Peel solution modifiers (Project/Distinct/Reduced/Slice), then evaluate
    // the conjunctive core (Join/Filter/Bgp).
    let mut project: Option<Vec<Variable>> = None;
    let mut distinct = false;
    let mut offset = 0usize;
    let mut limit: Option<usize> = None;
    let mut core = pattern;

    loop {
        match core {
            GraphPattern::Project { inner, variables } => {
                if project.is_none() {
                    project = Some(variables.clone());
                }
                core = inner;
            }
            GraphPattern::Distinct { inner } => {
                distinct = true;
                core = inner;
            }
            GraphPattern::Reduced { inner } => {
                core = inner;
            }
            GraphPattern::Slice { inner, start, length } => {
                offset = *start;
                limit = *length;
                core = inner;
            }
            _ => break,
        }
    }

    // Collect the conjunctive BGP + filters.
    let mut patterns: Vec<TriplePattern> = Vec::new();
    let mut filters: Vec<Expression> = Vec::new();
    collect_conjunction(core, &mut patterns, &mut filters)?;

    let mut bindings = eval_bgp(graph, &patterns)?;

    for f in &filters {
        apply_filter(graph, &mut bindings, f)?;
    }

    // Projection variable order. SELECT * exposes only real query variables,
    // never the synthetic variables we use for query blank nodes.
    let out_vars: Vec<Variable> = match project {
        Some(v) => v,
        None => bindings
            .vars
            .iter()
            .filter(|v| !v.as_str().starts_with(BNODE_VAR_PREFIX))
            .cloned()
            .collect(),
    };

    // Map each output row to terms.
    let col_of: Vec<Option<usize>> = out_vars.iter().map(|v| bindings.col(v)).collect();
    let mut rows: Vec<Vec<Option<Term>>> = Vec::with_capacity(bindings.rows.len());
    for row in &bindings.rows {
        let out: Vec<Option<Term>> = col_of
            .iter()
            .map(|c| c.map(|i| graph.dict.term(row[i]).clone()))
            .collect();
        rows.push(out);
    }

    if distinct {
        dedup_rows(&mut rows);
    }

    // OFFSET / LIMIT.
    if offset > 0 {
        rows.drain(0..offset.min(rows.len()));
    }
    if let Some(l) = limit {
        rows.truncate(l);
    }

    Ok(QueryResult { vars: out_vars, rows })
}

/// Flattens a tree of Join/Filter/Bgp into a triple-pattern set + filter list.
/// Anything else (OPTIONAL/UNION/paths/...) is deferred to a later milestone.
fn collect_conjunction(
    p: &GraphPattern,
    patterns: &mut Vec<TriplePattern>,
    filters: &mut Vec<Expression>,
) -> Result<(), String> {
    match p {
        GraphPattern::Bgp { patterns: tps } => {
            patterns.extend(tps.iter().cloned());
            Ok(())
        }
        GraphPattern::Join { left, right } => {
            collect_conjunction(left, patterns, filters)?;
            collect_conjunction(right, patterns, filters)
        }
        GraphPattern::Filter { expr, inner } => {
            collect_conjunction(inner, patterns, filters)?;
            filters.push(expr.clone());
            Ok(())
        }
        other => Err(format!("M1 does not support this pattern yet: {other:?}")),
    }
}

/// Evaluates a BGP into bindings using greedy join ordering and hash joins.
fn eval_bgp(graph: &Graph, patterns: &[TriplePattern]) -> Result<Bindings, String> {
    if patterns.is_empty() {
        return Ok(Bindings { vars: vec![], rows: vec![vec![]] }); // one empty solution
    }

    // Resolve each pattern to (id-pattern, var layout) and its estimate.
    struct Prepared {
        id_pat: IdPattern,
        // variable at each of the 3 positions (None if bound/constant)
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

    // Any unsatisfiable (bound term not in dict) pattern -> empty result.
    if prepared.iter().any(|p| p.unsatisfiable) {
        let vars = collect_vars(patterns);
        return Ok(Bindings { vars, rows: vec![] });
    }

    // Greedy: start with the smallest pattern.
    let mut order: Vec<usize> = (0..prepared.len()).collect();
    order.sort_by_key(|&i| prepared[i].est);

    let first = order[0];
    let mut result = scan_to_bindings(graph, &prepared[first].id_pat, &prepared[first].pos_vars);

    let mut done = vec![false; prepared.len()];
    done[first] = true;

    for _ in 1..prepared.len() {
        // Choose the next pattern: smallest estimate that shares a variable with
        // the current result (fall back to smallest remaining if disconnected).
        let mut best: Option<usize> = None;
        let mut best_connected = false;
        for &i in &order {
            if done[i] {
                continue;
            }
            let connected = prepared[i]
                .pos_vars
                .iter()
                .flatten()
                .any(|v| result.vars.contains(v));
            if connected {
                best = Some(i);
                best_connected = true;
                break;
            }
            if best.is_none() {
                best = Some(i);
            }
        }
        let i = best.unwrap();
        done[i] = true;
        let rhs = scan_to_bindings(graph, &prepared[i].id_pat, &prepared[i].pos_vars);
        result = if best_connected {
            hash_join(result, rhs)
        } else {
            cross_product(result, rhs)
        };
        if result.rows.is_empty() {
            break;
        }
    }

    Ok(result)
}

fn collect_vars(patterns: &[TriplePattern]) -> Vec<Variable> {
    let mut vars = Vec::new();
    for tp in patterns {
        for v in [tp_var(&tp.subject), nnp_var(&tp.predicate), tp_var(&tp.object)]
            .into_iter()
            .flatten()
        {
            if !vars.contains(&v) {
                vars.push(v);
            }
        }
    }
    vars
}

/// Resolves a triple pattern to an id-pattern + per-position variables.
/// Returns `unsatisfiable=true` if a bound term is absent from the dictionary.
fn prepare_pattern(
    graph: &Graph,
    tp: &TriplePattern,
) -> Result<(IdPattern, [Option<Variable>; 3], bool), String> {
    let mut id_pat: IdPattern = [None, None, None];
    let mut pos_vars: [Option<Variable>; 3] = [None, None, None];
    let mut unsat = false;

    // subject
    match &tp.subject {
        TermPattern::Variable(v) => pos_vars[0] = Some(v.clone()),
        // A blank node in a BGP is an existential variable, scoped to the query.
        TermPattern::BlankNode(b) => pos_vars[0] = Some(bnode_var(b)),
        other => {
            let t = term_pattern_to_term(other)?;
            match graph.id_of(&t) {
                Some(id) => id_pat[0] = Some(id),
                None => unsat = true,
            }
        }
    }
    // predicate
    match &tp.predicate {
        NamedNodePattern::Variable(v) => pos_vars[1] = Some(v.clone()),
        NamedNodePattern::NamedNode(n) => match graph.id_of(&Term::NamedNode(n.clone())) {
            Some(id) => id_pat[1] = Some(id),
            None => unsat = true,
        },
    }
    // object
    match &tp.object {
        TermPattern::Variable(v) => pos_vars[2] = Some(v.clone()),
        TermPattern::BlankNode(b) => pos_vars[2] = Some(bnode_var(b)),
        other => {
            let t = term_pattern_to_term(other)?;
            match graph.id_of(&t) {
                Some(id) => id_pat[2] = Some(id),
                None => unsat = true,
            }
        }
    }
    Ok((id_pat, pos_vars, unsat))
}

/// Scans a single pattern into bindings (handles a variable repeated across
/// positions by keeping only rows where those positions are equal).
fn scan_to_bindings(graph: &Graph, id_pat: &IdPattern, pos_vars: &[Option<Variable>; 3]) -> Bindings {
    // Distinct variables in position order, with the positions each occupies.
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

    let scan = graph.store.scan(id_pat);
    let mut rows: Vec<Vec<Id>> = Vec::with_capacity(scan.rows.len());
    for row in scan.rows {
        let spo = scan.to_spo(row);
        // Enforce equality for repeated variables.
        let mut ok = true;
        let mut out = Vec::with_capacity(vars.len());
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
        }
    }
    Bindings { vars, rows }
}

/// Hash join of two bindings on their shared variables.
fn hash_join(left: Bindings, right: Bindings) -> Bindings {
    // Build on the smaller side.
    let (build, probe) = if left.rows.len() <= right.rows.len() {
        (left, right)
    } else {
        (right, left)
    };

    let shared: Vec<(usize, usize)> = build
        .vars
        .iter()
        .enumerate()
        .filter_map(|(bi, v)| probe.col(v).map(|pi| (bi, pi)))
        .collect();

    // Output variable layout: build vars, then probe-only vars.
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

    // Hash table: key (build's shared values) -> build row indices.
    let mut table: FxHashMap<Vec<Id>, Vec<usize>> = FxHashMap::default();
    for (ri, row) in build.rows.iter().enumerate() {
        let key: Vec<Id> = shared.iter().map(|&(bi, _)| row[bi]).collect();
        table.entry(key).or_default().push(ri);
    }

    let mut rows: Vec<Vec<Id>> = Vec::new();
    for prow in &probe.rows {
        let key: Vec<Id> = shared.iter().map(|&(_, pi)| prow[pi]).collect();
        if let Some(matches) = table.get(&key) {
            for &bi in matches {
                let brow = &build.rows[bi];
                let mut combined = brow.clone();
                for &pi in &probe_only {
                    combined.push(prow[pi]);
                }
                rows.push(combined);
            }
        }
    }
    Bindings { vars: out_vars, rows }
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
    Bindings { vars: out_vars, rows }
}

fn dedup_rows(rows: &mut Vec<Vec<Option<Term>>>) {
    let mut seen = std::collections::HashSet::new();
    rows.retain(|r| {
        let key: Vec<String> = r
            .iter()
            .map(|t| t.as_ref().map(|t| t.to_string()).unwrap_or_default())
            .collect();
        seen.insert(key)
    });
}

// ---- FILTER (a useful subset) -------------------------------------------------

fn apply_filter(graph: &Graph, b: &mut Bindings, expr: &Expression) -> Result<(), String> {
    let mut keep = Vec::with_capacity(b.rows.len());
    for row in &b.rows {
        keep.push(eval_bool(graph, b, row, expr)?);
    }
    let mut i = 0;
    b.rows.retain(|_| {
        let k = keep[i];
        i += 1;
        k
    });
    Ok(())
}

/// Evaluate an expression to its SPARQL effective boolean value (EBV). A type
/// error / unbound value is false in our subset (FILTER removes the row).
fn eval_bool(graph: &Graph, b: &Bindings, row: &[Id], e: &Expression) -> Result<bool, String> {
    Ok(effective_boolean(&eval_expr(graph, b, row, e)?))
}

/// SPARQL effective boolean value: xsd:boolean -> its value; numeric -> != 0 and
/// not NaN; xsd:string / plain literal -> non-empty; everything else -> false.
fn effective_boolean(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Num(n) => *n != 0.0 && !n.is_nan(),
        Value::Unbound => false,
        Value::Term(Term::Literal(l)) => {
            let dt = l.datatype().as_str();
            if dt == "http://www.w3.org/2001/XMLSchema#boolean" {
                matches!(l.value(), "true" | "1")
            } else if is_numeric_dt(l) {
                l.value().parse::<f64>().map(|n| n != 0.0 && !n.is_nan()).unwrap_or(false)
            } else if dt == "http://www.w3.org/2001/XMLSchema#string"
                || dt == "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString"
            {
                !l.value().is_empty()
            } else {
                false
            }
        }
        Value::Term(_) => false,
    }
}

#[derive(Clone, Debug)]
enum Value {
    Bool(bool),
    Num(f64),
    Term(Term),
    Unbound,
}

fn eval_expr(graph: &Graph, b: &Bindings, row: &[Id], e: &Expression) -> Result<Value, String> {
    use Expression::*;
    match e {
        Variable(v) => match b.col(v) {
            Some(c) => Ok(Value::Term(graph.dict.term(row[c]).clone())),
            None => Ok(Value::Unbound),
        },
        NamedNode(n) => Ok(Value::Term(Term::NamedNode(n.clone()))),
        Literal(l) => Ok(Value::Term(Term::Literal(l.clone()))),
        And(a, c) => Ok(Value::Bool(
            eval_bool(graph, b, row, a)? && eval_bool(graph, b, row, c)?,
        )),
        Or(a, c) => Ok(Value::Bool(
            eval_bool(graph, b, row, a)? || eval_bool(graph, b, row, c)?,
        )),
        Not(a) => Ok(Value::Bool(!eval_bool(graph, b, row, a)?)),
        Equal(a, c) => cmp(graph, b, row, a, c, |o| o == std::cmp::Ordering::Equal),
        SameTerm(a, c) => {
            let x = eval_expr(graph, b, row, a)?;
            let y = eval_expr(graph, b, row, c)?;
            Ok(Value::Bool(value_term_eq(&x, &y)))
        }
        Greater(a, c) => cmp(graph, b, row, a, c, |o| o == std::cmp::Ordering::Greater),
        GreaterOrEqual(a, c) => cmp(graph, b, row, a, c, |o| o != std::cmp::Ordering::Less),
        Less(a, c) => cmp(graph, b, row, a, c, |o| o == std::cmp::Ordering::Less),
        LessOrEqual(a, c) => cmp(graph, b, row, a, c, |o| o != std::cmp::Ordering::Greater),
        Bound(v) => Ok(Value::Bool(b.col(v).map(|c| row[c] != 0).unwrap_or(false))),
        FunctionCall(_, _) => Err("M1: unsupported FILTER function".into()),
        other => Err(format!("M1: unsupported FILTER expression: {other:?}")),
    }
}

fn cmp(
    graph: &Graph,
    b: &Bindings,
    row: &[Id],
    a: &Expression,
    c: &Expression,
    f: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Value, String> {
    let x = eval_expr(graph, b, row, a)?;
    let y = eval_expr(graph, b, row, c)?;
    match compare_values(&x, &y) {
        Some(o) => Ok(Value::Bool(f(o))),
        None => Ok(Value::Bool(false)),
    }
}

fn as_num(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(*n),
        Value::Term(Term::Literal(l)) => l.value().parse::<f64>().ok().filter(|_| is_numeric_dt(l)),
        _ => None,
    }
}

fn is_numeric_dt(l: &oxrdf::Literal) -> bool {
    let dt = l.datatype().as_str();
    matches!(
        dt,
        "http://www.w3.org/2001/XMLSchema#integer"
            | "http://www.w3.org/2001/XMLSchema#decimal"
            | "http://www.w3.org/2001/XMLSchema#double"
            | "http://www.w3.org/2001/XMLSchema#float"
            | "http://www.w3.org/2001/XMLSchema#long"
            | "http://www.w3.org/2001/XMLSchema#int"
            | "http://www.w3.org/2001/XMLSchema#short"
            | "http://www.w3.org/2001/XMLSchema#byte"
            | "http://www.w3.org/2001/XMLSchema#nonNegativeInteger"
            | "http://www.w3.org/2001/XMLSchema#positiveInteger"
    )
}

fn compare_values(x: &Value, y: &Value) -> Option<std::cmp::Ordering> {
    if let (Some(a), Some(b)) = (as_num(x), as_num(y)) {
        return a.partial_cmp(&b);
    }
    // Fall back to lexical comparison of the term string for strings/IRIs.
    let xs = value_str(x)?;
    let ys = value_str(y)?;
    Some(xs.cmp(&ys))
}

fn value_str(v: &Value) -> Option<String> {
    match v {
        Value::Term(Term::Literal(l)) => Some(l.value().to_string()),
        Value::Term(Term::NamedNode(n)) => Some(n.as_str().to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Num(n) => Some(n.to_string()),
        _ => None,
    }
}

fn value_term_eq(x: &Value, y: &Value) -> bool {
    matches!((x, y), (Value::Term(a), Value::Term(b)) if a == b)
}

// ---- spargebra term helpers ---------------------------------------------------

fn term_pattern_to_term(tp: &TermPattern) -> Result<Term, String> {
    match tp {
        TermPattern::NamedNode(n) => Ok(Term::NamedNode(n.clone())),
        TermPattern::BlankNode(b) => Ok(Term::BlankNode(b.clone())),
        TermPattern::Literal(l) => Ok(Term::Literal(l.clone())),
        TermPattern::Variable(_) => Err("variable where a term was expected".into()),
        other => Err(format!("unsupported term pattern: {other:?}")),
    }
}

/// Prefix marking synthetic variables that stand in for query blank nodes.
const BNODE_VAR_PREFIX: &str = "__bn_";

/// The synthetic variable for a query blank node (repeated labels -> same var,
/// so they behave like a repeated variable, per SPARQL semantics).
fn bnode_var(b: &oxrdf::BlankNode) -> Variable {
    Variable::new_unchecked(format!("{BNODE_VAR_PREFIX}{}", b.as_str()))
}

/// The variable a term pattern contributes (a real variable, or the synthetic
/// variable of a blank node), if any.
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

// Re-export to silence unused warnings for NamedNode import used in signatures.
#[allow(unused_imports)]
use NamedNode as _NamedNode;
