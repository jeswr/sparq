//! [SONNET-4.6] The P8 cost/decidability spike acceptance suite (`sq-pfae.9`, issue #3281).
//!
//! Two obligations, both discharged with **deterministic** evidence only — no clock, no
//! elapsed duration, no allocator or host metadata is sampled anywhere in this file, because
//! work-box / EC2 timings are non-canonical in this repository and must never be gated:
//!
//! 1. **The admission-gate cost bound holds and is TIGHT.** Measured gate operations never
//!    exceed `admission_cost_bound`, componentwise, over a grid of `(rules, triples)`; and a
//!    saturated fixture — every triple passing every rule — attains the bound *exactly*. The
//!    two directions together are the mutation witness: inflating the closed form breaks the
//!    equality, deflating it breaks the domination.
//! 2. **Seeding directions are one-side-bound where the design says they must be** — and the
//!    one shipped ruleset that is NOT is pinned as such rather than quietly assumed safe.

#![cfg(feature = "cost-bound")]

use oxrdf::{BlankNode, Literal, NamedNode, NamedOrBlankNode, Term, Triple};
use sparq_trust::admit::{PresentedCredential, Session};
use sparq_trust::cost::{
    admission_cost_bound, admit_measured, analyse_seeding, require_one_side_bound, AdmissionShape,
    SeedingDenial, SeedingError, UnseededKind,
};
use sparq_trust::policy::{ShapeRef, TrustRule};
use sparq_zk::commit::commit_triples;
use sparq_zk::encode::salt_from_bytes;
use sparq_zk::sig::SecretKey;

const SCHEMA_AGE: &str = "http://schema.org/age";
const HOLDER: &str = "https://agent.example/alice";
const TARGET: &str = "https://pod.example/resource";
const SALT: [u8; 32] = [7u8; 32];
const NOW: i64 = 1_700_000_000;

/// The controller-authored `.acr` ABAC rule of design §3.1 — the rule text that actually
/// reaches the N3 evaluator on the admission path (`wire::derive_grants`).
const ACR_ABAC_RULE: &str = r#"@prefix schema: <http://schema.org/> .
@prefix math:   <http://www.w3.org/2000/10/swap/math#> .
@prefix auth:   <https://sparq.dev/ns/auth#> .
{ ?x schema:age ?y . ?y math:greaterThan 18 } => { ?x auth:read <https://pod.ex/resourceX> } .
"#;

fn iri(s: &str) -> NamedNode {
    NamedNode::new(s).expect("fixture IRIs are valid")
}

/// A single-predicate `forShape` (the `forPredicate` desugaring): target the subjects of
/// `SCHEMA_AGE` and require at least one such property.
fn age_shape(suffix: &str) -> ShapeRef {
    let root = BlankNode::new(format!("shape{}", suffix)).expect("valid fixture blank node");
    let property = BlankNode::new(format!("prop{}", suffix)).expect("valid fixture blank node");
    ShapeRef {
        root: Term::BlankNode(root.clone()),
        triples: vec![
            Triple::new(
                root.clone(),
                iri("http://www.w3.org/ns/shacl#targetSubjectsOf"),
                iri(SCHEMA_AGE),
            ),
            Triple::new(
                root,
                iri("http://www.w3.org/ns/shacl#property"),
                property.clone(),
            ),
            Triple::new(
                property.clone(),
                iri("http://www.w3.org/ns/shacl#path"),
                iri(SCHEMA_AGE),
            ),
            Triple::new(
                property,
                iri("http://www.w3.org/ns/shacl#minCount"),
                Literal::new_simple_literal("1"),
            ),
        ],
    }
}

/// A credential graph of `count` `<alice> schema:age N` triples — every triple of the
/// trusted statement-type, subject-bound to the holder, so nothing short-circuits.
fn saturating_graph(count: usize) -> Vec<Triple> {
    (0..count)
        .map(|n| {
            Triple::new(
                NamedOrBlankNode::NamedNode(iri(HOLDER)),
                iri(SCHEMA_AGE),
                Term::Literal(Literal::from(i64::try_from(n).expect("small fixture") + 18)),
            )
        })
        .collect()
}

/// `count` trust rules that all cover the target and all name the same (real) issuer key,
/// so every rule survives scope + signature + freshness + revocation.
fn saturating_rules(count: usize, key: &SecretKey) -> Vec<TrustRule> {
    (0..count)
        .map(|n| TrustRule {
            source: iri(&format!("https://authority.example/{}", n)),
            issuer_key: key.public_key(),
            shape: age_shape(&n.to_string()),
            scope: iri(TARGET),
            fresh_within_secs: 86_400,
        })
        .collect()
}

/// A credential signed by `key` over the RDFC-1.0 commitment of `graph`, exactly as the
/// gate verifies it.
fn signed_credential(key: &SecretKey, graph: Vec<Triple>) -> PresentedCredential {
    let salt = salt_from_bytes(&SALT);
    let commitment = commit_triples(&graph, salt).expect("fixture graph commits");
    PresentedCredential {
        issuer_signature_hex: key.sign_commitment(&commitment.commitment),
        graph,
        salt: SALT,
        issued_at_unix_secs: NOW,
        revoked: false,
    }
}

fn session() -> Session {
    Session {
        agent: iri(HOLDER),
        now_unix_secs: NOW,
    }
}

// ── 1. The admission-gate cost bound ──────────────────────────────────────────

#[test]
fn measured_gate_cost_never_exceeds_the_closed_form_bound() {
    let key = SecretKey::from_seed(0xC0FFEE);
    for rules in [0usize, 1, 3, 8] {
        for triples in [0usize, 1, 2, 5] {
            let cred = signed_credential(&key, saturating_graph(triples));
            let rule_set = saturating_rules(rules, &key);
            let (_, measured) = admit_measured(&cred, &rule_set, &session(), &iri(TARGET));
            let bound = admission_cost_bound(AdmissionShape {
                rules,
                graph_triples: triples,
            });
            assert!(
                bound.dominates(&measured),
                "R={} T={}: measured {:?} exceeds bound {:?}",
                rules,
                triples,
                measured,
                bound
            );
            // The published closed form, spelled out independently of the struct.
            assert!(
                measured.total() <= 2 + 4 * rules + 3 * rules * triples,
                "R={} T={}: total {} exceeds 2 + 4R + 3RT",
                rules,
                triples,
                measured.total()
            );
        }
    }
}

#[test]
fn a_saturated_credential_attains_the_bound_exactly_so_it_is_not_slack() {
    let key = SecretKey::from_seed(0xC0FFEE);
    let (rules, triples) = (4usize, 3usize);
    let cred = signed_credential(&key, saturating_graph(triples));
    let rule_set = saturating_rules(rules, &key);

    let (admitted, measured) = admit_measured(&cred, &rule_set, &session(), &iri(TARGET));
    // Every rule admitted every triple, so nothing short-circuited: the worst case ran.
    assert_eq!(admitted.len(), rules * triples);
    assert_eq!(
        measured,
        admission_cost_bound(AdmissionShape {
            rules,
            graph_triples: triples,
        }),
        "the saturated fixture must attain the bound exactly (a slack bound is a vacuous one)"
    );
    assert_eq!(measured.total(), 2 + 4 * rules + 3 * rules * triples);
}

#[test]
fn short_circuiting_strictly_reduces_cost_below_the_bound() {
    let key = SecretKey::from_seed(0xC0FFEE);
    let (rules, triples) = (4usize, 3usize);
    let cred = signed_credential(&key, saturating_graph(triples));
    // Same rules, but scoped to a resource the request is NOT for: every rule fails the
    // FIRST check, so the inner loop never runs.
    let mut rule_set = saturating_rules(rules, &key);
    for r in &mut rule_set {
        r.scope = iri("https://pod.example/other");
    }
    let (admitted, measured) = admit_measured(&cred, &rule_set, &session(), &iri(TARGET));
    assert!(admitted.is_empty(), "out-of-scope rules admit nothing");
    assert_eq!(measured.scope_checks, rules);
    assert_eq!(measured.signature_verifications, 0);
    assert_eq!(measured.shape_validations, 0);
    assert!(measured.total() < 2 + 4 * rules + 3 * rules * triples);
}

#[test]
fn the_bound_is_monotone_and_saturates_instead_of_overflowing() {
    let small = admission_cost_bound(AdmissionShape {
        rules: 2,
        graph_triples: 2,
    });
    let large = admission_cost_bound(AdmissionShape {
        rules: 3,
        graph_triples: 2,
    });
    assert!(large.dominates(&small) && large.total() > small.total());

    let absurd = admission_cost_bound(AdmissionShape {
        rules: usize::MAX,
        graph_triples: usize::MAX,
    });
    assert_eq!(absurd.shape_validations, usize::MAX);
    assert_eq!(absurd.total(), usize::MAX);
}

// ── 2. Seeding-direction analysis ─────────────────────────────────────────────

#[test]
fn the_acr_abac_rule_of_design_3_1_is_one_side_bound() {
    require_one_side_bound(ACR_ABAC_RULE).expect("the §3.1 worked-example rule seeds one-side");

    let report = analyse_seeding(ACR_ABAC_RULE).expect("parses");
    assert_eq!(report.rules.len(), 1);
    let rule = &report.rules[0];
    assert_eq!(rule.body_atoms, 2);
    // The `math:greaterThan 18` atom is the anchored seed (its OBJECT is a constant — a
    // constant predicate alone would not qualify), which binds `?y`; `?x schema:age ?y`
    // then joins on the bound `?y`. Neither step scans an unbounded extent.
    assert_eq!(rule.seeding_order, vec![1, 0]);
    assert!(rule.unseeded.is_empty() && rule.unsafe_head_vars.is_empty());
}

#[test]
fn a_two_unbound_atom_transitive_closure_rule_is_reported_not_silently_passed() {
    // The war story shape: neither atom has a bound subject or object, so no join order
    // avoids scanning the whole `p:linkedTo` extent.
    let n3 = "{ ?a p:linkedTo ?b . ?b p:linkedTo ?c } => { ?a p:linkedTo ?c } .\n";
    let report = analyse_seeding(n3).expect("parses");
    let rule = &report.rules[0];
    assert_eq!(rule.unseeded.len(), 2, "both atoms are two-unbound-atom seeds");
    assert!(rule
        .unseeded
        .iter()
        .all(|a| a.kind == UnseededKind::PredicateAnchored));
    assert!(!report.all_one_side_bound());
    assert_eq!(report.violations().len(), 1);
    assert!(matches!(
        require_one_side_bound(n3),
        Err(SeedingDenial::NotOneSideBound(_))
    ));
}

#[test]
fn a_wholly_unanchored_atom_is_classified_as_unanchored() {
    let n3 = "{ ?s ?p ?o } => { ?s p:seen ?o } .\n";
    let report = analyse_seeding(n3).expect("parses");
    assert_eq!(report.rules[0].unseeded.len(), 1);
    assert_eq!(report.rules[0].unseeded[0].kind, UnseededKind::Unanchored);
    assert_eq!(report.rules[0].unseeded[0].atom, "?s ?p ?o");
}

#[test]
fn one_constant_end_is_enough_to_seed_an_otherwise_unbound_conjunction() {
    // Same shape as the closure rule but anchored at one end: the seed is a constant
    // object, so a one-side-bound order exists and the greedy pick finds it.
    let n3 = "{ ?a p:linkedTo ?b . ?b p:linkedTo <urn:root> } => { ?a p:reaches <urn:root> } .\n";
    let report = analyse_seeding(n3).expect("parses");
    assert!(report.all_one_side_bound());
    // Atom 1 is the anchored seed; atom 0 becomes bound through `?b`.
    assert_eq!(report.rules[0].seeding_order, vec![1, 0]);
}

#[test]
fn a_head_variable_the_body_never_binds_is_a_range_restriction_failure() {
    let n3 = "{ ?a p:x ?b } => { ?c p:y ?a } .\n";
    let report = analyse_seeding(n3).expect("parses");
    assert_eq!(report.rules[0].unsafe_head_vars, vec!["?c".to_owned()]);
    assert!(!report.all_one_side_bound());
}

#[test]
fn predicate_object_and_object_lists_expand_before_the_analysis() {
    let n3 = "{ ?c p:op p:gteq ; p:left ?l , ?l2 } => { ?c p:ok ?l } .\n";
    let report = analyse_seeding(n3).expect("parses");
    let rule = &report.rules[0];
    assert_eq!(rule.body_atoms, 3, "`;` and `,` expand to three atoms");
    assert!(rule.is_one_side_bound(), "the constant object `p:gteq` seeds ?c");
    assert_eq!(rule.seeding_order, vec![0, 1, 2]);
}

#[test]
fn unmodelled_n3_constructs_fail_closed_rather_than_passing() {
    for (src, what) in [
        ("{ ?m p:has [ p:level ?l ] } => { ?m p:ok ?l } .\n", "blank-node property list"),
        ("{ ?m p:list ( ?a ?b ) } => { ?m p:ok ?a } .\n", "collection"),
        ("{ ?a p:x ?b } <= { ?b p:y ?a } .\n", "reverse implication"),
        ("{ ?a p:x { ?b p:y ?c } } => { ?a p:ok ?b } .\n", "nested formula"),
        ("{ ?a p:x } => { ?a p:ok ?a } .\n", "two-term statement"),
    ] {
        assert!(
            matches!(require_one_side_bound(src), Err(SeedingDenial::Unanalysable(_))),
            "{} must be refused, not silently accepted",
            what
        );
    }
}

#[test]
fn ground_facts_and_prefix_declarations_are_not_rules() {
    let report = analyse_seeding(
        "@prefix p: <https://example.org/ns#> .\n# a comment with a } brace\np:a p:b p:c .\n",
    )
    .expect("parses");
    assert!(report.rules.is_empty());
    // Vacuously one-side-bound: a document with no rules has no seeding direction at all.
    assert!(report.all_one_side_bound());
}

#[test]
fn an_unterminated_iri_is_an_error_not_a_pass() {
    assert!(matches!(
        analyse_seeding("?a <https://example.org/x ?b .\n"),
        Err(SeedingError::Unterminated(_))
    ));
    // An IRI left open INSIDE a rule swallows tokens up to the next `>`; the result is
    // still a refusal, never a silent pass.
    assert!(
        analyse_seeding("{ ?a <https://example.org/x ?b } => { ?a p:ok ?b } .\n").is_err(),
        "a malformed rule must not be reported as analysed"
    );
}

#[test]
fn denials_render_a_usable_reason() {
    let Err(denial) = require_one_side_bound("{ ?s ?p ?o } => { ?s p:seen ?o } .\n") else {
        panic!("an unanchored atom must be denied");
    };
    assert_eq!(denial.to_string(), "1 of 1 rule(s) are not one-side-bound");

    let Err(denial) = require_one_side_bound("{ ?m p:has [ p:level ?l ] } => { ?m p:ok ?l } .\n")
    else {
        panic!("a blank-node property list must be denied");
    };
    assert!(
        denial
            .to_string()
            .starts_with("document not analysable: unsupported N3 construct: "),
        "unexpected denial rendering: {}",
        denial
    );
}

/// The honest finding P8 records: the bundled admissibility ruleset is NOT wholly
/// one-side-bound. Its transitive-closure rule is a genuine two-unbound-atom seed, safe
/// only because its extent is the closed, bundled, constant `LEVEL_ORDERS` fact base —
/// not because the seeding is bounded. The discharge rule, by contrast, IS one-side-bound
/// (its `odrl:gteq` constant object seeds the conjunction).
#[cfg(feature = "secprop-admissibility")]
#[test]
fn the_bundled_admissibility_rulesets_are_pinned_as_analysed() {
    use sparq_trust::admissibility::{CLOSURE_RULES, DISCHARGE_RULE};

    let discharge = analyse_seeding(DISCHARGE_RULE).expect("the discharge rule parses");
    assert!(
        discharge.all_one_side_bound(),
        "the discharge rule must stay one-side-bound: {:?}",
        discharge.violations()
    );

    let closure = analyse_seeding(CLOSURE_RULES).expect("the closure rules parse");
    assert_eq!(closure.rules.len(), 3);
    // Rule 0 is the transitive closure — the known, documented two-unbound-atom seed.
    assert!(!closure.rules[0].unseeded.is_empty());
    assert!(closure.rules[0]
        .unseeded
        .iter()
        .all(|a| a.kind == UnseededKind::PredicateAnchored));
    // Every closure rule IS range-restricted, which is what its module docs claim.
    assert!(closure
        .rules
        .iter()
        .all(|r| r.unsafe_head_vars.is_empty()));
}
