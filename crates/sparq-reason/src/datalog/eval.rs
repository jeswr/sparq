//! [FABLE-5] sq-6tykl.3 — the **non-incremental per-stratum evaluator** (Phase 1).
//!
//! Strata run in ascending order; within a stratum, rules iterate to fixpoint in
//! naive rounds (derivations de-duplicate at store insertion — semi-naive delta
//! restriction is a beaded follow-up phase, see the design record §6). Because the
//! checker guarantees every `NOT`-negated / `AGGREGATE`-read predicate is complete
//! before its stratum runs:
//!
//! * `AGGREGATE` tables are computed **once at stratum entry** (their bodies read
//!   strictly-lower strata, which no longer change) and then joined like positive
//!   relations;
//! * `NOT` is a per-row absence check against the store's predicate index —
//!   the indexed relation is final, so no retraction can ever be needed.
//!
//! Positive-atom and aggregate-table joins drive the SHARED
//! [`sparq_substrate::join`] kernels via the same thin layout-adapter pattern as
//! `crate::substrate_join` and `crate::n3::compiled` — build side = binding rows,
//! probe side = candidate rows, combined rows reshaped back into the rule's slot
//! layout.

use super::{numeric_value, AggAtom, Atom, CmpOp, DTerm, FactStore, Program, Rule, Stratification};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};
use sparq_substrate::join::{self as sjoin, JoinKeys, NoBudget};
use sparq_substrate::rows::{Row, NO_ID};

const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

/// Evaluate `program` (already checked by [`super::stratify`]) over `facts` and
/// return the full closure (inputs + derivations, de-duplicated; treat as a set).
pub(super) fn eval_stratified(
    dict: &mut Dict,
    facts: &[[Id; 3]],
    program: &Program,
    strat: &Stratification,
) -> Vec<[Id; 3]> {
    let mut store = FactStore::default();
    for f in facts {
        store.insert(*f);
    }
    for s in 0..strat.n_strata {
        let rules: Vec<&Rule> = program
            .rules
            .iter()
            .zip(&strat.rule_stratum)
            .filter(|(_, rs)| **rs == s)
            .map(|(r, _)| r)
            .collect();
        if rules.is_empty() {
            continue;
        }
        // Aggregate tables: computed once per stratum (bodies read lower strata).
        let agg_tables: Vec<Vec<Vec<Row>>> = rules
            .iter()
            .map(|r| {
                r.aggregates
                    .iter()
                    .map(|a| aggregate_table(dict, a, &store))
                    .collect()
            })
            .collect();
        loop {
            let mut produced: Vec<[Id; 3]> = Vec::new();
            for (r, tables) in rules.iter().zip(&agg_tables) {
                run_rule(dict, r, tables, &store, &mut produced);
            }
            let mut any_new = false;
            for f in produced {
                any_new |= store.insert(f);
            }
            if !any_new {
                break;
            }
        }
    }
    store.list
}

/// `(binding slot, candidate column)` pairs mapping variable slots to the atom's
/// triple positions — used for equi-join keys and write-back slots alike.
type SlotCols = Vec<(usize, usize)>;
/// Per-position constants pinned for an atom's join probe (`s`, `p`, `o`).
type AtomConsts = [Option<Id>; 3];
/// The join plan for one atom: `(key_cols, new_writes, consts)`.
type AtomPlan = (SlotCols, SlotCols, AtomConsts);

/// The join plan for one atom given the running bound-slot set: equi-join key
/// column pairs `(binding slot, candidate column)` for already-bound variables,
/// write-back slots for new ones, and per-position constants. Marks the atom's
/// new variables as bound.
fn plan_atom(atom: &Atom, bound: &mut FxHashSet<u32>) -> AtomPlan {
    let mut key_cols = Vec::new();
    let mut new_writes = Vec::new();
    let mut consts = [None, None, None];
    for (i, t) in atom.t.iter().enumerate() {
        match t {
            DTerm::Const(id) => consts[i] = Some(*id),
            DTerm::Var(v) => {
                if bound.contains(v) {
                    key_cols.push((*v as usize, i));
                } else {
                    new_writes.push((*v as usize, i));
                }
            }
        }
    }
    for &(v, _) in &new_writes {
        bound.insert(v as u32);
    }
    (key_cols, new_writes, consts)
}

/// Candidate facts for one atom: the store's predicate index narrowed by the
/// constant pre-filters, as substrate probe rows `[s, p, o]`.
fn candidates(consts: &[Option<Id>; 3], pred: Id, store: &FactStore) -> Vec<Row> {
    let keep =
        |t: &[Id; 3]| consts[0].is_none_or(|c| t[0] == c) && consts[2].is_none_or(|c| t[2] == c);
    store
        .by_pred
        .get(&pred)
        .map_or(&[][..], |v| v.as_slice())
        .iter()
        .filter(|t| keep(t))
        .map(|t| Row::from_slice(t))
        .collect()
}

/// One join step over the SHARED substrate kernels: binding rows are the build
/// side, `cands` the probe side; each combined row (`bindings ++ candidate`) is
/// reshaped back into the `width`-slot layout by writing the new variables'
/// columns (a repeated new variable within one atom must agree).
fn join_rows(
    rows: &[Row],
    key_cols: &[(usize, usize)],
    new_writes: &[(usize, usize)],
    cands: &[Row],
    width: usize,
    cand_width: usize,
) -> Vec<Row> {
    if rows.is_empty() || cands.is_empty() {
        return Vec::new();
    }
    let keys = JoinKeys {
        key_cols: key_cols.to_vec(),
        right_only: Vec::new(),
    };
    let tables = vec![sjoin::build_table(rows, &keys)];
    let probe_only: Vec<usize> = (0..cand_width).collect();
    let mut combined: Vec<Row> = Vec::new();
    sjoin::hash_probe_serial(
        cands,
        &keys,
        rows,
        &tables,
        &probe_only,
        &NoBudget,
        &mut combined,
    );
    let mut out = Vec::with_capacity(combined.len());
    'row: for c in &combined {
        let (b, f) = c.split_at(width);
        let mut row = Row::from_slice(b);
        for &(v, i) in new_writes {
            if row[v] != NO_ID && row[v] != f[i] {
                continue 'row; // repeated new variable in one atom must agree
            }
            row[v] = f[i];
        }
        out.push(row);
    }
    out
}

/// Evaluate an `AGGREGATE` atom's positive body over the (stratum-complete) store,
/// de-duplicate the full binding tuples (set semantics), group by the `ON`
/// variables and COUNT the matches per group, minting the count as an
/// `xsd:integer` literal. Returns rows `[on₀ … onₖ₋₁, count]` in the OUTER join
/// layout (`ON` columns first, in `ON` order). Empty groups produce no row.
fn aggregate_table(dict: &mut Dict, agg: &AggAtom, store: &FactStore) -> Vec<Row> {
    let empty: Row = std::iter::repeat_n(NO_ID, agg.n_slots).collect();
    let mut rows: Vec<Row> = vec![empty];
    let mut bound: FxHashSet<u32> = FxHashSet::default();
    for atom in &agg.body {
        let (key_cols, new_writes, consts) = plan_atom(atom, &mut bound);
        let cands = candidates(&consts, atom.pred, store);
        rows = join_rows(&rows, &key_cols, &new_writes, &cands, agg.n_slots, 3);
    }
    // Distinct full tuples, then count per ON-group. (`COUNT(?v)` counts distinct
    // body matches — see the module docs; the counted slot is always bound in the
    // Phase-1 fragment, `parser` guarantees it occurs in the body.)
    let distinct: FxHashSet<Row> = rows.into_iter().collect();
    let mut groups: FxHashMap<Vec<Id>, u64> = FxHashMap::default();
    for r in &distinct {
        let key: Vec<Id> = agg.on.iter().map(|&(l, _)| r[l as usize]).collect();
        *groups.entry(key).or_insert(0) += 1;
    }
    let mut out = Vec::with_capacity(groups.len());
    for (key, cnt) in groups {
        let cnt_id = dict.intern_lit(&cnt.to_string(), XSD_INTEGER, None);
        let mut row: Row = Row::from_slice(&key);
        row.push(cnt_id);
        out.push(row);
    }
    out
}

/// Fire one rule against the current store, appending (possibly duplicate)
/// conclusions to `out` — the caller's store insertion de-duplicates.
fn run_rule(
    dict: &Dict,
    rule: &Rule,
    agg_tables: &[Vec<Row>],
    store: &FactStore,
    out: &mut Vec<[Id; 3]>,
) {
    let empty: Row = std::iter::repeat_n(NO_ID, rule.n_slots).collect();
    let mut rows: Vec<Row> = vec![empty];
    let mut bound: FxHashSet<u32> = FxHashSet::default();
    for atom in &rule.positive {
        let (key_cols, new_writes, consts) = plan_atom(atom, &mut bound);
        let cands = candidates(&consts, atom.pred, store);
        rows = join_rows(&rows, &key_cols, &new_writes, &cands, rule.n_slots, 3);
        if rows.is_empty() {
            return;
        }
    }
    // Aggregate tables join like positive relations: columns are the ON outer
    // slots (in order) then the output slot.
    for (agg, table) in rule.aggregates.iter().zip(agg_tables) {
        let cand_width = agg.on.len() + 1;
        let mut key_cols = Vec::new();
        let mut new_writes = Vec::new();
        for (i, &(_, outer)) in agg.on.iter().enumerate() {
            if bound.contains(&outer) {
                key_cols.push((outer as usize, i));
            } else {
                new_writes.push((outer as usize, i));
                bound.insert(outer);
            }
        }
        new_writes.push((agg.out as usize, agg.on.len()));
        bound.insert(agg.out);
        rows = join_rows(
            &rows,
            &key_cols,
            &new_writes,
            table,
            rule.n_slots,
            cand_width,
        );
        if rows.is_empty() {
            return;
        }
    }
    // NOT: per-row absence check against the (stratum-complete) predicate index.
    for atom in &rule.negated {
        rows.retain(|r| !naf_matches(atom, r, &bound, store));
        if rows.is_empty() {
            return;
        }
    }
    // FILTER: exact numeric comparison via the shared substrate tower; a
    // non-numeric operand fails the row (fail-closed).
    for f in &rule.filters {
        rows.retain(|r| {
            let val = |t: &DTerm| match t {
                DTerm::Const(id) => numeric_value(dict, *id),
                DTerm::Var(v) => numeric_value(dict, r[*v as usize]),
            };
            let (Some(a), Some(b)) = (val(&f.a), val(&f.b)) else {
                return false;
            };
            let Some(ord) = a.cmp(b) else { return false };
            match f.op {
                CmpOp::Eq => ord.is_eq(),
                CmpOp::Ne => ord.is_ne(),
                CmpOp::Lt => ord.is_lt(),
                CmpOp::Le => ord.is_le(),
                CmpOp::Gt => ord.is_gt(),
                CmpOp::Ge => ord.is_ge(),
            }
        });
        if rows.is_empty() {
            return;
        }
    }
    for r in &rows {
        for head in &rule.head {
            let g = [
                resolve(&head.t[0], r),
                resolve(&head.t[1], r),
                resolve(&head.t[2], r),
            ];
            if !store.all.contains(&g) {
                out.push(g);
            }
        }
    }
}

fn resolve(t: &DTerm, row: &Row) -> Id {
    match t {
        DTerm::Const(id) => *id,
        DTerm::Var(v) => row[*v as usize],
    }
}

/// Does the `NOT` atom match at least one stored fact under `row`? Bound
/// variables are correlated values; unbound ones are existential wildcards (a
/// repeated wildcard within the atom must match equal terms).
fn naf_matches(atom: &Atom, row: &Row, bound: &FxHashSet<u32>, store: &FactStore) -> bool {
    let facts = store
        .by_pred
        .get(&atom.pred)
        .map_or(&[][..], |v| v.as_slice());
    facts.iter().any(|f| {
        let mut wild: FxHashMap<u32, Id> = FxHashMap::default();
        atom.t.iter().enumerate().all(|(i, t)| match t {
            DTerm::Const(id) => f[i] == *id,
            DTerm::Var(v) if bound.contains(v) => f[i] == row[*v as usize],
            DTerm::Var(v) => *wild.entry(*v).or_insert(f[i]) == f[i],
        })
    })
}
