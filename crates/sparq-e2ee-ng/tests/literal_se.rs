//! Behavioral tests for the opt-in **Profile SE** encrypted-literal codec
//! (`se` feature, `research/e2ee-queryable-options.md` §3.c): seal/open
//! round-trips, fail-closed context binding, domain separation from block
//! envelopes, canonical-lexical parsing, padding buckets, and equality tags.
//!
//! Nothing here asserts a security *property* — these are behavioural checks on
//! a research-grade, externally unaudited construction (`sq-qhy4`).
#![cfg(feature = "se")]

use proptest::prelude::*;
use sparq_e2ee_ng::envelope::{open_block, seal_block_random, BlockContext, ObjectKind};
use sparq_e2ee_ng::error::Error;
use sparq_e2ee_ng::ids::{BranchId, Epoch, ObjectId, RepoId, Secret32};
use sparq_e2ee_ng::keyschedule::value_key;
use sparq_e2ee_ng::literal::{
    eqtag_from_lexical, eqtag_to_lexical, equality_tag, open_literal, seal_literal, tags_equal,
    EncryptedLiteral, ValueContext, EQTAG_LEN, SE_ENC_DATATYPE, SE_EQTAG_DATATYPE, SE_PAD_CLASSES,
};

const FOAF_NAME: &str = "http://xmlns.com/foaf/0.1/name";
const FOAF_AGE: &str = "http://xmlns.com/foaf/0.1/age";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const XSD_DATE: &str = "http://www.w3.org/2001/XMLSchema#date";
const ALICE: &str = "https://alice.example/card#me";
const BOB: &str = "https://bob.example/card#me";
const GRAPH_A: &str = "https://alice.example/health";

fn ctx<'a>() -> ValueContext<'a> {
    ValueContext {
        predicate: FOAF_NAME,
        graph: None,
        subject: Some(ALICE),
    }
}

// ============================================================================
// Round trip
// ============================================================================

#[test]
fn seal_open_round_trips_several_datatypes() {
    let dek = Secret32::random();
    let c = ctx();
    for (lexical, datatype) in [
        ("Alice", XSD_STRING),
        ("42", XSD_INTEGER),
        ("2026-07-26", XSD_DATE),
        ("", XSD_STRING),                      // empty lexical form
        ("線 — Ωμέγα — 🔐 naïve", XSD_STRING), // non-ASCII, multi-byte UTF-8
    ] {
        let lit = seal_literal(&dek, &c, lexical, datatype).expect("seal");
        assert_eq!(lit.datatype(), SE_ENC_DATATYPE);
        let got = open_literal(&dek, &c, &lit).expect("open");
        assert_eq!(got, (lexical.to_string(), datatype.to_string()));
    }
}

#[test]
fn lexical_form_round_trips_exactly() {
    let dek = Secret32::random();
    let c = ctx();
    let lit = seal_literal(&dek, &c, "Alice", XSD_STRING).expect("seal");
    let text = lit.to_lexical();
    assert!(
        text.starts_with("se0."),
        "canonical lexical form is se0.<nonce>.<ct>"
    );
    let parsed = EncryptedLiteral::from_lexical(&text).expect("parse own output");
    assert_eq!(parsed, lit);
    assert_eq!(parsed.to_lexical(), text);
    // The hex halves match the reference `hex` crate (the crate's own encoder is
    // hand-rolled to avoid a runtime dependency).
    let mut fields = text.split('.');
    assert_eq!(fields.next(), Some("se0"));
    assert_eq!(
        fields.next().map(str::to_string),
        Some(hex::encode(lit.nonce))
    );
    assert_eq!(
        fields.next().map(str::to_string),
        Some(hex::encode(&lit.ciphertext))
    );
    assert_eq!(fields.next(), None);
}

#[test]
fn nonce_is_randomized_not_deterministic() {
    // Sealing the SAME value twice under the SAME key/context must NOT produce
    // the same ciphertext: SE has no convergent/deterministic default.
    let dek = Secret32::random();
    let c = ctx();
    let a = seal_literal(&dek, &c, "Alice", XSD_STRING).expect("seal");
    let b = seal_literal(&dek, &c, "Alice", XSD_STRING).expect("seal");
    assert_ne!(a.nonce, b.nonce);
    assert_ne!(a.ciphertext, b.ciphertext);
    assert_ne!(a.to_lexical(), b.to_lexical());
}

// ============================================================================
// Fail-closed context binding
// ============================================================================

#[test]
fn wrong_predicate_fails_closed() {
    let dek = Secret32::random();
    let lit = seal_literal(&dek, &ctx(), "Alice", XSD_STRING).expect("seal");
    let wrong = ValueContext {
        predicate: FOAF_AGE,
        graph: None,
        subject: Some(ALICE),
    };
    assert_eq!(open_literal(&dek, &wrong, &lit), Err(Error::Decrypt));
}

#[test]
fn wrong_graph_fails_closed() {
    let dek = Secret32::random();
    let lit = seal_literal(&dek, &ctx(), "Alice", XSD_STRING).expect("seal");
    // default graph -> named graph
    let named = ValueContext {
        predicate: FOAF_NAME,
        graph: Some(GRAPH_A),
        subject: Some(ALICE),
    };
    assert_eq!(open_literal(&dek, &named, &lit), Err(Error::Decrypt));
    // named graph -> a *different* named graph, and -> the default graph
    let in_a = seal_literal(&dek, &named, "Alice", XSD_STRING).expect("seal");
    let other = ValueContext {
        graph: Some("https://alice.example/other"),
        ..named
    };
    assert_eq!(open_literal(&dek, &other, &in_a), Err(Error::Decrypt));
    assert_eq!(open_literal(&dek, &ctx(), &in_a), Err(Error::Decrypt));
    // `Some("")` must not collide with `None`.
    let empty = ValueContext {
        graph: Some(""),
        ..ctx()
    };
    assert_eq!(open_literal(&dek, &empty, &lit), Err(Error::Decrypt));
}

#[test]
fn wrong_subject_fails_closed_when_the_subject_is_pinned() {
    let dek = Secret32::random();
    let lit = seal_literal(&dek, &ctx(), "Alice", XSD_STRING).expect("seal");
    let relocated = ValueContext {
        subject: Some(BOB),
        ..ctx()
    };
    assert_eq!(open_literal(&dek, &relocated, &lit), Err(Error::Decrypt));
    // Dropping the pin is also a different binding, not a wildcard.
    let unpinned = ValueContext {
        subject: None,
        ..ctx()
    };
    assert_eq!(open_literal(&dek, &unpinned, &lit), Err(Error::Decrypt));
}

#[test]
fn unpinned_subject_permits_relocation_the_disclosed_integrity_limit() {
    // The DISCLOSED limit of `subject: None` (see `ValueContext::subject`): with
    // no subject bound, the ciphertext carries nothing tying it to a subject, so
    // a server may move it between subjects undetected. This test pins that
    // documented weakness so it cannot be silently "fixed" in the docs only.
    let dek = Secret32::random();
    let sealed_for_alice = ValueContext {
        predicate: FOAF_NAME,
        graph: None,
        subject: None,
    };
    let lit = seal_literal(&dek, &sealed_for_alice, "Alice", XSD_STRING).expect("seal");
    let read_as_bobs = ValueContext {
        predicate: FOAF_NAME,
        graph: None,
        subject: None,
    };
    assert_eq!(
        open_literal(&dek, &read_as_bobs, &lit).expect("opens").0,
        "Alice",
        "with no subject bound the same ciphertext opens at any subject"
    );
}

#[test]
fn wrong_dek_fails_closed() {
    let dek = Secret32::random();
    let other = Secret32::random();
    let lit = seal_literal(&dek, &ctx(), "Alice", XSD_STRING).expect("seal");
    assert_eq!(open_literal(&other, &ctx(), &lit), Err(Error::Decrypt));
}

#[test]
fn tampered_ciphertext_fails_closed() {
    let dek = Secret32::random();
    let mut lit = seal_literal(&dek, &ctx(), "Alice", XSD_STRING).expect("seal");
    lit.ciphertext[0] ^= 0x01;
    assert_eq!(open_literal(&dek, &ctx(), &lit), Err(Error::Decrypt));
}

// ============================================================================
// Domain separation: a block envelope is not a literal (and vice versa)
// ============================================================================

#[test]
fn a_sealed_block_does_not_open_as_a_literal() {
    let secret = Secret32::random();
    let bctx = BlockContext {
        repo: RepoId::random(),
        branch: BranchId::random(),
        epoch: Epoch(0),
        kind: ObjectKind::Operation,
    };
    let env = seal_block_random(&secret, &bctx, &ObjectId::random(), 0, 1, b"quads").expect("seal");
    // The block's smallest pad class (256) is also an SE class, so the block
    // ciphertext *parses* as a well-formed SE literal — and must still fail to
    // OPEN as one, on the AEAD, under the same input keying material.
    let smuggled = EncryptedLiteral {
        nonce: env.nonce,
        ciphertext: env.ciphertext.clone(),
    };
    assert!(SE_PAD_CLASSES.contains(&256));
    let text = smuggled.to_lexical();
    let parsed = EncryptedLiteral::from_lexical(&text).expect("a block ct is a well-formed SE hex");
    assert_eq!(open_literal(&secret, &ctx(), &parsed), Err(Error::Decrypt));

    // …and the reverse direction: an SE value must not open as a block.
    let lit = seal_literal(&secret, &ctx(), "Alice", XSD_STRING).expect("seal");
    let mut faked = env.clone();
    faked.nonce = lit.nonce;
    faked.ciphertext = lit.ciphertext;
    assert_eq!(open_block(&secret, &bctx, &faked), Err(Error::Decrypt));
}

#[test]
fn sealing_an_se_datatype_is_rejected() {
    let dek = Secret32::random();
    assert!(matches!(
        seal_literal(&dek, &ctx(), "se0.aa.bb", SE_ENC_DATATYPE),
        Err(Error::Schema(_))
    ));
    assert!(matches!(
        seal_literal(
            &dek,
            &ctx(),
            "00112233445566778899aabbccddeeff",
            SE_EQTAG_DATATYPE
        ),
        Err(Error::Schema(_))
    ));
}

// ============================================================================
// from_lexical is fail-closed
// ============================================================================

#[test]
fn from_lexical_rejects_malformed_and_non_canonical_input() {
    let dek = Secret32::random();
    let good = seal_literal(&dek, &ctx(), "Alice", XSD_STRING)
        .expect("seal")
        .to_lexical();
    let (nonce_hex, ct_hex) = {
        let mut it = good["se0.".len()..].split('.');
        (
            it.next().unwrap().to_string(),
            it.next().unwrap().to_string(),
        )
    };
    let join = |ver: &str, n: &str, c: &str| -> String {
        let mut s = String::from(ver);
        s.push('.');
        s.push_str(n);
        s.push('.');
        s.push_str(c);
        s
    };

    // wrong / missing tag
    assert!(matches!(
        EncryptedLiteral::from_lexical(""),
        Err(Error::Malformed(_))
    ));
    assert!(matches!(
        EncryptedLiteral::from_lexical("Alice"),
        Err(Error::Malformed(_))
    ));
    assert!(matches!(
        EncryptedLiteral::from_lexical(&join("xx0", &nonce_hex, &ct_hex)),
        Err(Error::Malformed(_))
    ));
    // no version separator at all
    assert!(matches!(
        EncryptedLiteral::from_lexical("se0"),
        Err(Error::Malformed(_))
    ));
    // unknown suite version (including a non-canonical "00" spelling of 0)
    assert_eq!(
        EncryptedLiteral::from_lexical(&join("se1", &nonce_hex, &ct_hex)),
        Err(Error::UnknownSuite)
    );
    assert_eq!(
        EncryptedLiteral::from_lexical(&join("se00", &nonce_hex, &ct_hex)),
        Err(Error::UnknownSuite)
    );
    // missing ciphertext separator
    let mut two_fields = String::from("se0.");
    two_fields.push_str(&nonce_hex);
    assert!(matches!(
        EncryptedLiteral::from_lexical(&two_fields),
        Err(Error::Malformed(_))
    ));
    // a trailing extra field
    let mut trailing = good.clone();
    trailing.push_str(".00");
    assert!(matches!(
        EncryptedLiteral::from_lexical(&trailing),
        Err(Error::Malformed(_))
    ));
    // NON-CANONICAL: uppercase hex is rejected, not normalized
    assert_eq!(
        EncryptedLiteral::from_lexical(&join("se0", &nonce_hex.to_uppercase(), &ct_hex)),
        Err(Error::NonCanonical("hex must be lowercase"))
    );
    assert_eq!(
        EncryptedLiteral::from_lexical(&join("se0", &nonce_hex, &ct_hex.to_uppercase())),
        Err(Error::NonCanonical("hex must be lowercase"))
    );
    // odd-length / non-hex / empty hex fields
    let mut odd = nonce_hex.clone();
    odd.pop();
    assert!(matches!(
        EncryptedLiteral::from_lexical(&join("se0", &odd, &ct_hex)),
        Err(Error::Malformed(_))
    ));
    let mut nonhex = nonce_hex.clone();
    nonhex.replace_range(0..2, "zz");
    assert!(matches!(
        EncryptedLiteral::from_lexical(&join("se0", &nonhex, &ct_hex)),
        Err(Error::Malformed(_))
    ));
    assert!(matches!(
        EncryptedLiteral::from_lexical(&join("se0", "", &ct_hex)),
        Err(Error::Malformed(_))
    ));
    assert!(matches!(
        EncryptedLiteral::from_lexical(&join("se0", &nonce_hex, "")),
        Err(Error::Malformed(_))
    ));
    // wrong nonce length (valid hex, wrong byte count)
    let mut short_nonce = nonce_hex.clone();
    short_nonce.truncate(nonce_hex.len() - 2);
    assert!(matches!(
        EncryptedLiteral::from_lexical(&join("se0", &short_nonce, &ct_hex)),
        Err(Error::Malformed(_))
    ));
    // ciphertext length that is not a pad class + AEAD tag (trailing bytes)
    let mut long_ct = ct_hex.clone();
    long_ct.push_str("00");
    assert!(matches!(
        EncryptedLiteral::from_lexical(&join("se0", &nonce_hex, &long_ct)),
        Err(Error::Malformed(_))
    ));
    // shorter than the AEAD tag
    assert!(matches!(
        EncryptedLiteral::from_lexical(&join("se0", &nonce_hex, "00")),
        Err(Error::Malformed(_))
    ));
    // an over-long field is rejected on the length ceiling, before any decode
    let huge = "ab".repeat(SE_PAD_CLASSES[SE_PAD_CLASSES.len() - 1] + 64);
    assert!(matches!(
        EncryptedLiteral::from_lexical(&join("se0", &nonce_hex, &huge)),
        Err(Error::LimitExceeded(_))
    ));
    let long_nonce = nonce_hex.clone() + "00";
    assert!(matches!(
        EncryptedLiteral::from_lexical(&join("se0", &long_nonce, &ct_hex)),
        Err(Error::LimitExceeded(_))
    ));
    // whitespace is not canonical
    let mut spaced = String::from(" ");
    spaced.push_str(&good);
    assert!(matches!(
        EncryptedLiteral::from_lexical(&spaced),
        Err(Error::Malformed(_))
    ));
}

// ============================================================================
// Padding buckets
// ============================================================================

#[test]
fn padding_hides_exact_value_length_within_a_class() {
    let dek = Secret32::random();
    let c = ctx();
    let short = seal_literal(&dek, &c, "a", XSD_STRING).expect("seal");
    let longer = seal_literal(&dek, &c, "abcdefgh", XSD_STRING).expect("seal");
    assert_eq!(
        short.ciphertext.len(),
        longer.ciphertext.len(),
        "two different value lengths in one pad class must give equal ciphertext lengths"
    );
    assert_eq!(short.to_lexical().len(), longer.to_lexical().len());
    assert_eq!(
        short.pad_class().expect("class"),
        longer.pad_class().expect("class")
    );

    // …and the test is not vacuous: a value that overflows the class lands in a
    // strictly larger one.
    let big = seal_literal(&dek, &c, &"x".repeat(400), XSD_STRING).expect("seal");
    assert!(big.pad_class().expect("class") > short.pad_class().expect("class"));
    assert!(big.ciphertext.len() > short.ciphertext.len());
    assert_eq!(
        open_literal(&dek, &c, &big).expect("open").0.len(),
        400,
        "a larger class still round-trips"
    );
}

#[test]
fn a_value_larger_than_the_largest_class_is_rejected() {
    let dek = Secret32::random();
    let largest = SE_PAD_CLASSES[SE_PAD_CLASSES.len() - 1];
    let too_big = "y".repeat(largest + 1);
    assert!(matches!(
        seal_literal(&dek, &ctx(), &too_big, XSD_STRING),
        Err(Error::LimitExceeded(_))
    ));
}

// ============================================================================
// Equality tags (SEPARATELY opt-in leakage)
// ============================================================================

#[test]
fn equal_values_give_equal_tags_and_different_values_do_not() {
    let dek = Secret32::random();
    let c = ctx();
    // The disclosed leakage, made explicit: equal (lexical, datatype) under one
    // predicate => EQUAL tags, which is the value-equality pattern the server
    // gets to see.
    let a = equality_tag(&dek, &c, "Alice", XSD_STRING);
    let b = equality_tag(&dek, &c, "Alice", XSD_STRING);
    assert!(tags_equal(&a, &b));
    assert_eq!(a, b);

    // a different lexical form, a different datatype, a different predicate, a
    // different graph, and a different DEK all give unrelated tags
    assert!(!tags_equal(&a, &equality_tag(&dek, &c, "Bob", XSD_STRING)));
    assert!(!tags_equal(&a, &equality_tag(&dek, &c, "Alice", XSD_DATE)));
    let other_pred = ValueContext {
        predicate: FOAF_AGE,
        ..c
    };
    assert!(!tags_equal(
        &a,
        &equality_tag(&dek, &other_pred, "Alice", XSD_STRING)
    ));
    let other_graph = ValueContext {
        graph: Some(GRAPH_A),
        ..c
    };
    assert!(!tags_equal(
        &a,
        &equality_tag(&dek, &other_graph, "Alice", XSD_STRING)
    ));
    assert!(!tags_equal(
        &a,
        &equality_tag(&Secret32::random(), &c, "Alice", XSD_STRING)
    ));

    // A concatenation ambiguity would break the length-delimited HMAC message.
    assert!(!tags_equal(
        &equality_tag(&dek, &c, "ab", "c"),
        &equality_tag(&dek, &c, "a", "bc")
    ));
}

#[test]
fn tags_are_comparable_across_subjects_by_design() {
    // A tag MUST NOT bind the subject, or it could not serve a value join; the
    // pinned-subject field is deliberately ignored here.
    let dek = Secret32::random();
    let alice = ValueContext {
        predicate: FOAF_NAME,
        graph: None,
        subject: Some(ALICE),
    };
    let bob = ValueContext {
        predicate: FOAF_NAME,
        graph: None,
        subject: Some(BOB),
    };
    let none = ValueContext {
        predicate: FOAF_NAME,
        graph: None,
        subject: None,
    };
    let t = equality_tag(&dek, &alice, "Smith", XSD_STRING);
    assert!(tags_equal(
        &t,
        &equality_tag(&dek, &bob, "Smith", XSD_STRING)
    ));
    assert!(tags_equal(
        &t,
        &equality_tag(&dek, &none, "Smith", XSD_STRING)
    ));
}

#[test]
fn seal_literal_never_emits_a_tag() {
    // The tag must be a deliberate, separate step: no part of a sealed value's
    // on-wire form may equal the value's equality tag.
    let dek = Secret32::random();
    let c = ctx();
    let lit = seal_literal(&dek, &c, "Alice", XSD_STRING).expect("seal");
    let tag = equality_tag(&dek, &c, "Alice", XSD_STRING);
    assert!(
        !lit.to_lexical().contains(&eqtag_to_lexical(&tag)),
        "a sealed value must not carry its equality tag"
    );
    assert!(!lit.ciphertext.windows(EQTAG_LEN).any(|w| w == tag));
    assert!(!lit.nonce.starts_with(&tag[..]));
}

#[test]
fn eqtag_lexical_form_round_trips_and_is_fail_closed() {
    let dek = Secret32::random();
    let tag = equality_tag(&dek, &ctx(), "Alice", XSD_STRING);
    let text = eqtag_to_lexical(&tag);
    assert_eq!(text.len(), 2 * EQTAG_LEN);
    assert_eq!(text, hex::encode(tag));
    assert_eq!(eqtag_from_lexical(&text).expect("parse"), tag);
    assert_eq!(
        eqtag_from_lexical(&text.to_uppercase()),
        Err(Error::NonCanonical("hex must be lowercase"))
    );
    assert!(matches!(
        eqtag_from_lexical("00ff"),
        Err(Error::Malformed(_))
    ));
    assert!(matches!(eqtag_from_lexical(""), Err(Error::Malformed(_))));
    let mut too_long = text.clone();
    too_long.push_str("00");
    assert!(matches!(
        eqtag_from_lexical(&too_long),
        Err(Error::Malformed(_))
    ));
}

// ============================================================================
// Golden vectors — pin the derivation bytes (wire/derivation format is
// load-bearing, exactly as tests/vectors.rs pins the block-profile ones)
// ============================================================================

/// A change to any of these bytes is a **format break**, not a refactor: it would
/// silently stop a peer (or an older sealed value) from being readable. They pin
/// the `value-key` HKDF label + context encoding, and the equality tag's
/// domain-separation prefix, length delimiting and truncation.
#[test]
fn vector_value_key_and_equality_tag() {
    let dek = Secret32([7u8; 32]);
    let c = ctx();
    assert_eq!(
        hex::encode(value_key(&dek, c.predicate, c.graph)),
        "bba37f7a607977d3266cb9d5fdc1c8b6b8746dd4ea9d45c38269f40ccb78ef3a",
        "value_key(default graph) derivation changed"
    );
    assert_eq!(
        hex::encode(value_key(&dek, c.predicate, Some(GRAPH_A))),
        "abc98c5d0d4bae51ce36ab500c30d55600bd4fdee6d23a99355d2d90c66bebd7",
        "value_key(named graph) derivation changed"
    );
    assert_eq!(
        eqtag_to_lexical(&equality_tag(&dek, &c, "Alice", XSD_STRING)),
        "39ac9a636a0aa5c2585230589b724028",
        "equality-tag derivation changed"
    );
}

// ============================================================================
// Key schedule: `value_key` is domain-separated and context-bound
// ============================================================================

#[test]
fn value_key_binds_predicate_and_graph() {
    let dek = Secret32::random();
    let base = value_key(&dek, FOAF_NAME, None);
    assert_eq!(base, value_key(&dek, FOAF_NAME, None), "deterministic");
    assert_ne!(base, value_key(&dek, FOAF_AGE, None), "predicate-bound");
    assert_ne!(
        base,
        value_key(&dek, FOAF_NAME, Some(GRAPH_A)),
        "graph-bound"
    );
    assert_ne!(
        base,
        value_key(&dek, FOAF_NAME, Some("")),
        "None (default graph) must not collide with Some(\"\")"
    );
    assert_ne!(
        value_key(&dek, "ab", Some("c")),
        value_key(&dek, "a", Some("bc")),
        "length-delimited context: no concatenation ambiguity"
    );
    assert_ne!(
        base,
        value_key(&Secret32::random(), FOAF_NAME, None),
        "DEK-bound"
    );
}

// ============================================================================
// Property test
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    #[test]
    fn arbitrary_values_round_trip_through_the_lexical_form(
        lexical in ".{0,120}",
        datatype in "http://example[.]org/dt/[a-z]{1,12}",
        predicate in "http://example[.]org/p/[a-z]{1,12}",
        graph in proptest::option::of("http://example[.]org/g/[a-z]{1,8}"),
        subject in proptest::option::of("http://example[.]org/s/[a-z]{1,8}"),
        seed in any::<[u8; 32]>(),
    ) {
        let dek = Secret32(seed);
        let c = ValueContext {
            predicate: &predicate,
            graph: graph.as_deref(),
            subject: subject.as_deref(),
        };
        let lit = seal_literal(&dek, &c, &lexical, &datatype).expect("seal");
        let parsed = EncryptedLiteral::from_lexical(&lit.to_lexical()).expect("parse");
        prop_assert_eq!(&parsed, &lit);
        let (got_lex, got_dt) = open_literal(&dek, &c, &parsed).expect("open");
        prop_assert_eq!(got_lex, lexical);
        prop_assert_eq!(got_dt, datatype);
    }
}
