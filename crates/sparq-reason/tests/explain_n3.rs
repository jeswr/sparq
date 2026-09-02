//! `why()` on [`MaterializedN3Graph`] + the id-level [`explain::n3_proof_tree`] bridge
//! (`explain` feature): hand-checked rule-firing chains, a differential sweep (every
//! derived fact explains; the proof's asserted leaves alone re-entail the conclusion under
//! the same rules), and retraction consistency.
#![cfg(feature = "explain")]

mod explain_common;

use explain_common::{check_proof, proof_leaves};
use rustc_hash::FxHashSet;
use sparq_reason::n3::Term;
use sparq_reason::{
    explain::n3_proof_tree, reason_n3_proof, reason_n3_terms, ExplainOpts, MaterializedN3Graph,
    N3Mode, ProofTree,
};

fn iri(s: &str) -> Term {
    Term::Iri(s.into())
}
fn ex(local: &str) -> Term {
    iri(&format!("http://ex/{local}"))
}

fn render(f: &[Term; 3]) -> [String; 3] {
    f.clone().map(|t| match t {
        Term::Iri(i) => format!("<{i}>"),
        Term::Lit(v, dt, None) if dt == "http://www.w3.org/2001/XMLSchema#string" => {
            format!("\"{v}\"")
        }
        Term::Lit(v, dt, None) => format!("\"{v}\"^^<{dt}>"),
        Term::Lit(v, _, Some(l)) => format!("\"{v}\"@{l}"),
        other => panic!("unexpected term shape in test: {other:?}"),
    })
}

fn rendered(base: &[[Term; 3]]) -> FxHashSet<[String; 3]> {
    base.iter().map(render).collect()
}

/// Re-entailment oracle: rules + the proof's asserted leaves must re-derive the conclusion.
/// (Leaf conclusions are already serialized N3 terms — feed them straight back.)
fn leaves_entail(rules: &str, tree: &ProofTree, conclusion: &[String; 3]) {
    let mut src = String::from(rules);
    src.push('\n');
    for l in proof_leaves(tree) {
        src.push_str(&format!("{} {} {} .\n", l[0], l[1], l[2]));
    }
    let closure = reason_n3_terms(&src, None).expect("leaf subset must re-parse");
    let ok = closure.facts.iter().any(|f| &render(f) == conclusion);
    assert!(
        ok,
        "proof leaves do not re-entail {conclusion:?}\nproof:\n{}",
        tree.to_text()
    );
}

const RULES: &str = r#"
@prefix : <http://ex/> .
{ ?x :parent ?y } => { ?x :ancestor ?y } .
{ ?x :ancestor ?y . ?y :ancestor ?z } => { ?x :ancestor ?z } .
{ ?x a :Human } => { ?x a :Mortal } .
"#;

#[test]
fn rule_chain_hand_checked() {
    let ty = iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
    let base = vec![
        [ex("a"), ex("parent"), ex("b")],
        [ex("b"), ex("parent"), ex("c")],
        [ex("socrates"), ty.clone(), ex("Human")],
    ];
    let g = MaterializedN3Graph::new(RULES, &base).expect("rules parse");
    assert_eq!(g.mode(), N3Mode::Counting);

    // socrates a :Mortal — one firing of rule 2.
    let f = [ex("socrates"), ty.clone(), ex("Mortal")];
    assert!(g.contains(&f));
    let tree = g.why(&f).expect("derived fact explains");
    check_proof(&tree, &rendered(&base), Some(&render(&f)));
    leaves_entail(RULES, &tree, &render(&f));
    let root = &tree.nodes()[tree.root() as usize];
    assert_eq!(root.rule, "n3-rule-2");
    assert_eq!(root.premises.len(), 1);
    assert_eq!(tree.nodes()[root.premises[0] as usize].rule, "asserted");

    // a :ancestor c — rule 1 over two rule-0 lifts.
    let f = [ex("a"), ex("ancestor"), ex("c")];
    assert!(g.contains(&f));
    let tree = g.why(&f).expect("recursive fact explains");
    check_proof(&tree, &rendered(&base), Some(&render(&f)));
    leaves_entail(RULES, &tree, &render(&f));
    let root = &tree.nodes()[tree.root() as usize];
    assert_eq!(root.rule, "n3-rule-1");
    assert!(
        tree.nodes()
            .iter()
            .filter(|n| n.rule == "n3-rule-0")
            .count()
            >= 2
    );
    assert!(tree.nodes().iter().filter(|n| n.rule == "asserted").count() == 2);

    // Asserted facts explain as single leaves; absent facts do not explain.
    let tree = g.why(&base[0]).unwrap();
    assert_eq!(tree.nodes().len(), 1);
    assert_eq!(tree.nodes()[0].rule, "asserted");
    assert!(g.why(&[ex("c"), ex("ancestor"), ex("a")]).is_none());

    // Determinism.
    let f = [ex("a"), ex("ancestor"), ex("c")];
    assert_eq!(g.why(&f).unwrap().to_json(), g.why(&f).unwrap().to_json());
}

#[test]
fn differential_every_derived_fact_explains() {
    // A chain of parents → quadratic ancestor closure; every derived fact must explain and
    // re-entail from its leaves.
    let mut base: Vec<[Term; 3]> = Vec::new();
    for i in 0..12 {
        base.push([
            ex(&format!("p{i}")),
            ex("parent"),
            ex(&format!("p{}", i + 1)),
        ]);
    }
    let g = MaterializedN3Graph::new(RULES, &base).expect("rules parse");
    assert_eq!(g.mode(), N3Mode::Counting);
    let base_set: FxHashSet<[Term; 3]> = base.iter().cloned().collect();
    let rendered_set = rendered(&base);
    let mut derived = 0usize;
    for f in g.closure() {
        let tree = g
            .why(&f)
            .unwrap_or_else(|| panic!("no proof for closure fact"));
        check_proof(&tree, &rendered_set, Some(&render(&f)));
        if !base_set.contains(&f) {
            leaves_entail(RULES, &tree, &render(&f));
            derived += 1;
        }
    }
    assert!(
        derived >= 78,
        "expected the full ancestor closure, got {derived}"
    );
}

#[test]
fn retraction_consistency() {
    let mut base: Vec<[Term; 3]> = vec![
        [ex("a"), ex("parent"), ex("b")],
        [ex("b"), ex("parent"), ex("c")],
        [ex("a"), ex("ancestor"), ex("c")], // ALSO asserted: second support
    ];
    let mut g = MaterializedN3Graph::new(RULES, &base).expect("rules parse");
    let f = [ex("a"), ex("ancestor"), ex("c")];
    assert!(g.contains(&f));
    // Asserted + derived: explains as the asserted leaf (a witness, not all witnesses).
    assert_eq!(g.why(&f).unwrap().nodes().len(), 1);

    // Retract the assertion: the derived support takes over.
    g.delete(&[base.pop().unwrap()]);
    assert!(g.contains(&f));
    let tree = g.why(&f).expect("derived support survives");
    check_proof(&tree, &rendered(&base), Some(&render(&f)));
    assert_eq!(tree.nodes()[tree.root() as usize].rule, "n3-rule-1");
    leaves_entail(RULES, &tree, &render(&f));

    // Retract a premise of the derivation: the fact must vanish and stop explaining.
    g.delete(&[[ex("b"), ex("parent"), ex("c")]]);
    assert!(!g.contains(&f));
    assert!(g.why(&f).is_none());
}

#[test]
fn fallback_mode_still_explains() {
    // A backward rule disqualifies the rule set from counting — why() re-runs the batch
    // engine, so explanations still work in fallback mode.
    let rules = r#"
@prefix : <http://ex/> .
{ ?x :ancestor ?y } <= { ?x :parent ?y } .
{ ?x :parent ?y . ?y :parent ?z } => { ?x :grandparent ?z } .
"#;
    let base = vec![
        [ex("a"), ex("parent"), ex("b")],
        [ex("b"), ex("parent"), ex("c")],
    ];
    let g = MaterializedN3Graph::new(rules, &base).expect("rules parse");
    assert_eq!(g.mode(), N3Mode::Fallback);
    let f = [ex("a"), ex("grandparent"), ex("c")];
    assert!(g.contains(&f));
    let tree = g.why(&f).expect("fallback-mode derivation explains");
    check_proof(&tree, &rendered(&base), Some(&render(&f)));
    assert_eq!(tree.nodes()[tree.root() as usize].premises.len(), 2);
}

#[test]
fn id_level_bridge_from_reason_n3_proof() {
    use sparq_core::dict::Dict;
    let src = r#"
@prefix : <http://ex/> .
:a :parent :b .
:b :parent :c .
{ ?x :parent ?y } => { ?x :ancestor ?y } .
{ ?x :ancestor ?y . ?y :ancestor ?z } => { ?x :ancestor ?z } .
"#;
    let mut dict = Dict::new();
    let (facts, steps) = reason_n3_proof(&mut dict, src).expect("reasoning succeeds");
    let (a, anc, c) = (
        dict.lookup(&oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(
            "http://ex/a",
        ))),
        dict.lookup(&oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(
            "http://ex/ancestor",
        ))),
        dict.lookup(&oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(
            "http://ex/c",
        ))),
    );
    let target = [a, anc, c];
    assert!(facts.contains(&target));
    let tree = n3_proof_tree(&dict, &steps, target, ExplainOpts::default())
        .expect("derived triple bridges to a proof tree");
    let asserted: FxHashSet<[String; 3]> = facts
        .iter()
        .filter(|t| !steps.iter().any(|s| s.conclusion == **t))
        .map(|&t| {
            [
                dict.term(t[0]).to_string(),
                dict.term(t[1]).to_string(),
                dict.term(t[2]).to_string(),
            ]
        })
        .collect();
    check_proof(&tree, &asserted, None);
    assert_eq!(tree.conclusion()[1], "<http://ex/ancestor>");
    // Inputs have no step: the bridge returns None for them (callers explain those as asserted).
    let b = dict.lookup(&oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(
        "http://ex/b",
    )));
    let par = dict.lookup(&oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(
        "http://ex/parent",
    )));
    assert!(n3_proof_tree(&dict, &steps, [a, par, b], ExplainOpts::default()).is_none());
}
