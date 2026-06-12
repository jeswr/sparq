//! Test chunk 0 for op:hexBinary-greater-than
//! Contains 29 tests

use dep::xpath::stub_ophexbinary_greater_than;

#[test]
fn hexbinary_gt_1() {
    //  Purpose: Evaluates The 'op:hexBinary-greater-than' operator     with the argume
    // XPath: xs:hexBinary(\"767479716c6a647663\") gt  xs:hexBinary(\"7674
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_2() {
    //  Purpose: Evaluates The 'op:hexBinary-greater-than' operator     with the argume
    // XPath: xs:hexBinary(\"6e7875626264756366\") gt  xs:hexBinary(\"7674
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_3() {
    //  Purpose: Evaluates The 'op:hexBinary-greater-than' operator     with the argume
    // XPath: xs:hexBinary(\"786174616d61616772\") gt  xs:hexBinary(\"7674
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_4() {
    //  Purpose: Evaluates The 'op:hexBinary-greater-than' operator     with the argume
    // XPath: xs:hexBinary(\"767479716c6a647663\") > xs:hexBinary(\"6e7875
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_5() {
    //  Purpose: Evaluates The 'op:hexBinary-greater-than' operator     with the argume
    // XPath: xs:hexBinary(\"767479716c6a647663\") > xs:hexBinary(\"786174
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_6() {
    //  Purpose: Evaluates The 'op:hexBinary-greater-than' operator     with the argume
    // XPath: xs:hexBinary(\"767479716c6a647663\") > xs:hexBinary(\"767479
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_7() {
    //  Purpose: Evaluates The 'op:hexBinary-greater-than' operator     with the argume
    // XPath: xs:hexBinary(\"6e7875626264756366\") > xs:hexBinary(\"767479
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_8() {
    //             Purpose: Evaluates The 'op:hexBinary-greater-than' operator     with
    // XPath: xs:hexBinary(\"786174616d61616772\") ge  xs:hexBinary(\"7674
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_9() {
    //             Purpose: Evaluates The 'op:hexBinary-greater-than' operator     with
    // XPath: xs:hexBinary(\"767479716c6a647663\") ge  xs:hexBinary(\"6e78
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_10() {
    //         Purpose: Evaluates The 'op:hexBinary-greater-than' operator     with the
    // XPath: xs:hexBinary(\"767479716c6a647663\") ge  xs:hexBinary(\"7861
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_11() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'and' and '
    // XPath: (xs:hexBinary(\"786174616d61616772\") >= xs:hexBinary(\"7674
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_12() {
    // imple Binary hex operation as part of a    logical expression.  Use 'and' and 'e
    // XPath: (xs:hexBinary(\"786174616d61616772\") >= xs:hexBinary(\"7674
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_13() {
    // Simple Binary hex operation
    // XPath: xs:hexBinary(\"786174616d61616772\") >= xs:hexBinary(\"78617
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_14() {
    // Compare hexBinary values as part of max() function
    // XPath: max((xs:hexBinary('aa'), xs:hexBinary('bb'), xs:hexBinary('8
    // Expected: xs:hexBinary('bb')
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_16() {
    // Compare two hexBinary values.
    // XPath: xs:hexBinary(\"FF\") gt  xs:hexBinary(\"ff\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_17() {
    // Compare two hexBinary values.
    // XPath: xs:hexBinary(\"FF\") ge  xs:hexBinary(\"aa\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_18() {
    // Compare two hexBinary values.
    // XPath: xs:hexBinary(\"FF\") gt  xs:hexBinary(\"aa\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_19() {
    // Complex comparison test of wrapped hexBinary/base64Binary constructor functions.
    // XPath: xs:hexBinary(xs:base64Binary(xs:hexBinary(\"03\"))) gt  xs:h
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_20() {
    // Complex comparison test of wrapped hexBinary/base64Binary constructor functions.
    // XPath: xs:hexBinary(xs:base64Binary(xs:hexBinary(\"14\"))) ge  xs:h
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_21() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"\") gt  xs:hexBinary(\"\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_22() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"\") gt  xs:hexBinary(\"01\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_23() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"00\") gt  xs:hexBinary(\"\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_24() {
    // Comparison involving implicit conversion of untypedAtomic to hexBinary
    // XPath: xs:untypedAtomic(\"0001\") > xs:hexBinary(\"0002\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_ge_27() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"\") ge  xs:hexBinary(\"\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_ge_28() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"\") ge  xs:hexBinary(\"01\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_ge_29() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"00\") gt  xs:hexBinary(\"\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_30() {
    //  test equality of xs:hexBinary
    // XPath: declare function local:hexBinary-value($arg as xs:boolean) a
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_31() {
    //  test equality of xs:hexBinary
    // XPath: declare function local:hexBinary-value($arg as xs:boolean) a
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}

#[test]
fn hexbinary_gt_32() {
    //  test equality of xs:hexBinary
    // XPath: declare function local:hexBinary-value($arg as xs:boolean) a
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-greater-than is implemented
    let _ = stub_ophexbinary_greater_than();
}
