//! Test chunk 0 for fn:false
//! Contains 22 tests

use dep::xpath::stub_fnfalse;

#[test]
fn fn_false_2() {
    //  Evaliation of 'fn:false' function as argument to fn:not function.
    // XPath: fn:not(fn:false())
    // Expected: true
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_3() {
    //  Evaluation of a 'false' function with a logical expression involving the 'and'
    // XPath: fn:false() and fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_4() {
    //  Evaluation of a 'fn:false' function with a logical expression involving the 'or
    // XPath: fn:false() or fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_5() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() eq fn:false()
    // Expected: true
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_6() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() ne fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_7() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() lt fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_8() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() le fn:false()
    // Expected: true
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_9() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() gt fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_10() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() ge fn:false()
    // Expected: true
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_11() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() = fn:false()
    // Expected: true
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_12() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() != fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_13() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() < fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_14() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() <= fn:false()
    // Expected: true
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_15() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() > fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_16() {
    //  Evaluation of an 'fn:false' function with a comparison expression involving the
    // XPath: fn:false() >= fn:false()
    // Expected: true
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_17() {
    //  Evaluation of an 'fn:false' function as an argument to an 'xs:boolean' function
    // XPath: xs:boolean(fn:false())
    // Expected: false
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_18() {
    //  Evaluation of an 'fn:false' function as an argument to an 'fn:string' function.
    // XPath: fn:string(fn:false())
    // Expected: "false"
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_19() {
    //  Evaluation of an 'fn:false' function as arguments to an 'fn:concat' function.
    // XPath: fn:concat(xs:string(fn:false()),xs:string(fn:false()))
    // Expected: falsefalse
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_20() {
    //  Evaluation of an 'fn:false' function as arguments to an 'fn:contains' function.
    // XPath: fn:contains(xs:string(fn:false()),xs:string(fn:false()))
    // Expected: true
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn fn_false_21() {
    //  Evaluation of an 'fn:false' function as an argument to an 'fn:string-length' fu
    // XPath: fn:string-length(xs:string(fn:false()))
    // Expected: 5
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn k_falsefunc_2() {
    //  A test whose essence is: `false() eq false()`.
    // XPath: false() eq false()
    // Expected: true
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}

#[test]
fn k_falsefunc_3() {
    //  A test whose essence is: `not(false())`.
    // XPath: not(false())
    // Expected: true
    // Calls stub function - will fail until fn:false is implemented
    let _ = stub_fnfalse();
}
