//! Test chunk 0 for fn:false
//! Contains 5 tests

use dep::xpath::{
    fn_false,
};

#[test]
fn fn_false_2() {
    // Evaliation of 'fn:false' function as argument to fn:not function.
    // XPath: fn:not(fn:false())
    // Expected: assert-true true
    assert(!(fn_false()) == true);
}

#[test]
fn fn_false_3() {
    // Evaluation of a 'false' function with a logical expression involving the 'and' operator.
    // XPath: fn:false() and fn:false()
    // Expected: assert-false false
    assert((fn_false()) & (fn_false()) == false);
}

#[test]
fn fn_false_4() {
    // Evaluation of a 'fn:false' function with a logical expression involving the 'or' operator.
    // XPath: fn:false() or fn:false()
    // Expected: assert-false false
    assert((fn_false()) | (fn_false()) == false);
}

#[test]
fn fn_false_17() {
    // Evaluation of an 'fn:false' function as an argument to an 'xs:boolean' function.
    // XPath: xs:boolean(fn:false())
    // Expected: assert-false false
    assert(fn_false() == false);
}

#[test]
fn k_falsefunc_3() {
    // A test whose essence is: `not(false())`.
    // XPath: not(false())
    // Expected: assert-true true
    assert(!(fn_false()) == true);
}
