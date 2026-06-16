//! SPARQL window functions (sq-5qz9) — a NON-STANDARD, OPT-IN extension.
//!
//! [OPUS-4.8] **There is no W3C-REC syntax for SPARQL window functions.** SPARQL
//! 1.1 has no `OVER (PARTITION BY … ORDER BY …)` clause, and no SPARQL-community
//! SEP defines one. Engines that offer windowing (Stardog's `WINDOW`/`OVER`,
//! AnzoGraph, GraphDB's ranking extensions) each invent their own surface. So
//! rather than smuggle a non-standard `OVER` clause into the (W3C-conformance
//! tracking) vendored parser, this module exposes window functions as a
//! **programmatic pass over a [`QueryResult`]** whose semantics follow the
//! SQL:2003 windowing model (`PARTITION BY`, then `ORDER BY` within a partition,
//! then a ranking / aggregate function) — the model Stardog and AnzoGraph
//! expose. The caller runs an ordinary SELECT, then applies a [`WindowSpec`].
//!
//! This keeps the extension HONEST (the engine's SPARQL surface stays exactly
//! SPARQL 1.1; the window layer is an explicit, separate API a caller opts into)
//! and avoids any conformance-harness regression.
//!
//! ## Supported functions
//!
//! * [`WindowFunction::RowNumber`] — `ROW_NUMBER()`: 1-based sequential position
//!   within the partition, in the window `ORDER BY` order (ties broken by input
//!   order, i.e. stable).
//! * [`WindowFunction::Rank`] — `RANK()`: 1-based, with GAPS after ties (rows
//!   that compare equal under the window `ORDER BY` share a rank; the next
//!   distinct row's rank skips by the size of the tie group).
//! * [`WindowFunction::DenseRank`] — `DENSE_RANK()`: 1-based, NO gaps (ties share
//!   a rank, the next distinct row is +1).
//! * [`WindowFunction::Aggregate`] — a windowed aggregate over the WHOLE
//!   partition (frame = the entire partition, the SQL default for an aggregate
//!   with `PARTITION BY` and no explicit frame): every row in a partition gets
//!   the same value, the aggregate of a chosen column over that partition.
//!
//! Each appends one new bound column (`new_var`) to every row; the row order of
//! the input `QueryResult` is preserved (window functions are computed, not a
//! re-sort).

use oxrdf::{Literal, NamedNode, Term, Variable};
use std::cmp::Ordering;

use crate::QueryResult;

/// A caller-supplied windowed-aggregate fold: maps a partition's per-row cells
/// (`None` ≡ unbound for that row) to the partition's window value (`None` ≡
/// unbound). Used by [`WindowAggregate::Custom`].
pub type WindowFold = Box<dyn Fn(&[Option<Term>]) -> Option<Term>>;

/// A window sort key: a variable plus a direction. Comparison follows a
/// SPARQL-ORDER-BY-like total order over the bound terms (see [`cmp_terms`]):
/// numeric literals by value, everything else by a stable lexical/kind order;
/// an unbound cell sorts first (ascending).
#[derive(Debug, Clone)]
pub struct SortKey {
    pub var: Variable,
    /// `true` = descending (`DESC`), `false` = ascending (`ASC`, the default).
    pub descending: bool,
}

impl SortKey {
    pub fn asc(var: Variable) -> Self {
        Self { var, descending: false }
    }
    pub fn desc(var: Variable) -> Self {
        Self { var, descending: true }
    }
}

/// The aggregate a windowed [`WindowFunction::Aggregate`] computes over a
/// partition. The numeric ones operate on the numeric VALUE of each bound,
/// non-null cell; `Count` counts bound cells (`Count`-of-`*` semantics for the
/// chosen column); a fully custom fold is [`Self::Custom`].
pub enum WindowAggregate {
    /// COUNT of bound (non-null) values of the column.
    Count,
    /// SUM of the numeric values; an empty/all-unbound partition is `0`.
    Sum,
    /// AVG of the numeric values; an empty partition yields an unbound result.
    Avg,
    /// MIN by the [`cmp_terms`] order over bound cells (unbound for empty).
    Min,
    /// MAX by the [`cmp_terms`] order over bound cells (unbound for empty).
    Max,
    /// A caller-supplied fold over the partition's per-row cells (`None` ≡
    /// unbound for that row): returns the partition's window value (`None` ≡
    /// unbound result).
    Custom(WindowFold),
}

/// The window function to evaluate per partition.
pub enum WindowFunction {
    /// `ROW_NUMBER()` — see the module docs.
    RowNumber,
    /// `RANK()` — gaps after ties.
    Rank,
    /// `DENSE_RANK()` — no gaps.
    DenseRank,
    /// A windowed aggregate over the whole partition, of the given column.
    Aggregate { agg: WindowAggregate, of: Variable },
}

/// A window specification: PARTITION BY zero or more columns, ORDER BY zero or
/// more sort keys, a [`WindowFunction`], and the name of the column to append.
///
/// Empty `partition_by` ≡ a single partition over the whole result. Empty
/// `order_by` for a ranking function ≡ every row is a peer (so `RANK`/
/// `DENSE_RANK` are all `1` and `ROW_NUMBER` follows input order).
pub struct WindowSpec {
    pub partition_by: Vec<Variable>,
    pub order_by: Vec<SortKey>,
    pub function: WindowFunction,
    /// The new variable the computed window value is bound to.
    pub new_var: Variable,
}

/// Applies `spec` to `result`, returning a new [`QueryResult`] with `spec.new_var`
/// appended as the last column. Row order is preserved.
///
/// `Err` if a referenced variable (`partition_by` / `order_by` / the aggregate's
/// `of`) is not a column of `result`.
pub fn apply_window(result: &QueryResult, spec: &WindowSpec) -> Result<QueryResult, String> {
    let col = |v: &Variable| -> Result<usize, String> {
        result
            .vars
            .iter()
            .position(|c| c == v)
            .ok_or_else(|| format!("window: variable ?{} is not a result column", v.as_str()))
    };
    let part_cols: Vec<usize> = spec.partition_by.iter().map(&col).collect::<Result<_, _>>()?;
    let order_cols: Vec<(usize, bool)> =
        spec.order_by.iter().map(|k| col(&k.var).map(|c| (c, k.descending))).collect::<Result<_, _>>()?;

    // Group row indices into partitions by their partition-key tuple, preserving
    // first-seen partition order and input row order within a partition.
    let mut partitions: Vec<Vec<usize>> = Vec::new();
    let mut keys: Vec<Vec<Option<Term>>> = Vec::new();
    for (ri, row) in result.rows.iter().enumerate() {
        let key: Vec<Option<Term>> = part_cols.iter().map(|&c| row[c].clone()).collect();
        match keys.iter().position(|k| *k == key) {
            Some(pi) => partitions[pi].push(ri),
            None => {
                keys.push(key);
                partitions.push(vec![ri]);
            }
        }
    }

    // The computed window value for each input row index.
    let mut values: Vec<Option<Term>> = vec![None; result.rows.len()];
    for part in &partitions {
        // Order the partition's rows by the window ORDER BY (stable, so input
        // order breaks ties — required for ROW_NUMBER determinism).
        let mut ordered: Vec<usize> = part.clone();
        ordered.sort_by(|&a, &b| order_rows(result, &order_cols, a, b));
        compute_partition(result, spec, part, &ordered, &order_cols, &mut values);
    }

    let mut vars = result.vars.clone();
    vars.push(spec.new_var.clone());
    let rows: Vec<Vec<Option<Term>>> = result
        .rows
        .iter()
        .enumerate()
        .map(|(ri, row)| {
            let mut r = row.clone();
            r.push(values[ri].take());
            r
        })
        .collect();
    Ok(QueryResult { vars, rows })
}

/// Computes the window value for one partition, writing into `values` at each
/// row's input index. `ordered` is the partition's rows in window-ORDER-BY order.
fn compute_partition(
    result: &QueryResult,
    spec: &WindowSpec,
    part: &[usize],
    ordered: &[usize],
    order_cols: &[(usize, bool)],
    values: &mut [Option<Term>],
) {
    match &spec.function {
        WindowFunction::RowNumber => {
            for (n, &ri) in ordered.iter().enumerate() {
                values[ri] = Some(int_term((n + 1) as i64));
            }
        }
        WindowFunction::Rank => {
            let mut rank: i64 = 0;
            let mut seen: i64 = 0;
            let mut prev: Option<usize> = None;
            for &ri in ordered {
                seen += 1;
                let tie = prev.is_some_and(|p| peers(result, order_cols, p, ri));
                if !tie {
                    rank = seen; // gap: jump to the running count
                }
                values[ri] = Some(int_term(rank));
                prev = Some(ri);
            }
        }
        WindowFunction::DenseRank => {
            let mut rank: i64 = 0;
            let mut prev: Option<usize> = None;
            for &ri in ordered {
                let tie = prev.is_some_and(|p| peers(result, order_cols, p, ri));
                if !tie {
                    rank += 1; // no gap
                }
                values[ri] = Some(int_term(rank));
                prev = Some(ri);
            }
        }
        WindowFunction::Aggregate { agg, of } => {
            let of_col = result.vars.iter().position(|c| c == of);
            let cells: Vec<Option<Term>> = part
                .iter()
                .map(|&ri| of_col.and_then(|c| result.rows[ri][c].clone()))
                .collect();
            let v = eval_window_aggregate(agg, &cells);
            // The whole-partition frame: every row in the partition gets `v`.
            for &ri in part {
                values[ri] = v.clone();
            }
        }
    }
}

/// Whether rows `a` and `b` are PEERS under the window ORDER BY (compare equal on
/// every order key). With no `ORDER BY`, all rows are peers.
fn peers(result: &QueryResult, order_cols: &[(usize, bool)], a: usize, b: usize) -> bool {
    order_rows(result, order_cols, a, b) == Ordering::Equal
}

/// Compare two rows by the window ORDER BY keys (direction applied), then Equal.
/// (Stable sort preserves input order for full ties, so ROW_NUMBER is deterministic.)
fn order_rows(result: &QueryResult, order_cols: &[(usize, bool)], a: usize, b: usize) -> Ordering {
    for &(c, desc) in order_cols {
        let ord = cmp_terms(result.rows[a][c].as_ref(), result.rows[b][c].as_ref());
        let ord = if desc { ord.reverse() } else { ord };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}

fn eval_window_aggregate(agg: &WindowAggregate, cells: &[Option<Term>]) -> Option<Term> {
    match agg {
        WindowAggregate::Count => Some(int_term(cells.iter().filter(|c| c.is_some()).count() as i64)),
        WindowAggregate::Sum => {
            let sum: f64 = cells.iter().filter_map(|c| c.as_ref()).filter_map(numeric_value).sum();
            Some(num_term(sum))
        }
        WindowAggregate::Avg => {
            let vals: Vec<f64> = cells.iter().filter_map(|c| c.as_ref()).filter_map(numeric_value).collect();
            if vals.is_empty() {
                None
            } else {
                Some(num_term(vals.iter().sum::<f64>() / vals.len() as f64))
            }
        }
        WindowAggregate::Min => cells
            .iter()
            .filter_map(|c| c.as_ref())
            .min_by(|a, b| cmp_terms(Some(a), Some(b)))
            .cloned(),
        WindowAggregate::Max => cells
            .iter()
            .filter_map(|c| c.as_ref())
            .max_by(|a, b| cmp_terms(Some(a), Some(b)))
            .cloned(),
        WindowAggregate::Custom(f) => f(cells),
    }
}

/// The numeric VALUE of a term, if it is a numeric literal (xsd integer / decimal
/// / float / double, or anything whose lexical form parses as an `f64`).
fn numeric_value(t: &Term) -> Option<f64> {
    match t {
        Term::Literal(l) => l.value().parse::<f64>().ok(),
        _ => None,
    }
}

/// A SPARQL-ORDER-BY-like total order over optional bound terms, self-contained
/// so the window pass needs no engine internals. Ordering: unbound (`None`) sorts
/// FIRST; then by KIND (blank < IRI < literal — a stable, deterministic order);
/// within literals, two NUMERIC literals compare by value, otherwise by
/// `(datatype, language, lexical)`. This is a TOTAL order (every pair is
/// comparable), which a window sort requires.
pub fn cmp_terms(a: Option<&Term>, b: Option<&Term>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(x), Some(y)) => cmp_bound(x, y),
    }
}

fn kind_rank(t: &Term) -> u8 {
    match t {
        Term::BlankNode(_) => 0,
        Term::NamedNode(_) => 1,
        Term::Literal(_) => 2,
        // oxrdf may expose a Triple term under rdf-star; order it last, stably.
        _ => 3,
    }
}

fn cmp_bound(x: &Term, y: &Term) -> Ordering {
    match (x, y) {
        (Term::Literal(lx), Term::Literal(ly)) => {
            if let (Some(nx), Some(ny)) = (numeric_value(&Term::Literal(lx.clone())), numeric_value(&Term::Literal(ly.clone()))) {
                if let Some(o) = nx.partial_cmp(&ny) {
                    if o != Ordering::Equal {
                        return o;
                    }
                }
            }
            lx.datatype()
                .as_str()
                .cmp(ly.datatype().as_str())
                .then_with(|| lx.language().unwrap_or("").cmp(ly.language().unwrap_or("")))
                .then_with(|| lx.value().cmp(ly.value()))
        }
        (Term::NamedNode(a), Term::NamedNode(b)) => a.as_str().cmp(b.as_str()),
        (Term::BlankNode(a), Term::BlankNode(b)) => a.as_str().cmp(b.as_str()),
        _ => kind_rank(x).cmp(&kind_rank(y)).then_with(|| x.to_string().cmp(&y.to_string())),
    }
}

/// An `xsd:integer` term for a rank / count.
fn int_term(n: i64) -> Term {
    Term::Literal(Literal::from(n))
}

/// A numeric term: an `xsd:integer` if the value is integral, else an
/// `xsd:double`. (Window SUM/AVG over integer inputs commonly want an integer
/// back when exact; AVG of an odd sum stays a double.)
fn num_term(v: f64) -> Term {
    if v.fract() == 0.0 && v.is_finite() && v.abs() < 9.007_199_254_740_992e15 {
        int_term(v as i64)
    } else {
        Term::Literal(Literal::new_typed_literal(
            {
                let mut s = v.to_string();
                if !s.contains(['.', 'e', 'E', 'N', 'i']) {
                    s.push_str(".0");
                }
                s
            },
            NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#double"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::NamedNode as NN;

    fn var(s: &str) -> Variable {
        Variable::new(s).unwrap()
    }
    fn iri(s: &str) -> Term {
        Term::NamedNode(NN::new(s).unwrap())
    }
    fn int(n: i64) -> Term {
        Term::Literal(Literal::from(n))
    }

    /// Three depts, sales values, to rank within partition.
    /// (?emp ?dept ?sales): a/eng/30, b/eng/30, c/eng/20, d/sales/10
    fn sample() -> QueryResult {
        QueryResult {
            vars: vec![var("emp"), var("dept"), var("sales")],
            rows: vec![
                vec![Some(iri("http://ex/a")), Some(iri("http://ex/eng")), Some(int(30))],
                vec![Some(iri("http://ex/b")), Some(iri("http://ex/eng")), Some(int(30))],
                vec![Some(iri("http://ex/c")), Some(iri("http://ex/eng")), Some(int(20))],
                vec![Some(iri("http://ex/d")), Some(iri("http://ex/sales")), Some(int(10))],
            ],
        }
    }

    fn val(r: &QueryResult, row: usize, col: usize) -> String {
        r.rows[row][col].as_ref().unwrap().to_string()
    }

    #[test]
    fn row_number_partition_order() {
        // PARTITION BY dept ORDER BY sales DESC -> ROW_NUMBER per dept.
        let spec = WindowSpec {
            partition_by: vec![var("dept")],
            order_by: vec![SortKey::desc(var("sales"))],
            function: WindowFunction::RowNumber,
            new_var: var("rn"),
        };
        let out = apply_window(&sample(), &spec).unwrap();
        // new column is appended at index 3; rows stay in input order.
        // eng partition (a=30,b=30,c=20) ordered desc -> a,b are 30 (tie, input order a<b), c=20.
        // a -> 1, b -> 2, c -> 3 ; sales partition d -> 1.
        assert_eq!(val(&out, 0, 3), int(1).to_string()); // a
        assert_eq!(val(&out, 1, 3), int(2).to_string()); // b
        assert_eq!(val(&out, 2, 3), int(3).to_string()); // c
        assert_eq!(val(&out, 3, 3), int(1).to_string()); // d
    }

    #[test]
    fn rank_has_gaps_dense_rank_does_not() {
        let mk = |f| WindowSpec {
            partition_by: vec![var("dept")],
            order_by: vec![SortKey::desc(var("sales"))],
            function: f,
            new_var: var("r"),
        };
        let rank = apply_window(&sample(), &mk(WindowFunction::Rank)).unwrap();
        // eng: a=30,b=30 -> both rank 1; c=20 -> rank 3 (GAP).
        assert_eq!(val(&rank, 0, 3), int(1).to_string());
        assert_eq!(val(&rank, 1, 3), int(1).to_string());
        assert_eq!(val(&rank, 2, 3), int(3).to_string());
        let dense = apply_window(&sample(), &mk(WindowFunction::DenseRank)).unwrap();
        // eng: a,b -> 1; c -> 2 (NO gap).
        assert_eq!(val(&dense, 0, 3), int(1).to_string());
        assert_eq!(val(&dense, 1, 3), int(1).to_string());
        assert_eq!(val(&dense, 2, 3), int(2).to_string());
    }

    #[test]
    fn windowed_sum_over_partition() {
        let spec = WindowSpec {
            partition_by: vec![var("dept")],
            order_by: vec![],
            function: WindowFunction::Aggregate { agg: WindowAggregate::Sum, of: var("sales") },
            new_var: var("total"),
        };
        let out = apply_window(&sample(), &spec).unwrap();
        // eng total = 30+30+20 = 80 for every eng row; sales total = 10.
        assert_eq!(val(&out, 0, 3), int(80).to_string());
        assert_eq!(val(&out, 1, 3), int(80).to_string());
        assert_eq!(val(&out, 2, 3), int(80).to_string());
        assert_eq!(val(&out, 3, 3), int(10).to_string());
    }

    #[test]
    fn windowed_count_and_custom() {
        let cnt = apply_window(
            &sample(),
            &WindowSpec {
                partition_by: vec![var("dept")],
                order_by: vec![],
                function: WindowFunction::Aggregate { agg: WindowAggregate::Count, of: var("sales") },
                new_var: var("n"),
            },
        )
        .unwrap();
        assert_eq!(val(&cnt, 0, 3), int(3).to_string()); // eng count
        assert_eq!(val(&cnt, 3, 3), int(1).to_string()); // sales count

        // Custom fold: max sales as a string, to prove the closure boundary.
        let custom = apply_window(
            &sample(),
            &WindowSpec {
                partition_by: vec![var("dept")],
                order_by: vec![],
                function: WindowFunction::Aggregate {
                    agg: WindowAggregate::Custom(Box::new(|cells: &[Option<Term>]| {
                        cells.iter().filter_map(|c| c.as_ref()).max_by(|a, b| cmp_terms(Some(a), Some(b))).cloned()
                    })),
                    of: var("sales"),
                },
                new_var: var("m"),
            },
        )
        .unwrap();
        assert_eq!(val(&custom, 0, 3), int(30).to_string()); // eng max
        assert_eq!(val(&custom, 3, 3), int(10).to_string()); // sales max
    }

    #[test]
    fn unknown_variable_is_error() {
        let spec = WindowSpec {
            partition_by: vec![var("nope")],
            order_by: vec![],
            function: WindowFunction::RowNumber,
            new_var: var("rn"),
        };
        assert!(apply_window(&sample(), &spec).is_err());
    }
}
