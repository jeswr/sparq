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

/// The `xsd:integer` datatype IRI (the only value-lane datatype class this slice
/// implements; decimal/double are a documented follow-up).
pub const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

fn blake3_field(bytes: &[u8]) -> Fr {
    field_from_hash_bytes(blake3::hash(bytes).as_bytes())
}

/// The `DATATYPE_CONST` for a datatype IRI: `blake3_field(IRI bytes)`. Folded
/// into `value_component` so a cross-datatype value collision (integer 5 vs the
/// bits for 5.0) is impossible.
pub fn datatype_const(datatype_iri: &str) -> Fr {
    blake3_field(datatype_iri.as_bytes())
}

/// The dual-leaf failure for a non-value-lane literal.
#[derive(Debug, PartialEq, Eq)]
pub enum DualLeafError {
    /// The literal is not an `xsd:integer` (this slice's only value-lane class).
    NotValueLane(String),
    /// The lexical form does not canonically parse to a non-negative `u64`
    /// integer value — fail closed (same-leaf co-binding, §6). This is the host
    /// mitigation that keeps sparq's own commitments INV-VL-consistent; it does
    /// NOT bind a malicious external issuer.
    NonCanonicalValue(String),
}

impl std::fmt::Display for DualLeafError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DualLeafError::NotValueLane(t) => {
                write!(f, "not a value-lane literal (xsd:integer only): {}", t)
            }
            DualLeafError::NonCanonicalValue(t) => {
                write!(f, "non-canonical value-lane literal (fail-closed co-binding): {}", t)
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
        assert!(matches!(encode_literal(&dbl), Err(DualLeafError::NotValueLane(_))));
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
}
