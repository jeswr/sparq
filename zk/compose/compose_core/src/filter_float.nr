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
use sparq_ieee754::f64;

/// Compare two f64 bit patterns under the public operator selector and
/// assert the public verdict.
pub fn filter_f64_check(a_bits: u64, b_bits: u64, op: u32, expected: bool) {
    assert(op <= OP_NE, "unknown comparison operator");
    let a = f64::new(a_bits);
    let b = f64::new(b_bits);
    // All six predicates are evaluated unconditionally -- gate count must not
    // depend on secret values (constant-shape rule).
    let result = if op == OP_LT {
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
    };
    assert_eq(result, expected, "float filter verdict mismatch");
}
