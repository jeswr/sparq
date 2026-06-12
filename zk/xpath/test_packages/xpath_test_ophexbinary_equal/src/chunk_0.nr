//! Test chunk 0 for op:hexBinary-equal
//! Contains 35 tests

use dep::xpath::stub_ophexbinary_equal;

#[test]
fn op_hexbinary_equal2args_1() {
    //  Purpose: Evaluates The 'op:hexBinary-equal' operator     with the arguments set
    // XPath: xs:hexBinary(\"767479716c6a647663\") eq xs:hexBinary(\"76747
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn op_hexbinary_equal2args_2() {
    //  Purpose: Evaluates The 'op:hexBinary-equal' operator     with the arguments set
    // XPath: xs:hexBinary(\"6e7875626264756366\") eq xs:hexBinary(\"76747
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn op_hexbinary_equal2args_3() {
    //  Purpose: Evaluates The 'op:hexBinary-equal' operator     with the arguments set
    // XPath: xs:hexBinary(\"786174616d61616772\") eq xs:hexBinary(\"76747
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn op_hexbinary_equal2args_4() {
    //  Purpose: Evaluates The 'op:hexBinary-equal' operator     with the arguments set
    // XPath: xs:hexBinary(\"767479716c6a647663\") eq xs:hexBinary(\"6e787
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn op_hexbinary_equal2args_5() {
    //  Purpose: Evaluates The 'op:hexBinary-equal' operator     with the arguments set
    // XPath: xs:hexBinary(\"767479716c6a647663\") eq xs:hexBinary(\"78617
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn op_hexbinary_equal2args_6() {
    //  Purpose: Evaluates The 'op:hexBinary-equal' operator     with the arguments set
    // XPath: xs:hexBinary(\"767479716c6a647663\") ne xs:hexBinary(\"76747
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn op_hexbinary_equal2args_7() {
    //  Purpose: Evaluates The 'op:hexBinary-equal' operator     with the arguments set
    // XPath: xs:hexBinary(\"6e7875626264756366\") ne xs:hexBinary(\"76747
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn op_hexbinary_equal2args_8() {
    //             Purpose: Evaluates The 'op:hexBinary-equal' operator     with the ar
    // XPath: xs:hexBinary(\"786174616d61616772\") ne xs:hexBinary(\"76747
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn op_hexbinary_equal2args_9() {
    //             Purpose: Evaluates The 'op:hexBinary-equal' operator     with the ar
    // XPath: xs:hexBinary(\"767479716c6a647663\") ne xs:hexBinary(\"6e787
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn op_hexbinary_equal2args_10() {
    //         Purpose: Evaluates The 'op:hexBinary-equal' operator     with the argume
    // XPath: xs:hexBinary(\"767479716c6a647663\") ne xs:hexBinary(\"78617
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_1() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'and' and '
    // XPath: (xs:hexBinary(\"786174616d61616772\") eq xs:hexBinary(\"7674
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_2() {
    // imple Binary hex operation as part of a    logical expression.  Use 'and' and 'e
    // XPath: (xs:hexBinary(\"786174616d61616772\") eq xs:hexBinary(\"7674
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_3() {
    // Simple Binary hex opeartion as part of argument to fn:not function.
    // XPath: fn:not((xs:hexBinary(\"786174616d61616772\") eq xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_4() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'and' and '
    // XPath: (xs:hexBinary(\"786174616d61616772\") eq xs:hexBinary(\"7674
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_5() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'and' and '
    // XPath: (xs:hexBinary(\"786174616d61616772\") eq xs:hexBinary(\"7674
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_6() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'or' and 'e
    // XPath: (xs:hexBinary(\"786174616d61616772\") eq xs:hexBinary(\"7674
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_7() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'or' and 'e
    // XPath: (xs:hexBinary(\"786174616d61616772\") eq xs:hexBinary(\"7674
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_8() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'and' and '
    // XPath: (xs:hexBinary(\"786174616d61616772\") ne xs:hexBinary(\"7674
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_9() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'and' and '
    // XPath: (xs:hexBinary(\"786174616d61616772\") ne xs:hexBinary(\"7674
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_10() {
    // Simple Binary hex opeartion as part of argument to fn:not function.
    // XPath: fn:not((xs:hexBinary(\"786174616d61616772\") ne xs:hexBinary
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_11() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'and' and '
    // XPath: (xs:hexBinary(\"786174616d61616772\") ne xs:hexBinary(\"7674
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_12() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'and' and '
    // XPath: (xs:hexBinary(\"786174616d61616772\") ne xs:hexBinary(\"7674
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_13() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'or' and 'n
    // XPath: (xs:hexBinary(\"786174616d61616772\") ne xs:hexBinary(\"7674
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn hexbinary_14() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'or' and 'n
    // XPath: (xs:hexBinary(\"786174616d61616772\") ne xs:hexBinary(\"7674
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn k_hexbinaryeq_1() {
    // Compare two hexBinary values.
    // XPath: xs:hexBinary(\"FF\") eq xs:hexBinary(\"ff\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn k_hexbinaryeq_2() {
    // Compare two hexBinary values.
    // XPath: xs:hexBinary(\"FF\") ne xs:hexBinary(\"aa\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn k_hexbinaryeq_3() {
    // Compare two hexBinary values.
    // XPath: not(xs:hexBinary(\"FF\") eq xs:hexBinary(\"aa\"))
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn k_hexbinaryeq_4() {
    // Complex comparison test of wrapped hexBinary/base64Binary constructor functions.
    // XPath: xs:hexBinary(xs:base64Binary(xs:hexBinary(\"03\"))) eq xs:he
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn k_hexbinaryeq_5() {
    // Complex comparison test of wrapped hexBinary/base64Binary constructor functions.
    // XPath: xs:hexBinary(xs:base64Binary(xs:hexBinary(\"03\"))) ne xs:he
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn k_hexbinaryeq_6() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"\") eq xs:hexBinary(\"\")
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn k_hexbinaryeq_7() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"\") eq xs:hexBinary(\"01\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn k_hexbinaryeq_8() {
    // Comparison involving empty hexBinary
    // XPath: xs:hexBinary(\"00\") eq xs:hexBinary(\"\")
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn cbcl_hexbinary_equal_001() {
    //  test equality of xs:hexBinary
    // XPath: declare function local:hexBinary-value($arg as xs:boolean) a
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn cbcl_hexbinary_equal_002() {
    //  test equality of xs:hexBinary
    // XPath: declare function local:hexBinary-value($arg as xs:boolean) a
    // Expected: true
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}

#[test]
fn cbcl_hexbinary_equal_003() {
    //  test equality of xs:hexBinary
    // XPath: declare function local:hexBinary-value($arg as xs:boolean) a
    // Expected: false
    // Calls stub function - will fail until op:hexBinary-equal is implemented
    let _ = stub_ophexbinary_equal();
}
