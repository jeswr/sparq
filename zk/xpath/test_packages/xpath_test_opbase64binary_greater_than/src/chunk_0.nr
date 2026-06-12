//! Test chunk 0 for op:base64Binary-greater-than
//! Contains 29 tests

use dep::xpath::stub_opbase64binary_greater_than;

#[test]
fn base64binary_gt_1() {
    //  Purpose: Evaluates The 'op:base64Binary-greater-than' operator     with the arg
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") gt xs:base64Binary(\"dnR5c
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_2() {
    //  Purpose: Evaluates The 'op:base64Binary-greater-than' operator     with the arg
    // XPath: xs:base64Binary(\"bnh1YmJkdWNm\") gt xs:base64Binary(\"dnR5c
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_3() {
    //  Purpose: Evaluates The 'op:base64Binary-greater-than' operator     with the arg
    // XPath: xs:base64Binary(\"eGF0YW1hYWdy\") gt xs:base64Binary(\"dnR5c
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_4() {
    //  Purpose: Evaluates The 'op:base64Binary-greater-than' operator     with the arg
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") > xs:base64Binary(\"bnh1Ym
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_5() {
    //  Purpose: Evaluates The 'op:base64Binary-greater-than' operator     with the arg
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") > xs:base64Binary(\"eGF0YW
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_6() {
    //  Purpose: Evaluates The 'op:base64Binary-greater-than' operator     with the arg
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") > xs:base64Binary(\"dnR5cW
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_7() {
    //  Purpose: Evaluates The 'op:base64Binary-greater-than' operator     with the arg
    // XPath: xs:base64Binary(\"bnh1YmJkdWNm\") > xs:base64Binary(\"dnR5cW
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_8() {
    //             Purpose: Evaluates The 'op:base64Binary-greater-than' operator     w
    // XPath: xs:base64Binary(\"eGF0YW1hYWdy\") ge  xs:base64Binary(\"dnR5
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_9() {
    //             Purpose: Evaluates The 'op:base64Binary-greater-than' operator     w
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") ge  xs:base64Binary(\"bnh1
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_10() {
    //         Purpose: Evaluates The 'op:base64Binary-greater-than' operator     with
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") ge  xs:base64Binary(\"eGF0
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_11() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'and' and '
    // XPath: (xs:base64Binary(\"eGF0YW1hYWdy\") >= xs:base64Binary(\"dnR5
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_12() {
    // imple Binary hex operation as part of a    logical expression.  Use 'and' and 'e
    // XPath: (xs:base64Binary(\"eGF0YW1hYWdy\") >= xs:base64Binary(\"dnR5
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_13() {
    // Simple Binary hex operation
    // XPath: xs:base64Binary(\"eGF0YW1hYWdy\") >= xs:base64Binary(\"eGF0Y
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_14() {
    // Compare base64Binary values as part of max() function
    // XPath: max((xs:base64Binary('qg=='), xs:base64Binary('uw=='), xs:ba
    // Expected: xs:base64Binary('uw==')
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_16() {
    // Compare two base64Binary values.
    // XPath: xs:base64Binary(\"/w==\") gt xs:base64Binary(\"/w==\")
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_17() {
    // Compare two base64Binary values.
    // XPath: xs:base64Binary(\"/w==\") ge  xs:base64Binary(\"qg==\")
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_18() {
    // Compare two base64Binary values.
    // XPath: xs:base64Binary(\"/w==\") gt xs:base64Binary(\"qg==\")
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_19() {
    // Complex comparison test of wrapped base64Binary/base64Binary constructor functio
    // XPath: xs:base64Binary(xs:base64Binary(xs:base64Binary(\"Aw==\")))
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_20() {
    // Complex comparison test of wrapped base64Binary/base64Binary constructor functio
    // XPath: xs:base64Binary(xs:base64Binary(xs:base64Binary(\"AAE=\")))
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_21() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"\") gt xs:base64Binary(\"\")
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_22() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"\") gt xs:base64Binary(\"AAI=\")
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_23() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"AA==\") gt xs:base64Binary(\"\")
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_24() {
    // Comparison involving implicit conversion of untypedAtomic to base64Binary
    // XPath: xs:untypedAtomic(\"AAE=\") > xs:base64Binary(\"AAI=\")
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_ge_27() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"\") ge  xs:base64Binary(\"\")
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_ge_28() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"\") ge  xs:base64Binary(\"AAE=\")
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_ge_29() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"AA==\") gt xs:base64Binary(\"\")
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_30() {
    //  test equality of xs:base64Binary
    // XPath: declare function local:base64Binary-value($arg as xs:boolean
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_31() {
    //  test equality of xs:base64Binary
    // XPath: declare function local:base64Binary-value($arg as xs:boolean
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}

#[test]
fn base64binary_gt_32() {
    //  test equality of xs:base64Binary
    // XPath: declare function local:base64Binary-value($arg as xs:boolean
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-greater-than is implemented
    let _ = stub_opbase64binary_greater_than();
}
