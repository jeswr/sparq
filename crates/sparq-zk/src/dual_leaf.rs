//! Dual-leaf value-lane host encoding (sq-xojl) — the host mirror of the
//! `filter_value_dl_int` Noir circuit member.
//!
//! OPT-IN, behind the `dual-leaf` cargo feature (OFF by default). This module is
//! compiled out of a normal build: the default `string-canonical` commitment
//! pipeline (`encode.rs`/`commit.rs`) is byte-unchanged.
//!
//! # The dual-leaf literal shape (`research/zk-field-native-encoding.md` §3.1)
//!
//! ```text
//! Enc_literal = h3(value_component, lexical_component, TYPE_CODE_LITERAL)
//!   value_component   = h3(VALUE_HOOK, DATATYPE_CONST, LANG_NONE)
//!   lexical_component = blake3_field(canonical N-Triples token)   // == today's
//!                                                                 // string-canonical h_s
//! ```
//!
//! - `VALUE_HOOK` is the numeric value handle. For `xsd:integer` it is the
//!   integer value itself as a field element (canonical by construction:
//!   `"05"` and `"5"` parse to the SAME hook — exactly the value-collapse the
//!   `lexical_component` disambiguates for identity ops).
//! - `DATATYPE_CONST = blake3_field(datatype IRI)` folds the datatype in so a
//!   cross-datatype value collision cannot occur.
//! - `LANG_NONE` is the reserved no-language sentinel (numeric datatypes have no
//!   language); it mirrors the Noir member's `LANG_NONE` global.
//! - `lexical_component` is exactly the string-canonical scheme's `h_s`
//!   (`blake3_field(literal.to_string())`), carried so identity ops
//!   (`sameTerm`/`DISTINCT`/`join`) keep term identity unchanged.
//!
//! This host encoder + the `filter_value_dl_int` circuit member recompute the
//! SAME leaf, so the verifier reconstructs the public `operand_enc` correctly
//! (`dual_leaf_value_components` is the cross-check seam).
//!
//! # INV-VL DOWNGRADE — DOCUMENTED RISK (load-bearing)
//!
//! The string-canonical pipeline enforces, in-circuit against an arbitrary
//! committer (including a malicious *trusted* issuer), that the compared value
//! equals `parse(committed lexical)` (the invariant INV-VL), because value and
//! binding derive from one witnessed digit array. The dual leaf witnesses
//! `VALUE_HOOK` and `lexical_component` INDEPENDENTLY, so it **REMOVES INV-VL**:
//! value↔lexical agreement on the value-FILTER lane moves from MACHINE-ENFORCED
//! to TRUSTED-ISSUER-HONESTY. A malicious *trusted* issuer can commit a leaf
//! whose `VALUE_HOOK` answers a value-FILTER as 18 while its `lexical_component`
//! answers `sameTerm`/`DISTINCT`/`join` as "5" — impossible in the
//! string-canonical pipeline. No *untrusted* party can exploit it (the issuer
//! signature chain is intact). The maintainer ACCEPTED this at research grade
//! (#769) and asked for it built WITH documentation; it is an open external-audit
//! obligation (gap CR-G8 / sq-qhy4). The honest host mitigation [`encode_literal`]
//! provides is **same-leaf co-binding**: it derives `VALUE_HOOK` and
//! `lexical_component` from the SAME parsed value and fails closed if the lexical
//! form does not canonically parse — so *sparq's own* commitments cannot
//! self-desync (`research/zk-field-native-encoding.md` §6). This binds honest
//! sparq ingest, NOT a malicious external issuer.
//!
//! The whole ZK estate is remediated + internally re-audited but **NOT externally
//! audited** (sq-qhy4, P0). Nothing here is a soundness or privacy guarantee.

use crate::encode::TYPE_CODE_LITERAL;
use crate::field::{field_from_hash_bytes, Fr};
use crate::poseidon2;
use oxrdf::Literal;

/// The reserved "no language" sentinel for the `LANG_NONE` slot — mirrors the
/// Noir `filter_value::LANG_NONE` global. A real language tag would be
/// `blake3(lang)`; numeric datatypes have no language, so they fold this fixed
/// field tag, which is distinct from any plausible blake3 output (a small
/// reserved value) so a value component can never collide a language-tagged one.
pub const LANG_NONE: u64 = 1;

/// The `xsd:integer` datatype IRI (the integer value-lane datatype class,
/// sq-xojl).
pub const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

/// The `xsd:double` datatype IRI (the IEEE-754 value-lane class, sq-2ezsx).
pub const XSD_DOUBLE: &str = "http://www.w3.org/2001/XMLSchema#double";

/// The `xsd:decimal` datatype IRI (the fixed-scale value-lane class, sq-2ezsx).
pub const XSD_DECIMAL: &str = "http://www.w3.org/2001/XMLSchema#decimal";

/// The canonical IEEE-754 quiet-NaN bit pattern the double value lane folds ALL
/// NaN payloads to — mirrors the Noir `filter_value::F64_CANONICAL_NAN`. Every
/// NaN is one SPARQL-numeric unordered class, so the value handle must not
/// distinguish payloads.
pub const F64_CANONICAL_NAN: u64 = 0x7ff8_0000_0000_0000;

fn blake3_field(bytes: &[u8]) -> Fr {
    field_from_hash_bytes(blake3::hash(bytes).as_bytes())
}

/// The `DATATYPE_CONST` for a datatype IRI: `blake3_field(IRI bytes)`. Folded
/// into `value_component` so a cross-datatype value collision (integer 5 vs the
/// bits for 5.0) is impossible.
pub fn datatype_const(datatype_iri: &str) -> Fr {
    blake3_field(datatype_iri.as_bytes())
}

/// The `DATATYPE_CONST` for an `xsd:decimal` value handle at a fixed fraction
/// SCALE: `blake3_field("<xsd:decimal IRI>@scale=<fd>")` — mirrors the Noir
/// `filter_value_dl_decimal` member's public `datatype_const` (which folds BOTH
/// the datatype AND the scale). This is the explicit canonical-SCALE bind (B4):
/// `"5.0"` (scale 1) and `"5.00"` (scale 2) have the SAME numeric value but are
/// DIFFERENT value handles because the scale is folded in, so a value at one scale
/// can never collide a value at another. `fd` is the fraction-digit count of the
/// canonical decimal lexical form.
pub fn decimal_datatype_const(fd: u32) -> Fr {
    blake3_field(format!("{}@scale={}", XSD_DECIMAL, fd).as_bytes())
}

/// Fold an `xsd:double` IEEE-754 bit pattern to its SPARQL-numeric CANONICAL bit
/// pattern — the host mirror of the Noir `filter_value::canonical_f64_bits`: any
/// NaN -> [`F64_CANONICAL_NAN`], `-0.0` -> `+0.0`, everything else unchanged. This
/// is the explicit IEEE-bit bind (B4) the dual-leaf double class needs so the
/// value handle is single-valued per SPARQL-numeric value.
pub fn canonical_f64_bits(bits: u64) -> u64 {
    let exp_all_ones = bits & 0x7ff0_0000_0000_0000 == 0x7ff0_0000_0000_0000;
    let mantissa_nonzero = bits & 0x000f_ffff_ffff_ffff != 0;
    if exp_all_ones && mantissa_nonzero {
        // NaN (exponent all ones, non-zero mantissa) -> the canonical qNaN.
        F64_CANONICAL_NAN
    } else if bits == 0x8000_0000_0000_0000 {
        // -0.0 -> +0.0 (numerically equal).
        0
    } else {
        bits
    }
}

/// The dual-leaf failure for a literal that cannot be encoded on the requested
/// value lane.
#[derive(Debug, PartialEq, Eq)]
pub enum DualLeafError {
    /// The literal's datatype is not the one the called encoder handles (e.g.
    /// passing an `xsd:double` to [`encode_literal`], which is `xsd:integer`-only).
    NotValueLane(String),
    /// The lexical form does not canonically parse to the value lane's value
    /// handle — fail closed (same-leaf co-binding, §6). This is the host
    /// mitigation that keeps sparq's own commitments INV-VL-consistent; it does
    /// NOT bind a malicious external issuer.
    NonCanonicalValue(String),
}

impl std::fmt::Display for DualLeafError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DualLeafError::NotValueLane(t) => {
                write!(f, "not the expected value-lane datatype: {}", t)
            }
            DualLeafError::NonCanonicalValue(t) => {
                write!(
                    f,
                    "non-canonical value-lane literal (fail-closed co-binding): {}",
                    t
                )
            }
        }
    }
}

impl std::error::Error for DualLeafError {}

/// The three field components of a dual-leaf literal value, exposed so the
/// verifier / cross-tests can reconstruct the public `operand_enc` exactly as
/// the circuit does. The leaf is
/// `h3(value_component, lexical_component, TYPE_CODE_LITERAL)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DualLeafComponents {
    /// `VALUE_HOOK`: the numeric value handle as a field element (the integer
    /// value for `xsd:integer`).
    pub value_hook: Fr,
    /// `DATATYPE_CONST = blake3_field(datatype IRI)`.
    pub datatype_const: Fr,
    /// `lexical_component = blake3_field(canonical N-Triples token)` — exactly the
    /// string-canonical scheme's `h_s`.
    pub lexical_component: Fr,
}

impl DualLeafComponents {
    /// `value_component = h3(VALUE_HOOK, DATATYPE_CONST, LANG_NONE)`.
    pub fn value_component(&self) -> Fr {
        poseidon2::hash(&[self.value_hook, self.datatype_const, Fr::from(LANG_NONE)])
    }

    /// The full dual leaf `Enc = h3(value_component, lexical_component, TYPE_CODE_LITERAL)`
    /// — the value committed as the literal's `operand_enc`, recomputed exactly
    /// as `filter_value_dl_int` does in-circuit.
    pub fn leaf(&self) -> Fr {
        poseidon2::hash(&[
            self.value_component(),
            self.lexical_component,
            Fr::from(TYPE_CODE_LITERAL),
        ])
    }
}

/// Encodes an `xsd:integer` literal under the dual-leaf method, with fail-closed
/// same-leaf co-binding (§6): `VALUE_HOOK` and `lexical_component` are derived
/// from the SAME canonical value, and a lexical form that does not canonically
/// parse to a non-negative `u64` integer is REJECTED (so sparq's own ingest
/// cannot self-desync). Returns the three components; `.leaf()` is the committed
/// `Enc`.
///
/// Honest scope: non-negative `xsd:integer`, magnitude `< 2^64` — the same
/// canonical-non-negative fragment the `filter_value_dl_int` member proves.
/// Negative integers / decimal / double are a documented follow-up.
pub fn encode_literal(literal: &Literal) -> Result<DualLeafComponents, DualLeafError> {
    let dt = literal.datatype();
    if dt.as_str() != XSD_INTEGER {
        return Err(DualLeafError::NotValueLane(literal.to_string()));
    }
    // Same-leaf co-binding: parse the lexical value once. Fail closed on a
    // non-canonical / out-of-range / signed lexical form — this is the host
    // mitigation that keeps sparq's own commitments INV-VL-consistent.
    let lex = literal.value();
    let value = canonical_nonneg_u64(lex)
        .ok_or_else(|| DualLeafError::NonCanonicalValue(literal.to_string()))?;
    Ok(DualLeafComponents {
        value_hook: Fr::from(value),
        datatype_const: datatype_const(XSD_INTEGER),
        // lexical_component is EXACTLY the string-canonical h_s over the canonical
        // N-Triples token (the same bytes `encode::encode_term` hashes), so a
        // dual-leaf graph's identity ops read the same lexical identity.
        lexical_component: blake3_field(literal.to_string().as_bytes()),
    })
}

/// Parse a canonical non-negative `xsd:integer` lexical form to a `u64`.
/// Canonical = ASCII digits only, no sign, no leading zero (except the single
/// digit "0"), value `< 2^64`. Returns `None` for any non-canonical / signed /
/// overflowing form (the §6 fail-closed predicate).
fn canonical_nonneg_u64(lexical: &str) -> Option<u64> {
    if lexical.is_empty() || !lexical.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    // Reject non-canonical leading zero ("05"), but accept the lone "0".
    if lexical.len() > 1 && lexical.starts_with('0') {
        return None;
    }
    lexical.parse::<u64>().ok()
}

/// Encodes an `xsd:double` literal under the dual-leaf method (sq-2ezsx), with
/// fail-closed same-leaf co-binding (§6): the value handle is the SPARQL-numeric
/// CANONICAL IEEE-754 bit pattern of the SAME lexical form the `lexical_component`
/// hashes, and a lexical form that does not parse as a finite/inf/NaN `xsd:double`
/// is REJECTED (so sparq's own ingest cannot self-desync). Returns the three
/// components; `.leaf()` is the committed `Enc`.
///
/// The value handle is [`canonical_f64_bits`]`(parsed.to_bits())` — exactly the
/// `filter_value_dl_f64` member's in-circuit canonical bind: `-0.0`/`+0.0` and all
/// NaN payloads collapse to ONE handle, the many-to-one-on-the-term property the
/// `lexical_component` disambiguates for identity ops.
///
/// Honest scope: any `xsd:double` Rust's `f64` lexical parser accepts (the same
/// floating fragment the SPARQL/XPath F&O semantics cover). The general fractional
/// /scientific in-circuit decimal→IEEE parse the blake3 lane defers is SIDESTEPPED
/// here: the value handle is the IEEE bits, committed off-circuit.
pub fn encode_double(literal: &Literal) -> Result<DualLeafComponents, DualLeafError> {
    if literal.datatype().as_str() != XSD_DOUBLE {
        return Err(DualLeafError::NotValueLane(literal.to_string()));
    }
    // Same-leaf co-binding: parse the lexical form once. `f64::from_str` accepts
    // the xsd:double lexical fragment (incl. "INF"/"-INF"/"NaN" via a small map);
    // a form it rejects is fail-closed (sparq cannot self-desync).
    let bits = parse_xsd_double_bits(literal.value())
        .ok_or_else(|| DualLeafError::NonCanonicalValue(literal.to_string()))?;
    Ok(DualLeafComponents {
        value_hook: Fr::from(canonical_f64_bits(bits)),
        datatype_const: datatype_const(XSD_DOUBLE),
        lexical_component: blake3_field(literal.to_string().as_bytes()),
    })
}

/// Encodes an `xsd:decimal` literal under the dual-leaf method (sq-2ezsx), with
/// fail-closed same-leaf co-binding (§6): the value handle is the SIGNED SCALED
/// MAGNITUDE `sign * round(|v| * 10^fd)` of the SAME lexical form the
/// `lexical_component` hashes, where `fd` is the fraction-digit count of that
/// lexical form; a non-canonical form is REJECTED. Returns the three components;
/// `.leaf()` is the committed `Enc`.
///
/// The B4 canonical-SCALE bind: `fd` is folded into `datatype_const`
/// ([`decimal_datatype_const`]), so `"5.0"` (fd=1) and `"5.00"` (fd=2) — the SAME
/// value, distinct lexical forms — get DIFFERENT handles (the scale is part of the
/// handle); within one scale the handle is canonical, and the `lexical_component`
/// disambiguates for identity ops. The sign is folded into the value handle (a
/// negative is the field negation), so a sign flip changes the leaf.
///
/// Honest scope: a canonical `xsd:decimal` lexical form `[-]?<int>.<frac>` whose
/// scaled magnitude `round(|v| * 10^fd)` fits `u64` (`fd >= 1`; the member's
/// fixed-point domain). An integer-only decimal (`"5"`, no point) or `>u64`
/// scaled magnitude is rejected fail-closed.
pub fn encode_decimal(literal: &Literal) -> Result<DualLeafComponents, DualLeafError> {
    if literal.datatype().as_str() != XSD_DECIMAL {
        return Err(DualLeafError::NotValueLane(literal.to_string()));
    }
    let (neg, scaled, fd) = canonical_decimal_scaled(literal.value())
        .ok_or_else(|| DualLeafError::NonCanonicalValue(literal.to_string()))?;
    let mag = Fr::from(scaled);
    let signed = if neg { -mag } else { mag };
    Ok(DualLeafComponents {
        value_hook: signed,
        datatype_const: decimal_datatype_const(fd),
        lexical_component: blake3_field(literal.to_string().as_bytes()),
    })
}

/// Parse an `xsd:double` lexical form to its IEEE-754 `u64` bit pattern. Accepts
/// the lexical fragment Rust's `f64` parser handles plus the xsd-canonical
/// `INF`/`-INF`/`NaN` spellings; returns `None` for any other form (fail-closed).
fn parse_xsd_double_bits(lexical: &str) -> Option<u64> {
    let v: f64 = match lexical {
        "INF" | "+INF" => f64::INFINITY,
        "-INF" => f64::NEG_INFINITY,
        "NaN" => f64::NAN,
        // Reject the bare integer-less forms Rust accepts but xsd:double does not
        // need special-casing here; `f64::from_str` covers "1.0E1", "42", "-0.0".
        other => other.parse::<f64>().ok()?,
    };
    Some(v.to_bits())
}

/// Parse a canonical `xsd:decimal` lexical form `[-]?<int>.<frac>` to `(neg,
/// round(|v| * 10^fd), fd)`. Canonical = optional leading `-`, integer part with
/// no leading zero (except the lone `0`), a `.`, and `fd >= 1` fraction digits;
/// `-0...` is rejected (no `-0`); the scaled magnitude must fit `u64`. Returns
/// `None` for any non-canonical / integer-only / overflowing form (the §6
/// fail-closed predicate, mirroring the `filter_signed::filter_decimal_check`
/// canonical-form discipline).
fn canonical_decimal_scaled(lexical: &str) -> Option<(bool, u64, u32)> {
    let (neg, rest) = match lexical.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, lexical),
    };
    let (int_part, frac_part) = rest.split_once('.')?;
    // Integer + fraction parts: ASCII digits, fraction non-empty.
    if int_part.is_empty() || frac_part.is_empty() {
        return None;
    }
    if !int_part.bytes().all(|b| b.is_ascii_digit())
        || !frac_part.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    // Canonical integer part: no leading zero unless it is the lone "0".
    if int_part.len() > 1 && int_part.starts_with('0') {
        return None;
    }
    let fd = frac_part.len() as u32;
    let int_val: u64 = int_part.parse().ok()?;
    let frac_val: u64 = frac_part.parse().ok()?;
    let scale = 10u64.checked_pow(fd)?;
    let scaled = int_val.checked_mul(scale)?.checked_add(frac_val)?;
    // No "-0.0...0": a negative sign on a zero scaled magnitude is not canonical.
    if neg && scaled == 0 {
        return None;
    }
    Some((neg, scaled, fd))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::NamedNode;

    fn int_lit(v: &str) -> Literal {
        Literal::new_typed_literal(v, NamedNode::new(XSD_INTEGER).unwrap())
    }

    #[test]
    fn integer_round_trips_to_a_dual_leaf() {
        let c = encode_literal(&int_lit("18")).unwrap();
        assert_eq!(c.value_hook, Fr::from(18u64));
        // The leaf is well-defined and stable.
        assert_eq!(c.leaf(), c.leaf());
        // value_component folds the datatype so 18-as-integer != a bare 18 hash.
        assert_ne!(c.value_component(), Fr::from(18u64));
    }

    #[test]
    fn lexical_component_equals_string_canonical_h_s() {
        // The dual leaf's lexical_component MUST be byte-identical to the
        // string-canonical scheme's h_s, so identity ops are unchanged.
        let lit = int_lit("18");
        let c = encode_literal(&lit).unwrap();
        let string_canonical_hs = blake3_field(lit.to_string().as_bytes());
        assert_eq!(c.lexical_component, string_canonical_hs);
    }

    #[test]
    fn host_leaf_matches_the_circuit_construction() {
        // The leaf the host commits MUST equal h3(h3(hook, dt, LANG_NONE),
        // lexical, TYPE_CODE_LITERAL) — exactly the filter_value_dl_int member's
        // construction. This is the load-bearing host<->circuit cross-check.
        let c = encode_literal(&int_lit("42")).unwrap();
        let vc = poseidon2::hash(&[c.value_hook, c.datatype_const, Fr::from(LANG_NONE)]);
        let leaf = poseidon2::hash(&[vc, c.lexical_component, Fr::from(TYPE_CODE_LITERAL)]);
        assert_eq!(c.leaf(), leaf);
    }

    #[test]
    fn value_collapse_05_and_5_is_intended_for_the_value_hook() {
        // "05" is NON-canonical and fail-closed-rejected at ingest (the §6
        // co-binding), so honest sparq never commits it. Canonical "5" parses.
        assert!(matches!(
            encode_literal(&int_lit("05")),
            Err(DualLeafError::NonCanonicalValue(_))
        ));
        let c5 = encode_literal(&int_lit("5")).unwrap();
        assert_eq!(c5.value_hook, Fr::from(5u64));
    }

    #[test]
    fn non_value_lane_literal_is_rejected() {
        let plain = Literal::new_simple_literal("hello");
        assert!(matches!(
            encode_literal(&plain),
            Err(DualLeafError::NotValueLane(_))
        ));
        let dbl = Literal::new_typed_literal(
            "1.5",
            NamedNode::new("http://www.w3.org/2001/XMLSchema#double").unwrap(),
        );
        assert!(matches!(
            encode_literal(&dbl),
            Err(DualLeafError::NotValueLane(_))
        ));
    }

    #[test]
    fn signed_and_overflow_are_fail_closed() {
        // The §6 co-binding rejects signed forms (this slice is non-negative
        // only) and out-of-u64-range forms — fail closed, never silent.
        assert!(matches!(
            encode_literal(&int_lit("-5")),
            Err(DualLeafError::NonCanonicalValue(_))
        ));
        let huge = "99999999999999999999999999"; // > u64::MAX
        assert!(matches!(
            encode_literal(&int_lit(huge)),
            Err(DualLeafError::NonCanonicalValue(_))
        ));
    }

    #[test]
    fn zero_is_canonical() {
        let c = encode_literal(&int_lit("0")).unwrap();
        assert_eq!(c.value_hook, Fr::from(0u64));
    }

    #[test]
    fn distinct_values_give_distinct_leaves() {
        let a = encode_literal(&int_lit("17")).unwrap().leaf();
        let b = encode_literal(&int_lit("18")).unwrap().leaf();
        assert_ne!(a, b);
    }

    // ---- xsd:double class (sq-2ezsx) ----

    fn dbl_lit(v: &str) -> Literal {
        Literal::new_typed_literal(v, NamedNode::new(XSD_DOUBLE).unwrap())
    }

    #[test]
    fn double_round_trips_to_a_dual_leaf() {
        let c = encode_double(&dbl_lit("2.5E0")).unwrap();
        assert_eq!(c.value_hook, Fr::from(2.5f64.to_bits()));
        assert_eq!(c.datatype_const, datatype_const(XSD_DOUBLE));
        assert_eq!(c.leaf(), c.leaf());
    }

    #[test]
    fn double_host_leaf_matches_the_circuit_construction() {
        // h3(h3(canon_bits, dt, LANG_NONE), lexical, TYPE_CODE_LITERAL) — exactly
        // the filter_value_dl_f64 member's construction.
        let lit = dbl_lit("1.0E1");
        let c = encode_double(&lit).unwrap();
        let vc = poseidon2::hash(&[c.value_hook, c.datatype_const, Fr::from(LANG_NONE)]);
        let leaf = poseidon2::hash(&[vc, c.lexical_component, Fr::from(TYPE_CODE_LITERAL)]);
        assert_eq!(c.leaf(), leaf);
    }

    #[test]
    fn double_neg_zero_collapses_to_pos_zero_value_handle() {
        // THE B4 INVARIANT (double): -0.0 and +0.0 are numerically equal, so the
        // CANONICAL value handle is identical (both fold to +0.0 bits). The lexical
        // forms differ, so the leaves still differ (identity ops disambiguated).
        let neg = encode_double(&dbl_lit("-0.0E0")).unwrap();
        let pos = encode_double(&dbl_lit("0.0E0")).unwrap();
        assert_eq!(neg.value_hook, Fr::from(0u64));
        assert_eq!(pos.value_hook, Fr::from(0u64));
        assert_eq!(neg.value_component(), pos.value_component());
        // The lexical_component differs ("-0.0E0" vs "0.0E0"), so leaves differ.
        assert_ne!(neg.lexical_component, pos.lexical_component);
        assert_ne!(neg.leaf(), pos.leaf());
    }

    #[test]
    fn double_nan_payloads_share_the_value_handle() {
        // THE B4 INVARIANT (double): every NaN is one SPARQL-numeric class, so the
        // canonical handle is the same regardless of payload. The host canonicalises
        // the bits before the value handle.
        let c = encode_double(&dbl_lit("NaN")).unwrap();
        assert_eq!(c.value_hook, Fr::from(F64_CANONICAL_NAN));
    }

    #[test]
    fn double_canonical_bits_helper() {
        assert_eq!(canonical_f64_bits(0x8000_0000_0000_0000), 0); // -0.0 -> +0.0
        assert_eq!(canonical_f64_bits(0x7ff0_0000_0000_0001), F64_CANONICAL_NAN); // sNaN
        assert_eq!(canonical_f64_bits(0x7ff8_0000_0000_0000), F64_CANONICAL_NAN); // qNaN
                                                                                  // +inf and a finite value are unchanged.
        assert_eq!(
            canonical_f64_bits(0x7ff0_0000_0000_0000),
            0x7ff0_0000_0000_0000
        );
        assert_eq!(canonical_f64_bits(2.5f64.to_bits()), 2.5f64.to_bits());
    }

    #[test]
    fn double_rejects_non_double_and_bad_lexical() {
        assert!(matches!(
            encode_double(&int_lit("5")),
            Err(DualLeafError::NotValueLane(_))
        ));
        assert!(matches!(
            encode_double(&dbl_lit("not-a-number")),
            Err(DualLeafError::NonCanonicalValue(_))
        ));
    }

    // ---- xsd:decimal class (sq-2ezsx) ----

    fn dec_lit(v: &str) -> Literal {
        Literal::new_typed_literal(v, NamedNode::new(XSD_DECIMAL).unwrap())
    }

    #[test]
    fn decimal_round_trips_to_a_dual_leaf() {
        // "123.45" at fd=2 -> scaled 12345, non-negative.
        let c = encode_decimal(&dec_lit("123.45")).unwrap();
        assert_eq!(c.value_hook, Fr::from(12345u64));
        assert_eq!(c.datatype_const, decimal_datatype_const(2));
        assert_eq!(c.leaf(), c.leaf());
    }

    #[test]
    fn decimal_host_leaf_matches_the_circuit_construction() {
        let c = encode_decimal(&dec_lit("2.50")).unwrap();
        // Mirror the circuit: signed_value = scaled (non-negative).
        let vc = poseidon2::hash(&[c.value_hook, c.datatype_const, Fr::from(LANG_NONE)]);
        let leaf = poseidon2::hash(&[vc, c.lexical_component, Fr::from(TYPE_CODE_LITERAL)]);
        assert_eq!(c.leaf(), leaf);
    }

    #[test]
    fn decimal_negative_folds_sign_into_value_handle() {
        // -2.50 (scaled 250, neg) — the value handle is the field negation of 250.
        let c = encode_decimal(&dec_lit("-2.50")).unwrap();
        assert_eq!(c.value_hook, -Fr::from(250u64));
        // The negative and positive of the same magnitude give DIFFERENT handles.
        let pos = encode_decimal(&dec_lit("2.50")).unwrap();
        assert_ne!(c.value_hook, pos.value_hook);
        assert_ne!(c.leaf(), pos.leaf());
    }

    #[test]
    fn decimal_scale_is_folded_so_5_0_and_5_00_differ() {
        // THE B4 CANONICAL-SCALE INVARIANT: "5.0" (fd=1) and "5.00" (fd=2) are the
        // SAME numeric value but DIFFERENT value handles (the scale is folded into
        // datatype_const), so a value at one scale cannot collide a value at another.
        let a = encode_decimal(&dec_lit("5.0")).unwrap();
        let b = encode_decimal(&dec_lit("5.00")).unwrap();
        assert_eq!(a.value_hook, Fr::from(50u64)); // 5.0 * 10^1
        assert_eq!(b.value_hook, Fr::from(500u64)); // 5.00 * 10^2
        assert_ne!(a.datatype_const, b.datatype_const);
        assert_ne!(a.value_component(), b.value_component());
        assert_ne!(a.leaf(), b.leaf());
    }

    #[test]
    fn decimal_rejects_non_canonical_forms() {
        assert!(matches!(
            encode_decimal(&int_lit("5")),
            Err(DualLeafError::NotValueLane(_))
        ));
        // Integer-only decimal (no point) is rejected (fixed-point member needs fd>=1).
        assert!(matches!(
            encode_decimal(&dec_lit("5")),
            Err(DualLeafError::NonCanonicalValue(_))
        ));
        // Leading zero on the integer part is non-canonical.
        assert!(matches!(
            encode_decimal(&dec_lit("05.0")),
            Err(DualLeafError::NonCanonicalValue(_))
        ));
        // "-0.0" is rejected (no negative zero).
        assert!(matches!(
            encode_decimal(&dec_lit("-0.0")),
            Err(DualLeafError::NonCanonicalValue(_))
        ));
        // Lone "0.0" is canonical (non-negative zero).
        assert_eq!(
            encode_decimal(&dec_lit("0.0")).unwrap().value_hook,
            Fr::from(0u64)
        );
    }
}
