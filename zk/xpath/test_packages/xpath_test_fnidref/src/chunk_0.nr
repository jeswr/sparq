//! Test chunk 0 for fn:idref
//! Contains 18 tests

use dep::xpath::stub_fnidref;

#[test]
fn fn_idref_dtd_5() {
    //  Evaluation of fn:idref with given ID matching a single element.
    // XPath: fn:idref(\"id1\",/IDS[1])/name(..)
    // Expected: elementwithidrefattr-1
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_7() {
    //  Evaluation of fn:idref with given ID matching multiple elements.
    // XPath: fn:idref(\"id4\", /IDS[1])/name(..)
    // Expected: elementwithidrefattr-4
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_9() {
    //  Evaluation of fn:idref with multiple ID, but only one matching one element.
    // XPath: fn:idref((\"id1\", \"nomatching\"), /IDS[1])/name(..)
    // Expected: elementwithidrefattr-1
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_10() {
    //  Evaluation of fn:idref with multiple ID, but none matching one element.
    // XPath: fn:count(fn:idref(\"nomatching1 nomatching2\", /IDS[1]))
    // Expected: 0
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_11() {
    //  Evaluation of fn:idref with ID set to empty string.
    // XPath: fn:count(fn:idref(\"\", /IDS[1]))
    // Expected: 0
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_12() {
    //  Evaluation of fn:idref used as an argument to function fn:node-name()
    // XPath: fn:node-name(fn:idref(\"id2\", /IDS[1]))
    // Expected: anIdRef
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_13() {
    //  Evaluation of fn:idref used as part of a node expression ('is' operand). Compar
    // XPath: (fn:idref(\"id1\", /IDS[1])) is (fn:idref(\"id1\", /IDS[1]))
    // Expected: true
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_14() {
    //  Evaluation of fn:idref used as part of a node expression ('is' operand). Compar
    // XPath: (fn:idref(\"id1\", /IDS[1])) is (fn:idref(\"id2\", /IDS[1]))
    // Expected: false
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_15() {
    //  Evaluation of fn:idref for which ID list have the same value for all its member
    // XPath: count(fn:idref((\"id1\",\"id1\"), /IDS[1]))
    // Expected: 1
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_16() {
    //  Evaluation of fn:idref for which ID list have the same value for all its member
    // XPath: count(fn:idref((\"id1\",\"ID1\"), /IDS[1]))
    // Expected: 1
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_17() {
    //  Evaluation of fn:idref for which ID uses the fn lower case function.
    // XPath: fn:idref(fn:lower-case(\"ID1\"), /IDS[1])/name(..)
    // Expected: elementwithidrefattr-1
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_18() {
    //  Evaluation of fn:idref for which ID uses the fn:upper-case function.
    // XPath: fn:idref(fn:upper-case(\"id5\"), /IDS[1])/name(..)
    // Expected: elementwithidrefattr-6
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_19() {
    //  Evaluation of fn:idref for which $arg uses the fn:concat function.
    // XPath: fn:idref(fn:concat(\"i\",\"d1\"), /IDS[1])/name(..)
    // Expected: elementwithidrefattr-1
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_20() {
    //  Evaluation of fn:idref for which $arg uses the xs:string function.
    // XPath: fn:idref(xs:string(\"id1\"), /IDS[1])/name(..)
    // Expected: elementwithidrefattr-1
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_21() {
    //  Evaluation of fn:idref for which $arg uses the fn:string-join function.
    // XPath: fn:idref(fn:string-join((\"id\",\"1\"),\"\"), /IDS[1])/name(
    // Expected: elementwithidrefattr-1
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn fn_idref_dtd_25() {
    // Evaluation of fn:idref with no second argument.
    // XPath: fn:idref(\"id1\")/name(..)
    // Expected: elementwithidrefattr-1
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn cbcl_idref_001() {
    //  test fn:idref on the empty sequence
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: true
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}

#[test]
fn cbcl_idref_003() {
    //  test fn:idref in an axis expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: true
    // Calls stub function - will fail until fn:idref is implemented
    let _ = stub_fnidref();
}
