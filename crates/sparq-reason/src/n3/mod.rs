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
/// evaluating builtins as filters).
fn match_premise(premise: &[[Term; 3]], facts: &FxHashSet<[Term; 3]>) -> Vec<Binding> {
    let mut bindings: Vec<Binding> = vec![Binding::new()];
    for pat in premise {
        if let Some(op) = builtin(&pat[1]) {
            bindings.retain(|b| eval_builtin(op, &pat[0], &pat[2], b));
            continue;
        }
        let mut next = Vec::new();
        for b in &bindings {
            for f in facts {
                if let Some(nb) = unify(pat, f, b) {
                    next.push(nb);
                }
            }
        }
        bindings = next;
        if bindings.is_empty() {
            break;
        }
    }
    bindings
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
    Gt,
    Lt,
    NotGt,
    NotLt,
    MathEq,
    MathNe,
    LogEq,
    LogNe,
}

fn builtin(p: &Term) -> Option<Builtin> {
    let Term::Iri(i) = p else { return None };
    let m = i.strip_prefix(MATH);
    let l = i.strip_prefix(LOG);
    Some(match (m, l) {
        (Some("greaterThan"), _) => Builtin::Gt,
        (Some("lessThan"), _) => Builtin::Lt,
        (Some("notGreaterThan"), _) => Builtin::NotGt,
        (Some("notLessThan"), _) => Builtin::NotLt,
        (Some("equalTo"), _) => Builtin::MathEq,
        (Some("notEqualTo"), _) => Builtin::MathNe,
        (_, Some("equalTo")) => Builtin::LogEq,
        (_, Some("notEqualTo")) => Builtin::LogNe,
        _ => return None,
    })
}

fn eval_builtin(op: Builtin, s: &Term, o: &Term, b: &Binding) -> bool {
    let (s, o) = (apply(s, b), apply(o, b));
    match op {
        Builtin::LogEq => s == o,
        Builtin::LogNe => s != o,
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

/// The numeric value of a literal term (for `math:` builtins).
fn num(t: &Term) -> Option<f64> {
    match t {
        Term::Lit(v, _, _) => v.parse::<f64>().ok(),
        _ => None,
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
