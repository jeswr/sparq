//! Test chunk 0 for fn:codepoint-equal
//! Contains 30 tests

use dep::xpath::stub_fncodepoint_equal;

#[test]
fn fn_codepoint_equal_3() {
    //  Evaluation of an 'fn:codepoint-equal' with arguments set to zero length string.
    // XPath: fn:codepoint-equal(\"\",\"\")
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_4() {
    //  Evaluation of an 'fn:codepoint-equal' with arguments set to same value 'a'.
    // XPath: fn:codepoint-equal(\"a\",\"a\")
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_5() {
    //  Evaluation of an 'fn:codepoint-equal' with arguments set to different values 'a
    // XPath: fn:codepoint-equal(\"a\",\"b\")
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_6() {
    //  Evaluation of an 'fn:codepoint-equal' as argument to fn:not. Returns true
    // XPath: fn:not(fn:codepoint-equal(\"a\",\"b\"))
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_7() {
    //  Evaluation of an 'fn:codepoint-equal' as argument to fn:not. Returns false
    // XPath: fn:not(fn:codepoint-equal(\"a\",\"a\"))
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_8() {
    //  Evaluation of an 'fn:codepoint-equal' whose arguments use fn:string() for a num
    // XPath: fn:codepoint-equal(fn:string(1),fn:string(1))
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_9() {
    //  Evaluation of an 'fn:codepoint-equal' whose arguments use fn:string() for a str
    // XPath: fn:codepoint-equal(fn:string(\"aa\"),fn:string(\"aa\"))
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_12() {
    //  Evaluation of an 'fn:codepoint-equal' with arguments set to 'aa' and 'AA' respe
    // XPath: fn:codepoint-equal(\"aa\",\"AA\")
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_13() {
    //  Evaluation of an 'fn:codepoint-equal' with arguments set to 'aa' and lower-case
    // XPath: fn:codepoint-equal(\"aa\",fn:lower-case(\"AA\"))
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_14() {
    //  Evaluation of an 'fn:codepoint-equal' with arguments set to 'aa' and upper-case
    // XPath: fn:codepoint-equal(\"aa\",fn:upper-case(\"aa\"))
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_15() {
    //  Evaluation of an 'fn:codepoint-equal' as an argument to xs:boolean.
    // XPath: xs:boolean(fn:codepoint-equal(\"aa\",\"aa\"))
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_16() {
    //  Evaluation of an 'fn:codepoint-equal' as part of boolean expression ('and' and
    // XPath: fn:codepoint-equal(\"aa\",\"aa\") and fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_17() {
    //  Evaluation of an 'fn:codepoint-equal' as part of boolean expression ('and' and
    // XPath: fn:codepoint-equal(\"aa\",\"aa\") and fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_18() {
    //  Evaluation of an 'fn:codepoint-equal' as part of boolean expression ('or' and f
    // XPath: fn:codepoint-equal(\"aa\",\"aa\") or fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_19() {
    //  Evaluation of an 'fn:codepoint-equal' as part of boolean expression ('or' and f
    // XPath: fn:codepoint-equal(\"aa\",\"aa\") or fn:false()
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_20() {
    //  Evaluation of an 'fn:codepoint-equal' as part of boolean expression involving t
    // XPath: fn:codepoint-equal(\"aa\",\"aa\") and fn:codepoint-equal(\"a
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_21() {
    //  Evaluation of an 'fn:codepoint-equal' as part of boolean expression involving t
    // XPath: fn:codepoint-equal(\"aa\",\"aa\") or fn:codepoint-equal(\"aa
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn fn_codepoint_equal_22() {
    //  Different normalization forms of the same string are not codepoint-equal
    // XPath: fn:codepoint-equal(normalize-unicode(\"gar?on\", \"NFC\"), n
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn k2_codepointequal_1() {
    //  Compare two values returned from fn:lower-case().
    // XPath: codepoint-equal(lower-case(concat(\"B STRING\", current-time
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn k2_codepointequal_2() {
    //  Compare two values returned from fn:upper-case().
    // XPath: codepoint-equal(upper-case(concat(\"B STRING\", current-time
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn k2_codepointequal_3() {
    //  Compare two non-matching values returned from fn:lower-case().
    // XPath: codepoint-equal(lower-case(concat(\"B STRING\", current-time
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn k2_codepointequal_4() {
    //  Compare two non-matching values returned from fn:upper-case().
    // XPath: codepoint-equal(upper-case(concat(\"B STRING\", current-time
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn k2_codepointequal_5() {
    //  Compare two non-matching values returned from fn:lower-case() and fn:upper-case
    // XPath: codepoint-equal(upper-case(concat(\"B STRING\", current-time
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn k2_codepointequal_6() {
    //  Compare two non-matching values returned from fn:lower-case() and fn:upper-case
    // XPath: codepoint-equal(lower-case(concat(\"B STRING\", current-time
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn cbcl_codepoint_equal_001() {
    //  test fn:codepoint-equal with empty $arg1
    // XPath: declare function local:yes-empty($arg as xs:boolean) as xs:s
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn cbcl_codepoint_equal_002() {
    //  test fn:codepoint-equal with empty $arg2
    // XPath: declare function local:yes-empty($arg as xs:boolean) as xs:s
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn cbcl_codepoint_equal_003() {
    //  test fn:codepoint-equal with non-empty arguments
    // XPath: declare function local:yes-no($arg as xs:boolean) as xs:stri
    // Expected: true
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn cbcl_codepoint_equal_004() {
    //  test fn:boolean on fn:codepoint-equal with empty $arg2
    // XPath: declare function local:yes-empty($arg as xs:boolean) as xs:s
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn cbcl_codepoint_equal_005() {
    //  test fn:boolean on fn:codepoint-equal with empty $arg1
    // XPath: declare function local:yes-empty($arg as xs:boolean) as xs:s
    // Expected: false
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}

#[test]
fn cbcl_codepoint_equal_006() {
    //  test fn:codepoint-equal evaluate to item
    // XPath: declare function local:yes-no($arg as xs:boolean) as xs:stri
    // Expected: 2
    // Calls stub function - will fail until fn:codepoint-equal is implemented
    let _ = stub_fncodepoint_equal();
}
