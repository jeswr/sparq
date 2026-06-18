//! `xsd:double` FILTER comparison building block over `sparq_ieee754`.
//!
//! STATUS (v1, empirical honesty): a gate-counted, tested BUILDING BLOCK,
//! not yet composable into the proof manifest. The missing piece is the
//! operand binding: an `xsd:double` literal's canonical lexical form (e.g.
//! "1.0E1") would have to be re-derived from the f64 bit pattern in-circuit
//! to anchor `operand_enc` the way `filter_int` does -- float-to-canonical-
//! decimal printing in-circuit is unbudgeted and deferred (alternatives:
//! a numeric value lane in the sparq-zk encoding; see README "sparq-zk API
//! gaps"). Until then, hidden-operand double filters are outside the
//! supported fragment; this module pins the comparison cost and the
//! sparq_ieee754 integration the eventual circuit will reuse.
//!
//! IEEE semantics note (sparql-formal-semantics / xpath F&O): SPARQL numeric
//! comparison over doubles follows IEEE 754 -- NaN is unordered against
//! everything (all of lt/le/gt/ge/eq false, ne true), which is exactly the
//! `sparq_ieee754` predicate contract (deliberately NOT std::cmp::Ord; see
//! noir-circuit-patterns "Standard library traits").

use crate::filter_int::{OP_EQ, OP_GE, OP_GT, OP_LE, OP_LT, OP_NE};
use crate::hashes::{h2, TYPE_CODE_LITERAL};
use sparq_ieee754::f64;

/// Compose the float predicate verdict over two f64 values under the public
/// operator selector. Shared by [`filter_f64_check`] (raw bits) and
/// [`filter_f64_composable_check`] (operand-bound). All six predicates are
/// evaluated unconditionally -- gate count must not depend on secret values
/// (constant-shape rule; noir-circuit-patterns "Filters").
fn f64_verdict(a: f64, b: f64, op: u32) -> bool {
    if op == OP_LT {
        a.lt(b)
    } else if op == OP_LE {
        a.le(b)
    } else if op == OP_GT {
        a.gt(b)
    } else if op == OP_GE {
        a.ge(b)
    } else if op == OP_EQ {
        a.eq(b)
    } else {
        a.ne(b)
    }
}

/// [OPUS-4.8] sq-q7e + sq-tat: MANIFEST-COMPOSABLE hidden-operand `xsd:double`
/// FILTER over the INTEGER-VALUED double fragment.
///
/// # What this makes composable (and why it is sound)
/// `filter_f64_check` (below) is a raw-bits BUILDING BLOCK: its `a_bits` is
/// prover-free, so it could not be assembled into a manifest without letting a
/// prover pick any `a_bits` (a soundness hole). This relation closes that by
/// binding the hidden operand to the COMMITTED literal exactly as `filter_int`
/// does, then deriving the IEEE bits FROM the bound value rather than trusting a
/// free `a_bits`:
///
/// 1. **Operand binding (same mechanism as `filter_int`).** Witness the canonical
///    decimal digits of the value; rebuild the canonical N-Triples token
///    `"<digits>"^^<http://www.w3.org/2001/XMLSchema#double>`, blake3-hash it
///    (blackbox, one compression for short tokens), truncate to the low 31 bytes
///    big-endian (mirroring `sparq_zk::field::field_from_hash_bytes`), and assert
///    `h2(LITERAL, hs) == operand_enc`. So `operand_enc` (the scan-proof anchor)
///    pins the EXACT committed literal -- a prover cannot substitute a different
///    value than the scan disclosed (the `binding_consistency` edge).
/// 2. **Sound bits derivation (no free `a_bits`).** The value is recovered from
///    the SAME digits (`value = sum digit*10^i`), and the IEEE-754 double bit
///    pattern is `f64::from(value)` -- the in-circuit integer->double conversion
///    (`sparq_ieee754`'s `From<u64>`, round-to-nearest-even, exact for the
///    integer fragment `value <= 2^53`). There is NO independent `a_bits` witness
///    the prover could lie with: the bits are a CONSTRAINED function of the bound
///    operand. This is the key difference from the raw building block.
/// 3. **IEEE comparison.** Compare `f64::from(value)` against the public `b_bits`
///    under `op`, assert the public `expected` (IEEE/NaN-correct via
///    `sparq_ieee754`; `b_bits` is the FILTER's constant double, NaN-handled by
///    the predicate contract).
///
/// # Scope (HONEST deferral -- the general fractional/rounded fragment)
/// This supports the INTEGER-VALUED `xsd:double` fragment written in plain
/// canonical decimal-integer lexical form (`"42"`, `"1000"`), where decimal->f64
/// is the exact, soundly-constrained `f64::from(u64)` path. The GENERAL fragment
/// (fractional mantissas, scientific `1.5E3` notation, rounded values needing a
/// full in-circuit decimal->IEEE parser with round-to-nearest-even over an
/// arbitrary lexical form) remains DEFERRED -- that parser is the unbudgeted work
/// the `filter_float` module docs and the README record. The fragment restriction
/// is enforced HERE by the digit-only token (no '.', 'E', or sign), so an
/// out-of-fragment operand is simply unprovable by this member rather than
/// silently mis-bound. This refines the supported fragment honestly
/// (noir-circuit-patterns "When to escalate") instead of shipping an unsound free
/// `a_bits`.
///
/// `D` is the value's decimal digit count (a circuit-family parameter, the blake3
/// blackbox needs a comptime token length -- mirrors `filter_int_d{D}`). `D <= 15`
/// keeps `value <= 10^15 - 1 < 2^53`, so `f64::from` is EXACT (no rounding) over
/// the whole member.
pub fn filter_f64_composable_check<let D: u32>(
    operand_enc: Field,
    op: u32,
    b_bits: u64,
    expected: bool,
    digits: [u8; D],
) {
    std::static_assert(D >= 1, "at least one digit");
    // value <= 10^15 - 1 < 2^53 => f64::from(value) is EXACT (no rounding), so the
    // integer-valued-double fragment is fully sound for this member.
    std::static_assert(
        D <= 15,
        "integer-valued double fragment: value must be exact in f64 (< 2^53)",
    );
    assert(op <= OP_NE, "unknown comparison operator");

    // Canonical decimal digits: ASCII, no leading zero unless D == 1. This also
    // RESTRICTS the composable fragment to plain integer-valued doubles: a token
    // with '.', 'E'/'e', or '-' is not buildable here, so the general fractional/
    // scientific fragment is out of this member (and unprovable, not mis-bound).
    for i in 0..D {
        assert((digits[i] >= 48) & (digits[i] <= 57), "not a decimal digit");
    }
    if D > 1 {
        assert(digits[0] != 48, "non-canonical leading zero");
    }

    // The numeric value (u64; D <= 15 keeps it < 2^53).
    let mut value: u64 = 0;
    for i in 0..D {
        value = value * 10 + ((digits[i] - 48) as u64);
    }

    // (1) Operand binding: rebuild the canonical N-Triples token and re-derive the
    // encoding -- identical mechanism to `filter_int`, but with the xsd:double
    // datatype IRI. Token: '"' + digits + '"' + "^^<...#double>".
    let suffix: [u8; 43] = "^^<http://www.w3.org/2001/XMLSchema#double>".as_bytes();
    let mut token: [u8; D + 45] = [0; D + 45];
    token[0] = 34; // '"'
    for i in 0..D {
        token[i + 1] = digits[i];
    }
    token[D + 1] = 34; // '"'
    for i in 0..43 {
        token[D + 2 + i] = suffix[i];
    }
    let digest: [u8; 32] = std::hash::blake3(token);
    // Big-endian over the low 31 bytes (digest[1..32]) -- mirrors
    // `sparq_zk::field::field_from_hash_bytes` (truncation, 248 bits < modulus).
    let mut hs: Field = 0;
    for i in 1..32 {
        hs = hs * 256 + (digest[i] as Field);
    }
    assert_eq(h2(TYPE_CODE_LITERAL, hs), operand_enc, "operand encoding mismatch");

    // (2) Sound bits: derive the IEEE double FROM the bound value (no free a_bits).
    let a: f64 = f64::from(value);
    let b = f64::new(b_bits);

    // (3) The IEEE comparison verdict.
    let result = f64_verdict(a, b, op);
    assert_eq(result, expected, "float filter verdict mismatch");
}

/// Compare two f64 bit patterns under the public operator selector and
/// assert the public verdict.
pub fn filter_f64_check(a_bits: u64, b_bits: u64, op: u32, expected: bool) {
    assert(op <= OP_NE, "unknown comparison operator");
    let a = f64::new(a_bits);
    let b = f64::new(b_bits);
    let result = f64_verdict(a, b, op);
    assert_eq(result, expected, "float filter verdict mismatch");
}

/// [OPUS-4.8] sq-mslu (sq-7lrq part-2 / sq-lxi7): the GENERAL `xsd:double`
/// decimal -> IEEE-754 conversion CORE for a FRACTIONAL / SCIENTIFIC lexical
/// form, decomposed as `sign * mantissa_int * 10^dexp`.
///
/// # Why this exists (and what it is honestly NOT)
/// The composable `filter_f64_composable_check` above covers only the
/// INTEGER-VALUED `xsd:double` fragment written in plain decimal-integer form
/// (`"42"`), where decimal->f64 is the exact `f64::from(u64)`. The DEFERRED
/// piece (the bead) is the conversion for an arbitrary lexical form -- a
/// fractional mantissa (`123.45`) or scientific notation (`1.5E3`) -- which
/// reduces to: given the (already-parsed) integer mantissa `mantissa_int` and a
/// base-10 exponent `dexp`, recover the correctly-rounded IEEE-754 double of
/// `mantissa_int * 10^dexp`.
///
/// This function implements that conversion for the SOUNDLY-REACHABLE subset
/// and REJECTS (in-circuit `assert`) the rest, rather than ship an unsound
/// shortcut:
///
/// 1. **`dexp >= 0` (integer value).** `mantissa_int * 10^dexp` is an exact
///    integer; if it fits in `u128` then `f64::from(u128)` is a SINGLE
///    correctly-rounded conversion (round-to-nearest-even, the IEEE default) --
///    so the result is the correctly-rounded double of the decimal value. This
///    covers `"1.5E3"` (= 1500), `"100.0"` (= 100), `"1.0E1"` (= 10): a
///    fractional/scientific LEXICAL form whose VALUE is an integer.
/// 2. **`dexp < 0` (genuinely fractional value).** The correctly-rounded double
///    of `mantissa_int / 10^|dexp|` is NOT, in general, recoverable by a
///    composition of the library's individually-correctly-rounded ops:
///    `f64::from(mantissa_int) / f64::from(10^|dexp|)` DOUBLE-ROUNDS (round the
///    numerator, round the denominator, round the quotient) and can differ from
///    the true nearest double by 1 ULP on adversarial inputs. A SOUND general
///    fractional parser needs the full strtod core (exact bignum
///    `mantissa_int * 2^? ` vs `10^|dexp|` comparison with an explicit
///    round-to-nearest-even tie decision -- Clinger / Eisel-Lemire). That core
///    is the unbudgeted work; until it lands (or #514's value-lane signs the
///    IEEE-754 bits directly and removes in-circuit parsing entirely) the
///    fractional-VALUE case is OUT of the sound fragment and is REJECTED here.
///
/// So this member generalises the LEXICAL form (fractional/scientific input is
/// now parseable and bindable) while keeping the VALUE in the
/// single-rounding-exact fragment -- an honest refinement, not an unsound
/// fractional result. The fractional-value case (point 2) is STRUCTURALLY
/// out of this signature: `dexp` is a `u32` and the value is `mantissa_int *
/// 10^dexp`, so a non-integer value (which would need `dexp < 0`) simply cannot
/// be expressed -- the caller's lexical parse maps `123.45` to no representable
/// `(mantissa_int, dexp)` here and must escalate to the (deferred) strtod core.
/// `mantissa_int` and `dexp` are the outputs of the lexical parse (caller-
/// supplied here so the conversion core can be gate-measured and tested in
/// isolation); the operand-binding member that rebuilds the canonical token is
/// tracked separately (it additionally needs in-circuit shortest-round-trip
/// canonical PRINTING -- see the module docs + README "sparq-zk API gaps").
///
/// Returns the IEEE-754 bit pattern of the (sound) double value.
pub fn decimal_to_f64_integer_valued(sign: bool, mantissa_int: u128, dexp: u32) -> u64 {
    // The represented value is mantissa_int * 10^dexp with dexp >= 0 (a
    // genuinely-fractional value would need dexp < 0, which this u32 signature
    // cannot express -- see the doc comment). u128 holds up to
    // ~3.4e38, so >38 decimal scalings cannot stay exact: bound dexp so the
    // exact integer fits in u128 and `f64::from` is the SINGLE correctly-rounded
    // op. The loop runs a fixed comptime number of iterations (constant-shape
    // gate count); `i < dexp` selects the live factors without a data-dependent
    // bound.
    assert(dexp <= 38, "base-10 exponent out of u128 exact range");
    let mut value: u128 = mantissa_int;
    for i in 0..38 {
        let factor: u128 = if (i as u32) < dexp { 10 } else { 1 };
        value = value * factor;
    }

    // Single correctly-rounded conversion (round-to-nearest-even). For value <
    // 2^53 this is exact; above it, it is still the unique nearest double.
    let magnitude: f64 = f64::from(value);
    // Re-apply the sign onto the bit pattern (f64::from is non-negative). The
    // signbit is bit 63; zero stays canonical +0/-0 per IEEE (we keep the
    // witnessed sign, matching the lexical "-0.0" form).
    let bits: u64 = magnitude.bits();
    if sign {
        bits | 0x8000000000000000
    } else {
        bits
    }
}

/// [OPUS-4.8] sq-mslu: FILTER comparison over a fractional/scientific
/// `xsd:double` operand, using [`decimal_to_f64_integer_valued`] for the sound
/// (integer-valued) fragment. Building block (raw `b_bits`, parsed operand
/// components) -- the manifest-composable operand binding is deferred with the
/// canonical-printing gap (module docs).
pub fn filter_f64_decimal_check(
    sign: bool,
    mantissa_int: u128,
    dexp: u32,
    b_bits: u64,
    op: u32,
    expected: bool,
) {
    assert(op <= OP_NE, "unknown comparison operator");
    let a_bits = decimal_to_f64_integer_valued(sign, mantissa_int, dexp);
    let a = f64::new(a_bits);
    let b = f64::new(b_bits);
    let result = f64_verdict(a, b, op);
    assert_eq(result, expected, "float filter verdict mismatch");
}
