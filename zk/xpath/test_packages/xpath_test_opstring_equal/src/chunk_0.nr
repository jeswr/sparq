//! Test chunk 0 for op:string-equal
//! Contains 15 tests

use dep::xpath::stub_opstring_equal;

#[test]
fn k_stringequal_1() {
    //  A test whose essence is: `'equals' eq 'equals'`.
    // XPath: 'equals' eq 'equals'
    // Expected: true
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn k_stringequal_2() {
    //  A test whose essence is: `'' eq ''`.
    // XPath: '' eq ''
    // Expected: true
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn k_stringequal_3() {
    //  A test whose essence is: `not('nada' eq 'equalness')`.
    // XPath: not('nada' eq 'equalness')
    // Expected: true
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn k_stringequal_4() {
    //  A test whose essence is: `'not' ne 'equal'`.
    // XPath: 'not' ne 'equal'
    // Expected: true
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn k_stringequal_5() {
    //  A test whose essence is: `not('equal' ne 'equal')`.
    // XPath: not('equal' ne 'equal')
    // Expected: true
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn k2_stringequal_1() {
    //  Compare two values returned from fn:lower-case().
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: true
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn k2_stringequal_2() {
    //  Compare two values returned from fn:upper-case().
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: true
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn k2_stringequal_3() {
    //  Compare two non-matching values returned from fn:lower-case().
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: false
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn k2_stringequal_4() {
    //  Compare two non-matching values returned from fn:upper-case().
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: false
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn k2_stringequal_5() {
    //  Compare two non-matching values returned from fn:lower-case() and fn:upper-case
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: false
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn k2_stringequal_6() {
    //  Compare two non-matching values returned from fn:lower-case() and fn:upper-case
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: false
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn cbcl_string_equal_001() {
    //  test string comparison
    // XPath: not(string(current-time()) eq \"now\")
    // Expected: true
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn cbcl_string_equal_002() {
    //  test string comparison
    // XPath: not(string(current-time()) ne \"now\")
    // Expected: false
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn cbcl_string_equal_003() {
    //  test string comparison
    // XPath: not(xs:untypedAtomic(current-time()) eq xs:untypedAtomic(\"n
    // Expected: true
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}

#[test]
fn cbcl_string_equal_004() {
    //  test string comparison
    // XPath: not(xs:untypedAtomic(current-time()) ne xs:untypedAtomic(\"n
    // Expected: false
    // Calls stub function - will fail until op:string-equal is implemented
    let _ = stub_opstring_equal();
}
