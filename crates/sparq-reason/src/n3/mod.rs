//! Notation3 (N3) rule reasoning — a forward-chaining engine toward EYE-reasoner parity.
//!
//! N3 adds rules (`{ premise } => { conclusion }`), variables (`?x`), formulae (`{ … }`) and
//! **builtins** (`math:`, `string:`, `log:`, …) on top of Turtle. EYE is a mature reasoner
//! with hundreds of builtins; this is the foundation: a parser (`parser`), a term model
//! (`model`), and a semi-naive forward chainer that applies rules to a fixpoint with variable
//! binding and a growing set of builtins. Coverage expands builtin-by-builtin, validated
//! against EYE's own test cases (see the `eye_cases` tests).
//!
//! v1 builtins: comparison/equality (`math:greaterThan`/`lessThan`/`notGreaterThan`/
//! `notLessThan`/`equalTo`/`notEqualTo`, `log:equalTo`/`notEqualTo`). Functional builtins with
//! list arguments (`math:sum`/`difference`/`product`/`quotient` over `( … )`) are the next
//! increment (need in-rule list resolution).

mod model;
mod parser;

pub use model::{Rule, Term};
use rustc_hash::FxHashSet;
use sparq_core::dict::{Dict, Id};
use std::collections::HashMap;

const MATH: &str = "http://www.w3.org/2000/10/swap/math#";
const LOG: &str = "http://www.w3.org/2000/10/swap/log#";
const STRING: &str = "http://www.w3.org/2000/10/swap/string#";
const LIST: &str = "http://www.w3.org/2000/10/swap/list#";

/// Parse N3 `src`, run the rule closure, and return the entailed GROUND triples interned into
/// `dict`. The rules/formulae/variables are consumed by reasoning; only ground facts remain.
pub fn reason_n3(dict: &mut Dict, src: &str) -> Result<Vec<[Id; 3]>, String> {
    let parsed = parser::parse(src)?;
    let mut facts: FxHashSet<[Term; 3]> = parsed.facts.into_iter().collect();

    // Semi-naive-ish fixpoint: re-run every rule against the full fact set until no rule
    // produces a new fact. (Rule sets are small; the data join dominates.)
    loop {
        let mut produced: Vec<[Term; 3]> = Vec::new();
        for rule in &parsed.rules {
            for b in match_premise(&rule.premise, &facts) {
                for c in &rule.conclusion {
                    if let Some(g) = ground_triple(c, &b) {
                        if !facts.contains(&g) {
                            produced.push(g);
                        }
                    }
                }
            }
        }
        let mut changed = false;
        for t in produced {
            changed |= facts.insert(t);
        }
        if !changed {
            break;
        }
    }

    // Intern the ground closure into the dictionary.
    let mut out = Vec::with_capacity(facts.len());
    for t in &facts {
        out.push([intern(dict, &t[0])?, intern(dict, &t[1])?, intern(dict, &t[2])?]);
    }
    Ok(out)
}

type Binding = HashMap<String, Term>;

/// All variable bindings under which every premise pattern holds (joining against `facts`,
/// evaluating builtins as filters/computations). N3 collections `( … )` in the premise are
/// rule-local list STRUCTURE (rdf:first/rest over fresh bnodes), not data to match — they are
/// extracted up front and consumed by the functional builtins (e.g. `math:sum`).
fn match_premise(premise: &[[Term; 3]], facts: &FxHashSet<[Term; 3]>) -> Vec<Binding> {
    let lists = extract_lists(premise);
    let mut bindings: Vec<Binding> = vec![Binding::new()];
    for pat in premise {
        if is_list_struct(pat, &lists) {
            continue; // structural, handled by list resolution
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
            let mut next = Vec::new();
            for b in &bindings {
                for fact in facts {
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

/// Functional `math:`/`string:`/`list:` builtins: subject is a `( … )` list, object computed.
#[derive(Clone, Copy)]
enum Func {
    Sum,
    Difference,
    Product,
    Quotient,
    Max,
    Min,
    Concat, // string:concatenation
    Length, // list:length
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
            _ => return None,
        });
    }
    match (i.strip_prefix(STRING), i.strip_prefix(LIST)) {
        (Some("concatenation"), _) => Some(Func::Concat),
        (_, Some("length")) => Some(Func::Length),
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
    let members = lists.get(subj)?;
    let args: Vec<Term> = members.iter().map(|m| apply(m, &b)).collect();
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
        _ => {
            let nums: Vec<f64> = args.iter().map(num).collect::<Option<_>>()?;
            if nums.is_empty() {
                return None;
            }
            let v = match f {
                Func::Sum => nums.iter().sum(),
                Func::Product => nums.iter().product(),
                Func::Max => nums.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                Func::Min => nums.iter().copied().fold(f64::INFINITY, f64::min),
                Func::Difference => {
                    if nums.len() != 2 {
                        return None;
                    }
                    nums[0] - nums[1]
                }
                Func::Quotient => {
                    if nums.len() != 2 || nums[1] == 0.0 {
                        return None;
                    }
                    nums[0] / nums[1]
                }
                Func::Concat | Func::Length => unreachable!(),
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
