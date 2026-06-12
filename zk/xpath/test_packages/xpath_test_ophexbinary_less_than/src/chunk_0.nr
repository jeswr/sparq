//! Test chunk 0 for op:hexBinary-less-than
//! Contains 29 tests

use dep::xpath::stub_ophexbinary_less_than;

#[test]
fn hexbinary_lt_1() {
    //  Purpose: Evaluates The 'op:hexBinary-less-than' operator     with the arguments
    // XPath: xs:hexBinary(\"767479716c6a647663\") lt xs:hexBinary(\"76747
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_2() {
    //  Purpose: Evaluates The 'op:hexBinary-less-than' operator     with the arguments
    // XPath: xs:hexBinary(\"6e7875626264756366\") lt xs:hexBinary(\"76747
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_3() {
    //  Purpose: Evaluates The 'op:hexBinary-less-than' operator     with the arguments
    // XPath: xs:hexBinary(\"786174616d61616772\") lt xs:hexBinary(\"76747
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_4() {
    //  Purpose: Evaluates The 'op:hexBinary-less-than' operator     with the arguments
    // XPath: xs:hexBinary(\"767479716c6a647663\") < xs:hexBinary(\"6e7875
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_5() {
    //  Purpose: Evaluates The 'op:hexBinary-less-than' operator     with the arguments
    // XPath: xs:hexBinary(\"767479716c6a647663\") < xs:hexBinary(\"786174
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_6() {
    //  Purpose: Evaluates The 'op:hexBinary-less-than' operator     with the arguments
    // XPath: xs:hexBinary(\"767479716c6a647663\") < xs:hexBinary(\"767479
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_7() {
    //  Purpose: Evaluates The 'op:hexBinary-less-than' operator     with the arguments
    // XPath: xs:hexBinary(\"6e7875626264756366\") < xs:hexBinary(\"767479
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_8() {
    //             Purpose: Evaluates The 'op:hexBinary-less-than' operator     with th
    // XPath: xs:hexBinary(\"786174616d61616772\") le xs:hexBinary(\"76747
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_9() {
    //             Purpose: Evaluates The 'op:hexBinary-less-than' operator     with th
    // XPath: xs:hexBinary(\"767479716c6a647663\") le xs:hexBinary(\"6e787
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_10() {
    //         Purpose: Evaluates The 'op:hexBinary-less-than' operator     with the ar
    // XPath: xs:hexBinary(\"767479716c6a647663\") le xs:hexBinary(\"78617
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_11() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'and' and '
    // XPath: (xs:hexBinary(\"786174616d61616772\") <= xs:hexBinary(\"7674
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_12() {
    // imple Binary hex operation as part of a    logical expression.  Use 'and' and 'e
    // XPath: (xs:hexBinary(\"786174616d61616772\") <= xs:hexBinary(\"7674
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_13() {
    // Simple Binary hex operation
    // XPath: xs:hexBinary(\"786174616d61616772\") <= xs:hexBinary(\"78617
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_14() {
    // Compare hexBinary values as part of min() function
    // XPath: min((xs:hexBinary('aa'), xs:hexBinary('bb'), xs:hexBinary('8
    // Expected: xs:hexBinary('88')
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_16() {
    // Compare two hexBinary values.
    // XPath: xs:hexBinary(\"FF\") lt xs:hexBinary(\"ff\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_17() {
    // Compare two hexBinary values.
    // XPath: xs:hexBinary(\"FF\") le xs:hexBinary(\"aa\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_18() {
    // Compare two hexBinary values.
    // XPath: xs:hexBinary(\"FF\") lt xs:hexBinary(\"aa\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_19() {
    // Complex comparison test of wrapped hexBinary/base64Binary constructor functions.
    // XPath: xs:hexBinary(xs:base64Binary(xs:hexBinary(\"03\"))) lt xs:he
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_20() {
    // Complex comparison test of wrapped hexBinary/base64Binary constructor functions.
    // XPath: xs:hexBinary(xs:base64Binary(xs:hexBinary(\"03\"))) le xs:he
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_21() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"\") lt xs:hexBinary(\"\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_22() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"\") lt xs:hexBinary(\"01\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_23() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"00\") lt xs:hexBinary(\"\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_24() {
    // Comparison involving implicit conversion of untypedAtomic to hexBinary
    // XPath: xs:untypedAtomic(\"0001\") < xs:hexBinary(\"0002\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_le_27() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"\") le xs:hexBinary(\"\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_le_28() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"\") le xs:hexBinary(\"01\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_le_29() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"00\") lt xs:hexBinary(\"\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_30() {
    //  test equality of xs:hexBinary
    // XPath: declare function local:hexBinary-value($arg as xs:boolean) a
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_31() {
    //  test equality of xs:hexBinary
    // XPath: declare function local:hexBinary-value($arg as xs:boolean) a
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}

#[test]
fn hexbinary_lt_32() {
    //  test equality of xs:hexBinary
    // XPath: declare function local:hexBinary-value($arg as xs:boolean) a
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-less-than is implemented
    let _ = stub_ophexbinary_less_than();
}
