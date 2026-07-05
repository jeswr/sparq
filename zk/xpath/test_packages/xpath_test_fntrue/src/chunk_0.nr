//! Test chunk 0 for fn:true
//! Contains 4 tests

use dep::xpath::{
    fn_true,
};

#[test]
fn fn_true_2() {
    // Evaluation of 'fn:true' function as an argument to 'fn:not' function.
    // XPath: fn:not(fn:true())
    // Expected: assert-false false
    assert(!(fn_true()) == false);
}

#[test]
fn fn_true_3() {
    // Evaluation of a 'true' function with a logical expression involving the 'and' operator.
    // XPath: fn:true() and fn:true()
    // Expected: assert-true true
    assert((fn_true()) & (fn_true()) == true);
}

#[test]
fn fn_true_4() {
    // Evaluation of a 'fn:true' function with a logical expression involving the 'or' operator.
    // XPath: fn:true() or fn:true()
    // Expected: assert-true true
    assert((fn_true()) | (fn_true()) == true);
}

#[test]
fn fn_true_17() {
    // Evaluation of an 'fn:true' function as an argument to an 'xs:boolean' function.
    // XPath: xs:boolean(fn:true())
    // Expected: assert-true true
    assert(fn_true() == true);
}
