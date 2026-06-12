//! Test chunk 0 for fn:true
//! Contains 23 tests

use dep::xpath::stub_fntrue;

#[test]
fn fn_true_2() {
    //  Evaluation of 'fn:true' function as an argument to 'fn:not' function.
    // XPath: fn:not(fn:true())
    // Expected: false
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_3() {
    //  Evaluation of a 'true' function with a logical expression involving the 'and' o
    // XPath: fn:true() and fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_4() {
    //  Evaluation of a 'fn:true' function with a logical expression involving the 'or'
    // XPath: fn:true() or fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_5() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() eq fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_6() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() ne fn:true()
    // Expected: false
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_7() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() lt fn:true()
    // Expected: false
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_8() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() le fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_9() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() gt fn:true()
    // Expected: false
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_10() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() ge fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_11() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() = fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_12() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() != fn:true()
    // Expected: false
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_13() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() < fn:true()
    // Expected: false
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_14() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() <= fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_15() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() > fn:true()
    // Expected: false
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_16() {
    //  Evaluation of an 'fn:true' function with a comparison expression involving the
    // XPath: fn:true() >= fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_17() {
    //  Evaluation of an 'fn:true' function as an argument to an 'xs:boolean' function.
    // XPath: xs:boolean(fn:true())
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_18() {
    //  Evaluation of an 'fn:true' function as an argument to an 'fn:string' function.
    // XPath: fn:string(fn:true())
    // Expected: "true"
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_19() {
    //  Evaluation of an 'fn:true' function as arguments to an 'fn:concat' function.
    // XPath: fn:concat(xs:string(fn:true()),xs:string(fn:true()))
    // Expected: truetrue
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_20() {
    //  Evaluation of an 'fn:true' function as arguments to an 'fn:contains' function.
    // XPath: fn:contains(xs:string(fn:true()),xs:string(fn:true()))
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_21() {
    //  Evaluation of an 'fn:true' function as an argument to an 'fn:string-length' fun
    // XPath: fn:string-length(xs:string(fn:true()))
    // Expected: 4
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn fn_true_22() {
    // true() as a function item
    // XPath: let $t := true#0 return 3[$t()]
    // Expected: 3
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn k_truefunc_2() {
    //  A test whose essence is: `true() eq true()`.
    // XPath: true() eq true()
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}

#[test]
fn k_truefunc_3() {
    //  A test whose essence is: `true()`.
    // XPath: true()
    // Expected: true
    // Calls stub function - will fail until fn:true is implemented
    let _ = stub_fntrue();
}
