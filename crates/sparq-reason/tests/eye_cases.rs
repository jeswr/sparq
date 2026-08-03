//! EYE differential test gate for the N3 reasoner.
//!
//! Each case is an `input.n3` + an `*-answer.n3` (the expected entailed triples). We run our
//! forward-chaining closure on the input and assert it contains every triple in EYE's answer
//! (a superset, since EYE answers are often query-projected to the conclusion while we emit
//! the full closure). `socrates.*` is vendored verbatim from the EYE repo
//! (eyereasoner/eye `reasoning/socrates`); the rest follow EYE's documented builtin semantics
//! for features our v1 covers (forward `=>` rules + comparison/functional `math:` builtins).
//!
//! This is the parity gate the project grows: as backward chaining (`<=`), path syntax
//! (`!`/`^`), and more builtins land, drop the corresponding EYE `reasoning/<case>` files in
//! here and add a `check!`.
//!
//! EYE runs most cases as `eye <data+rules> --query <query.n3>`: the query file's
//! `{goal} => {goal}` rule both poses the goal (driving backward `<=` rules) and projects
//! the output. Our harness concatenates data + query into one document — same semantics
//! for these cases, since a query rule is just a forward rule.

use sparq_core::dict::Dict;
use sparq_reason::reason_n3;
use std::collections::HashSet;

/// The closure of an N3 document as a set of canonical (s, p, o) strings.
fn closure_strings(src: &str) -> HashSet<(String, String, String)> {
    let mut d = Dict::new();
    let triples = reason_n3(&mut d, src).expect("reasoning failed");
    triples
        .iter()
        .map(|[s, p, o]| {
            (
                d.term(*s).to_string(),
                d.term(*p).to_string(),
                d.term(*o).to_string(),
            )
        })
        .collect()
}

/// Assert our closure of `input` contains every triple in EYE's `answer`.
fn check(name: &str, input: &str, answer: &str) {
    let closure = closure_strings(input);
    let expected = closure_strings(answer);
    for t in &expected {
        assert!(
            closure.contains(t),
            "[{name}] EYE answer triple not derived by sparq-reason: {t:?}\nderived closure: {closure:#?}"
        );
    }
}

#[test]
fn eye_socrates() {
    // Vendored verbatim from eyereasoner/eye reasoning/socrates — a forward subClassOf rule.
    check(
        "socrates",
        include_str!("eye/socrates.n3"),
        include_str!("eye/socrates-answer.n3"),
    );
}

#[test]
fn eye_math_sum() {
    // EYE math:sum builtin semantics over an N3 forward rule.
    check(
        "math-sum",
        include_str!("eye/math-sum.n3"),
        include_str!("eye/math-sum-answer.n3"),
    );
}

#[test]
fn eye_backward() {
    // Vendored verbatim from eyereasoner/eye reasoning/backward: a `<=` rule whose premise
    // is a pure builtin (`?X math:greaterThan ?Y`) — provable ONLY goal-directed, once the
    // query rule binds ?X/?Y. The canonical proof that `<=` must not be reversed into a
    // forward rule.
    let input = format!(
        "{}\n{}",
        include_str!("eye/backward.n3"),
        include_str!("eye/backward-query.n3")
    );
    check("backward", &input, include_str!("eye/backward-answer.n3"));
}

#[test]
fn eye_witch() {
    // Vendored verbatim from eyereasoner/eye reasoning/witch (the Monty Python "burn the
    // witch" syllogism): chained forward rules + a query goal.
    let input = format!(
        "{}\n{}",
        include_str!("eye/witch.n3"),
        include_str!("eye/witch-goal.n3")
    );
    check("witch", &input, include_str!("eye/witch-answer.n3"));
}

#[test]
fn eye_bi_subset() {
    // EYE's own builtin unit-test suite (reasoning/bi/biP.n3), restricted to the builtins
    // we implement — every test line verbatim; see the header of bi-subset.n3 for the
    // exclusion list. Exercises log:conjunction/uri, string:scrape/containsIgnoringCase,
    // math:memberCount, formula containment, and the math/string/list/time families.
    check(
        "bi-subset",
        include_str!("eye/bi-subset.n3"),
        include_str!("eye/bi-subset-answer.n3"),
    );
}
