//! Identity-op + desync regression guard (sq-in0rz) — the EXPANDED §11 bead 4
//! of `research/zk-field-native-encoding.md`, scoped to the on-main host estate.
//!
//! THE reject-list (v) witness: the dual-leaf method's value handle is
//! deliberately MANY-TO-ONE on the term for several datatype classes (IEEE
//! `-0.0`/`+0.0`, NaN payloads, cross-scale decimals). Identity ops —
//! scan-row equality, `join_eq`, `DISTINCT`, `sameTerm` — run over the
//! string-canonical `encode_term` identities / committed leaves, so they must
//! separate every pair the value lane collapses. This guard pins that per
//! fixture, plus the §6 fail-closed desync predicate at the encoder level.
//!
//! Named prerequisite witness for the §13.6 offset-normalisation widening:
//! any future widening that makes a hook many-to-one on the term MUST add its
//! fixture pair here. (Boolean `"true"`/`"1"` and dateTime offset fixtures
//! join this file once their host encoders — #2089 / #2104 — merge.)
//!
//! TEST-ONLY: no source change; the ingest-PIPELINE-level desync guard moves
//! with sq-j506 (commit-pipeline integration). NO production-soundness claim —
//! inherits the documented INV-VL downgrade (CR-G8 / sq-qhy4).
#![cfg(feature = "dual-leaf")]

use oxrdf::{Literal, NamedNode, Term};
use sparq_zk::dual_leaf::{
    canonical_f64_bits, decimal_datatype_const, encode_decimal, encode_double, encode_literal,
    DualLeafError, F64_CANONICAL_NAN, XSD_DECIMAL, XSD_DOUBLE, XSD_INTEGER,
};
use sparq_zk::encode::{encode_term, TYPE_CODE_LITERAL};
use sparq_zk::poseidon2;
use sparq_zk::Fr;

fn lit(value: &str, dt: &str) -> Literal {
    Literal::new_typed_literal(value, NamedNode::new(dt).unwrap())
}

/// The string-canonical identity of a literal, exactly as the commitment
/// pipeline computes it (`encode_term`'s literal branch; literals ignore the
/// graph salt).
fn identity(l: &Literal) -> Fr {
    encode_term(&Term::Literal(l.clone()), &Fr::from(0u64)).unwrap()
}

#[test]
fn double_neg_zero_pair_is_separated_on_the_identity_lane_only() {
    // -0.0 and +0.0: SPARQL-numerically equal, so the value lane collapses
    // them (same canonical hook, same value_component) — the identity lane
    // (lexical_component -> leaf -> encode_term) is the ONLY separator, so
    // join/DISTINCT/sameTerm must never consult the value side.
    let neg = lit("-0.0E0", XSD_DOUBLE);
    let pos = lit("0.0E0", XSD_DOUBLE);
    let n = encode_double(&neg).unwrap();
    let p = encode_double(&pos).unwrap();
    assert_eq!(n.value_hook, p.value_hook);
    assert_eq!(n.value_component(), p.value_component());
    // The identity lane separates the pair at every level identity ops read.
    assert_ne!(n.lexical_component, p.lexical_component);
    assert_ne!(n.leaf(), p.leaf());
    assert_ne!(identity(&neg), identity(&pos));
}

#[test]
fn nan_payload_bits_collapse_on_the_value_lane() {
    // Every NaN payload is ONE SPARQL-numeric unordered class: the canonical
    // bind folds all payloads (quiet or signalling) to the single canonical
    // hook, so no committed hook can smuggle payload identity into a FILTER.
    let payloads = [
        0x7ff8_0000_0000_0000u64, // canonical qNaN
        0x7ff0_0000_0000_0001,    // sNaN payload 1
        0x7ff8_dead_beef_cafe,    // qNaN with payload
        0xfff8_0000_0000_0000,    // negative-sign qNaN
    ];
    for bits in payloads {
        assert_eq!(canonical_f64_bits(bits), F64_CANONICAL_NAN);
    }
    // The one hookable NaN LEXICAL commits exactly the canonical class hook.
    let c = encode_double(&lit("NaN", XSD_DOUBLE)).unwrap();
    assert_eq!(c.value_hook, Fr::from(F64_CANONICAL_NAN));
}

#[test]
fn decimal_cross_scale_pair_is_separated_everywhere_identity_ops_look() {
    // "5.0" (fd=1) and "5.00" (fd=2): the SAME numeric value. The B4 scale
    // bind puts them in different value sub-lanes (scale-folded consts), and
    // the identity lane separates them as terms — they must not join/dedup.
    let a_lit = lit("5.0", XSD_DECIMAL);
    let b_lit = lit("5.00", XSD_DECIMAL);
    let a = encode_decimal(&a_lit).unwrap();
    let b = encode_decimal(&b_lit).unwrap();
    assert_eq!(a.datatype_const, decimal_datatype_const(1));
    assert_eq!(b.datatype_const, decimal_datatype_const(2));
    assert_ne!(a.value_component(), b.value_component());
    assert_ne!(a.lexical_component, b.lexical_component);
    assert_ne!(a.leaf(), b.leaf());
    assert_ne!(identity(&a_lit), identity(&b_lit));
}

#[test]
fn cross_datatype_five_never_shares_a_value_component() {
    // Integer "5", decimal "5.0", double "5.0E0": one mathematical value,
    // three datatypes. The datatype-folded consts keep every value_component
    // distinct (cross-datatype separation), and the identity lane separates
    // all three terms.
    let i = encode_literal(&lit("5", XSD_INTEGER)).unwrap();
    let d = encode_decimal(&lit("5.0", XSD_DECIMAL)).unwrap();
    let f = encode_double(&lit("5.0E0", XSD_DOUBLE)).unwrap();
    assert_ne!(i.value_component(), d.value_component());
    assert_ne!(i.value_component(), f.value_component());
    assert_ne!(d.value_component(), f.value_component());
    let ids = [
        identity(&lit("5", XSD_INTEGER)),
        identity(&lit("5.0", XSD_DECIMAL)),
        identity(&lit("5.0E0", XSD_DOUBLE)),
    ];
    assert_ne!(ids[0], ids[1]);
    assert_ne!(ids[0], ids[2]);
    assert_ne!(ids[1], ids[2]);
}

#[test]
fn identity_lane_is_exactly_the_string_canonical_encoder() {
    // THE load-bearing cross-check: for every hookable fixture, the dual
    // leaf's lexical_component is byte-identical to the blake3 inner of
    // encode_term's literal branch — i.e. the committed string-canonical
    // identity is h2(TYPE_CODE_LITERAL, lexical_component). If this ever
    // breaks, dual-leaf graphs and string-canonical graphs disagree on term
    // identity and every identity op is desynced.
    let fixtures: Vec<(Literal, sparq_zk::dual_leaf::DualLeafComponents)> = vec![
        (
            lit("18", XSD_INTEGER),
            encode_literal(&lit("18", XSD_INTEGER)).unwrap(),
        ),
        (
            lit("0", XSD_INTEGER),
            encode_literal(&lit("0", XSD_INTEGER)).unwrap(),
        ),
        (
            lit("-2.50", XSD_DECIMAL),
            encode_decimal(&lit("-2.50", XSD_DECIMAL)).unwrap(),
        ),
        (
            lit("5.00", XSD_DECIMAL),
            encode_decimal(&lit("5.00", XSD_DECIMAL)).unwrap(),
        ),
        (
            lit("-0.0E0", XSD_DOUBLE),
            encode_double(&lit("-0.0E0", XSD_DOUBLE)).unwrap(),
        ),
        (
            lit("NaN", XSD_DOUBLE),
            encode_double(&lit("NaN", XSD_DOUBLE)).unwrap(),
        ),
    ];
    for (l, c) in fixtures {
        let reconstructed = poseidon2::hash(&[Fr::from(TYPE_CODE_LITERAL), c.lexical_component]);
        assert_eq!(
            reconstructed,
            identity(&l),
            "identity-lane drift for {l}: encode_term no longer equals h2(TYPE_CODE_LITERAL, lexical_component)"
        );
    }
}

#[test]
fn value_component_is_structurally_absent_from_the_identity_lane() {
    // Two literals with the SAME lexical bytes under different value-lane
    // treatment cannot exist (one lexical = one identity); the converse
    // guard: identical value_components with distinct lexicals never yield
    // the same identity. Sweep every collision pair in this file.
    let pairs = [
        (lit("-0.0E0", XSD_DOUBLE), lit("0.0E0", XSD_DOUBLE)),
        (lit("5.0", XSD_DECIMAL), lit("5.00", XSD_DECIMAL)),
    ];
    for (a, b) in pairs {
        assert_ne!(
            identity(&a),
            identity(&b),
            "identity ops would wrongly join/dedup {a} and {b}"
        );
    }
}

#[test]
fn fail_closed_rejects_every_non_canonical_hookable_lexical() {
    // The §6 desync-detection guard at the encoder level: a lexical that is
    // hookable-datatyped but non-canonical must be REJECTED (never silently
    // canonicalised, never a desynced leaf). Pipeline-level enforcement moves
    // with sq-j506.
    let rejected: Vec<Literal> = vec![
        // integer: leading zero, sign (slice is non-negative), overflow, junk
        lit("05", XSD_INTEGER),
        lit("-5", XSD_INTEGER),
        lit("99999999999999999999999999", XSD_INTEGER),
        lit("5 ", XSD_INTEGER),
        // decimal: integer-only form, leading zero, negative zero, junk
        lit("5", XSD_DECIMAL),
        lit("05.0", XSD_DECIMAL),
        lit("-0.00", XSD_DECIMAL),
        lit("1.2.3", XSD_DECIMAL),
        // double: unparseable
        lit("not-a-number", XSD_DOUBLE),
        lit("0x1p3", XSD_DOUBLE),
    ];
    for l in rejected {
        let res = match l.datatype().as_str() {
            XSD_INTEGER => encode_literal(&l),
            XSD_DECIMAL => encode_decimal(&l),
            _ => encode_double(&l),
        };
        assert!(
            matches!(res, Err(DualLeafError::NonCanonicalValue(_))),
            "expected fail-closed NonCanonicalValue for {l}, got {res:?}"
        );
    }
}
