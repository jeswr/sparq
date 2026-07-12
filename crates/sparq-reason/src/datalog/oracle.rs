//! [FABLE-5] sq-6tykl.3 — the **independent reference evaluator** for the
//! differential suite (test-only). Deliberately shares NOTHING with the production
//! evaluator's machinery: no substrate join kernels, no predicate index, no binding
//! rows — just textbook naive stratified evaluation by recursive substitution
//! (environments are plain `var → Id` maps, every atom scans every fact). Integer
//! `FILTER` comparison is re-implemented over `i128` lexical parsing so even the
//! numeric path is independent for the `xsd:integer` values the fixtures use.
//!
//! Slow on purpose — fixture scale only. Agreement between this evaluator and
//! [`super::eval`] on the fixtures and the seed-randomised graphs is the Phase-1
//! acceptance differential.

use super::{numeric_value, AggAtom, Atom, DTerm, Program, Rule, Stratification};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};

const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

type Env = FxHashMap<u32, Id>;

/// Naive stratified evaluation: the reference closure (inputs + derivations).
pub(super) fn eval_naive(
    dict: &mut Dict,
    facts: &[[Id; 3]],
    program: &Program,
    strat: &Stratification,
) -> FxHashSet<[Id; 3]> {
    let mut store: FxHashSet<[Id; 3]> = facts.iter().copied().collect();
    for s in 0..strat.n_strata {
        loop {
            let mut fresh: Vec<[Id; 3]> = Vec::new();
            for (rule, rs) in program.rules.iter().zip(&strat.rule_stratum) {
                if *rs != s {
                    continue;
                }
                for env in rule_matches(dict, rule, &store) {
                    for h in &rule.head {
                        let g = [subst(&h.t[0], &env), subst(&h.t[1], &env), subst(&h.t[2], &env)];
                        if !store.contains(&g) {
                            fresh.push(g);
                        }
                    }
                }
            }
            let mut any = false;
            for f in fresh {
                any |= store.insert(f);
            }
            if !any {
                break;
            }
        }
    }
    store
}

fn subst(t: &DTerm, env: &Env) -> Id {
    match t {
        DTerm::Const(id) => *id,
        DTerm::Var(v) => env[v],
    }
}

/// All satisfying environments for a rule body: backtrack over the positive atoms,
/// then extend through the aggregates, then filter by NOT and FILTER.
fn rule_matches(dict: &mut Dict, rule: &Rule, store: &FxHashSet<[Id; 3]>) -> Vec<Env> {
    let mut envs = vec![Env::default()];
    for atom in &rule.positive {
        envs = envs
            .iter()
            .flat_map(|env| atom_matches(atom, env, store))
            .collect();
    }
    for agg in &rule.aggregates {
        let table = agg_table(dict, agg, store);
        let mut next = Vec::new();
        for env in &envs {
            for (key, cnt_id) in &table {
                let mut e = env.clone();
                let mut ok = true;
                for (&(_, outer), &kv) in agg.on.iter().zip(key) {
                    match e.get(&outer) {
                        Some(&b) if b != kv => {
                            ok = false;
                            break;
                        }
                        Some(_) => {}
                        None => {
                            e.insert(outer, kv);
                        }
                    }
                }
                if ok {
                    e.insert(agg.out, *cnt_id);
                    next.push(e);
                }
            }
        }
        envs = next;
    }
    for atom in &rule.negated {
        envs.retain(|env| !store.iter().any(|f| unifies(atom, f, env)));
    }
    for f in &rule.filters {
        envs.retain(|env| {
            let val = |t: &DTerm| match t {
                DTerm::Const(id) => num(dict, *id),
                DTerm::Var(v) => env.get(v).and_then(|&id| num(dict, id)),
            };
            match (val(&f.a), val(&f.b)) {
                (Some(a), Some(b)) => {
                    use super::CmpOp::*;
                    match f.op {
                        Eq => a == b,
                        Ne => a != b,
                        Lt => a < b,
                        Le => a <= b,
                        Gt => a > b,
                        Ge => a >= b,
                    }
                }
                _ => false,
            }
        });
    }
    envs
}

/// Match one positive atom against every stored fact by substitution.
fn atom_matches(atom: &Atom, env: &Env, store: &FxHashSet<[Id; 3]>) -> Vec<Env> {
    let mut out = Vec::new();
    for f in store {
        let mut e = env.clone();
        if atom.t.iter().enumerate().all(|(i, t)| match t {
            DTerm::Const(id) => f[i] == *id,
            DTerm::Var(v) => match e.get(v) {
                Some(&b) => b == f[i],
                None => {
                    e.insert(*v, f[i]);
                    true
                }
            },
        }) {
            out.push(e);
        }
    }
    out
}

/// Does the NOT atom unify with `f` under `env` (unbound vars are wildcards; a
/// repeated wildcard must match equal terms)?
fn unifies(atom: &Atom, f: &[Id; 3], env: &Env) -> bool {
    let mut wild: Env = Env::default();
    atom.t.iter().enumerate().all(|(i, t)| match t {
        DTerm::Const(id) => f[i] == *id,
        DTerm::Var(v) => match env.get(v) {
            Some(&b) => b == f[i],
            None => *wild.entry(*v).or_insert(f[i]) == f[i],
        },
    })
}

/// The aggregate's grouped COUNT table: `(group key, count-literal id)` rows over
/// the distinct full matches of the aggregate body.
fn agg_table(
    dict: &mut Dict,
    agg: &AggAtom,
    store: &FxHashSet<[Id; 3]>,
) -> Vec<(Vec<Id>, Id)> {
    let mut envs = vec![Env::default()];
    for atom in &agg.body {
        envs = envs
            .iter()
            .flat_map(|env| atom_matches(atom, env, store))
            .collect();
    }
    // Distinct full tuples over the aggregate's local slots.
    let distinct: FxHashSet<Vec<Id>> = envs
        .iter()
        .map(|e| {
            (0..agg.n_slots as u32)
                .map(|v| e.get(&v).copied().unwrap_or(0))
                .collect()
        })
        .collect();
    let mut groups: FxHashMap<Vec<Id>, u64> = FxHashMap::default();
    for tup in &distinct {
        let key: Vec<Id> = agg.on.iter().map(|&(l, _)| tup[l as usize]).collect();
        *groups.entry(key).or_insert(0) += 1;
    }
    groups
        .into_iter()
        .map(|(k, c)| {
            let id = dict.intern_lit(&c.to_string(), XSD_INTEGER, None);
            (k, id)
        })
        .collect()
}

/// Independent exact numeric view for the fixtures' value space: `xsd:integer`
/// via `i128` lexical parsing (no shared code); anything else falls back to the
/// shared tower (the differential still exercises the full join/NAF/aggregate
/// pipeline independently).
fn num(dict: &Dict, id: Id) -> Option<i128> {
    let oxrdf::Term::Literal(l) = dict.term(id) else {
        return None;
    };
    if l.datatype().as_str() == "http://www.w3.org/2001/XMLSchema#integer" {
        return l.value().parse::<i128>().ok();
    }
    // Non-integer numerics: approximate via the shared tower's f64 view — the
    // fixtures keep FILTER operands integral, so this branch is a safety net.
    numeric_value(dict, id).map(|d| (d.f64() * 1e6) as i128)
}
