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

use super::{numeric_value, AggAtom, AggFunc, Atom, DTerm, Program, Rule, Stratification};
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
                        let g = [
                            subst(&h.t[0], &env),
                            subst(&h.t[1], &env),
                            subst(&h.t[2], &env),
                        ];
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

/// The aggregate's grouped table over the distinct full matches of its body.
fn agg_table(dict: &mut Dict, agg: &AggAtom, store: &FxHashSet<[Id; 3]>) -> Vec<(Vec<Id>, Id)> {
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
    let mut groups: FxHashMap<Vec<Id>, Vec<(Id, OracleNum)>> = FxHashMap::default();
    let mut counts: FxHashMap<Vec<Id>, u64> = FxHashMap::default();
    for tup in &distinct {
        let key: Vec<Id> = agg.on.iter().map(|&(l, _)| tup[l as usize]).collect();
        if agg.func == AggFunc::Count {
            *counts.entry(key).or_insert(0) += 1;
        } else {
            let id = tup[agg.value.expect("numeric aggregate has a value slot") as usize];
            if let Some(n) = oracle_num(dict, id) {
                groups.entry(key).or_default().push((id, n));
            }
        }
    }
    let mut out: Vec<(Vec<Id>, Id)> = counts
        .into_iter()
        .map(|(k, c)| {
            let id = dict.intern_lit(&c.to_string(), XSD_INTEGER, None);
            (k, id)
        })
        .collect();
    for (key, values) in groups {
        let id = match agg.func {
            AggFunc::Count => unreachable!(),
            AggFunc::Min | AggFunc::Max => values
                .iter()
                .copied()
                .reduce(|a, b| {
                    let keep_a = if agg.func == AggFunc::Min {
                        a.1.as_f64() <= b.1.as_f64()
                    } else {
                        a.1.as_f64() >= b.1.as_f64()
                    };
                    if keep_a {
                        a
                    } else {
                        b
                    }
                })
                .map(|(id, _)| id),
            AggFunc::Sum | AggFunc::Avg => oracle_sum(&values).map(|sum| {
                if agg.func == AggFunc::Avg {
                    let lexical = format_decimal_ratio(sum, values.len() as i128);
                    let datatype = match sum {
                        OracleNum::Exact(_) => "http://www.w3.org/2001/XMLSchema#decimal",
                        OracleNum::Float(_) => "http://www.w3.org/2001/XMLSchema#double",
                    };
                    dict.intern_lit(&lexical, datatype, None)
                } else {
                    match sum {
                        OracleNum::Exact(v) => dict.intern_lit(&v.to_string(), XSD_INTEGER, None),
                        OracleNum::Float(v) => dict.intern_lit(
                            &canonical_double(v),
                            "http://www.w3.org/2001/XMLSchema#double",
                            None,
                        ),
                    }
                }
            }),
        };
        if let Some(id) = id {
            out.push((key, id));
        }
    }
    out
}

/// [GPT-5.6] Independent oracle number: integral fixtures stay exact in `i128`;
/// every other XSD numeric is compared/folded through a separately parsed `f64`.
#[derive(Clone, Copy)]
enum OracleNum {
    Exact(i128),
    Float(f64),
}

impl OracleNum {
    fn as_f64(self) -> f64 {
        match self {
            Self::Exact(v) => v as f64,
            Self::Float(v) => v,
        }
    }
}

fn oracle_num(dict: &Dict, id: Id) -> Option<OracleNum> {
    let oxrdf::Term::Literal(l) = dict.term(id) else {
        return None;
    };
    let dt = l.datatype().as_str();
    if sparq_core::is_integer_datatype(dt) {
        return l.value().parse::<i128>().ok().map(OracleNum::Exact);
    }
    if matches!(
        dt,
        "http://www.w3.org/2001/XMLSchema#decimal"
            | "http://www.w3.org/2001/XMLSchema#float"
            | "http://www.w3.org/2001/XMLSchema#double"
    ) {
        return l.value().parse::<f64>().ok().map(OracleNum::Float);
    }
    None
}

fn oracle_sum(values: &[(Id, OracleNum)]) -> Option<OracleNum> {
    values.iter().try_fold(OracleNum::Exact(0), |acc, (_, n)| {
        Some(match (acc, *n) {
            (OracleNum::Exact(a), OracleNum::Exact(b)) => a
                .checked_add(b)
                .map(OracleNum::Exact)
                .unwrap_or_else(|| OracleNum::Float(a as f64 + b as f64)),
            (a, b) => OracleNum::Float(a.as_f64() + b.as_f64()),
        })
    })
}

fn format_decimal_ratio(sum: OracleNum, count: i128) -> String {
    match sum {
        OracleNum::Exact(v) => {
            let neg = v < 0;
            let mag = v.unsigned_abs();
            let den = count as u128;
            for scale in 1..=18 {
                if let Some(scaled) = mag.checked_mul(10u128.pow(scale)) {
                    if scaled % den == 0 {
                        return decimal_lexical(scaled / den, scale, neg);
                    }
                }
            }
            let scaled = mag.saturating_mul(10u128.pow(18));
            let rounded = scaled / den + u128::from((scaled % den) * 2 >= den);
            decimal_lexical(rounded, 18, neg)
        }
        OracleNum::Float(v) => canonical_double(v / count as f64),
    }
}

fn decimal_lexical(mant: u128, scale: u32, neg: bool) -> String {
    let digits = mant.to_string();
    let split = digits.len().saturating_sub(scale as usize);
    let mut out = String::new();
    if neg && mant != 0 {
        out.push('-');
    }
    if split == 0 {
        out.push_str("0.");
        out.extend(std::iter::repeat_n('0', scale as usize - digits.len()));
        out.push_str(&digits);
    } else {
        out.push_str(&digits[..split]);
        out.push('.');
        out.push_str(&digits[split..]);
    }
    out
}

fn canonical_double(v: f64) -> String {
    if v.is_nan() {
        return "NaN".into();
    }
    if v == f64::INFINITY {
        return "INF".into();
    }
    if v == f64::NEG_INFINITY {
        return "-INF".into();
    }
    let s = format!("{v:E}");
    match s.split_once('E') {
        Some((m, e)) if !m.contains('.') => format!("{m}.0E{e}"),
        _ => s,
    }
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
