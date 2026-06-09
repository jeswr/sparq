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

use sparq_core::dict::Dict;
use sparq_reason::reason_n3;
use std::collections::HashSet;

/// The closure of an N3 document as a set of canonical (s, p, o) strings.
fn closure_strings(src: &str) -> HashSet<(String, String, String)> {
    let mut d = Dict::new();
    let triples = reason_n3(&mut d, src).expect("reasoning failed");
    triples
        .iter()
        .map(|[s, p, o]| (d.term(*s).to_string(), d.term(*p).to_string(), d.term(*o).to_string()))
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
