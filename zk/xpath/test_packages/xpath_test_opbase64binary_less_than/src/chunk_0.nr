//! Test chunk 0 for op:base64Binary-less-than
//! Contains 29 tests

use dep::xpath::stub_opbase64binary_less_than;

#[test]
fn base64binary_lt_1() {
    //  Purpose: Evaluates The 'op:base64Binary-less-than' operator     with the argume
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") lt xs:base64Binary(\"dnR5c
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_2() {
    //  Purpose: Evaluates The 'op:base64Binary-less-than' operator     with the argume
    // XPath: xs:base64Binary(\"bnh1YmJkdWNm\") lt xs:base64Binary(\"dnR5c
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_3() {
    //  Purpose: Evaluates The 'op:base64Binary-less-than' operator     with the argume
    // XPath: xs:base64Binary(\"eGF0YW1hYWdy\") lt xs:base64Binary(\"dnR5c
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_4() {
    //  Purpose: Evaluates The 'op:base64Binary-less-than' operator     with the argume
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") < xs:base64Binary(\"bnh1Ym
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_5() {
    //  Purpose: Evaluates The 'op:base64Binary-less-than' operator     with the argume
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") < xs:base64Binary(\"eGF0YW
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_6() {
    //  Purpose: Evaluates The 'op:base64Binary-less-than' operator     with the argume
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") < xs:base64Binary(\"dnR5cW
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_7() {
    //  Purpose: Evaluates The 'op:base64Binary-less-than' operator     with the argume
    // XPath: xs:base64Binary(\"bnh1YmJkdWNm\") < xs:base64Binary(\"dnR5cW
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_8() {
    //             Purpose: Evaluates The 'op:base64Binary-less-than' operator     with
    // XPath: xs:base64Binary(\"eGF0YW1hYWdy\") le xs:base64Binary(\"dnR5c
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_9() {
    //             Purpose: Evaluates The 'op:base64Binary-less-than' operator     with
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") le xs:base64Binary(\"bnh1Y
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_10() {
    //         Purpose: Evaluates The 'op:base64Binary-less-than' operator     with the
    // XPath: xs:base64Binary(\"dnR5cWxqZHZj\") le xs:base64Binary(\"eGF0Y
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_11() {
    // Simple Binary hex operation as part of a    logical expression.  Use 'and' and '
    // XPath: (xs:base64Binary(\"eGF0YW1hYWdy\") <= xs:base64Binary(\"dnR5
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_12() {
    // imple Binary hex operation as part of a    logical expression.  Use 'and' and 'e
    // XPath: (xs:base64Binary(\"eGF0YW1hYWdy\") <= xs:base64Binary(\"dnR5
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_13() {
    // Simple Binary hex operation
    // XPath: xs:base64Binary(\"eGF0YW1hYWdy\") <= xs:base64Binary(\"eGF0Y
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_14() {
    // Compare base64Binary values as part of min() function
    // XPath: min((xs:base64Binary('qg=='), xs:base64Binary('uw=='), xs:ba
    // Expected: xs:base64Binary('iA==')
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_16() {
    // Compare two base64Binary values.
    // XPath: xs:base64Binary(\"/w==\") lt xs:base64Binary(\"/w==\")
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_17() {
    // Compare two base64Binary values.
    // XPath: xs:base64Binary(\"/w==\") le xs:base64Binary(\"qg==\")
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_18() {
    // Compare two base64Binary values.
    // XPath: xs:base64Binary(\"/w==\") lt xs:base64Binary(\"qg==\")
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_19() {
    // Complex comparison test of wrapped base64Binary/base64Binary constructor functio
    // XPath: xs:base64Binary(xs:base64Binary(xs:base64Binary(\"Aw==\")))
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_20() {
    // Complex comparison test of wrapped base64Binary/base64Binary constructor functio
    // XPath: xs:base64Binary(xs:base64Binary(xs:base64Binary(\"AAE=\")))
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_21() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"\") lt xs:base64Binary(\"\")
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_22() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"\") lt xs:base64Binary(\"AAI=\")
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_23() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"AA==\") lt xs:base64Binary(\"\")
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_24() {
    // Comparison involving implicit conversion of untypedAtomic to base64Binary
    // XPath: xs:untypedAtomic(\"AAE=\") < xs:base64Binary(\"AAI=\")
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_le_27() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"\") le xs:base64Binary(\"\")
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_le_28() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"\") le xs:base64Binary(\"AAE=\")
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_le_29() {
    // Comparison involving empty base64Binary
    // XPath: xs:base64Binary(\"AA==\") lt xs:base64Binary(\"\")
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_30() {
    //  test equality of xs:base64Binary
    // XPath: declare function local:base64Binary-value($arg as xs:boolean
    // Expected: false
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_31() {
    //  test equality of xs:base64Binary
    // XPath: declare function local:base64Binary-value($arg as xs:boolean
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}

#[test]
fn base64binary_lt_32() {
    //  test equality of xs:base64Binary
    // XPath: declare function local:base64Binary-value($arg as xs:boolean
    // Expected: true
    // Calls stub function - will fail until op:base64Binary-less-than is implemented
    let _ = stub_opbase64binary_less_than();
}
