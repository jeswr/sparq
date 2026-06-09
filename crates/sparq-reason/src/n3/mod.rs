//! Notation3 (N3) rule reasoning — a forward-chaining engine toward EYE-reasoner parity.
//!
//! N3 adds rules (`{ premise } => { conclusion }`), variables (`?x`), formulae (`{ … }`) and
//! **builtins** (`math:`, `string:`, `log:`, …) on top of Turtle. EYE is a mature reasoner
//! with hundreds of builtins; this is the foundation: a parser (`parser`), a term model
//! (`model`), and a semi-naive forward chainer that applies rules to a fixpoint with variable
//! binding and a growing set of builtins. Coverage expands builtin-by-builtin, validated
//! against EYE's own test cases (see the `eye_cases` tests).
//!
//! Builtins implemented (roadmap T12 — growing toward EYE parity):
//!   * `math:` comparisons (greaterThan/lessThan/notGreaterThan/notLessThan/equalTo/notEqualTo)
//!     and functional (sum/difference/product/quotient/max/min/exponentiation/negation/
//!     absoluteValue/rounded/floor/ceiling) over `( … )` list arguments;
//!   * `string:` concatenation/length/contains/startsWith/endsWith/greaterThan/lessThan;
//!   * `list:` length;  `time:` year/month/day/hours/minutes/seconds;  `log:` equalTo/notEqualTo.
//! Next increments (T12): `list:` member/first/append, more `string:` (matches/replace),
//! `log:` includes/conjunction, backward chaining (`<=`).

mod model;
mod parser;

pub use model::{Rule, Term};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};
use std::collections::HashMap;

/// Facts + access indexes, so a rule-body join atom is an O(1)/O(matches) lookup instead of a
/// full scan. Indexed by (predicate, subject)→objects, (predicate, object)→subjects, and
/// predicate→facts — the patterns that arise when a rule atom's predicate (and often one
/// argument) is already bound. Maintained incrementally as the closure grows. Without this,
/// even semi-naive evaluation degrades to O(N²) on recursive rule chains (DeepTaxonomy).
#[derive(Default)]
struct FactIndex {
    all: FxHashSet<[Term; 3]>,
    ps: FxHashMap<(Term, Term), Vec<Term>>, // (pred, subj) -> objects
    po: FxHashMap<(Term, Term), Vec<Term>>, // (pred, obj) -> subjects
    p: FxHashMap<Term, Vec<[Term; 3]>>,     // pred -> facts (predicate-only-bound)
}

impl FactIndex {
    fn from_iter(facts: impl IntoIterator<Item = [Term; 3]>) -> FactIndex {
        let mut ix = FactIndex::default();
        for f in facts {
            ix.insert(f);
        }
        ix
    }
    fn contains(&self, t: &[Term; 3]) -> bool {
        self.all.contains(t)
    }
    fn len(&self) -> usize {
        self.all.len()
    }
    fn insert(&mut self, t: [Term; 3]) -> bool {
        if !self.all.insert(t.clone()) {
            return false;
        }
        let [s, p, o] = &t;
        self.ps.entry((p.clone(), s.clone())).or_default().push(o.clone());
        self.po.entry((p.clone(), o.clone())).or_default().push(s.clone());
        self.p.entry(p.clone()).or_default().push(t.clone());
        true
    }
    /// Candidate facts matching a (partially-ground) pattern, via the most selective index.
    fn candidates(&self, s: &Term, p: &Term, o: &Term) -> Vec<[Term; 3]> {
        let (sg, pg, og) = (s.is_ground(), p.is_ground(), o.is_ground());
        if pg && sg {
            self.ps
                .get(&(p.clone(), s.clone()))
                .map(|os| os.iter().map(|ob| [s.clone(), p.clone(), ob.clone()]).collect())
                .unwrap_or_default()
        } else if pg && og {
            self.po
                .get(&(p.clone(), o.clone()))
                .map(|ss| ss.iter().map(|sb| [sb.clone(), p.clone(), o.clone()]).collect())
                .unwrap_or_default()
        } else if pg {
            self.p.get(p).cloned().unwrap_or_default()
        } else {
            self.all.iter().cloned().collect() // predicate unbound — rare; fall back to scan
        }
    }
}

const MATH: &str = "http://www.w3.org/2000/10/swap/math#";
const LOG: &str = "http://www.w3.org/2000/10/swap/log#";
const STRING: &str = "http://www.w3.org/2000/10/swap/string#";
const LIST: &str = "http://www.w3.org/2000/10/swap/list#";
const TIME: &str = "http://www.w3.org/2000/10/swap/time#";

/// One derivation step: a `conclusion` triple was produced by rule `rule` (its index in the
/// document's rule order) from the ground `premises` (the supporting facts under the binding).
pub struct ProofStep {
    pub conclusion: [Id; 3],
    pub rule: usize,
    pub premises: Vec<[Id; 3]>,
}

/// Parse N3 `src`, run the rule closure, and return the entailed GROUND triples interned into
/// `dict`. The rules/formulae/variables are consumed by reasoning; only ground facts remain.
pub fn reason_n3(dict: &mut Dict, src: &str) -> Result<Vec<[Id; 3]>, String> {
    Ok(reason_n3_proof(dict, src)?.0)
}

/// As [`reason_n3`], but also return the derivation (a [`ProofStep`] for each NEWLY-derived
/// triple, in derivation order) — the EYE `--proof` analogue.
pub fn reason_n3_proof(dict: &mut Dict, src: &str) -> Result<(Vec<[Id; 3]>, Vec<ProofStep>), String> {
    let parsed = parser::parse(src)?;
    let mut facts = FactIndex::from_iter(parsed.facts);
    // Derivation steps at the term level (interned to ids once at the end).
    let mut steps: Vec<([Term; 3], usize, Vec<[Term; 3]>)> = Vec::new();

    // SEMI-NAIVE fixpoint: each round, a positive rule only fires on bindings that involve at
    // least one NEWLY-derived fact (the `delta`) — run once per join-atom position, with that
    // atom restricted to `delta` and the rest to all facts. This avoids re-deriving the whole
    // closure every round (the naive blow-up on recursive rule chains). Rules with scoped
    // negation are non-monotonic, so they re-evaluate against ALL facts each round (correct,
    // matches the prior behaviour); pure-builtin rules (no join atom) fire only in round 0.
    let rule_meta: Vec<(Vec<usize>, bool)> = parsed
        .rules
        .iter()
        .map(|r| {
            let lists = extract_lists(&r.premise);
            let joins: Vec<usize> =
                r.premise.iter().enumerate().filter(|(_, p)| is_join_atom(p, &lists)).map(|(i, _)| i).collect();
            let has_neg = r.premise.iter().any(|p| scoped_negation(&p[1]).is_some());
            (joins, has_neg)
        })
        .collect();

    let mut delta: FxHashSet<[Term; 3]> = facts.all.clone(); // round 0: every fact is "new"
    let mut first_round = true;
    loop {
        let mut produced: Vec<([Term; 3], usize, Vec<[Term; 3]>)> = Vec::new();
        for (ri, rule) in parsed.rules.iter().enumerate() {
            let (joins, has_neg) = &rule_meta[ri];
            let bindings: Vec<Binding> = if *has_neg || joins.is_empty() {
                // non-monotonic / constant rule: full evaluation (negation) or round-0 only.
                if *has_neg || first_round {
                    match_premise(&rule.premise, &facts)
                } else {
                    Vec::new()
                }
            } else {
                // Semi-naive: union over delta-at-each-join-position (dedup via facts.insert).
                let mut bs = Vec::new();
                for &k in joins {
                    bs.extend(match_premise_seeded(&rule.premise, &facts, &Binding::new(), Some((&delta, k))));
                }
                bs
            };
            for b in bindings {
                for c in &rule.conclusion {
                    if let Some(g) = ground_triple(c, &b) {
                        if !facts.contains(&g) {
                            // The supporting facts: premise patterns instantiated under b that
                            // are actual facts (excludes builtins / list structure).
                            let prem: Vec<[Term; 3]> = rule
                                .premise
                                .iter()
                                .filter_map(|p| ground_triple(p, &b))
                                .filter(|t| facts.contains(t))
                                .collect();
                            produced.push((g, ri, prem));
                        }
                    }
                }
            }
        }
        let mut new_delta: FxHashSet<[Term; 3]> = FxHashSet::default();
        for (g, ri, prem) in produced {
            if facts.insert(g.clone()) {
                new_delta.insert(g.clone());
                steps.push((g, ri, prem));
            }
        }
        first_round = false;
        if new_delta.is_empty() {
            break;
        }
        delta = new_delta;
    }

    // Intern the ground closure into the dictionary.
    let mut out = Vec::with_capacity(facts.len());
    for t in &facts.all {
        out.push([intern(dict, &t[0])?, intern(dict, &t[1])?, intern(dict, &t[2])?]);
    }
    // Intern the proof steps.
    let mut proof = Vec::with_capacity(steps.len());
    for (g, ri, prem) in &steps {
        let it = |t: &[Term; 3], d: &mut Dict| -> Result<[Id; 3], String> {
            Ok([intern(d, &t[0])?, intern(d, &t[1])?, intern(d, &t[2])?])
        };
        let conclusion = it(g, dict)?;
        let premises = prem.iter().map(|p| it(p, dict)).collect::<Result<Vec<_>, _>>()?;
        proof.push(ProofStep { conclusion, rule: *ri, premises });
    }
    Ok((out, proof))
}

type Binding = HashMap<String, Term>;

/// All variable bindings under which every premise pattern holds (joining against `facts`,
/// evaluating builtins as filters/computations). N3 collections `( … )` in the premise are
/// rule-local list STRUCTURE (rdf:first/rest over fresh bnodes), not data to match — they are
/// extracted up front and consumed by the functional builtins (e.g. `math:sum`).
fn match_premise(premise: &[[Term; 3]], facts: &FactIndex) -> Vec<Binding> {
    match_premise_seeded(premise, facts, &Binding::new(), None)
}

/// Match `premise` starting from an existing partial binding `seed`. For SEMI-NAIVE
/// evaluation, `delta_at = Some((delta, k))` restricts the join atom at premise index `k` to
/// match only the `delta` set (the newly-derived facts) rather than the full `facts` — so the
/// driver, by running once per join-atom index, considers only bindings that involve ≥1 new
/// fact. `None` = naive (every join atom matches all facts); also used for the negation
/// sub-formula recursion.
fn match_premise_seeded(
    premise: &[[Term; 3]],
    facts: &FactIndex,
    seed: &Binding,
    delta_at: Option<(&FxHashSet<[Term; 3]>, usize)>,
) -> Vec<Binding> {
    // Semi-naive: seed from the DELTA atom first (delta is small → most selective), then
    // evaluate the rest against the index. Doing the delta atom first makes it prune early
    // instead of letting a non-delta atom do a full predicate-index scan.
    if let Some((delta, k)) = delta_at {
        let mut seeds = Vec::new();
        for fact in delta {
            if let Some(nb) = unify(&premise[k], fact, seed) {
                seeds.push(nb);
            }
        }
        if seeds.is_empty() {
            return Vec::new();
        }
        let rest: Vec<[Term; 3]> =
            premise.iter().enumerate().filter(|&(i, _)| i != k).map(|(_, p)| p.clone()).collect();
        let mut out = Vec::new();
        for s in &seeds {
            out.extend(match_premise_seeded(&rest, facts, s, None));
        }
        return out;
    }
    let lists = extract_lists(premise);
    let mut bindings: Vec<Binding> = vec![seed.clone()];
    for pat in premise {
        if is_list_struct(pat, &lists) {
            continue; // structural, handled by list resolution
        }
        // log:includes / log:notIncludes — does the object formula hold (scoped negation as
        // failure for notIncludes)? The subject formula is treated as the current store.
        if let Some(is_not) = scoped_negation(&pat[1]) {
            let inner: &[[Term; 3]] = match &pat[2] {
                Term::Formula(t) => t,
                _ => &[],
            };
            bindings.retain(|b| {
                let holds = !match_premise_seeded(inner, facts, b, None).is_empty();
                if is_not {
                    !holds
                } else {
                    holds
                }
            });
            if bindings.is_empty() {
                break;
            }
            continue;
        }
        if let Some(gen) = list_generator(&pat[1]) {
            // list:member / list:in — generate one binding per list member.
            let (list_pos, var_pos) = match gen {
                ListGen::Member => (&pat[0], &pat[2]),
                ListGen::In => (&pat[2], &pat[0]),
            };
            let mut next = Vec::new();
            for b in &bindings {
                let head = apply(list_pos, b);
                if let Some(members) = lists.get(&head) {
                    for m in members {
                        let mv = apply(m, b);
                        let mut nb = b.clone();
                        if unify_term(var_pos, &mv, &mut nb) {
                            next.push(nb);
                        }
                    }
                }
            }
            bindings = next;
        } else if let Some(f) = functional_builtin(&pat[1]) {
            bindings = bindings
                .into_iter()
                .filter_map(|b| eval_functional(f, &pat[0], &pat[2], &lists, b))
                .collect();
        } else if let Some(op) = builtin(&pat[1]) {
            bindings.retain(|b| eval_builtin(op, &pat[0], &pat[2], b));
        } else {
            // Join atom: selective FactIndex lookup (no full scan) for each current binding.
            let mut next = Vec::new();
            for b in &bindings {
                let cands = facts.candidates(&apply(&pat[0], b), &apply(&pat[1], b), &apply(&pat[2], b));
                for fact in &cands {
                    if let Some(nb) = unify(pat, fact, b) {
                        next.push(nb);
                    }
                }
            }
            bindings = next;
        }
        if bindings.is_empty() {
            break;
        }
    }
    bindings
}

/// Whether a premise pattern is a JOIN atom (matched against facts), as opposed to a builtin,
/// list generator/structure, or scoped-negation atom.
fn is_join_atom(pat: &[Term; 3], lists: &HashMap<Term, Vec<Term>>) -> bool {
    builtin(&pat[1]).is_none()
        && functional_builtin(&pat[1]).is_none()
        && list_generator(&pat[1]).is_none()
        && scoped_negation(&pat[1]).is_none()
        && !is_list_struct(pat, lists)
}

/// Resolve every list node in `premise` (rooted at an rdf:first) to its member sequence.
fn extract_lists(premise: &[[Term; 3]]) -> HashMap<Term, Vec<Term>> {
    use parser::{RDF_FIRST, RDF_NIL, RDF_REST};
    let (is, ir) = (Term::Iri(RDF_FIRST.into()), Term::Iri(RDF_REST.into()));
    let nil = Term::Iri(RDF_NIL.into());
    let mut first: HashMap<Term, Term> = HashMap::new();
    let mut rest: HashMap<Term, Term> = HashMap::new();
    for [s, p, o] in premise {
        if *p == is {
            first.insert(s.clone(), o.clone());
        } else if *p == ir {
            rest.insert(s.clone(), o.clone());
        }
    }
    let mut lists = HashMap::new();
    for head in first.keys() {
        let mut members = Vec::new();
        let mut cur = head.clone();
        // follow first/rest to nil (bounded by node count to avoid cycles)
        for _ in 0..first.len() + 1 {
            match first.get(&cur) {
                Some(m) => members.push(m.clone()),
                None => break,
            }
            match rest.get(&cur) {
                Some(n) if *n != nil => cur = n.clone(),
                _ => break,
            }
        }
        lists.insert(head.clone(), members);
    }
    lists
}

/// Is `pat` an rdf:first/rest triple belonging to an extracted list (rule structure)?
fn is_list_struct(pat: &[Term; 3], lists: &HashMap<Term, Vec<Term>>) -> bool {
    use parser::{RDF_FIRST, RDF_REST};
    matches!(&pat[1], Term::Iri(i) if i == RDF_FIRST || i == RDF_REST) && lists.contains_key(&pat[0])
}

/// Try to unify pattern triple `pat` with ground fact `f`, extending binding `b`.
fn unify(pat: &[Term; 3], f: &[Term; 3], b: &Binding) -> Option<Binding> {
    let mut nb = b.clone();
    for i in 0..3 {
        if !unify_term(&pat[i], &f[i], &mut nb) {
            return None;
        }
    }
    Some(nb)
}

fn unify_term(pat: &Term, val: &Term, b: &mut Binding) -> bool {
    match pat {
        Term::Var(v) => match b.get(v) {
            Some(existing) => existing == val,
            None => {
                b.insert(v.clone(), val.clone());
                true
            }
        },
        other => other == val,
    }
}

/// Substitute bound variables in `t`; returns the term (possibly still containing free vars).
fn apply(t: &Term, b: &Binding) -> Term {
    match t {
        Term::Var(v) => b.get(v).cloned().unwrap_or_else(|| t.clone()),
        _ => t.clone(),
    }
}

/// Instantiate a conclusion triple under binding `b`; `None` if any term stays non-ground.
fn ground_triple(t: &[Term; 3], b: &Binding) -> Option<[Term; 3]> {
    let g = [apply(&t[0], b), apply(&t[1], b), apply(&t[2], b)];
    if g.iter().all(|x| x.is_ground()) {
        Some(g)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
enum Builtin {
    // numeric (math:)
    Gt,
    Lt,
    NotGt,
    NotLt,
    MathEq,
    MathNe,
    // term (log:)
    LogEq,
    LogNe,
    // string (string:)
    StrContains,
    StrStarts,
    StrEnds,
    StrGt,
    StrLt,
}

fn builtin(p: &Term) -> Option<Builtin> {
    let Term::Iri(i) = p else { return None };
    if let Some(f) = i.strip_prefix(MATH) {
        return Some(match f {
            "greaterThan" => Builtin::Gt,
            "lessThan" => Builtin::Lt,
            "notGreaterThan" => Builtin::NotGt,
            "notLessThan" => Builtin::NotLt,
            "equalTo" => Builtin::MathEq,
            "notEqualTo" => Builtin::MathNe,
            _ => return None,
        });
    }
    if let Some(f) = i.strip_prefix(LOG) {
        return Some(match f {
            "equalTo" => Builtin::LogEq,
            "notEqualTo" => Builtin::LogNe,
            _ => return None,
        });
    }
    if let Some(f) = i.strip_prefix(STRING) {
        return Some(match f {
            "contains" => Builtin::StrContains,
            "startsWith" => Builtin::StrStarts,
            "endsWith" => Builtin::StrEnds,
            "greaterThan" => Builtin::StrGt,
            "lessThan" => Builtin::StrLt,
            _ => return None,
        });
    }
    None
}

fn eval_builtin(op: Builtin, s: &Term, o: &Term, b: &Binding) -> bool {
    let (s, o) = (apply(s, b), apply(o, b));
    match op {
        Builtin::LogEq => s == o,
        Builtin::LogNe => s != o,
        Builtin::StrContains | Builtin::StrStarts | Builtin::StrEnds | Builtin::StrGt | Builtin::StrLt => {
            let (Some(x), Some(y)) = (lex(&s), lex(&o)) else { return false };
            match op {
                Builtin::StrContains => x.contains(y),
                Builtin::StrStarts => x.starts_with(y),
                Builtin::StrEnds => x.ends_with(y),
                Builtin::StrGt => x > y,
                Builtin::StrLt => x < y,
                _ => unreachable!(),
            }
        }
        _ => {
            let (Some(x), Some(y)) = (num(&s), num(&o)) else { return false };
            match op {
                Builtin::Gt => x > y,
                Builtin::Lt => x < y,
                Builtin::NotGt => x <= y,
                Builtin::NotLt => x >= y,
                Builtin::MathEq => x == y,
                Builtin::MathNe => x != y,
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Clone, Copy)]
enum ListGen {
    Member, // ?list list:member ?x
    In,     // ?x list:in ?list
}

fn list_generator(p: &Term) -> Option<ListGen> {
    let Term::Iri(i) = p else { return None };
    match i.strip_prefix(LIST) {
        Some("member") => Some(ListGen::Member),
        Some("in") => Some(ListGen::In),
        _ => None,
    }
}

/// `log:includes` (→ `Some(false)`) / `log:notIncludes` (→ `Some(true)`, negation as failure).
fn scoped_negation(p: &Term) -> Option<bool> {
    let Term::Iri(i) = p else { return None };
    match i.strip_prefix(LOG) {
        Some("includes") => Some(false),
        Some("notIncludes") => Some(true),
        _ => None,
    }
}

/// The lexical string of a literal term (for `string:` builtins).
fn lex(t: &Term) -> Option<&str> {
    match t {
        Term::Lit(v, _, _) => Some(v.as_str()),
        _ => None,
    }
}

/// The numeric value of a literal term (for `math:` builtins).
fn num(t: &Term) -> Option<f64> {
    match t {
        Term::Lit(v, _, _) => v.parse::<f64>().ok(),
        _ => None,
    }
}

/// Functional `math:`/`string:`/`list:`/`time:` builtins: the subject is a `( … )` list (or a
/// single value for the unary ops), and the object is computed.
#[derive(Clone, Copy)]
enum Func {
    // list-arg
    Sum,
    Difference,
    Product,
    Quotient,
    Max,
    Min,
    Exponentiation,
    Concat,    // string:concatenation
    Length,    // list:length
    StrLength, // string:length (Unicode scalar count)
    First,     // list:first
    Last,      // list:last
    // single-value-arg (unary math)
    Negation,
    AbsoluteValue,
    Rounded,
    Floor,
    Ceiling,
    // single-value-arg (time: components of an xsd:dateTime)
    Year,
    Month,
    Day,
    Hours,
    Minutes,
    Seconds,
}

fn functional_builtin(p: &Term) -> Option<Func> {
    let Term::Iri(i) = p else { return None };
    if let Some(f) = i.strip_prefix(MATH) {
        return Some(match f {
            "sum" => Func::Sum,
            "difference" => Func::Difference,
            "product" => Func::Product,
            "quotient" => Func::Quotient,
            "max" => Func::Max,
            "min" => Func::Min,
            "exponentiation" => Func::Exponentiation,
            "negation" => Func::Negation,
            "absoluteValue" => Func::AbsoluteValue,
            "rounded" => Func::Rounded,
            "floor" => Func::Floor,
            "ceiling" => Func::Ceiling,
            _ => return None,
        });
    }
    if let Some(f) = i.strip_prefix(TIME) {
        return Some(match f {
            "year" => Func::Year,
            "month" => Func::Month,
            "day" => Func::Day,
            "hours" => Func::Hours,
            "minutes" => Func::Minutes,
            "seconds" => Func::Seconds,
            _ => return None,
        });
    }
    match (i.strip_prefix(STRING), i.strip_prefix(LIST)) {
        (Some("concatenation"), _) => Some(Func::Concat),
        (Some("length"), _) => Some(Func::StrLength),
        (_, Some("length")) => Some(Func::Length),
        (_, Some("first")) => Some(Func::First),
        (_, Some("last")) => Some(Func::Last),
        _ => None,
    }
}

/// Evaluate a functional builtin `(members) op object`: resolve the list members under `b`,
/// compute, then either bind the object variable to the result or filter if it is ground.
fn eval_functional(
    f: Func,
    subj: &Term,
    obj: &Term,
    lists: &HashMap<Term, Vec<Term>>,
    b: Binding,
) -> Option<Binding> {
    // Arguments: the list members, or a singleton for the unary (math:/time:) ops.
    let args: Vec<Term> = match lists.get(subj) {
        Some(members) => members.iter().map(|m| apply(m, &b)).collect(),
        None => vec![apply(subj, &b)],
    };
    if args.is_empty() {
        return None;
    }
    let result: Term = match f {
        Func::Concat => {
            let mut s = String::new();
            for a in &args {
                match a {
                    Term::Lit(v, _, _) => s.push_str(v),
                    _ => return None,
                }
            }
            Term::Lit(s, "http://www.w3.org/2001/XMLSchema#string".into(), None)
        }
        Func::Length => number_term(args.len() as f64),
        Func::StrLength => number_term(lex(&args[0])?.chars().count() as f64),
        Func::First => args.first()?.clone(),
        Func::Last => args.last()?.clone(),
        Func::Year | Func::Month | Func::Day | Func::Hours | Func::Minutes | Func::Seconds => {
            number_term(datetime_part(lex(&args[0])?, f)? as f64)
        }
        _ => {
            let nums: Vec<f64> = args.iter().map(num).collect::<Option<_>>()?;
            let two = |n: &[f64]| if n.len() == 2 { Some((n[0], n[1])) } else { None };
            let v = match f {
                Func::Sum => nums.iter().sum(),
                Func::Product => nums.iter().product(),
                Func::Max => nums.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                Func::Min => nums.iter().copied().fold(f64::INFINITY, f64::min),
                Func::Difference => {
                    let (a, b) = two(&nums)?;
                    a - b
                }
                Func::Quotient => {
                    let (a, b) = two(&nums)?;
                    if b == 0.0 {
                        return None;
                    }
                    a / b
                }
                Func::Exponentiation => {
                    let (a, b) = two(&nums)?;
                    a.powf(b)
                }
                Func::Negation => -nums[0],
                Func::AbsoluteValue => nums[0].abs(),
                Func::Rounded => nums[0].round(),
                Func::Floor => nums[0].floor(),
                Func::Ceiling => nums[0].ceil(),
                _ => unreachable!(),
            };
            number_term(v)
        }
    };
    // Bind the object variable, or (if already ground) filter on equality.
    let mut nb = b;
    if unify_term(obj, &result, &mut nb) {
        Some(nb)
    } else {
        None
    }
}

/// Extract a component of an `xsd:dateTime`/`xsd:date` lexical form for `time:` builtins.
/// Lexical: `[-]YYYY-MM-DD[Thh:mm:ss[.sss]][Z|±hh:mm]`.
fn datetime_part(s: &str, f: Func) -> Option<i64> {
    let (date, time) = s.split_once('T').unwrap_or((s, ""));
    let neg = date.starts_with('-');
    let mut dparts = date.trim_start_matches('-').split('-');
    match f {
        Func::Year => dparts.next()?.parse::<i64>().ok().map(|y| if neg { -y } else { y }),
        Func::Month => dparts.nth(1)?.parse().ok(),
        Func::Day => dparts.nth(2)?.parse().ok(),
        Func::Hours | Func::Minutes | Func::Seconds => {
            // strip any timezone (Z, +hh:mm, -hh:mm) — the time itself has no +/-/Z.
            let t = time.split(['+', '-', 'Z']).next().unwrap_or(time);
            let idx = match f {
                Func::Hours => 0,
                Func::Minutes => 1,
                Func::Seconds => 2,
                _ => unreachable!(),
            };
            let part = t.split(':').nth(idx)?;
            part.split('.').next().unwrap_or(part).parse().ok()
        }
        _ => None,
    }
}

/// Render an `f64` result as an N3 numeric literal (integer when whole, else decimal).
fn number_term(v: f64) -> Term {
    if v.fract() == 0.0 && v.abs() < 9.007e15 {
        Term::Lit((v as i64).to_string(), "http://www.w3.org/2001/XMLSchema#integer".into(), None)
    } else {
        Term::Lit(format!("{v}"), "http://www.w3.org/2001/XMLSchema#decimal".into(), None)
    }
}

/// Intern an N3 ground term into the dictionary.
fn intern(dict: &mut Dict, t: &Term) -> Result<Id, String> {
    Ok(match t {
        Term::Iri(i) => dict.intern_iri(i),
        Term::Lit(v, dt, lang) => dict.intern_lit(v, dt, lang.as_deref()),
        Term::Blank(b) => dict.intern_blank(b),
        Term::Var(_) | Term::Formula(_) => return Err("non-ground term in closure".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn closure(src: &str) -> (Dict, FxHashSet<[Id; 3]>) {
        let mut dict = Dict::new();
        let triples = reason_n3(&mut dict, src).unwrap();
        let set = triples.into_iter().collect();
        (dict, set)
    }
    fn id(dict: &Dict, iri: &str) -> Id {
        use oxrdf::{NamedNode, Term as OT};
        dict.lookup(&OT::NamedNode(NamedNode::new_unchecked(iri.to_string())))
    }
    fn has(dict: &Dict, set: &FxHashSet<[Id; 3]>, s: &str, p: &str, o: &str) -> bool {
        let (a, b, c) = (id(dict, s), id(dict, p), id(dict, o));
        a != 0 && b != 0 && c != 0 && set.contains(&[a, b, c])
    }

    #[test]
    fn proof_records_derivations() {
        // Chained rules: each derived triple should have a proof step naming its premise.
        let src = r#"
            @prefix : <http://ex/> .
            :Socrates a :Man .
            { ?x a :Man } => { ?x a :Mortal } .
            { ?x a :Mortal } => { ?x a :Being } .
        "#;
        let mut dict = Dict::new();
        let (_triples, proof) = reason_n3_proof(&mut dict, src).unwrap();
        // Two derivations: Socrates a Mortal (from Socrates a Man), Socrates a Being (from Mortal).
        assert_eq!(proof.len(), 2, "two derivation steps");
        let mortal = dict.intern_iri("http://ex/Mortal");
        let man = dict.intern_iri("http://ex/Man");
        let socrates = dict.intern_iri("http://ex/Socrates");
        let ty = dict.intern_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        let step = proof.iter().find(|s| s.conclusion == [socrates, ty, mortal]).expect("Mortal step");
        assert_eq!(step.premises, vec![[socrates, ty, man]], "Mortal derived from (Socrates a Man)");
    }

    #[test]
    fn simple_rule_socrates() {
        // The canonical N3 rule: every Man is Mortal.
        let src = r#"
            @prefix : <http://ex/> .
            :Socrates a :Man .
            { ?x a :Man } => { ?x a :Mortal } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/Socrates", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://ex/Mortal"));
    }

    #[test]
    fn backward_rule_arrow() {
        // `{ conclusion } <= { premise }` — same closure as `premise => conclusion`.
        let src = r#"
            @prefix : <http://ex/> .
            :Socrates a :Man .
            { ?x a :Mortal } <= { ?x a :Man } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/Socrates", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://ex/Mortal"));
    }

    #[test]
    fn transitive_via_rule() {
        // Define transitivity with an N3 rule and close a chain.
        let src = r#"
            @prefix : <http://ex/> .
            :a :before :b . :b :before :c . :c :before :d .
            { ?x :before ?y . ?y :before ?z } => { ?x :before ?z } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/a", "http://ex/before", "http://ex/c"));
        assert!(has(&d, &s, "http://ex/a", "http://ex/before", "http://ex/d"));
        assert!(has(&d, &s, "http://ex/b", "http://ex/before", "http://ex/d"));
    }

    #[test]
    fn functional_math_sum_and_product() {
        // (?a ?b) math:sum ?s computes ?s; chained with product.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :rect :width 4 ; :height 5 .
            { ?r :width ?w . ?r :height ?h . (?w ?h) math:product ?area } => { ?r :area ?area } .
            { ?r :width ?w . ?r :height ?h . (?w ?h) math:sum ?half } => { ?r :perimeterHalf ?half } .
        "#;
        let (mut d, s) = closure(src);
        let int = "http://www.w3.org/2001/XMLSchema#integer";
        let area_id = d.intern_lit("20", int, None); // 4*5
        let half_id = d.intern_lit("9", int, None); // 4+5
        assert!(s.contains(&[id(&d, "http://ex/rect"), id(&d, "http://ex/area"), area_id]), "math:product 4*5=20");
        assert!(s.contains(&[id(&d, "http://ex/rect"), id(&d, "http://ex/perimeterHalf"), half_id]), "math:sum 4+5=9");
    }

    #[test]
    fn functional_string_length() {
        // ?w string:length ?n  — Unicode scalar count (T12).
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :a :word "hello" .
            { ?x :word ?w . ?w string:length ?n } => { ?x :wordLen ?n } .
        "#;
        let (mut d, s) = closure(src);
        let five = d.intern_lit("5", "http://www.w3.org/2001/XMLSchema#integer", None);
        assert!(s.contains(&[id(&d, "http://ex/a"), id(&d, "http://ex/wordLen"), five]), "string:length(hello)=5");
    }

    #[test]
    fn functional_list_first_last() {
        // ( … ) list:first / list:last over a rule-local collection (T12).
        let src = r#"
            @prefix : <http://ex/> .
            @prefix list: <http://www.w3.org/2000/10/swap/list#> .
            { ( :a :b :c ) list:first ?f . ( :a :b :c ) list:last ?z } => { :s :first ?f . :s :last ?z } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/s", "http://ex/first", "http://ex/a"));
        assert!(has(&d, &s, "http://ex/s", "http://ex/last", "http://ex/c"));
    }

    #[test]
    fn path_syntax_forward() {
        // `?x!:mother :knows ?f` — the path ?x!:mother is the mother node; desugars to
        // (?x :mother _m)(_m :knows ?f).
        let src = r#"
            @prefix : <http://ex/> .
            :alice :mother :mary .
            :mary :knows :bob .
            { ?x!:mother :knows ?f } => { ?x :motherKnows ?f } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/alice", "http://ex/motherKnows", "http://ex/bob"), "forward path !");
    }

    #[test]
    fn path_syntax_backward() {
        // `?x^:mother` — the subject whose :mother is ?x (i.e. ?x's children).
        let src = r#"
            @prefix : <http://ex/> .
            :alice :mother :mary .
            { ?child :mother ?m . ?m^:mother :name ?cn } => { ?m :hasChildNamed ?cn } .
            :alice :name "Alice" .
        "#;
        // ?m^:mother is a child of ?m; simpler: verify ^ desugars to a backward triple.
        let (d, s) = closure(src);
        // ?m=mary: mary^:mother = alice (alice :mother mary); alice :name "Alice"
        // ⊢ mary :hasChildNamed "Alice".
        assert!(
            s.iter().any(|[a, p, _]| *a == id(&d, "http://ex/mary") && *p == id(&d, "http://ex/hasChildNamed")),
            "backward path ^ derived mary :hasChildNamed"
        );
    }

    #[test]
    fn scoped_negation_not_includes() {
        // log:notIncludes — negation as failure: a Person with no recorded email is :NoEmail.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix log: <http://www.w3.org/2000/10/swap/log#> .
            :alice a :Person .
            :bob a :Person .
            :bob :hasEmail "bob@x" .
            { ?x a :Person . { } log:notIncludes { ?x :hasEmail ?e } } => { ?x a :NoEmail } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/alice", ty, "http://ex/NoEmail"), "alice has no email → NoEmail");
        assert!(!has(&d, &s, "http://ex/bob", ty, "http://ex/NoEmail"), "bob has email → excluded");
    }

    #[test]
    fn string_contains_filter() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :a :label "hello world" .
            :b :label "goodbye" .
            { ?x :label ?l . ?l string:contains "world" } => { ?x a :Matched } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/a", ty, "http://ex/Matched"), "string:contains match");
        assert!(!has(&d, &s, "http://ex/b", ty, "http://ex/Matched"), "non-match excluded");
    }

    #[test]
    fn list_member_generator() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix list: <http://www.w3.org/2000/10/swap/list#> .
            :s :p :o .
            { ( :a :b :c ) list:member ?x } => { ?x a :Listed } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        for m in ["a", "b", "c"] {
            assert!(has(&d, &s, &format!("http://ex/{m}"), ty, "http://ex/Listed"), "list:member {m}");
        }
    }

    #[test]
    fn time_components_and_unary_math() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix time: <http://www.w3.org/2000/10/swap/time#> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :e :when "2024-03-15T10:30:45"^^<http://www.w3.org/2001/XMLSchema#dateTime> .
            :e :temp -5 .
            { ?x :when ?d . ?d time:year ?y . ?d time:month ?mo . ?d time:day ?dd } => { ?x :y ?y ; :mo ?mo ; :dd ?dd } .
            { ?x :temp ?t . ?t math:absoluteValue ?a } => { ?x :absTemp ?a } .
        "#;
        let (mut d, s) = closure(src);
        let int = "http://www.w3.org/2001/XMLSchema#integer";
        let lit = |d: &mut Dict, n: &str| d.intern_lit(n, int, None);
        let (y, mo, dd, a) = (lit(&mut d, "2024"), lit(&mut d, "3"), lit(&mut d, "15"), lit(&mut d, "5"));
        let e = id(&d, "http://ex/e");
        assert!(s.contains(&[e, id(&d, "http://ex/y"), y]), "time:year = 2024");
        assert!(s.contains(&[e, id(&d, "http://ex/mo"), mo]), "time:month = 3");
        assert!(s.contains(&[e, id(&d, "http://ex/dd"), dd]), "time:day = 15");
        assert!(s.contains(&[e, id(&d, "http://ex/absTemp"), a]), "math:absoluteValue(-5) = 5");
    }

    #[test]
    fn math_max_and_list_length() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            @prefix list: <http://www.w3.org/2000/10/swap/list#> .
            :d :seed "x" .
            { ( 3 7 2 ) math:max ?m . ( 3 7 2 ) list:length ?n } => { :d :maxVal ?m ; :count ?n } .
        "#;
        let (mut d, s) = closure(src);
        let int = "http://www.w3.org/2001/XMLSchema#integer";
        let seven = d.intern_lit("7", int, None);
        let three = d.intern_lit("3", int, None);
        assert!(s.contains(&[id(&d, "http://ex/d"), id(&d, "http://ex/maxVal"), seven]), "math:max = 7");
        assert!(s.contains(&[id(&d, "http://ex/d"), id(&d, "http://ex/count"), three]), "list:length = 3");
    }

    #[test]
    fn math_builtin_filter() {
        // math:greaterThan as a premise filter: adults are people over 17.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :alice :age 30 . :bob :age 12 .
            { ?p :age ?a . ?a math:greaterThan 17 } => { ?p a :Adult } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/alice", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://ex/Adult"), "alice is adult");
        assert!(!has(&d, &s, "http://ex/bob", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://ex/Adult"), "bob is NOT adult");
    }
}
