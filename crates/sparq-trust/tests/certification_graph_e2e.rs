//! End-to-end + ADVERSARIAL edge-forgery matrix for the depth-bounded certification-edge
//! trust-graph closure (`sparq_trust::graph::derive_effective_rules`, `sq-pfae.15`).
//!
//! This is the load-bearing evidence for the bead's HARD invariant: **fail-closed,
//! attenuation-ONLY**. The closure runs AHEAD of the UNCHANGED admission gate and produces
//! the `Vec<TrustRule>` the gate consumes; a certification can only NARROW, never widen,
//! the certifier's own authority. The matrix drives EVERY escalation shape a broadening or
//! forged edge would take and asserts each DENIES (contributes nothing), plus a positive
//! end-to-end case and the strict-additivity byte-identical property.
//!
//! A broadening case the code lets through is a PRIVILEGE-ESCALATION bug, not a test to
//! weaken. Run: `cargo test -p sparq-trust --features cert-graph --test certification_graph_e2e`.
//!
//! [OPUS-4.8] sq-pfae.15 (epic sq-pfae, issue #940). Written while Fable unavailable — flag
//! for re-review when Fable returns. 🤖 SPARQ agent — certification-edge closure adversarial matrix.
#![cfg(feature = "cert-graph")]

use oxrdf::{BlankNode, Literal, NamedNode, Term, Triple};
use sparq_trust::graph::{
    certification_message, derive_effective_rules, explain_edge, CertScope, Certification,
    EdgeRejection,
};
use sparq_trust::policy::{ShapeRef, TrustRule};
use sparq_zk::sig::{public_key_to_hex, PublicKey, SecretKey};

// ── fixtures ────────────────────────────────────────────────────────────────

const GOV: &str = "https://gov.example/framework"; // the framework operator / certifier
const ISSUER: &str = "https://issuer.example/dvs"; // the issuer the cert vouches for
const AGE: &str = "https://schema.org/age";
const NAME: &str = "https://schema.org/name";
const RES: &str = "https://pod.ex/resourceX";
const NOW: i64 = 1_700_000_000;
const FRESH: i64 = 30 * 86400;

fn iri(s: &str) -> NamedNode {
    NamedNode::new(s).unwrap()
}

fn keypair(seed: u64) -> (SecretKey, PublicKey) {
    let sk = SecretKey::from_seed(seed);
    let pk = sk.public_key();
    (sk, pk)
}

/// A `trust:forPredicate P` desugared into a single-predicate SHACL node-shape (subjects
/// of `p`, `p` minCount 1) — the crate's ONE statement-type idiom.
fn predicate_shape(p: &str) -> ShapeRef {
    let root = BlankNode::default();
    let prop = BlankNode::default();
    let pred = iri(p);
    ShapeRef {
        root: Term::BlankNode(root.clone()),
        triples: vec![
            Triple::new(
                root.clone(),
                iri("http://www.w3.org/ns/shacl#targetSubjectsOf"),
                pred.clone(),
            ),
            Triple::new(
                root,
                iri("http://www.w3.org/ns/shacl#property"),
                prop.clone(),
            ),
            Triple::new(prop.clone(), iri("http://www.w3.org/ns/shacl#path"), pred),
            Triple::new(
                prop,
                iri("http://www.w3.org/ns/shacl#minCount"),
                Literal::new_simple_literal("1"),
            ),
        ],
    }
}

/// A shape that adds a constraint to `predicate_shape(p)` (a strictly-tighter shape: same
/// constraints PLUS a `sh:datatype` restriction) — a provable NARROWING of `predicate_shape`.
fn narrower_shape(p: &str) -> ShapeRef {
    let mut s = predicate_shape(p);
    // The property node is the subject of the last two triples; reuse it to add a datatype.
    let prop = s.triples[1].object.clone();
    if let Term::BlankNode(prop_bn) = prop {
        s.triples.push(Triple::new(
            prop_bn,
            iri("http://www.w3.org/ns/shacl#datatype"),
            iri("http://www.w3.org/2001/XMLSchema#integer"),
        ));
    }
    s
}

/// A shape that ADDS an extra `sh:targetSubjectsOf q` root triple to `predicate_shape(p)`.
/// Every constraint of `predicate_shape(p)` is present PLUS one more target predicate — an
/// injective structural SUPERSET of `predicate_shape(p)`. It is a **broadening**, not a
/// narrowing: SHACL selects focus nodes by the UNION of target predicates, so the extra
/// `sh:targetSubjectsOf q` WIDENS the admitted set to subjects of `p` OR `q` (the
/// privilege-escalation shape the `sq-pfae.15` review found). The containment check must
/// treat selection vocab as CONTRAVARIANT and reject this.
fn additive_target_shape(p: &str, extra_target: &str) -> ShapeRef {
    let mut s = predicate_shape(p);
    if let Term::BlankNode(root) = s.root.clone() {
        s.triples.push(Triple::new(
            root,
            iri("http://www.w3.org/ns/shacl#targetSubjectsOf"),
            iri(extra_target),
        ));
    }
    s
}

/// The set of `sh:targetSubjectsOf` object IRIs a shape declares at its root — the
/// target-predicate set the admission gate (`admit::shape_targets_subject`) selects focus
/// nodes by (the UNION). Used to assert the actual consumed-by-admission invariant: a derived
/// rule's target-predicate set must be `⊆` the anchor's.
fn target_predicate_set(shape: &ShapeRef) -> std::collections::BTreeSet<String> {
    let root = match &shape.root {
        Term::BlankNode(b) => format!("B{}", b.as_str()),
        Term::NamedNode(n) => format!("I{}", n.as_str()),
        other => format!("{other:?}"),
    };
    shape
        .triples
        .iter()
        .filter(|t| {
            let subj = match &t.subject {
                oxrdf::NamedOrBlankNode::BlankNode(b) => format!("B{}", b.as_str()),
                oxrdf::NamedOrBlankNode::NamedNode(n) => format!("I{}", n.as_str()),
            };
            subj == root
                && t.predicate.as_str() == "http://www.w3.org/ns/shacl#targetSubjectsOf"
        })
        .filter_map(|t| match &t.object {
            Term::NamedNode(n) => Some(n.as_str().to_string()),
            _ => None,
        })
        .collect()
}

fn anchor(source: &str, key: PublicKey, shape: ShapeRef, scope: &str) -> TrustRule {
    TrustRule {
        source: iri(source),
        issuer_key: key,
        shape,
        scope: iri(scope),
        fresh_within_secs: FRESH,
    }
}

/// A certification signed by `certifier_sk` over the domain-separated message.
fn signed_cert(
    certifier: &str,
    certifier_sk: &SecretKey,
    certified: &str,
    certified_key: PublicKey,
    scope: CertScope,
    valid_from: i64,
    valid_until: i64,
) -> Certification {
    let mut cert = Certification {
        certifier: iri(certifier),
        certifier_key: certifier_sk.public_key(),
        certified_issuer: iri(certified),
        certified_key,
        scope,
        valid_from_unix_secs: valid_from,
        valid_until_unix_secs: valid_until,
        signature_hex: String::new(),
    };
    cert.signature_hex = certifier_sk.sign_commitment(&certification_message(&cert));
    cert
}

/// The standard AnyService-scoped, in-window, signed cert from GOV → ISSUER.
fn good_cert() -> (TrustRule, Certification, PublicKey) {
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let cert = signed_cert(
        GOV,
        &gov_sk,
        ISSUER,
        iss_pk,
        CertScope::AnyService,
        NOW - 86400,
        NOW + 86400,
    );
    (anchor_rule, cert, iss_pk)
}

/// Assert the closure produced EXACTLY the anchors (no derived rule) — the fail-closed
/// outcome for a denied edge.
fn assert_no_derivation(anchors: &[TrustRule], certs: &[Certification]) {
    let out = derive_effective_rules(anchors, certs, NOW, 1);
    assert_eq!(
        out.len(),
        anchors.len(),
        "a denied edge must contribute NOTHING (output == anchors only)"
    );
}

// ── POSITIVE end-to-end ───────────────────────────────────────────────────────

#[test]
fn positive_certified_issuer_admitted_within_intersected_scope() {
    let (anchor_rule, cert, iss_pk) = good_cert();
    let out = derive_effective_rules(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&cert), NOW, 1);
    assert_eq!(out.len(), 2, "anchor + one derived rule");
    // Anchor first, verbatim.
    assert_eq!(out[0].source.as_str(), GOV);
    // The derived rule authorises the CERTIFIED issuer with ITS key, over the anchor's
    // resource scope + (AnyService ⇒ anchor's) shape, no fresher than the anchor.
    let d = &out[1];
    assert_eq!(d.source.as_str(), ISSUER);
    assert_eq!(public_key_to_hex(&d.issuer_key), public_key_to_hex(&iss_pk));
    assert_eq!(d.scope.as_str(), RES, "resource scope is the certifier ceiling, unchanged");
    assert!(
        d.fresh_within_secs <= anchor_rule.fresh_within_secs,
        "attenuation: derived freshness never exceeds the anchor's"
    );
    // And the explained path agrees.
    assert!(explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 1).is_ok());
}

#[test]
fn positive_shape_scoped_cert_narrows_to_the_tighter_shape() {
    // The certifier is anchored for `age` (broad); the cert narrows to the integer-typed
    // `age` sub-shape. The derived rule must carry the STRICTLY TIGHTER shape.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let cert = signed_cert(
        GOV,
        &gov_sk,
        ISSUER,
        iss_pk,
        CertScope::Shape(narrower_shape(AGE)),
        NOW - 10,
        NOW + 10,
    );
    let d = explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 1)
        .expect("a narrowing shape-scope is admitted");
    // The derived shape must be the tighter one (more constraint triples than the anchor).
    assert!(
        d.shape.triples.len() > anchor_rule.shape.triples.len(),
        "derived shape is the strictly-tighter cert shape (a narrowing)"
    );
}

// ── STRICT ADDITIVITY ─────────────────────────────────────────────────────────

#[test]
fn zero_certifications_is_direct_rules_byte_identical() {
    let (_sk, pk) = keypair(1);
    let rules = vec![
        anchor("https://a.ex", pk, predicate_shape(AGE), "https://pod.ex/"),
        anchor("https://b.ex", pk, predicate_shape(NAME), "https://pod.ex/x"),
    ];
    let out = derive_effective_rules(&rules, &[], NOW, 1);
    assert_eq!(out.len(), rules.len(), "no certs ⇒ no derived rules");
    for (a, b) in rules.iter().zip(out.iter()) {
        assert_eq!(a.source.as_str(), b.source.as_str());
        assert_eq!(a.scope.as_str(), b.scope.as_str());
        assert_eq!(a.fresh_within_secs, b.fresh_within_secs);
        assert_eq!(a.shape.triples.len(), b.shape.triples.len());
        assert_eq!(
            public_key_to_hex(&a.issuer_key),
            public_key_to_hex(&b.issuer_key)
        );
    }
}

#[test]
fn all_edges_denied_leaves_direct_rules_byte_identical() {
    // A pile of edges that ALL deny (forged, broadening, expired, cyclic) must leave the
    // anchors exactly as they were — strict additivity holds even under adversarial input.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let (_atk_sk, atk_pk) = keypair(9);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);

    // forged (attacker-signed), expired, self-cert cycle, no-anchor
    let mut forged = signed_cert(GOV, &gov_sk, ISSUER, iss_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    forged.signature_hex = SecretKey::from_seed(9).sign_commitment(&certification_message(&forged));
    let expired = signed_cert(GOV, &gov_sk, ISSUER, iss_pk, CertScope::AnyService, NOW - 100, NOW - 50);
    let self_cert = signed_cert(GOV, &gov_sk, GOV, atk_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    let (unknown_sk, _unknown_pk) = keypair(7);
    let no_anchor = signed_cert("https://rogue.ex", &unknown_sk, ISSUER, iss_pk, CertScope::AnyService, NOW - 10, NOW + 10);

    assert_no_derivation(
        std::slice::from_ref(&anchor_rule),
        &[forged, expired, self_cert, no_anchor],
    );
}

// ── ADVERSARIAL matrix: each edge-forgery shape DENIES ─────────────────────────

#[test]
fn forged_certifier_signature_is_denied() {
    // The genuine GOV → ISSUER edge, but signed by an ATTACKER key (or corrupted). The
    // certifier_key still names GOV's key, so the anchor matches — the SIGNATURE is the gate.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let mut cert = signed_cert(GOV, &gov_sk, ISSUER, iss_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    // Re-sign under an attacker key while leaving certifier_key = GOV's key.
    cert.signature_hex = SecretKey::from_seed(42).sign_commitment(&certification_message(&cert));
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 1),
        Err(EdgeRejection::SignatureInvalid)
    ));
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&cert));
}

#[test]
fn absent_signature_is_denied() {
    let (anchor_rule, mut cert, _iss_pk) = good_cert();
    cert.signature_hex = String::new();
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 1),
        Err(EdgeRejection::SignatureInvalid)
    ));
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&cert));
}

#[test]
fn tampered_scope_after_signing_is_denied() {
    // A forger lifts a genuine narrow cert and re-presents it with a BROADER scope
    // (AnyService) WITHOUT re-signing (it cannot — it lacks GOV's key). The scope is folded
    // into the signed message, so the signature no longer verifies.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let mut cert = signed_cert(
        GOV,
        &gov_sk,
        ISSUER,
        iss_pk,
        CertScope::Shape(narrower_shape(AGE)),
        NOW - 10,
        NOW + 10,
    );
    // Tamper: widen the scope to AnyService, keep the old signature.
    cert.scope = CertScope::AnyService;
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 1),
        Err(EdgeRejection::SignatureInvalid)
    ));
}

#[test]
fn key_substitution_on_certified_issuer_is_denied() {
    // A forger captures the GOV → ISSUER edge and substitutes its OWN key as the certified
    // issuer's key (to then present its own credentials as ISSUER). The certified_key is
    // folded into the signed message, so the substitution breaks GOV's signature.
    let (anchor_rule, mut cert, _iss_pk) = good_cert();
    let (_atk_sk, atk_pk) = keypair(66);
    cert.certified_key = atk_pk; // substitute — but do NOT (cannot) re-sign under GOV's key
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 1),
        Err(EdgeRejection::SignatureInvalid)
    ));
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&cert));
}

#[test]
fn no_matching_anchor_is_denied() {
    // The certifier names no anchor rule (the pod does not anchor "rogue.ex"). Even a
    // perfectly-signed edge confers nothing without an anchor to narrow.
    let (_gov_sk, gov_pk) = keypair(1);
    let (rogue_sk, _rogue_pk) = keypair(5);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let cert = signed_cert(
        "https://rogue.ex",
        &rogue_sk,
        ISSUER,
        iss_pk,
        CertScope::AnyService,
        NOW - 10,
        NOW + 10,
    );
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 1),
        Err(EdgeRejection::NoAnchor)
    ));
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&cert));
}

#[test]
fn certifier_named_with_wrong_key_is_denied() {
    // The certifier IRI matches an anchor, but certifier_key is a DIFFERENT key (and the
    // edge is signed under that different key so the sig would verify in isolation). The
    // anchor-key binding rejects it: the pod anchors GOV's real key, not this one.
    let (_gov_sk, gov_pk) = keypair(1);
    let (wrong_sk, _wrong_pk) = keypair(3);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    // signed_cert sets certifier_key = wrong_sk.public_key(); the IRI is GOV.
    let cert = signed_cert(GOV, &wrong_sk, ISSUER, iss_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 1),
        Err(EdgeRejection::NoAnchor)
    ));
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&cert));
}

#[test]
fn scope_broadening_shape_cert_is_denied() {
    // The certifier is anchored ONLY for the integer-typed `age` sub-shape (narrow). The
    // cert attempts to certify ISSUER for the BROADER plain-`age` shape (dropping the
    // datatype constraint) — a broadening. It must contribute nothing.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let narrow_anchor = anchor(GOV, gov_pk, narrower_shape(AGE), RES);
    let broadening = signed_cert(
        GOV,
        &gov_sk,
        ISSUER,
        iss_pk,
        CertScope::Shape(predicate_shape(AGE)), // broader than the anchor
        NOW - 10,
        NOW + 10,
    );
    assert!(matches!(
        explain_edge(std::slice::from_ref(&narrow_anchor), &broadening, NOW, 1),
        Err(EdgeRejection::Broadening)
    ));
    assert_no_derivation(std::slice::from_ref(&narrow_anchor), std::slice::from_ref(&broadening));
}

#[test]
fn partial_shape_dropping_an_anchor_constraint_is_denied() {
    // The anchor requires the FULL predicate shape (targetSubjectsOf + property/minCount).
    // The cert presents a PARTIAL shape that keeps only the targetSubjectsOf triple (drops
    // the property constraint) — it admits MORE nodes than the anchor ⇒ a broadening. The
    // injective structural matcher must NOT report containment; the edge is denied.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    // A partial shape: same root, ONLY the targetSubjectsOf triple.
    let root = BlankNode::default();
    let partial = ShapeRef {
        root: Term::BlankNode(root.clone()),
        triples: vec![Triple::new(
            root,
            iri("http://www.w3.org/ns/shacl#targetSubjectsOf"),
            iri(AGE),
        )],
    };
    let cert = signed_cert(
        GOV,
        &gov_sk,
        ISSUER,
        iss_pk,
        CertScope::Shape(partial),
        NOW - 10,
        NOW + 10,
    );
    assert!(
        matches!(
            explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 1),
            Err(EdgeRejection::Broadening)
        ),
        "a cert shape that DROPS an anchor constraint admits more nodes ⇒ broadening ⇒ denied"
    );
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&cert));
}

#[test]
fn cert_shape_for_a_different_predicate_is_denied() {
    // The anchor is for `age`; the cert scopes to `name` — a DISJOINT (not narrower) shape.
    // It shares no constraint with the anchor, so it does not contain it ⇒ denied (a cert
    // cannot swing the certifier's `age` authority onto a `name` scope).
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let cert = signed_cert(
        GOV,
        &gov_sk,
        ISSUER,
        iss_pk,
        CertScope::Shape(predicate_shape(NAME)),
        NOW - 10,
        NOW + 10,
    );
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 1),
        Err(EdgeRejection::Broadening)
    ));
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&cert));
}

#[test]
fn any_service_cert_over_shape_scoped_anchor_never_broadens_shape() {
    // AnyService imposes no statement-type narrowing of its own — but it must NOT broaden a
    // shape-scoped anchor either: the derived shape must be the ANCHOR's (narrow) shape,
    // never an implicit "any predicate" widening.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let narrow_anchor = anchor(GOV, gov_pk, narrower_shape(AGE), RES);
    let cert = signed_cert(GOV, &gov_sk, ISSUER, iss_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    let d = explain_edge(std::slice::from_ref(&narrow_anchor), &cert, NOW, 1)
        .expect("AnyService inherits the anchor shape (a valid narrowing = identity)");
    assert_eq!(
        d.shape.triples.len(),
        narrow_anchor.shape.triples.len(),
        "AnyService derives the ANCHOR shape unchanged — never broadens it"
    );
}

#[test]
fn expired_certification_is_denied() {
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let expired = signed_cert(GOV, &gov_sk, ISSUER, iss_pk, CertScope::AnyService, NOW - 100, NOW - 50);
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &expired, NOW, 1),
        Err(EdgeRejection::OutOfWindow)
    ));
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&expired));
}

#[test]
fn not_yet_valid_certification_is_denied() {
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let future = signed_cert(GOV, &gov_sk, ISSUER, iss_pk, CertScope::AnyService, NOW + 50, NOW + 100);
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &future, NOW, 1),
        Err(EdgeRejection::OutOfWindow)
    ));
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&future));
}

#[test]
fn inverted_window_certification_is_denied() {
    // valid_until < valid_from — an ill-formed window (the shape a "revoked/missing positive
    // status" would present as: no covering window). Fail-closed.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let inverted = signed_cert(GOV, &gov_sk, ISSUER, iss_pk, CertScope::AnyService, NOW + 100, NOW - 100);
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &inverted, NOW, 1),
        Err(EdgeRejection::OutOfWindow)
    ));
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&inverted));
}

#[test]
fn self_certification_cycle_is_denied() {
    // GOV certifies GOV — a self-cert cycle. Even perfectly signed, it can confer no new
    // authority and is the shape a cycle would use to launder a broadening. Denied.
    let (gov_sk, gov_pk) = keypair(1);
    let (_atk_sk, atk_pk) = keypair(9);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let self_cert = signed_cert(GOV, &gov_sk, GOV, atk_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &self_cert, NOW, 1),
        Err(EdgeRejection::Cyclic)
    ));
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&self_cert));
}

#[test]
fn certified_issuer_already_anchoring_certifier_is_a_cycle_and_denied() {
    // Two anchors: GOV and ISSUER (ISSUER is ALREADY a direct anchor with key iss_pk). A
    // cert GOV → ISSUER (same key) would form a cycle / add nothing — denied as Cyclic so a
    // mutual-certification loop cannot amplify authority.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let gov_anchor = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let issuer_anchor = anchor(ISSUER, iss_pk, predicate_shape(NAME), RES);
    let cert = signed_cert(GOV, &gov_sk, ISSUER, iss_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    let anchors = vec![gov_anchor, issuer_anchor];
    assert!(matches!(
        explain_edge(&anchors, &cert, NOW, 1),
        Err(EdgeRejection::Cyclic)
    ));
    // Output is the two anchors, unchanged — the cert added nothing.
    let out = derive_effective_rules(&anchors, std::slice::from_ref(&cert), NOW, 1);
    assert_eq!(out.len(), 2);
}

#[test]
fn over_depth_is_denied_at_depth_zero() {
    // depth_bound = 0 short-circuits: NO edge is derived (OverDepth via explain_edge).
    let (anchor_rule, cert, _iss_pk) = good_cert();
    assert!(matches!(
        explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 0),
        Err(EdgeRejection::OverDepth)
    ));
    let out = derive_effective_rules(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&cert), NOW, 0);
    assert_eq!(out.len(), 1, "depth 0 ⇒ anchors only");
}

#[test]
fn derived_rule_is_never_reused_as_an_anchor_no_transitive_chain() {
    // `depth_bound = 1`: GOV certifies ISSUER (derived), and a second edge ISSUER certifies
    // THIRD. At depth 1 the ONLY anchors are the direct rules, so the ISSUER → THIRD edge
    // finds no anchor and contributes nothing — the closure does NOT transitively chain at
    // this bound. THIRD must NOT appear. (At `depth_bound = 2` it would — see
    // `two_hop_narrowing_chain_admits_at_depth_two` in the depth-N matrix below.)
    let (gov_sk, gov_pk) = keypair(1);
    let (iss_sk, iss_pk) = keypair(2);
    let (_third_sk, third_pk) = keypair(3);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let gov_to_issuer = signed_cert(GOV, &gov_sk, ISSUER, iss_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    // ISSUER signs an edge to a THIRD party — but ISSUER is not a direct anchor.
    let issuer_to_third = signed_cert(
        ISSUER,
        &iss_sk,
        "https://third.ex",
        third_pk,
        CertScope::AnyService,
        NOW - 10,
        NOW + 10,
    );
    let out = derive_effective_rules(
        std::slice::from_ref(&anchor_rule),
        &[gov_to_issuer, issuer_to_third],
        NOW,
        1,
    );
    // anchor + ISSUER only — THIRD is NOT derived (depth-1; no transitive chaining).
    assert_eq!(out.len(), 2, "only the direct GOV→ISSUER edge derives; ISSUER→THIRD does not");
    assert!(out.iter().all(|r| r.source.as_str() != "https://third.ex"));
}

#[test]
fn attribute_scoped_issuer_cannot_meta_escalate_to_certify_another_issuer() {
    // Meta-scope escalation: an issuer anchored ONLY for an attribute shape (`age`) is NOT a
    // framework operator and holds no authority to certify OTHER issuers OUTSIDE that shape.
    // This is the shape-ceiling property tested from the escalation angle, and it is what
    // holds the depth-N closure together at every hop: an ISSUER anchored for `age` presents
    // a cert ISSUER → ATTACKER.
    // ISSUER *is* a direct anchor here (anchored for age) — but the derived ATTACKER rule can
    // only NARROW ISSUER's `age` authority, never gain a broader/meta authority. An attempt
    // to certify ATTACKER for a DIFFERENT predicate (`name`, outside ISSUER's `age` anchor)
    // is a broadening and is denied.
    let (iss_sk, iss_pk) = keypair(2);
    let (_atk_sk, atk_pk) = keypair(8);
    // ISSUER anchored ONLY for `age`.
    let issuer_anchor = anchor(ISSUER, iss_pk, predicate_shape(AGE), RES);
    // ISSUER tries to certify ATTACKER for `name` — outside ISSUER's own `age` scope.
    let escalation = signed_cert(
        ISSUER,
        &iss_sk,
        "https://attacker.ex",
        atk_pk,
        CertScope::Shape(predicate_shape(NAME)),
        NOW - 10,
        NOW + 10,
    );
    assert!(
        matches!(
            explain_edge(std::slice::from_ref(&issuer_anchor), &escalation, NOW, 1),
            Err(EdgeRejection::Broadening)
        ),
        "certifying an issuer for a predicate OUTSIDE the certifier's own scope is a broadening"
    );
    assert_no_derivation(std::slice::from_ref(&issuer_anchor), std::slice::from_ref(&escalation));
}

#[test]
fn multi_anchor_certifier_narrowed_by_the_matching_anchor() {
    // The certifier is anchored for BOTH `age` and `name`. A cert scoped to the `name`
    // shape must narrow the `name` anchor (not be wrongly denied because `age` happened to
    // be listed first). Completeness without weakening the invariant: the chosen anchor is
    // always genuine pod authority.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let age_anchor = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let name_anchor = anchor(GOV, gov_pk, predicate_shape(NAME), RES);
    let anchors = vec![age_anchor, name_anchor];
    // Cert scoped to `name` (a narrowing of the `name` anchor).
    let cert = signed_cert(
        GOV,
        &gov_sk,
        ISSUER,
        iss_pk,
        CertScope::Shape(predicate_shape(NAME)),
        NOW - 10,
        NOW + 10,
    );
    let d = explain_edge(&anchors, &cert, NOW, 1)
        .expect("the `name`-scoped cert narrows the `name` anchor");
    assert_eq!(d.source.as_str(), ISSUER);
    // The full closure derives the ISSUER rule (2 anchors + 1 derived).
    let out = derive_effective_rules(&anchors, std::slice::from_ref(&cert), NOW, 1);
    assert_eq!(out.len(), 3);
}

#[test]
fn derived_freshness_is_capped_by_the_certification_window() {
    // The anchor is fresh-within 30d, but the cert window closes in 100s. The derived rule's
    // freshness must be the SMALLER (100s) — a cert can only shrink admitted staleness.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES); // FRESH = 30d
    let short = signed_cert(GOV, &gov_sk, ISSUER, iss_pk, CertScope::AnyService, NOW - 10, NOW + 100);
    let d = explain_edge(std::slice::from_ref(&anchor_rule), &short, NOW, 1)
        .expect("in-window, signed, narrowing edge is admitted");
    assert!(
        d.fresh_within_secs <= 100,
        "derived freshness capped by the (narrow) certification window, got {}",
        d.fresh_within_secs
    );
    assert!(d.fresh_within_secs <= anchor_rule.fresh_within_secs);
}

// ── sq-pfae.15 REVIEW FIX: target-set contravariance (additive-target broadening) ──────

#[test]
fn additive_target_predicate_cert_is_denied_as_broadening() {
    // THE escalation the escalated review found. The certifier is anchored ONLY for `age`.
    // The cert scope shape = the `age` predicate-shape PLUS an extra
    // `root sh:targetSubjectsOf schema:email` — an injective structural SUPERSET of the anchor
    // shape. A uniform "more edges ⇒ narrower" matcher wrongly reports this as a narrowing;
    // but SHACL selects focus nodes by the UNION of target predicates, so the extra target
    // WIDENS the admitted set (subjects of `age` OR `email`). The certified issuer would gain
    // authority over `email` the certifier never held — privilege escalation. Selection vocab
    // is CONTRAVARIANT: the cert must be denied as `Broadening`.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let email = "https://schema.org/email";
    let broadening = signed_cert(
        GOV,
        &gov_sk,
        ISSUER,
        iss_pk,
        CertScope::Shape(additive_target_shape(AGE, email)),
        NOW - 10,
        NOW + 10,
    );
    assert!(
        matches!(
            explain_edge(std::slice::from_ref(&anchor_rule), &broadening, NOW, 1),
            Err(EdgeRejection::Broadening)
        ),
        "an ADDITIVE target predicate WIDENS the admitted set ⇒ broadening ⇒ denied"
    );
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&broadening));
}

#[test]
fn every_derived_rule_target_set_is_subset_of_the_anchor_target_set() {
    // The actual CONSUMED-BY-ADMISSION invariant (not merely the matcher's internal verdict):
    // the admission gate (`admit::shape_targets_subject`) selects focus nodes by the UNION of a
    // rule shape's `sh:targetSubjectsOf` predicates, so a derived rule that targets a predicate
    // OUTSIDE the certifier's anchor is a live escalation regardless of what the matcher
    // reported. Drive a mix of edges (a genuine narrowing, an additive-target broadening, an
    // AnyService edge) and assert EVERY derived rule's target-predicate set ⊆ its certifier
    // anchor's. The anchor is `age` only, so the derived target set must be a subset of {age}.
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let (_iss2_sk, iss2_pk) = keypair(4);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let anchor_targets = target_predicate_set(&anchor_rule.shape);

    let email = "https://schema.org/email";
    let broadening = signed_cert(
        GOV, &gov_sk, ISSUER, iss_pk,
        CertScope::Shape(additive_target_shape(AGE, email)),
        NOW - 10, NOW + 10,
    );
    let narrowing = signed_cert(
        GOV, &gov_sk, "https://issuer.example/two", iss2_pk,
        CertScope::Shape(narrower_shape(AGE)),
        NOW - 10, NOW + 10,
    );
    let anyservice = signed_cert(
        GOV, &gov_sk, "https://issuer.example/three", keypair(5).1,
        CertScope::AnyService, NOW - 10, NOW + 10,
    );

    let out = derive_effective_rules(
        std::slice::from_ref(&anchor_rule),
        &[broadening, narrowing, anyservice],
        NOW,
        1,
    );
    // The broadening edge must contribute nothing; the two legitimate narrowings do. Anchor +
    // (narrowing, anyservice) = 3 rules; the additive-target broadening is dropped.
    assert_eq!(out.len(), 3, "additive-target broadening dropped; two narrowings derived");
    // EVERY derived rule (index ≥ 1) must target a SUBSET of the anchor's target predicates.
    for derived in &out[1..] {
        let derived_targets = target_predicate_set(&derived.shape);
        assert!(
            derived_targets.is_subset(&anchor_targets),
            "derived rule for {} targets {:?} ⊄ anchor targets {:?} — escalation",
            derived.source.as_str(),
            derived_targets,
            anchor_targets
        );
    }
    // And no derived rule may target `email` (the predicate the broadening tried to smuggle).
    assert!(
        out[1..].iter().all(|r| !target_predicate_set(&r.shape).contains(email)),
        "no derived rule may target the smuggled `email` predicate"
    );
}

#[test]
fn un_modelled_selection_predicate_cert_fails_closed() {
    // Fail-closed on an UN-MODELLED selection predicate. The cert scope shape = the `age`
    // predicate-shape PLUS a `root sh:targetObjectsOf schema:email` triple — a SHACL target
    // predicate the matcher recognises by the `sh:target*` prefix but which the desugared
    // anchor shape does not carry. Because it is a selection predicate the anchor lacks, it is
    // a broadening (and any future/unknown `sh:target…` is likewise treated as selection ⇒
    // fail-closed, never silently admitted as a conformance constraint).
    let (gov_sk, gov_pk) = keypair(1);
    let (_iss_sk, iss_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    // Add an unrelated SHACL target predicate at the root.
    let mut shape = predicate_shape(AGE);
    if let Term::BlankNode(root) = shape.root.clone() {
        shape.triples.push(Triple::new(
            root,
            iri("http://www.w3.org/ns/shacl#targetObjectsOf"),
            iri("https://schema.org/email"),
        ));
    }
    let cert = signed_cert(
        GOV, &gov_sk, ISSUER, iss_pk,
        CertScope::Shape(shape),
        NOW - 10, NOW + 10,
    );
    assert!(
        matches!(
            explain_edge(std::slice::from_ref(&anchor_rule), &cert, NOW, 1),
            Err(EdgeRejection::Broadening)
        ),
        "an un-modelled / extra SHACL target predicate ⇒ fail-closed (Broadening), never admitted"
    );
    assert_no_derivation(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&cert));
}

// ── DEPTH-N matrix: framework-of-frameworks chains (sq-13096) ──────────────────
//
// The v1 closure was DEPTH-1 ONLY: a derived rule was never re-used as an anchor, so
// `GOV certifies MID, MID certifies LEAF` derived nothing for LEAF. `derive_effective_rules`
// now iterates up to `depth_bound` hops. The HARD invariant is unchanged and now has to hold
// at EVERY depth: a broadening at ANY hop is a privilege escalation, and a cycle must never
// amplify across rounds. Each case below drives one multi-hop shape and asserts the outcome.

const MID: &str = "https://mid.example/registrar"; // the intermediate framework in a chain
const LEAF: &str = "https://leaf.example/issuer"; // the issuer at the end of a chain
/// `trustx:certifies` — the predicate a REGISTRAR-scoped shape must select for its holder to
/// act as a certification edge source at a deeper hop (the meta-scope gate).
const TRUSTX_CERTIFIES: &str = "https://sparq.dev/ns/trust#certifies";

/// A REGISTRAR shape: `predicate_shape(p)` PLUS a root `sh:targetSubjectsOf trustx:certifies`
/// selection edge, so it selects certification statements as well as `p` statements. A rule
/// carrying it has an incoming certification scope that covers certification-issuing itself,
/// which is what the meta-scope gate requires of any DERIVED edge source. A plain
/// attribute shape is an ISSUER scope and confers no authority to certify.
fn registrar_shape(p: &str) -> ShapeRef {
    additive_target_shape(p, TRUSTX_CERTIFIES)
}

/// The root anchor + the first hop `GOV → MID` (AnyService, in-window, GOV-signed). GOV is
/// anchored as a REGISTRAR for `age`, so MID inherits that shape (certification authority
/// included), GOV's resource scope, and a freshness <= GOV's.
fn root_and_first_hop() -> (TrustRule, Certification, SecretKey, PublicKey) {
    let (gov_sk, gov_pk) = keypair(1);
    let (mid_sk, mid_pk) = keypair(2);
    let anchor_rule = anchor(GOV, gov_pk, registrar_shape(AGE), RES);
    let hop1 = signed_cert(
        GOV,
        &gov_sk,
        MID,
        mid_pk,
        CertScope::AnyService,
        NOW - 86400,
        NOW + 86400,
    );
    (anchor_rule, hop1, mid_sk, mid_pk)
}

#[test]
fn two_hop_narrowing_chain_admits_at_depth_two() {
    // The legitimate framework-of-frameworks case. GOV (anchored for `age`) certifies MID;
    // MID certifies LEAF, NARROWING to the integer-typed `age` sub-shape. At depth 2 LEAF is
    // derived, and its authority telescopes: LEAF ⊆ MID ⊆ GOV on every axis.
    let (anchor_rule, hop1, mid_sk, _mid_pk) = root_and_first_hop();
    let (_leaf_sk, leaf_pk) = keypair(3);
    let hop2 = signed_cert(
        MID,
        &mid_sk,
        LEAF,
        leaf_pk,
        CertScope::Shape(narrower_shape(AGE)),
        NOW - 10,
        NOW + 10,
    );
    let out = derive_effective_rules(
        std::slice::from_ref(&anchor_rule),
        &[hop1, hop2],
        NOW,
        2,
    );
    assert_eq!(out.len(), 3, "anchor + MID (hop 1) + LEAF (hop 2)");
    assert_eq!(out[1].source.as_str(), MID, "shallowest hop first");
    let leaf = &out[2];
    assert_eq!(leaf.source.as_str(), LEAF);
    assert_eq!(
        public_key_to_hex(&leaf.issuer_key),
        public_key_to_hex(&leaf_pk),
        "the derived rule binds the key MID's signature attested"
    );
    // ATTENUATION AT EVERY HOP — the telescoping ceiling.
    assert_eq!(
        leaf.scope.as_str(),
        RES,
        "resource scope stays the ROOT anchor's ceiling across both hops"
    );
    assert!(
        leaf.fresh_within_secs <= out[1].fresh_within_secs
            && out[1].fresh_within_secs <= anchor_rule.fresh_within_secs,
        "freshness narrows monotonically: LEAF <= MID <= GOV"
    );
    // The consumed-by-admission invariant: the target-predicate set only ever SHRINKS.
    let root_targets = target_predicate_set(&anchor_rule.shape);
    let mid_targets = target_predicate_set(&out[1].shape);
    let leaf_targets = target_predicate_set(&leaf.shape);
    assert!(mid_targets.is_subset(&root_targets), "hop 1 targets ⊆ anchor targets");
    assert!(leaf_targets.is_subset(&mid_targets), "hop 2 targets ⊆ hop 1 targets");
    assert!(
        leaf_targets.len() < mid_targets.len(),
        "hop 2 carried a STRICTLY tighter selection than hop 1 (the narrowing it signed)"
    );
    // The narrowing it signed also added a conformance constraint hop 1 did not carry.
    let datatype = "http://www.w3.org/ns/shacl#datatype";
    assert!(
        leaf.shape.triples.iter().any(|t| t.predicate.as_str() == datatype)
            && !out[1].shape.triples.iter().any(|t| t.predicate.as_str() == datatype),
        "hop 2's derived shape is the tighter cert shape, not hop 1's inherited one"
    );
    // META-SCOPE: LEAF's narrowed shape no longer SELECTS certification statements, so LEAF
    // is an issuer, not a registrar — the chain terminates here even at a deeper bound.
    assert!(
        !leaf_targets.contains(TRUSTX_CERTIFIES) && mid_targets.contains(TRUSTX_CERTIFIES),
        "MID may certify (registrar scope); LEAF, narrowed to `age` alone, may not"
    );
}

#[test]
fn two_hop_chain_broadening_at_the_second_hop_is_denied() {
    // THE ESCALATION SHAPE. Hop 1 is legitimate (GOV → MID, AnyService ⇒ MID inherits `age`).
    // Hop 2 tries to certify LEAF for `name` — a predicate OUTSIDE MID's own derived `age`
    // ceiling. Attenuation is re-derived per hop, so hop 2 is a broadening and LEAF must
    // never appear, at ANY depth bound.
    let (anchor_rule, hop1, mid_sk, _mid_pk) = root_and_first_hop();
    let (_leaf_sk, leaf_pk) = keypair(3);
    let broadening_hop2 = signed_cert(
        MID,
        &mid_sk,
        LEAF,
        leaf_pk,
        CertScope::Shape(predicate_shape(NAME)),
        NOW - 10,
        NOW + 10,
    );
    for depth in [2u32, 3, 16] {
        let out = derive_effective_rules(
            std::slice::from_ref(&anchor_rule),
            &[hop1.clone(), broadening_hop2.clone()],
            NOW,
            depth,
        );
        assert_eq!(
            out.len(),
            2,
            "depth {depth}: only the legitimate hop 1 derives; the broadening hop 2 contributes nothing"
        );
        assert!(
            out.iter().all(|r| r.source.as_str() != LEAF),
            "depth {depth}: a broadening at hop 2 must never grant LEAF authority"
        );
    }
    // The explained path names the gate. Hop 2 is explained against hop 1's own output —
    // exactly the anchor set the closure held when it evaluated that hop.
    let after_hop1 =
        derive_effective_rules(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&hop1), NOW, 1);
    assert!(
        matches!(
            explain_edge(&after_hop1, &broadening_hop2, NOW, 2),
            Err(EdgeRejection::Broadening)
        ),
        "the second hop is denied AT THAT HOP as Broadening"
    );
}

#[test]
fn two_hop_chain_with_an_additive_target_at_the_second_hop_is_denied() {
    // The contravariant-selection escalation, moved to hop 2: MID inherits GOV's `age`
    // targets, then tries to certify LEAF for a shape that keeps every `age` constraint but
    // ADDS `sh:targetSubjectsOf schema:email`. It is an injective structural superset yet
    // selects strictly MORE nodes — a broadening that must be denied at the deeper hop too.
    let (anchor_rule, hop1, mid_sk, _mid_pk) = root_and_first_hop();
    let (_leaf_sk, leaf_pk) = keypair(3);
    let smuggle = signed_cert(
        MID,
        &mid_sk,
        LEAF,
        leaf_pk,
        CertScope::Shape(additive_target_shape(AGE, "https://schema.org/email")),
        NOW - 10,
        NOW + 10,
    );
    let out = derive_effective_rules(
        std::slice::from_ref(&anchor_rule),
        &[hop1, smuggle],
        NOW,
        2,
    );
    assert_eq!(out.len(), 2, "the additive-target hop 2 contributes nothing");
    let root_targets = target_predicate_set(&anchor_rule.shape);
    for rule in &out {
        assert!(
            target_predicate_set(&rule.shape).is_subset(&root_targets),
            "no rule at any depth may target a predicate the root anchor never selected"
        );
    }
}

#[test]
fn cross_round_cycle_derives_nothing_new() {
    // A → B → A. Hop 1 derives MID. Hop 2 would close the loop back onto GOV, which already
    // holds a matching-key rule, so the cycle gate denies it and the closure converges: the
    // output is the anchor + MID at EVERY depth. A cycle must never re-enter and amplify.
    let (anchor_rule, hop1, mid_sk, _mid_pk) = root_and_first_hop();
    let (_gov_sk, gov_pk) = keypair(1);
    let back_edge = signed_cert(
        MID,
        &mid_sk,
        GOV,
        gov_pk,
        CertScope::AnyService,
        NOW - 10,
        NOW + 10,
    );
    for depth in [1u32, 2, 5, 32] {
        let out = derive_effective_rules(
            std::slice::from_ref(&anchor_rule),
            &[hop1.clone(), back_edge.clone()],
            NOW,
            depth,
        );
        assert_eq!(
            out.len(),
            2,
            "depth {depth}: the cycle adds nothing — anchor + MID only"
        );
        assert_eq!(out[1].source.as_str(), MID);
    }
    // And the gate that denied it is named, evaluated against the closure state at that hop.
    let after_hop1 =
        derive_effective_rules(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&hop1), NOW, 1);
    assert!(
        matches!(
            explain_edge(&after_hop1, &back_edge, NOW, 2),
            Err(EdgeRejection::Cyclic)
        ),
        "the back edge of a cross-round cycle is denied as Cyclic"
    );
}

#[test]
fn over_depth_chain_denies_the_tail() {
    // A THREE-hop chain GOV → MID → LEAF → TAIL evaluated at each bound. The closure admits
    // exactly `depth_bound` hops and leaves the tail beyond it UNDERIVED — never approximated,
    // never admitted "because the prefix was fine".
    const TAIL: &str = "https://tail.example/issuer";
    let (anchor_rule, hop1, mid_sk, _mid_pk) = root_and_first_hop();
    let (leaf_sk, leaf_pk) = keypair(3);
    let (_tail_sk, tail_pk) = keypair(4);
    let hop2 = signed_cert(MID, &mid_sk, LEAF, leaf_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    let hop3 = signed_cert(LEAF, &leaf_sk, TAIL, tail_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    let chain = [hop1, hop2, hop3];

    // depth 1 ⇒ MID only; depth 2 ⇒ + LEAF; depth 3 ⇒ + TAIL; deeper ⇒ no more.
    let expected: [(u32, &[&str]); 5] = [
        (1, &[MID]),
        (2, &[MID, LEAF]),
        (3, &[MID, LEAF, TAIL]),
        (4, &[MID, LEAF, TAIL]),
        (9, &[MID, LEAF, TAIL]),
    ];
    for (depth, want) in expected {
        let out = derive_effective_rules(std::slice::from_ref(&anchor_rule), &chain, NOW, depth);
        let derived: Vec<&str> = out[1..].iter().map(|r| r.source.as_str()).collect();
        assert_eq!(derived, want, "depth {depth}: the tail beyond the bound stays underived");
        // The no-amplification bound holds at every depth: one rule per edge, at most.
        assert!(derived.len() <= chain.len());
    }
    // Attenuation still telescopes across all three hops (the resource ceiling is the root's).
    let full = derive_effective_rules(std::slice::from_ref(&anchor_rule), &chain, NOW, 3);
    assert!(full.iter().all(|r| r.scope.as_str() == RES));
    let root_targets = target_predicate_set(&anchor_rule.shape);
    assert!(full
        .iter()
        .all(|r| target_predicate_set(&r.shape).is_subset(&root_targets)));
}

#[test]
fn a_denied_first_hop_denies_the_whole_chain() {
    // Fail-closed composition: if hop 1 is denied (here: an EXPIRED GOV → MID edge), MID is
    // never derived, so hop 2 finds no anchor and LEAF is never derived either. A chain is
    // only as admissible as its weakest hop.
    let (gov_sk, gov_pk) = keypair(1);
    let (mid_sk, mid_pk) = keypair(2);
    let (_leaf_sk, leaf_pk) = keypair(3);
    let anchor_rule = anchor(GOV, gov_pk, registrar_shape(AGE), RES);
    let expired_hop1 = signed_cert(GOV, &gov_sk, MID, mid_pk, CertScope::AnyService, NOW - 100, NOW - 50);
    let good_hop2 = signed_cert(MID, &mid_sk, LEAF, leaf_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    let out = derive_effective_rules(
        std::slice::from_ref(&anchor_rule),
        &[expired_hop1, good_hop2],
        NOW,
        4,
    );
    assert_eq!(out.len(), 1, "a denied hop 1 leaves the whole chain underived");
}

#[test]
fn no_edge_ever_contributes_more_than_one_rule() {
    // The no-amplification bound, observed behaviourally: two certifiers both vouch for MID
    // at hop 1 (both derive, exactly as at depth-1 — the per-round snapshot keeps that
    // order-independent), and no edge fires a second time at a deeper hop, so the derived
    // count never exceeds the edge count however deep the bound. The mechanism that enforces
    // this is the CYCLE gate; the `(certifier, certified_issuer)` visited set states the same
    // bound redundantly (see the module docs — disabling it turns no test red).
    let (gov_sk, gov_pk) = keypair(1);
    let (other_sk, other_pk) = keypair(5);
    let (_mid_sk, mid_pk) = keypair(2);
    let gov_anchor = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let other_anchor = anchor("https://other.example/framework", other_pk, predicate_shape(AGE), RES);
    let anchors = vec![gov_anchor, other_anchor];
    let from_gov = signed_cert(GOV, &gov_sk, MID, mid_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    let from_other = signed_cert(
        "https://other.example/framework",
        &other_sk,
        MID,
        mid_pk,
        CertScope::AnyService,
        NOW - 10,
        NOW + 10,
    );
    let edges = [from_gov, from_other];
    for depth in [1u32, 2, 7] {
        let out = derive_effective_rules(&anchors, &edges, NOW, depth);
        assert_eq!(
            out.len(),
            anchors.len() + 2,
            "depth {depth}: both certifiers derive ONCE each — no depth-driven duplication"
        );
        assert!(out[2..].iter().all(|r| r.source.as_str() == MID));
    }
}

#[test]
fn meta_scope_escalation_an_attribute_issuer_cannot_certify_at_a_deeper_hop() {
    // THE META-SCOPE ESCALATION (`research/trust-graph-authorisation-2026-07.md` §2.4 rule 3,
    // which §2.5 made the stated precondition for depth > 1).
    //
    // GOV is anchored ONLY for `age` — an ISSUER scope, not a REGISTRAR scope. GOV certifies
    // MID for `age`: legitimate, and MID's rule derives. MID then certifies LEAF for `age` —
    // strictly INSIDE MID's own derived shape, in-window, correctly signed, so every
    // attenuation / signature / time gate PASSES. It must still be denied: MID was certified
    // to ISSUE age statements, never to CONFER age authority on anyone else. Admitting it
    // would let a chain of ordinary issuers launder pod authority onto an issuer neither GOV
    // nor the pod ever vouched for — a broadening in the META dimension, invisible to the
    // shape ceiling.
    let (gov_sk, gov_pk) = keypair(1);
    let (mid_sk, mid_pk) = keypair(2);
    let (_leaf_sk, leaf_pk) = keypair(3);
    // Attribute-only anchor — deliberately NOT `registrar_shape`.
    let issuer_only_anchor = anchor(GOV, gov_pk, predicate_shape(AGE), RES);
    let hop1 = signed_cert(GOV, &gov_sk, MID, mid_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    let hop2 = signed_cert(MID, &mid_sk, LEAF, leaf_pk, CertScope::AnyService, NOW - 10, NOW + 10);

    for depth in [2u32, 3, 16] {
        let out = derive_effective_rules(
            std::slice::from_ref(&issuer_only_anchor),
            &[hop1.clone(), hop2.clone()],
            NOW,
            depth,
        );
        assert_eq!(out.len(), 2, "depth {depth}: MID derives; LEAF must NOT");
        assert!(
            out.iter().all(|r| r.source.as_str() != LEAF),
            "depth {depth}: an attribute-scoped issuer must not confer authority on anyone"
        );
    }

    // The EXPLAINED path agrees with the fast path at the SAME hop — the meta-scope gate
    // lives in the one shared per-hop gate, so `explain_edge` cannot report a rule the
    // closure refused to derive. MID holds no CERTIFIER anchor at hop 2, hence `NoAnchor`.
    let after_hop1 = derive_effective_rules(
        std::slice::from_ref(&issuer_only_anchor),
        std::slice::from_ref(&hop1),
        NOW,
        1,
    );
    assert!(
        matches!(
            explain_edge(&after_hop1, &hop2, NOW, 2),
            Err(EdgeRejection::NoAnchor)
        ),
        "at hop 2 an attribute-scoped derived anchor is not a certifier ⇒ NoAnchor"
    );
    // Non-vacuity of that assertion: the SAME edge and SAME anchor set at hop 1 — i.e. asking
    // "what if the pod had anchored MID's rule DIRECTLY?" — is admitted. So `NoAnchor` above
    // is the meta-scope exemption boundary, not a broken fixture.
    assert!(
        explain_edge(&after_hop1, &hop2, NOW, 1).is_ok(),
        "hop 1 exempts DIRECT anchors from the meta-scope gate"
    );

    // CONTROL — the gate is denying on meta-scope, not on some unrelated defect of the
    // fixture. Swap ONLY the anchor shape for a registrar shape (so MID's incoming
    // certification scope now covers certification-issuing itself) and the very same hop 2 is
    // admitted.
    let registrar_anchor = anchor(GOV, gov_pk, registrar_shape(AGE), RES);
    let out = derive_effective_rules(
        std::slice::from_ref(&registrar_anchor),
        &[hop1, hop2],
        NOW,
        2,
    );
    assert_eq!(out.len(), 3, "a registrar-scoped chain admits the same hop 2");
    assert_eq!(out[2].source.as_str(), LEAF);
    // And even then LEAF stays inside the ROOT ceiling — the meta-scope gate widens nothing.
    assert!(target_predicate_set(&out[2].shape).is_subset(&target_predicate_set(&registrar_anchor.shape)));
}

#[test]
fn a_direct_anchor_may_certify_regardless_of_meta_scope() {
    // The meta-scope gate applies to DERIVED edge sources only. A DIRECT anchor is the pod's
    // own explicit decision about who it trusts to confer, so an attribute-scoped direct
    // anchor still certifies at hop 1 — which is exactly what keeps `depth_bound = 1`
    // byte-identical to the pre-sq-13096 closure. (The shape ceiling still binds it: see
    // `attribute_scoped_issuer_cannot_meta_escalate_to_certify_another_issuer`.)
    let (anchor_rule, cert, _iss_pk) = good_cert(); // attribute-only `age` anchor
    for depth in [1u32, 2, 4] {
        let out = derive_effective_rules(
            std::slice::from_ref(&anchor_rule),
            std::slice::from_ref(&cert),
            NOW,
            depth,
        );
        assert_eq!(out.len(), 2, "depth {depth}: the direct anchor still certifies");
        assert_eq!(out[1].source.as_str(), ISSUER);
    }
}

#[test]
fn an_older_round_certifier_cannot_re_admit_an_edge_at_a_later_hop() {
    // THE FRONTIER-vs-CLOSURE-SO-FAR question. `derive_effective_rules` evaluates hop k
    // against EXACTLY round k-1's output (the frontier) for GATE 2, but against the WHOLE
    // closure so far for GATE 1. `explain_edge` is handed one slice —
    // `derive_effective_rules(.., k - 1)` — which is exactly right for GATE 1 and a strict
    // SUPERSET for GATE 2: it also carries the direct anchors and every round older than
    // k-1. So the explained path offers GATE 2 candidate anchors the closure could NOT use
    // at that hop. This test pins that the over-approximation cannot turn a denial into an
    // admission, on the two shapes where it could — a certification-capable DIRECT anchor,
    // and a registrar derived in an OLDER round — and that GATE 1 (which reads the whole
    // closure so far) is the interlock that makes it so.
    const TAIL_A: &str = "https://tail-a.example/issuer"; // certified by GOV   (fires at hop 1)
    const TAIL_B: &str = "https://tail-b.example/issuer"; // certified by MID   (fires at hop 2)
    const TAIL_C: &str = "https://tail-c.example/issuer"; // certified by MID, BROADENING

    let (anchor_rule, hop1, mid_sk, _mid_pk) = root_and_first_hop();
    let (_leaf_sk, leaf_pk) = keypair(3);
    let (_a_sk, a_pk) = keypair(4);
    let (_b_sk, b_pk) = keypair(5);
    let (_c_sk, c_pk) = keypair(6);
    let (gov_sk, _gov_pk) = keypair(1);

    let hop2 = signed_cert(MID, &mid_sk, LEAF, leaf_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    // GOV is a DIRECT anchor, so this edge is admitted at hop 1 and GOV is never in the
    // frontier again — yet it stays in the cumulative set `explain_edge` is handed.
    let gov_tail_a =
        signed_cert(GOV, &gov_sk, TAIL_A, a_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    // MID is derived in round 1, so it is the frontier at hop 2 ONLY.
    let mid_tail_b =
        signed_cert(MID, &mid_sk, TAIL_B, b_pk, CertScope::AnyService, NOW - 10, NOW + 10);
    // …and this one broadens past MID's `age` ceiling, so it is denied at hop 2 and MID is
    // gone from the frontier from hop 3 on. It must stay denied.
    let mid_tail_c = signed_cert(
        MID,
        &mid_sk,
        TAIL_C,
        c_pk,
        CertScope::Shape(predicate_shape(NAME)),
        NOW - 10,
        NOW + 10,
    );
    let edges = [
        hop1.clone(),
        hop2,
        gov_tail_a.clone(),
        mid_tail_b.clone(),
        mid_tail_c.clone(),
    ];
    let anchors = std::slice::from_ref(&anchor_rule);

    // The closure converges after round 2: GOV→{MID, TAIL_A} at hop 1, MID→{LEAF, TAIL_B} at
    // hop 2, nothing at hop 3 — and deepening the bound never re-admits an older certifier.
    let after_hop2 = derive_effective_rules(anchors, &edges, NOW, 2);
    let sources_at_2: Vec<&str> = after_hop2.iter().map(|r| r.source.as_str()).collect();
    assert_eq!(sources_at_2, [GOV, MID, TAIL_A, LEAF, TAIL_B], "the depth-2 closure");
    for depth in [3u32, 4, 8] {
        let deeper = derive_effective_rules(anchors, &edges, NOW, depth);
        let sources: Vec<&str> = deeper.iter().map(|r| r.source.as_str()).collect();
        assert_eq!(
            sources, sources_at_2,
            "depth {depth}: no older-round certifier may derive anything at a later hop"
        );
        assert!(
            deeper.iter().all(|r| r.source.as_str() != TAIL_C),
            "depth {depth}: a broadening stays denied once its certifier leaves the frontier"
        );
    }

    // THE AGREEMENT PROPERTY AT hop 3 — every edge whose certifier is out of the frontier is
    // denied by the explained path too, even though the cumulative set still offers that
    // certifier as a candidate anchor.
    for (cert, who, want) in [
        // GOV is a DIRECT anchor in the set handed to `explain_edge`, and still
        // certification-capable — but the edge already fired at hop 1, so TAIL_A holds a
        // matching-key rule and GATE 1 denies the re-attempt.
        (&gov_tail_a, TAIL_A, EdgeRejection::Cyclic),
        // Same interlock one round deeper: MID derived at round 1, fired at hop 2.
        (&mid_tail_b, TAIL_B, EdgeRejection::Cyclic),
        // The edge that NEVER fired: MID is offered to GATE 2 from round 1, so the explained
        // path reaches GATE 5 — and the attenuation ceiling denies it there. The closure
        // denies it at hop 3 earlier still (MID is not in the frontier); both contribute
        // NOTHING, which is the property that matters.
        (&mid_tail_c, TAIL_C, EdgeRejection::Broadening),
    ] {
        // (`TrustRule` has no `PartialEq`, so compare the rejection side.)
        assert_eq!(
            explain_edge(&after_hop2, cert, NOW, 3).err(),
            Some(want),
            "hop 3: the edge certifying {who} must be denied, not laundered through an \
             out-of-frontier anchor"
        );
    }

    // NON-VACUITY — the denials above are the hop interlock, not a broken fixture: the very
    // same edges ARE admitted at the hop whose frontier really did hold their certifier.
    assert!(
        explain_edge(anchors, &gov_tail_a, NOW, 1).is_ok(),
        "GOV certifies TAIL_A at hop 1"
    );
    let after_hop1 = derive_effective_rules(anchors, &edges, NOW, 1);
    assert!(
        explain_edge(&after_hop1, &mid_tail_b, NOW, 2).is_ok(),
        "MID certifies TAIL_B at hop 2"
    );
}
