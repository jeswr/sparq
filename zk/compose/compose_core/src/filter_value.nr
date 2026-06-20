//! Dual-leaf value-lane FILTER over a committed numeric literal (sq-xojl).
//!
//! THE GATE WIN, and THE PRICE. The blake3-bound `filter_int` family
//! (`filter_int.nr`) re-derives the operand binding by witnessing the digit
//! bytes, rebuilding the canonical N-Triples token, and hashing it with the
//! in-circuit `blake3` BLACKBOX (~14k gates -- the dominant cost). This member
//! AVOIDS that hash entirely: it proves a value-FILTER against the **dual-leaf**
//! commitment shape (`research/zk-field-native-encoding.md` S3.1), in which the
//! committed literal leaf is
//!
//! ```text
//! Enc_literal = h3(value_component, lexical_component, TYPE_CODE_LITERAL)
//!   value_component   = h3(VALUE_HOOK, DATATYPE_CONST, LANG_CONST)
//!   lexical_component = field_from_hash(blake3(canonical N-Triples token))   // OFF-circuit
//! ```
//!
//! so the FILTER binds the operand with TWO Poseidon2 permutations over the
//! witnessed `VALUE_HOOK` instead of one in-circuit blake3 -- and takes
//! `lexical_component` as a FREE witness (it never reconstructs or hashes the
//! lexical bytes; the scan-anchored leaf binding pins it). This is the
//! "value handle" comparison the dual-leaf method exists to enable.
//!
//! ## INV-VL DOWNGRADE -- DOCUMENTED RISK (load-bearing, read this)
//!
//! The blake3 family enforces, IN-CIRCUIT against an arbitrary committer
//! (including a malicious *trusted* issuer), the invariant **INV-VL**: *the
//! compared value equals the parse of the committed lexical bytes* -- because
//! the value AND the leaf binding both derive from the SAME witnessed digit
//! array (`filter_int.nr:67-92`). This member **REMOVES INV-VL**: `VALUE_HOOK`
//! and `lexical_component` are INDEPENDENT witnesses, so the leaf binds *that
//! the leaf contains both* but NOT that `VALUE_HOOK == parse(lexical)`. Binding
//! them in-circuit would re-introduce the exact blake3-over-the-token parse the
//! value handle exists to avoid -- defeating the gate win. So a malicious
//! *trusted* issuer can mint ONE signed credential whose `VALUE_HOOK` answers a
//! value-FILTER as 18 while its `lexical_component` answers
//! sameTerm/DISTINCT/join as "5". This is IMPOSSIBLE in the blake3 family. It is
//! a STRICT trust-model regression for the value-FILTER lane: value<->lexical
//! agreement moves from MACHINE-ENFORCED to TRUSTED-ISSUER-HONESTY. No
//! *untrusted* party gains anything (the issuer-signature chain is intact). The
//! maintainer ACKNOWLEDGED AND ACCEPTED this attack at research grade (#769) and
//! asked for it built WITH documentation. It is registered as an open external-
//! audit obligation (gap CR-G8 / sq-qhy4). The whole ZK estate is remediated +
//! internally re-audited but NOT externally audited -- nothing here is a
//! soundness or privacy guarantee.
//!
//! Honest mitigations (NOT substitutes for the audit): host-side same-leaf
//! co-binding at ingest (`research/zk-field-native-encoding.md` S6, fail-closed)
//! makes sparq's OWN commitments INV-VL-consistent; canonical-form issuance
//! (S5.6) is the named precondition that restores INV-VL for conforming issuers.
//!
//! ## B1 (range-decomposition) is INSTANTIATED, not named
//!
//! `VALUE_HOOK` arrives as a `Field`. A `Field` is NOT a `u64`: a prover could
//! supply a value congruent to a small number mod the BN254 modulus (modular
//! wrap -- reject-list (i)). So before any comparison this member RANGE-
//! DECOMPOSES `VALUE_HOOK` into the typed `u64` domain (`assert_max_bit_size::<64>()`,
//! the in-circuit range proof used across this stack) and the SAME
//! range-decomposed value feeds BOTH the operand binding (`value_component`) AND
//! the typed comparison -- so A1 binds "the same in-range value the comparison
//! uses", not merely "the same field". Omitting B1 would silently downgrade
//! in-range-ness from a prover-side circuit guarantee to an issuer/ingest
//! assumption (the S5.4 (2) escalation). This member does NOT omit it.
//!
//! ## B4 (canonical-form) for the integer datatype class
//!
//! `xsd:integer` VALUE_HOOK is the integer value itself (NOT the digit string),
//! so the value handle is canonical BY CONSTRUCTION: there is exactly one
//! `u64` magnitude per value, and `"05"` and `"5"` map to the SAME VALUE_HOOK.
//! That is precisely the value-collapse the dual leaf carries the
//! `lexical_component` to disambiguate for IDENTITY ops -- and exactly why
//! reject-list (v) (no identity op may read `value_component`) is load-bearing
//! and structurally enforced OUTSIDE this member (the dispatch matrix, sq-cfmv).
//! For this value-FILTER member, the single-encoding property B4 needs holds:
//! one value <-> one VALUE_HOOK, no second integer encoding. The decimal/double
//! datatype classes (whose VALUE_HOOK is many-to-one on the TERM and whose B4
//! needs an explicit canonical-scale / IEEE-bit bind) are a documented follow-up
//! (the integer class is the load-bearing first member; sq follow-up bead).
//!
//! `operand_enc` is PUBLIC: the hidden column's disclosed term encoding in the
//! manifest binding table, which the scan proof ties to a committed triple slot.
//! Composition is unchanged from `filter_int`: the verifier checks the same
//! `operand_enc` appears in both the scan proof's rows and this proof's public
//! inputs (the binding_consistency edge).

use crate::hashes::{h3, TYPE_CODE_LITERAL};

/// Comparison operator selectors (public input) -- identical encoding to
/// `filter_int` (`OP_LT=0 .. OP_NE=5`), reused so the host/verifier op mapping
/// is shared across the value lane and the blake3 lane.
pub global OP_LT: u32 = 0;
pub global OP_LE: u32 = 1;
pub global OP_GT: u32 = 2;
pub global OP_GE: u32 = 3;
pub global OP_EQ: u32 = 4;
pub global OP_NE: u32 = 5;

/// A reserved "no language" tag constant for the value lane's `LANG_CONST`
/// slot, mirroring the host `sparq_zk::dual_leaf::LANG_NONE`. A real lang tag
/// is `blake3(lang)`; numeric datatypes have no language, so they fold this
/// fixed sentinel. Distinct from any plausible `blake3` output by construction
/// (a small reserved field tag), so a value_component can never collide a
/// language-tagged one.
pub global LANG_NONE: Field = 1;

/// Range-decompose `v` (a `Field` witness) into a `u64`, proving it lies in the
/// `u64` domain with NO modular wrap (B1). `assert_max_bit_size::<64>()` is the
/// in-circuit range proof (the idiom used across this stack, e.g. the vendored
/// `json_parser`): it constrains `v < 2^64`, so the subsequent `v as u64` is the
/// exact value with no truncation / modular reduction. A prover supplying a
/// `VALUE_HOOK` congruent to a small value mod the BN254 modulus is UNPROVABLE.
fn range_decompose_u64(v: Field) -> u64 {
    v.assert_max_bit_size::<64>();
    v as u64
}

/// Evaluate the comparison verdict for `value <op> bound`. Both arms are paid
/// unconditionally (flatten_cfg; noir-optimisation S13). Shared with the blake3
/// lane's verdict shape so the disclosed `expected` semantics are identical.
fn integer_verdict(value: u64, op: u32, bound: u64) -> bool {
    assert(op <= OP_NE, "unknown comparison operator");
    let lt = value < bound;
    let eq = value == bound;
    if op == OP_LT {
        lt
    } else if op == OP_LE {
        lt | eq
    } else if op == OP_GT {
        !lt & !eq
    } else if op == OP_GE {
        !lt
    } else if op == OP_EQ {
        eq
    } else {
        !eq
    }
}

/// Dual-leaf value-FILTER over a committed NON-NEGATIVE `xsd:integer`.
///
/// Proves the disclosed verdict `expected` for `value <op> bound` where `value`
/// is the committed literal's hidden numeric value, WITHOUT an in-circuit
/// blake3 (the gate win) -- by binding the witnessed `VALUE_HOOK` into the
/// dual-leaf `operand_enc` via two Poseidon2 permutations and range-decomposing
/// it (B1) before the typed comparison.
///
/// - `operand_enc`     : PUBLIC. The scan-anchored dual-leaf literal encoding.
/// - `op`, `bound`     : PUBLIC. The FILTER operator + constant.
/// - `expected`        : PUBLIC. The disclosed verdict.
/// - `value_hook`      : PRIVATE. The numeric value handle (a `Field`).
/// - `lexical_component`: PRIVATE. The OFF-circuit blake3 lexical hash, carried
///   as a FREE witness -- the leaf binding pins it; this member NEVER hashes the
///   lexical bytes (the saving). Its INDEPENDENCE from `value_hook` is exactly
///   the removed INV-VL (module doc).
/// - `datatype_const`  : PUBLIC. `blake3(datatype IRI)` as a field, a constant
///   for the known FILTER datatypes; folded into `value_component` so a cross-
///   datatype value collision (integer 5 vs the bits for 5.0) cannot occur.
pub fn filter_value_dl_int(
    operand_enc: Field,
    op: u32,
    bound: u64,
    expected: bool,
    value_hook: Field,
    lexical_component: Field,
    datatype_const: Field,
) {
    // B1: range-decompose VALUE_HOOK to the typed u64 domain, no modular wrap.
    // The SAME decomposed value feeds the binding AND the comparison.
    let value: u64 = range_decompose_u64(value_hook);

    // value_component = h3(VALUE_HOOK, DATATYPE_CONST, LANG_NONE).
    // Rebind from the RANGE-DECOMPOSED value (cast back to Field) so the bound
    // handle is provably the in-range one the comparison uses, not a free field.
    let value_canonical: Field = value as Field;
    let value_component = h3(value_canonical, datatype_const, LANG_NONE);

    // leaf = h3(value_component, lexical_component, TYPE_CODE_LITERAL).
    let leaf = h3(value_component, lexical_component, TYPE_CODE_LITERAL);

    // A1: bind to the scan-anchored committed leaf. No in-circuit blake3.
    assert_eq(leaf, operand_enc, "dual-leaf operand encoding mismatch");

    // B2/B3: the typed comparison verdict, asserted equal to the disclosed one.
    let result = integer_verdict(value, op, bound);
    assert_eq(result, expected, "filter verdict mismatch");
}

#[test]
fn accepts_true_ge_18() {
    // age = 18, FILTER age >= 18 -> true. Build the dual-leaf operand_enc the
    // SAME way the member does, then prove the verdict.
    let value_hook: Field = 18;
    let dt: Field = 0xabc; // a stand-in xsd:integer DATATYPE_CONST
    let lexical: Field = 0xdef; // a stand-in lexical_component (blake3 off-circuit)
    let vc = h3(value_hook, dt, LANG_NONE);
    let operand_enc = h3(vc, lexical, TYPE_CODE_LITERAL);
    filter_value_dl_int(operand_enc, OP_GE, 18, true, value_hook, lexical, dt);
}

#[test]
fn accepts_false_ge_18_when_under() {
    // age = 17, FILTER age >= 18 -> false (disclosed verdict false is provable).
    let value_hook: Field = 17;
    let dt: Field = 0xabc;
    let lexical: Field = 0xdef;
    let vc = h3(value_hook, dt, LANG_NONE);
    let operand_enc = h3(vc, lexical, TYPE_CODE_LITERAL);
    filter_value_dl_int(operand_enc, OP_GE, 18, false, value_hook, lexical, dt);
}

#[test(should_fail_with = "filter verdict mismatch")]
fn rejects_lying_verdict() {
    // age = 17 but the prover claims age >= 18 is true -> unprovable.
    let value_hook: Field = 17;
    let dt: Field = 0xabc;
    let lexical: Field = 0xdef;
    let vc = h3(value_hook, dt, LANG_NONE);
    let operand_enc = h3(vc, lexical, TYPE_CODE_LITERAL);
    filter_value_dl_int(operand_enc, OP_GE, 18, true, value_hook, lexical, dt);
}

#[test(should_fail_with = "dual-leaf operand encoding mismatch")]
fn rejects_unbound_value_hook() {
    // A prover supplies a value_hook NOT bound to operand_enc (the operand_enc
    // was built for value 18; the prover tries to compare value 5). A3/A1: the
    // value handle is anchored to the scan-bound leaf, so a free value is
    // unprovable. This is the binding that keeps the removed INV-VL a matter of
    // ISSUER trust only -- an UNTRUSTED prover still cannot swap the value.
    let dt: Field = 0xabc;
    let lexical: Field = 0xdef;
    let vc = h3(18, dt, LANG_NONE);
    let operand_enc = h3(vc, lexical, TYPE_CODE_LITERAL);
    // value_hook = 5 does not reproduce operand_enc.
    filter_value_dl_int(operand_enc, OP_GE, 5, true, 5, lexical, dt);
}

#[test(should_fail_with = "dual-leaf operand encoding mismatch")]
fn rejects_wrong_lexical_component() {
    // The lexical_component is a FREE witness, but the leaf binding pins it: a
    // prover cannot present a different lexical_component than the one committed.
    // (This is the binding; the removed INV-VL is that a *trusted issuer* may
    // have committed a lexical INCONSISTENT with value_hook -- documented risk,
    // NOT a prover-side forgery, which this rejects.)
    let value_hook: Field = 18;
    let dt: Field = 0xabc;
    let committed_lexical: Field = 0xdef;
    let vc = h3(value_hook, dt, LANG_NONE);
    let operand_enc = h3(vc, committed_lexical, TYPE_CODE_LITERAL);
    let attacker_lexical: Field = 0x999;
    filter_value_dl_int(operand_enc, OP_GE, 18, true, value_hook, attacker_lexical, dt);
}

#[test(should_fail_with = "call to assert_max_bit_size")]
fn rejects_modular_wrap_value_hook() {
    // B1: a value_hook of 2^64 (just past the u64 domain) must be rejected by the
    // range-decomposition, NOT silently reduced. Without B1 a prover could supply
    // a field congruent to a small value mod p.
    let value_hook: Field = 0x10000000000000000; // 2^64
    let dt: Field = 0xabc;
    let lexical: Field = 0xdef;
    let vc = h3(value_hook, dt, LANG_NONE);
    let operand_enc = h3(vc, lexical, TYPE_CODE_LITERAL);
    filter_value_dl_int(operand_enc, OP_GE, 0, true, value_hook, lexical, dt);
}

#[test(should_fail_with = "unknown comparison operator")]
fn rejects_unknown_op() {
    let value_hook: Field = 18;
    let dt: Field = 0xabc;
    let lexical: Field = 0xdef;
    let vc = h3(value_hook, dt, LANG_NONE);
    let operand_enc = h3(vc, lexical, TYPE_CODE_LITERAL);
    filter_value_dl_int(operand_enc, 6, 18, true, value_hook, lexical, dt);
}
