//! Shared NAIVE proof checker for the `explain` tests: re-validates every step of a
//! [`ProofTree`] against an independent rule table (premise shapes written straight from
//! the RDFS / OWL 2 RL spec tables), plus the structural guarantees (flat
//! premises-before-conclusion order, root last, asserted leaves really asserted).
//!
//! Deliberately independent of the prover: it works on the RENDERED term strings only, so
//! a prover bug cannot leak into the oracle through shared code.

use rustc_hash::FxHashSet;
use sparq_reason::ProofTree;

pub const TY: &str = "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>";
pub const SC: &str = "<http://www.w3.org/2000/01/rdf-schema#subClassOf>";
pub const SP: &str = "<http://www.w3.org/2000/01/rdf-schema#subPropertyOf>";
pub const DOM: &str = "<http://www.w3.org/2000/01/rdf-schema#domain>";
pub const RNG: &str = "<http://www.w3.org/2000/01/rdf-schema#range>";
pub const EQC: &str = "<http://www.w3.org/2002/07/owl#equivalentClass>";
pub const EQP: &str = "<http://www.w3.org/2002/07/owl#equivalentProperty>";
pub const INV: &str = "<http://www.w3.org/2002/07/owl#inverseOf>";
pub const SYM: &str = "<http://www.w3.org/2002/07/owl#SymmetricProperty>";
pub const TRANS: &str = "<http://www.w3.org/2002/07/owl#TransitiveProperty>";
pub const DIF: &str = "<http://www.w3.org/2002/07/owl#differentFrom>";
pub const OWL_CLASS: &str = "<http://www.w3.org/2002/07/owl#Class>";
pub const OWL_THING: &str = "<http://www.w3.org/2002/07/owl#Thing>";
pub const OWL_NOTHING: &str = "<http://www.w3.org/2002/07/owl#Nothing>";

/// The XSD numeric-tower subclass axioms the engine may emit as `axiom-xsd` leaves.
const XSD_PAIRS: &[(&str, &str)] = &[
    ("byte", "short"),
    ("short", "int"),
    ("int", "long"),
    ("long", "integer"),
    ("integer", "decimal"),
    ("unsignedByte", "unsignedShort"),
    ("unsignedShort", "unsignedInt"),
    ("unsignedInt", "unsignedLong"),
    ("unsignedLong", "nonNegativeInteger"),
    ("nonNegativeInteger", "integer"),
    ("positiveInteger", "nonNegativeInteger"),
    ("negativeInteger", "nonPositiveInteger"),
    ("nonPositiveInteger", "integer"),
];

fn xsd(frag: &str) -> String {
    format!("<http://www.w3.org/2001/XMLSchema#{frag}>")
}

/// Validate `tree` (which must conclude `expect`, if given) against `asserted` (rendered
/// base triples). Every rule application is re-checked independently; panics with a
/// description on the first invalid step.
pub fn check_proof(
    tree: &ProofTree,
    asserted: &FxHashSet<[String; 3]>,
    expect: Option<&[String; 3]>,
) {
    let nodes = tree.nodes();
    assert!(!nodes.is_empty(), "empty proof");
    assert_eq!(
        tree.root() as usize,
        nodes.len() - 1,
        "root must be the last node"
    );
    if let Some(e) = expect {
        assert_eq!(tree.conclusion(), e, "proof concludes the wrong triple");
    }
    for (i, n) in nodes.iter().enumerate() {
        for &p in &n.premises {
            assert!(
                (p as usize) < i,
                "node {i}: premise {p} not before conclusion"
            );
        }
        let prem: Vec<&[String; 3]> = n
            .premises
            .iter()
            .map(|&p| &nodes[p as usize].conclusion)
            .collect();
        let c = &n.conclusion;
        let fail = |msg: &str| panic!("node {i} [{}] {c:?} ← {prem:?}: {msg}", n.rule);
        let eq = |a: &str, b: &str, msg: &str| {
            if a != b {
                fail(msg);
            }
        };
        match n.rule.as_str() {
            "asserted" => {
                if !prem.is_empty() {
                    fail("asserted leaf with premises");
                }
                if !asserted.contains(c) {
                    fail("asserted leaf not in the base");
                }
            }
            "axiom-xsd" => {
                if !prem.is_empty() {
                    fail("axiom leaf with premises");
                }
                let ok = c[1] == SC
                    && XSD_PAIRS
                        .iter()
                        .any(|&(a, b)| c[0] == xsd(a) && c[2] == xsd(b));
                if !ok {
                    fail("not an XSD numeric-tower subclass axiom");
                }
            }
            "axiom-owl" => {
                if !prem.is_empty() {
                    fail("axiom leaf with premises");
                }
                let ok =
                    c[1] == TY && c[2] == OWL_CLASS && (c[0] == OWL_THING || c[0] == OWL_NOTHING);
                if !ok {
                    fail("not the owl:Thing/Nothing typing axiom");
                }
            }
            // (c sc d), (x type c) ⊢ (x type d)
            "rdfs9" | "cax-sco" => {
                let [p0, p1] = two(&prem);
                eq(&p0[1], SC, "premise 0 not subClassOf");
                eq(&p1[1], TY, "premise 1 not type");
                eq(&p1[2], &p0[0], "type premise class != subclass subject");
                eq(&c[0], &p1[0], "subject mismatch");
                eq(&c[1], TY, "conclusion not type");
                eq(&c[2], &p0[2], "class mismatch");
            }
            // (a R m), (m R n) ⊢ (a R n) for R = subClassOf / subPropertyOf
            "rdfs11" | "scm-sco" | "rdfs5" | "scm-spo" => {
                let r = if matches!(n.rule.as_str(), "rdfs11" | "scm-sco") {
                    SC
                } else {
                    SP
                };
                let [p0, p1] = two(&prem);
                eq(&p0[1], r, "premise 0 wrong predicate");
                eq(&p1[1], r, "premise 1 wrong predicate");
                eq(&p0[2], &p1[0], "chain does not connect");
                eq(&c[0], &p0[0], "subject mismatch");
                eq(&c[1], r, "conclusion wrong predicate");
                eq(&c[2], &p1[2], "object mismatch");
            }
            // (p sp q), (s p o) ⊢ (s q o)
            "rdfs7" | "prp-spo1" => {
                let [p0, p1] = two(&prem);
                eq(&p0[1], SP, "premise 0 not subPropertyOf");
                eq(&p1[1], &p0[0], "data premise predicate != sub-property");
                eq(&c[0], &p1[0], "subject mismatch");
                eq(&c[1], &p0[2], "conclusion predicate != super-property");
                eq(&c[2], &p1[2], "object mismatch");
            }
            // (p domain c), (s p o) ⊢ (s type c)
            "rdfs2" | "prp-dom" => {
                let [p0, p1] = two(&prem);
                eq(&p0[1], DOM, "premise 0 not domain");
                eq(&p1[1], &p0[0], "data premise predicate != property");
                eq(&c[0], &p1[0], "subject mismatch");
                eq(&c[1], TY, "conclusion not type");
                eq(&c[2], &p0[2], "class mismatch");
            }
            // (p range c), (s p o) ⊢ (o type c)
            "rdfs3" | "prp-rng" => {
                let [p0, p1] = two(&prem);
                eq(&p0[1], RNG, "premise 0 not range");
                eq(&p1[1], &p0[0], "data premise predicate != property");
                eq(&c[0], &p1[2], "subject != data object");
                eq(&c[1], TY, "conclusion not type");
                eq(&c[2], &p0[2], "class mismatch");
            }
            // (p inverseOf q), (x p y) ⊢ (y q x)
            "prp-inv1" => {
                let [p0, p1] = two(&prem);
                eq(&p0[1], INV, "premise 0 not inverseOf");
                eq(&p1[1], &p0[0], "data premise predicate != p");
                eq(&c[0], &p1[2], "conclusion subject != data object");
                eq(&c[1], &p0[2], "conclusion predicate != q");
                eq(&c[2], &p1[0], "conclusion object != data subject");
            }
            // (p inverseOf q), (x q y) ⊢ (y p x)
            "prp-inv2" => {
                let [p0, p1] = two(&prem);
                eq(&p0[1], INV, "premise 0 not inverseOf");
                eq(&p1[1], &p0[2], "data premise predicate != q");
                eq(&c[0], &p1[2], "conclusion subject != data object");
                eq(&c[1], &p0[0], "conclusion predicate != p");
                eq(&c[2], &p1[0], "conclusion object != data subject");
            }
            // (p type Symmetric), (x p y) ⊢ (y p x)
            "prp-symp" => {
                let [p0, p1] = two(&prem);
                if p0[1] != TY || p0[2] != SYM {
                    fail("premise 0 not a SymmetricProperty typing");
                }
                eq(&p1[1], &p0[0], "data premise predicate != p");
                eq(&c[0], &p1[2], "conclusion subject != data object");
                eq(&c[1], &p1[1], "predicate mismatch");
                eq(&c[2], &p1[0], "conclusion object != data subject");
            }
            // (p eqp q), (x p y) ⊢ (x q y)   /   (p eqp q), (x q y) ⊢ (x p y)
            "prp-eqp1" | "prp-eqp2" => {
                let [p0, p1] = two(&prem);
                eq(&p0[1], EQP, "premise 0 not equivalentProperty");
                let (from, to) = if n.rule == "prp-eqp1" {
                    (&p0[0], &p0[2])
                } else {
                    (&p0[2], &p0[0])
                };
                eq(&p1[1], from, "data premise predicate mismatch");
                eq(&c[0], &p1[0], "subject mismatch");
                eq(&c[1], to, "conclusion predicate mismatch");
                eq(&c[2], &p1[2], "object mismatch");
            }
            // (p type Transitive), (x p y), (y p z) ⊢ (x p z)
            "prp-trp" => {
                let [p0, p1, p2] = three(&prem);
                if p0[1] != TY || p0[2] != TRANS {
                    fail("premise 0 not a TransitiveProperty typing");
                }
                eq(&p1[1], &p0[0], "premise 1 predicate != p");
                eq(&p2[1], &p0[0], "premise 2 predicate != p");
                eq(&p1[2], &p2[0], "chain does not connect");
                eq(&c[0], &p1[0], "subject mismatch");
                eq(&c[1], &p0[0], "predicate mismatch");
                eq(&c[2], &p2[2], "object mismatch");
            }
            // (a eqc b) ⊢ (a sc b) / (b sc a);   property analogue scm-eqp1
            "scm-eqc1" | "scm-eqp1" => {
                let (epred, rpred) = if n.rule == "scm-eqc1" {
                    (EQC, SC)
                } else {
                    (EQP, SP)
                };
                let [p0] = one(&prem);
                eq(&p0[1], epred, "premise not an equivalence");
                eq(&c[1], rpred, "conclusion wrong predicate");
                let fwd = c[0] == p0[0] && c[2] == p0[2];
                let bwd = c[0] == p0[2] && c[2] == p0[0];
                if !fwd && !bwd {
                    fail("conclusion not either direction of the equivalence");
                }
            }
            // (a sc b), (b sc a) ⊢ (a eqc b);   property analogue scm-eqp2
            "scm-eqc2" | "scm-eqp2" => {
                let (epred, rpred) = if n.rule == "scm-eqc2" {
                    (EQC, SC)
                } else {
                    (EQP, SP)
                };
                let [p0, p1] = two(&prem);
                eq(&p0[1], rpred, "premise 0 wrong predicate");
                eq(&p1[1], rpred, "premise 1 wrong predicate");
                if !(p0[0] == p1[2] && p0[2] == p1[0]) {
                    fail("premises not mutual subsumption");
                }
                eq(&c[1], epred, "conclusion wrong predicate");
                if !(c[0] == p0[0] && c[2] == p0[2]) {
                    fail("conclusion does not match premise orientation");
                }
            }
            // (p dom c), (c sc d) ⊢ (p dom d);   range analogue scm-rng1
            "scm-dom1" | "scm-rng1" => {
                let dr = if n.rule == "scm-dom1" { DOM } else { RNG };
                let [p0, p1] = two(&prem);
                eq(&p0[1], dr, "premise 0 wrong predicate");
                eq(&p1[1], SC, "premise 1 not subClassOf");
                eq(&p0[2], &p1[0], "class chain does not connect");
                eq(&c[0], &p0[0], "property mismatch");
                eq(&c[1], dr, "conclusion wrong predicate");
                eq(&c[2], &p1[2], "class mismatch");
            }
            // (q dom c), (p sp q) ⊢ (p dom c);   range analogue scm-rng2
            "scm-dom2" | "scm-rng2" => {
                let dr = if n.rule == "scm-dom2" { DOM } else { RNG };
                let [p0, p1] = two(&prem);
                eq(&p0[1], dr, "premise 0 wrong predicate");
                eq(&p1[1], SP, "premise 1 not subPropertyOf");
                eq(&p1[2], &p0[0], "property chain does not connect");
                eq(&c[0], &p1[0], "property mismatch");
                eq(&c[1], dr, "conclusion wrong predicate");
                eq(&c[2], &p0[2], "class mismatch");
            }
            // ENGINE rule (semantics of inverseOf, not in the RL table):
            // (p inv q | q inv p), (p dom c) ⊢ (q rng c)  /  (p rng c) ⊢ (q dom c)
            "inv-dom" | "inv-rng" => {
                let (src_dr, dst_dr) = if n.rule == "inv-dom" {
                    (RNG, DOM)
                } else {
                    (DOM, RNG)
                };
                let [p0, p1] = two(&prem);
                eq(&p0[1], INV, "premise 0 not inverseOf");
                eq(&p1[1], src_dr, "premise 1 wrong predicate");
                let q = &c[0];
                let p = &p1[0];
                let related = (p0[0] == *p && p0[2] == *q) || (p0[0] == *q && p0[2] == *p);
                if !related {
                    fail("inverse premise does not relate the two properties");
                }
                eq(&c[1], dst_dr, "conclusion wrong predicate");
                eq(&c[2], &p1[2], "class mismatch");
            }
            // ENGINE rule (semantics of differentFrom): (x df y) ⊢ (y df x)
            "sym-dif" => {
                let [p0] = one(&prem);
                eq(&p0[1], DIF, "premise not differentFrom");
                eq(&c[0], &p0[2], "subject mismatch");
                eq(&c[1], DIF, "conclusion not differentFrom");
                eq(&c[2], &p0[0], "object mismatch");
            }
            r if r.starts_with("n3-rule-") => {
                // User-supplied rule: structural checks only here; semantic validity is
                // covered by the leaf-subset re-entailment check in the N3 tests.
                if prem.is_empty() {
                    fail("rule application with no premises");
                }
            }
            r => fail(&format!("unknown rule '{r}'")),
        }
    }
}

fn one<'a>(prem: &[&'a [String; 3]]) -> [&'a [String; 3]; 1] {
    assert_eq!(prem.len(), 1, "expected 1 premise, got {}", prem.len());
    [prem[0]]
}
fn two<'a>(prem: &[&'a [String; 3]]) -> [&'a [String; 3]; 2] {
    assert_eq!(prem.len(), 2, "expected 2 premises, got {}", prem.len());
    [prem[0], prem[1]]
}
fn three<'a>(prem: &[&'a [String; 3]]) -> [&'a [String; 3]; 3] {
    assert_eq!(prem.len(), 3, "expected 3 premises, got {}", prem.len());
    [prem[0], prem[1], prem[2]]
}

/// The asserted leaves of a proof (rendered triples).
pub fn proof_leaves(tree: &ProofTree) -> Vec<[String; 3]> {
    tree.nodes()
        .iter()
        .filter(|n| n.rule == "asserted")
        .map(|n| n.conclusion.clone())
        .collect()
}
