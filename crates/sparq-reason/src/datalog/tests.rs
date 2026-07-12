//! [FABLE-5] sq-6tykl.3 — Phase-1 acceptance suite: parser + stratification-checker
//! unit tests, hand-computed eval fixtures (exact expected sets — a mutation flips
//! them red), and the DIFFERENTIAL harness against the independent naive oracle
//! (`super::oracle`) on fixed programs × seed-randomised graphs.

use super::{eval, oracle, parse_program, stratify};
use rustc_hash::FxHashSet;
use sparq_core::dict::{Dict, Id};

const EX: &str = "http://ex/";
const P: &str = "@prefix ex: <http://ex/> .\n";

fn iri(d: &mut Dict, local: &str) -> Id {
    d.intern_iri(&format!("{EX}{local}"))
}

fn int(d: &mut Dict, n: u64) -> Id {
    d.intern_lit(&n.to_string(), "http://www.w3.org/2001/XMLSchema#integer", None)
}

fn s(d: &mut Dict, v: &str) -> Id {
    d.intern_lit(v, "http://www.w3.org/2001/XMLSchema#string", None)
}

/// Evaluate and return ONLY the derived facts (closure minus inputs), as a set.
fn derived(d: &mut Dict, facts: &[[Id; 3]], src: &str) -> FxHashSet<[Id; 3]> {
    let p = parse_program(d, src).expect("parse");
    let closure = eval(d, facts, &p).expect("stratifiable");
    let inputs: FxHashSet<[Id; 3]> = facts.iter().copied().collect();
    closure.into_iter().filter(|f| !inputs.contains(f)).collect()
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

#[test]
fn parse_basics_and_a_sugar() {
    let mut d = Dict::new();
    let p = parse_program(
        &mut d,
        &format!("{P}[?x, a, ex:Mortal] :- [?x, a, ex:Man] .\n# comment\n"),
    )
    .unwrap();
    assert_eq!(p.n_rules(), 1);
    // `a` expanded to rdf:type in both the head and the body atom.
    let ty = d.intern_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
    assert_eq!(p.rules[0].head[0].pred, ty);
    assert_eq!(p.rules[0].positive[0].pred, ty);
}

#[test]
fn parse_rejects_variable_predicate() {
    let mut d = Dict::new();
    let e = parse_program(&mut d, &format!("{P}[?x, ex:q, ?y] :- [?x, ?p, ?y] ."))
        .unwrap_err();
    assert!(e.contains("variable predicates"), "{e}");
}

#[test]
fn parse_rejects_unimplemented_aggregate_functions_loudly() {
    let mut d = Dict::new();
    let e = parse_program(
        &mut d,
        &format!("{P}[?x, ex:t, ?s] :- AGGREGATE([?x, ex:v, ?y] ON ?x BIND SUM(?y) AS ?s) ."),
    )
    .unwrap_err();
    assert!(e.contains("SUM") && e.contains("not implemented"), "{e}");
}

#[test]
fn parse_rejects_unsafe_head_variable() {
    let mut d = Dict::new();
    let e = parse_program(&mut d, &format!("{P}[?x, ex:q, ?z] :- [?x, ex:p, ?y] ."))
        .unwrap_err();
    assert!(e.contains("unsafe rule") && e.contains("?z"), "{e}");
}

#[test]
fn parse_rejects_head_var_bound_only_by_not() {
    let mut d = Dict::new();
    let e = parse_program(
        &mut d,
        &format!("{P}[?y, ex:q, \"y\"] :- [?x, a, ex:N], NOT [?x, ex:p, ?y] ."),
    )
    .unwrap_err();
    assert!(e.contains("unsafe rule"), "{e}");
}

#[test]
fn parse_rejects_aggregate_local_capture() {
    let mut d = Dict::new();
    // ?y occurs both inside the aggregate body (non-ON) and as an outer atom var.
    let e = parse_program(
        &mut d,
        &format!(
            "{P}[?x, ex:t, ?c] :- [?x, ex:p, ?y], \
             AGGREGATE([?x, ex:v, ?y] ON ?x BIND COUNT(?y) AS ?c) ."
        ),
    )
    .unwrap_err();
    assert!(e.contains("aggregate-local"), "{e}");
}

#[test]
fn parse_rejects_on_var_missing_from_body_and_undeclared_prefix() {
    let mut d = Dict::new();
    let e = parse_program(
        &mut d,
        &format!("{P}[?z, ex:t, ?c] :- AGGREGATE([?x, ex:v, ?y] ON ?z BIND COUNT(?y) AS ?c) ."),
    )
    .unwrap_err();
    assert!(e.contains("does not occur"), "{e}");
    let e2 = parse_program(&mut d, "[?x, foo:p, ?y] :- [?x, foo:q, ?y] .").unwrap_err();
    assert!(e2.contains("undeclared prefix"), "{e2}");
}

#[test]
fn parse_rejects_duplicate_aggregate_outputs_and_unbound_filter_var() {
    let mut d = Dict::new();
    let e = parse_program(
        &mut d,
        &format!(
            "{P}[?x, ex:t, ?c] :- AGGREGATE([?x, ex:v, ?y] ON ?x BIND COUNT(?y) AS ?c), \
             AGGREGATE([?x, ex:w, ?z] ON ?x BIND COUNT(?z) AS ?c) ."
        ),
    )
    .unwrap_err();
    assert!(e.contains("same output variable"), "{e}");
    let e2 = parse_program(
        &mut d,
        &format!("{P}[?x, ex:q, ?y] :- [?x, ex:p, ?y], FILTER(?nope > 1) ."),
    )
    .unwrap_err();
    assert!(e2.contains("FILTER variable"), "{e2}");
}

#[test]
fn parse_rejects_fresh_violation_of_aggregate_output() {
    let mut d = Dict::new();
    let e = parse_program(
        &mut d,
        &format!(
            "{P}[?x, ex:t, ?c] :- [?x, ex:p, ?c], \
             AGGREGATE([?x, ex:v, ?y] ON ?x BIND COUNT(?y) AS ?c) ."
        ),
    )
    .unwrap_err();
    assert!(e.contains("must be fresh"), "{e}");
}

// ---------------------------------------------------------------------------
// Stratification checker
// ---------------------------------------------------------------------------

#[test]
fn stratify_pure_positive_recursion_is_one_stratum() {
    let mut d = Dict::new();
    let p = parse_program(
        &mut d,
        &format!("{P}[?x, ex:reach, ?y] :- [?x, ex:edge, ?y] .\n[?x, ex:reach, ?z] :- [?x, ex:reach, ?y], [?y, ex:edge, ?z] ."),
    )
    .unwrap();
    let s = stratify(&d, &p).unwrap();
    assert_eq!(s.n_strata(), 1);
}

#[test]
fn stratify_naf_over_derived_needs_a_higher_stratum() {
    let mut d = Dict::new();
    let p = parse_program(
        &mut d,
        &format!(
            "{P}[?x, ex:reach, \"y\"] :- [?x, ex:seed, \"y\"] .\n\
             [?y, ex:reach, \"y\"] :- [?x, ex:reach, \"y\"], [?x, ex:edge, ?y] .\n\
             [?x, ex:unreach, \"y\"] :- [?x, a, ex:Node], NOT [?x, ex:reach, \"y\"] ."
        ),
    )
    .unwrap();
    let s = stratify(&d, &p).unwrap();
    // reach: stratum 0 wrt seed/edge (EDB); unreach must sit strictly above reach.
    assert_eq!(s.n_strata(), 2);
    assert_eq!(s.rule_stratum, vec![0, 0, 1]);
}

#[test]
fn stratify_rejects_negation_cycle_naming_the_predicate() {
    let mut d = Dict::new();
    // win(X) :- move(X,Y), NOT win(Y).
    let p = parse_program(
        &mut d,
        &format!("{P}[?x, ex:win, \"y\"] :- [?x, ex:move, ?y], NOT [?y, ex:win, \"y\"] ."),
    )
    .unwrap();
    let e = stratify(&d, &p).unwrap_err();
    assert!(e.contains("NOT stratifiable") && e.contains("http://ex/win"), "{e}");
}

#[test]
fn stratify_rejects_aggregation_cycle() {
    let mut d = Dict::new();
    // p counts itself: no stratified model.
    let p = parse_program(
        &mut d,
        &format!("{P}[?x, ex:p, ?c] :- AGGREGATE([?x, ex:p, ?y] ON ?x BIND COUNT(?y) AS ?c) ."),
    )
    .unwrap();
    assert!(stratify(&d, &p).is_err());
}

#[test]
fn stratify_chains_strata_through_aggregates() {
    let mut d = Dict::new();
    let p = parse_program(
        &mut d,
        &format!(
            "{P}[?x, ex:deg, ?c] :- AGGREGATE([?x, ex:edge, ?y] ON ?x BIND COUNT(?y) AS ?c) .\n\
             [?x, ex:isolated, \"y\"] :- [?x, a, ex:Node], NOT [?x, ex:deg, ?c] ."
        ),
    )
    .unwrap();
    let s = stratify(&d, &p).unwrap();
    // edge (EDB, 0) → deg (1) → isolated (2).
    assert_eq!(s.n_strata(), 3);
    assert_eq!(s.rule_stratum, vec![1, 2]);
}

#[test]
fn stratify_co_head_predicates_share_a_stratum() {
    let mut d = Dict::new();
    let p = parse_program(
        &mut d,
        &format!(
            "{P}[?x, ex:p, \"y\"], [?x, ex:q, \"y\"] :- [?x, a, ex:N], NOT [?x, ex:r, \"y\"] .\n\
             [?x, ex:r, \"y\"] :- [?x, ex:base, \"y\"] ."
        ),
    )
    .unwrap();
    let s = stratify(&d, &p).unwrap();
    assert_eq!(s.rule_stratum, vec![1, 0]);
}

#[test]
fn stratify_is_class_granular_for_rdf_type() {
    let mut d = Dict::new();
    // Deriving `a ex:Hub` and negating `a ex:Hub` from a DIFFERENT class head must
    // stratify (rdf:type is not one monolithic relation) …
    let p = parse_program(
        &mut d,
        &format!(
            "{P}[?x, ex:deg, ?c] :- AGGREGATE([?x, ex:edge, ?y] ON ?x BIND COUNT(?y) AS ?c) .\n\
             [?x, a, ex:Hub] :- [?x, ex:deg, ?c], FILTER(?c >= 2) .\n\
             [?x, a, ex:Leaf] :- [?x, a, ex:Node], NOT [?x, a, ex:Hub] ."
        ),
    )
    .unwrap();
    let s = stratify(&d, &p).unwrap();
    assert_eq!(s.rule_stratum, vec![1, 1, 2]);
    // … while a genuine cycle through one class is still rejected, named as a class.
    let p2 = parse_program(
        &mut d,
        &format!("{P}[?x, a, ex:Win] :- [?x, ex:move, ?y], NOT [?y, a, ex:Win] ."),
    )
    .unwrap();
    let e = stratify(&d, &p2).unwrap_err();
    assert!(e.contains("class") && e.contains("http://ex/Win"), "{e}");
}

#[test]
fn stratify_variable_class_reads_sit_above_every_class() {
    let mut d = Dict::new();
    // `NOT [?x, a, ?c]` (variable class) must sit above the derived ex:Hub class.
    let p = parse_program(
        &mut d,
        &format!(
            "{P}[?x, a, ex:Hub] :- [?x, ex:big, \"y\"] .\n\
             [?x, ex:untyped, \"y\"] :- [?x, ex:thing, \"y\"], NOT [?x, a, ?c] ."
        ),
    )
    .unwrap();
    let s = stratify(&d, &p).unwrap();
    assert_eq!(s.rule_stratum, vec![0, 1]);
    // And a variable-class HEAD feeding a negated class is rejected.
    let p2 = parse_program(
        &mut d,
        &format!(
            "{P}[?x, a, ?c] :- [?x, ex:says, ?c], NOT [?x, a, ex:Banned] ."
        ),
    )
    .unwrap();
    assert!(stratify(&d, &p2).is_err());
}

// ---------------------------------------------------------------------------
// Evaluation fixtures (hand-computed expected sets)
// ---------------------------------------------------------------------------

#[test]
fn eval_transitive_closure() {
    let mut d = Dict::new();
    let (a, b, c, edge, reach) = (
        iri(&mut d, "a"),
        iri(&mut d, "b"),
        iri(&mut d, "c"),
        iri(&mut d, "edge"),
        iri(&mut d, "reach"),
    );
    let facts = vec![[a, edge, b], [b, edge, c]];
    let got = derived(
        &mut d,
        &facts,
        &format!(
            "{P}[?x, ex:reach, ?y] :- [?x, ex:edge, ?y] .\n\
             [?x, ex:reach, ?z] :- [?x, ex:reach, ?y], [?y, ex:edge, ?z] ."
        ),
    );
    let want: FxHashSet<[Id; 3]> =
        [[a, reach, b], [b, reach, c], [a, reach, c]].into_iter().collect();
    assert_eq!(got, want);
}

#[test]
fn eval_naf_absence_check_across_strata() {
    let mut d = Dict::new();
    let (a, b, c, ty, node, edge, seed, reach, unreach, y) = (
        iri(&mut d, "a"),
        iri(&mut d, "b"),
        iri(&mut d, "c"),
        d.intern_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
        iri(&mut d, "Node"),
        iri(&mut d, "edge"),
        iri(&mut d, "seed"),
        iri(&mut d, "reach"),
        iri(&mut d, "unreach"),
        s(&mut d, "y"),
    );
    // a → b, c disconnected; a seeded.
    let facts = vec![
        [a, ty, node],
        [b, ty, node],
        [c, ty, node],
        [a, edge, b],
        [a, seed, y],
    ];
    let got = derived(
        &mut d,
        &facts,
        &format!(
            "{P}[?x, ex:reach, \"y\"] :- [?x, ex:seed, \"y\"] .\n\
             [?y, ex:reach, \"y\"] :- [?x, ex:reach, \"y\"], [?x, ex:edge, ?y] .\n\
             [?x, ex:unreach, \"y\"] :- [?x, a, ex:Node], NOT [?x, ex:reach, \"y\"] ."
        ),
    );
    let want: FxHashSet<[Id; 3]> =
        [[a, reach, y], [b, reach, y], [c, unreach, y]].into_iter().collect();
    assert_eq!(got, want);
}

#[test]
fn eval_count_per_group_and_threshold_filter() {
    let mut d = Dict::new();
    let (g1, g2, m1, m2, m3, member, deg, ty, hub) = (
        iri(&mut d, "g1"),
        iri(&mut d, "g2"),
        iri(&mut d, "m1"),
        iri(&mut d, "m2"),
        iri(&mut d, "m3"),
        iri(&mut d, "member"),
        iri(&mut d, "deg"),
        d.intern_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
        iri(&mut d, "Hub"),
    );
    let facts = vec![[g1, member, m1], [g1, member, m2], [g1, member, m3], [g2, member, m1]];
    let got = derived(
        &mut d,
        &facts,
        &format!(
            "{P}[?g, ex:deg, ?c] :- AGGREGATE([?g, ex:member, ?m] ON ?g BIND COUNT(?m) AS ?c) .\n\
             [?g, a, ex:Hub] :- [?g, ex:deg, ?c], FILTER(?c >= 2) ."
        ),
    );
    let three = int(&mut d, 3);
    let one = int(&mut d, 1);
    let want: FxHashSet<[Id; 3]> =
        [[g1, deg, three], [g2, deg, one], [g1, ty, hub]].into_iter().collect();
    assert_eq!(got, want);
}

#[test]
fn eval_global_count_without_on() {
    let mut d = Dict::new();
    let (a, b, ty, node, total, w) = (
        iri(&mut d, "a"),
        iri(&mut d, "b"),
        d.intern_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
        iri(&mut d, "Node"),
        iri(&mut d, "total"),
        iri(&mut d, "world"),
    );
    let facts = vec![[a, ty, node], [b, ty, node]];
    let got = derived(
        &mut d,
        &facts,
        &format!("{P}[ex:world, ex:total, ?c] :- AGGREGATE([?x, a, ex:Node] BIND COUNT(?x) AS ?c) ."),
    );
    let two = int(&mut d, 2);
    let want: FxHashSet<[Id; 3]> = [[w, total, two]].into_iter().collect();
    assert_eq!(got, want);
}

#[test]
fn eval_count_over_derived_predicate() {
    let mut d = Dict::new();
    let (a, b, c, edge, nreach, w) = (
        iri(&mut d, "a"),
        iri(&mut d, "b"),
        iri(&mut d, "c"),
        iri(&mut d, "edge"),
        iri(&mut d, "nreach"),
        iri(&mut d, "world"),
    );
    let facts = vec![[a, edge, b], [b, edge, c]];
    // reach is derived (stratum 0); the count reads it from stratum 1.
    let got = derived(
        &mut d,
        &facts,
        &format!(
            "{P}[?x, ex:reach, ?y] :- [?x, ex:edge, ?y] .\n\
             [?x, ex:reach, ?z] :- [?x, ex:reach, ?y], [?y, ex:edge, ?z] .\n\
             [ex:world, ex:nreach, ?c] :- AGGREGATE([?x, ex:reach, ?y] BIND COUNT(?x) AS ?c) ."
        ),
    );
    // reach = {(a,b),(b,c),(a,c)} → 3 distinct matches.
    let three = int(&mut d, 3);
    assert!(got.contains(&[w, nreach, three]), "count over DERIVED reach");
    assert_eq!(got.len(), 4); // 3 reach facts + the count fact
}

#[test]
fn eval_naf_repeated_wildcard_requires_equal_terms() {
    let mut d = Dict::new();
    let (a, b, p, ok, ty, n, y) = (
        iri(&mut d, "a"),
        iri(&mut d, "b"),
        iri(&mut d, "p"),
        iri(&mut d, "ok"),
        d.intern_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
        iri(&mut d, "N"),
        s(&mut d, "y"),
    );
    // No self-loop exists: NOT [?z, ex:p, ?z] holds even though (a,p,b) exists.
    let facts = vec![[a, ty, n], [a, p, b]];
    let got = derived(
        &mut d,
        &facts,
        &format!("{P}[?x, ex:ok, \"y\"] :- [?x, a, ex:N], NOT [?z, ex:p, ?z] ."),
    );
    let want: FxHashSet<[Id; 3]> = [[a, ok, y]].into_iter().collect();
    assert_eq!(got, want);
    // Add the self-loop: the wildcard NAF now fails and nothing derives.
    let facts2 = vec![[a, ty, n], [a, p, b], [b, p, b]];
    let got2 = derived(
        &mut d,
        &facts2,
        &format!("{P}[?x, ex:ok, \"y\"] :- [?x, a, ex:N], NOT [?z, ex:p, ?z] ."),
    );
    assert!(got2.is_empty());
}

#[test]
fn eval_multi_head_rule_derives_both() {
    let mut d = Dict::new();
    let (a, b, p, q, r) = (
        iri(&mut d, "a"),
        iri(&mut d, "b"),
        iri(&mut d, "p"),
        iri(&mut d, "q"),
        iri(&mut d, "r"),
    );
    let facts = vec![[a, p, b]];
    let got = derived(
        &mut d,
        &facts,
        &format!("{P}[?x, ex:q, ?y], [?y, ex:r, ?x] :- [?x, ex:p, ?y] ."),
    );
    let want: FxHashSet<[Id; 3]> = [[a, q, b], [b, r, a]].into_iter().collect();
    assert_eq!(got, want);
}

#[test]
fn eval_rejects_non_stratifiable_program() {
    let mut d = Dict::new();
    let p = parse_program(
        &mut d,
        &format!("{P}[?x, ex:win, \"y\"] :- [?x, ex:move, ?y], NOT [?y, ex:win, \"y\"] ."),
    )
    .unwrap();
    assert!(eval(&mut d, &[], &p).is_err());
}

#[test]
fn eval_filter_is_fail_closed_on_non_numeric_operands() {
    let mut d = Dict::new();
    let (a, p, b) = (iri(&mut d, "a"), iri(&mut d, "p"), iri(&mut d, "b"));
    // ?y is an IRI — the numeric FILTER must fail the row, deriving nothing.
    let facts = vec![[a, p, b]];
    let got = derived(
        &mut d,
        &facts,
        &format!("{P}[?x, ex:q, ?y] :- [?x, ex:p, ?y], FILTER(?y > 0) ."),
    );
    assert!(got.is_empty());
}

// ---------------------------------------------------------------------------
// Differential vs the independent naive oracle
// ---------------------------------------------------------------------------

/// Deterministic LCG (no rand dep; fixed seeds → reproducible graphs).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self, bound: u64) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) % bound
    }
}

/// Random node/edge facts over `n` nodes: every node typed, ~`e` random edges.
fn random_graph(d: &mut Dict, seed: u64, n: u64, e: u64) -> Vec<[Id; 3]> {
    let ty = d.intern_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
    let node = iri(d, "Node");
    let edge = iri(d, "edge");
    let seed_p = iri(d, "seed");
    let y = s(d, "y");
    let mut rng = Lcg(seed.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(1));
    let nodes: Vec<Id> = (0..n).map(|i| iri(d, &format!("n{i}"))).collect();
    let mut facts: Vec<[Id; 3]> = nodes.iter().map(|&x| [x, ty, node]).collect();
    for _ in 0..e {
        let (i, j) = (rng.next(n) as usize, rng.next(n) as usize);
        facts.push([nodes[i], edge, nodes[j]]);
    }
    // One seeded node so reachability programs have a source.
    facts.push([nodes[rng.next(n) as usize], seed_p, y]);
    facts
}

/// The differential body: engine closure == oracle closure, as sets.
fn assert_differential(src: &str, seeds: std::ops::Range<u64>) {
    for seed in seeds {
        let mut d = Dict::new();
        let program = parse_program(&mut d, src).expect("parse");
        let strat = stratify(&d, &program).expect("stratifiable");
        let facts = random_graph(&mut d, seed, 6, 9);
        let engine: FxHashSet<[Id; 3]> =
            eval(&mut d, &facts, &program).unwrap().into_iter().collect();
        let reference = oracle::eval_naive(&mut d, &facts, &program, &strat);
        assert_eq!(
            engine, reference,
            "engine/oracle divergence: seed {seed} program:\n{src}"
        );
    }
}

#[test]
fn differential_recursion_plus_naf() {
    assert_differential(
        &format!(
            "{P}[?x, ex:reach, \"y\"] :- [?x, ex:seed, \"y\"] .\n\
             [?y, ex:reach, \"y\"] :- [?x, ex:reach, \"y\"], [?x, ex:edge, ?y] .\n\
             [?x, ex:unreach, \"y\"] :- [?x, a, ex:Node], NOT [?x, ex:reach, \"y\"] ."
        ),
        0..25,
    );
}

#[test]
fn differential_count_threshold() {
    assert_differential(
        &format!(
            "{P}[?x, ex:deg, ?c] :- AGGREGATE([?x, ex:edge, ?y] ON ?x BIND COUNT(?y) AS ?c) .\n\
             [?x, a, ex:Hub] :- [?x, ex:deg, ?c], FILTER(?c >= 2) .\n\
             [?x, a, ex:Leaf] :- [?x, a, ex:Node], NOT [?x, a, ex:Hub] ."
        ),
        0..25,
    );
}

#[test]
fn differential_count_over_derived_with_wildcard_naf() {
    assert_differential(
        &format!(
            "{P}[?x, ex:reach, ?y] :- [?x, ex:edge, ?y] .\n\
             [?x, ex:reach, ?z] :- [?x, ex:reach, ?y], [?y, ex:edge, ?z] .\n\
             [?x, ex:fan, ?c] :- AGGREGATE([?x, ex:reach, ?y] ON ?x BIND COUNT(?y) AS ?c) .\n\
             [ex:world, ex:quiet, \"y\"] :- [ex:world, a, ex:Nothing], NOT [?z, ex:edge, ?z] .\n\
             [?x, ex:big, \"y\"] :- [?x, ex:fan, ?c], FILTER(?c > 3) ."
        ),
        0..25,
    );
}

#[test]
fn differential_multi_head_and_global_count() {
    assert_differential(
        &format!(
            "{P}[?x, ex:out, ?y], [?y, ex:in, ?x] :- [?x, ex:edge, ?y] .\n\
             [ex:world, ex:edges, ?c] :- AGGREGATE([?x, ex:out, ?y] BIND COUNT(?x) AS ?c) ."
        ),
        0..25,
    );
}

// ---------------------------------------------------------------------------
// Semi-naive vs naive (sq-8sve7): SET-identical closures + strictly less work
// ---------------------------------------------------------------------------

/// Three-way differential on one (program, facts) input: the semi-naive engine,
/// the same engine forced to full (naive) re-runs, and the independent oracle
/// must produce the SAME closure set.
fn assert_semi_naive_equals_naive(src: &str, facts_of: &dyn Fn(&mut Dict) -> Vec<[Id; 3]>) {
    let mut d = Dict::new();
    let program = parse_program(&mut d, src).expect("parse");
    let strat = stratify(&d, &program).expect("stratifiable");
    let facts = facts_of(&mut d);
    let mut semi_stats = eval::EvalStats::default();
    let semi: FxHashSet<[Id; 3]> =
        eval::eval_stratified_with_stats(&mut d, &facts, &program, &strat, &mut semi_stats, false)
            .into_iter()
            .collect();
    let mut naive_stats = eval::EvalStats::default();
    let naive: FxHashSet<[Id; 3]> =
        eval::eval_stratified_with_stats(&mut d, &facts, &program, &strat, &mut naive_stats, true)
            .into_iter()
            .collect();
    assert_eq!(
        semi, naive,
        "semi-naive/naive closure divergence ({} facts) program:\n{src}",
        facts.len()
    );
    let reference = oracle::eval_naive(&mut d, &facts, &program, &strat);
    assert_eq!(
        semi, reference,
        "engine/oracle closure divergence ({} facts) program:\n{src}",
        facts.len()
    );
}

/// The sq-8sve7 soundness invariant, as a battery: recursive, NAF, aggregate and
/// multi-stratum programs × {empty graph, single fact, seeded random graphs} —
/// semi-naive output == naive output as SET equality on every input.
///
/// NON-VACUITY (verified by hand-run mutation, see the PR body): restricting the
/// semi-naive loop to `k = 0` only — or starting the delta empty instead of
/// all-facts — flips this test red on the recursive programs.
#[test]
fn differential_semi_naive_equals_naive_battery() {
    let programs = [
        // Pure positive recursion (transitive closure).
        format!(
            "{P}[?x, ex:reach, ?y] :- [?x, ex:edge, ?y] .\n\
             [?x, ex:reach, ?z] :- [?x, ex:reach, ?y], [?y, ex:edge, ?z] ."
        ),
        // Transitive closure with the RECURSIVE atom in the SECOND body position —
        // the delta restriction must cover every positive-atom index k, not just
        // k = 0 (a k=0-only mutation goes red exactly here).
        format!(
            "{P}[?x, ex:reach, ?y] :- [?x, ex:edge, ?y] .\n\
             [?x, ex:reach, ?z] :- [?y, ex:edge, ?z], [?x, ex:reach, ?y] ."
        ),
        // Recursion + NAF across strata (needs_full path + delta path together).
        format!(
            "{P}[?x, ex:reach, \"y\"] :- [?x, ex:seed, \"y\"] .\n\
             [?y, ex:reach, \"y\"] :- [?x, ex:reach, \"y\"], [?x, ex:edge, ?y] .\n\
             [?x, ex:unreach, \"y\"] :- [?x, a, ex:Node], NOT [?x, ex:reach, \"y\"] ."
        ),
        // Aggregate + threshold FILTER + NAF over a derived class (3 strata).
        format!(
            "{P}[?x, ex:deg, ?c] :- AGGREGATE([?x, ex:edge, ?y] ON ?x BIND COUNT(?y) AS ?c) .\n\
             [?x, a, ex:Hub] :- [?x, ex:deg, ?c], FILTER(?c >= 2) .\n\
             [?x, a, ex:Leaf] :- [?x, a, ex:Node], NOT [?x, a, ex:Hub] ."
        ),
        // Multi-head + recursion over a derived predicate + global count over the
        // recursive closure + a wildcard-NAF constant rule.
        format!(
            "{P}[?x, ex:out, ?y], [?y, ex:in, ?x] :- [?x, ex:edge, ?y] .\n\
             [?x, ex:reach, ?y] :- [?x, ex:out, ?y] .\n\
             [?x, ex:reach, ?z] :- [?x, ex:reach, ?y], [?y, ex:out, ?z] .\n\
             [ex:world, ex:nreach, ?c] :- AGGREGATE([?x, ex:reach, ?y] BIND COUNT(?x) AS ?c) .\n\
             [ex:world, ex:quiet, \"y\"] :- [ex:world, a, ex:Nothing], NOT [?z, ex:edge, ?z] ."
        ),
    ];
    for src in &programs {
        // Empty input graph.
        assert_semi_naive_equals_naive(src, &|_| Vec::new());
        // Single fact.
        assert_semi_naive_equals_naive(src, &|d| {
            let (a, e, b) = (iri(d, "a"), iri(d, "edge"), iri(d, "b"));
            vec![[a, e, b]]
        });
        // Seed-randomised graphs (typed nodes + random edges + one seed).
        for seed in 0..10 {
            assert_semi_naive_equals_naive(src, &move |d| random_graph(d, seed, 6, 9));
        }
    }
}

/// sq-8sve7 acceptance: on a 30-node linear-chain transitive closure, semi-naive
/// delta restriction feeds STRICTLY fewer candidate tuples into the join steps
/// than the forced-full (naive) discipline — while deriving the identical set,
/// cross-checked against the independent oracle. Deterministic counters only.
#[test]
fn semi_naive_considers_fewer_tuples_than_naive() {
    let mut d = Dict::new();
    let src = format!(
        "{P}[?x, ex:reach, ?y] :- [?x, ex:edge, ?y] .\n\
         [?x, ex:reach, ?z] :- [?x, ex:reach, ?y], [?y, ex:edge, ?z] ."
    );
    let program = parse_program(&mut d, &src).expect("parse");
    let strat = stratify(&d, &program).expect("stratifiable");
    let edge = iri(&mut d, "edge");
    let nodes: Vec<Id> = (0..30).map(|i| iri(&mut d, &format!("c{i}"))).collect();
    let facts: Vec<[Id; 3]> = nodes.windows(2).map(|w| [w[0], edge, w[1]]).collect();

    let mut semi_stats = eval::EvalStats::default();
    let semi: FxHashSet<[Id; 3]> =
        eval::eval_stratified_with_stats(&mut d, &facts, &program, &strat, &mut semi_stats, false)
            .into_iter()
            .collect();
    let mut naive_stats = eval::EvalStats::default();
    let naive: FxHashSet<[Id; 3]> =
        eval::eval_stratified_with_stats(&mut d, &facts, &program, &strat, &mut naive_stats, true)
            .into_iter()
            .collect();

    // Identical derived sets, and both equal the independent oracle.
    let reference = oracle::eval_naive(&mut d, &facts, &program, &strat);
    assert_eq!(semi, reference);
    assert_eq!(naive, reference);
    // 30-node chain: 29 input edges + C(30,2) = 435 reach facts.
    assert_eq!(semi.len(), 29 + 435);

    // The point of the phase: strictly less join work, same fixpoint depth order.
    assert!(
        semi_stats.tuples_considered < naive_stats.tuples_considered,
        "semi-naive must consider strictly fewer tuples: semi={} naive={}",
        semi_stats.tuples_considered,
        naive_stats.tuples_considered
    );
    assert!(semi_stats.rounds >= 2, "chain TC needs multiple rounds");
    assert!(semi_stats.tuples_considered > 0, "counter must be live");
}
