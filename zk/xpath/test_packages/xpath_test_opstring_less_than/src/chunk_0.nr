//! Test chunk 0 for op:string-less-than
//! Contains 10 tests

use dep::xpath::stub_opstring_less_than;

#[test]
fn k_stringlt_1() {
    //  A test whose essence is: `'a' lt 'abc'`.
    // XPath: 'a' lt 'abc'
    // Expected: true
    // Calls stub function - will fail until op:string-less-than is implemented
    let _ = stub_opstring_less_than();
}

#[test]
fn k_stringlt_2() {
    //  A test whose essence is: `not('abc' lt 'a')`.
    // XPath: not('abc' lt 'a')
    // Expected: true
    // Calls stub function - will fail until op:string-less-than is implemented
    let _ = stub_opstring_less_than();
}

#[test]
fn k_stringlt_3() {
    //  A test whose essence is: `'a' le 'abc'`.
    // XPath: 'a' le 'abc'
    // Expected: true
    // Calls stub function - will fail until op:string-less-than is implemented
    let _ = stub_opstring_less_than();
}

#[test]
fn k_stringlt_4() {
    //  A test whose essence is: `not('abc' le 'a')`.
    // XPath: not('abc' le 'a')
    // Expected: true
    // Calls stub function - will fail until op:string-less-than is implemented
    let _ = stub_opstring_less_than();
}

#[test]
fn k_stringlt_5() {
    //  A test whose essence is: `'abc' le 'abc'`.
    // XPath: 'abc' le 'abc'
    // Expected: true
    // Calls stub function - will fail until op:string-less-than is implemented
    let _ = stub_opstring_less_than();
}

#[test]
fn k2_stringlt_1() {
    //  Compare two large codepoints.
    // XPath: \"?\" lt \"?\"
    // Expected: true
    // Calls stub function - will fail until op:string-less-than is implemented
    let _ = stub_opstring_less_than();
}

#[test]
fn cbcl_string_less_than_001() {
    //  test string comparison
    // XPath: not(string(current-time()) lt \"now\")
    // Expected: false
    // Calls stub function - will fail until op:string-less-than is implemented
    let _ = stub_opstring_less_than();
}

#[test]
fn cbcl_string_less_than_002() {
    //  test string comparison
    // XPath: not(string(current-time()) ge \"now\")
    // Expected: true
    // Calls stub function - will fail until op:string-less-than is implemented
    let _ = stub_opstring_less_than();
}

#[test]
fn cbcl_string_less_than_003() {
    //  test string comparison
    // XPath: not(xs:untypedAtomic(current-time()) lt xs:untypedAtomic(\"n
    // Expected: false
    // Calls stub function - will fail until op:string-less-than is implemented
    let _ = stub_opstring_less_than();
}

#[test]
fn cbcl_string_less_than_004() {
    //  test string comparison
    // XPath: not(xs:untypedAtomic(current-time()) ge xs:untypedAtomic(\"n
    // Expected: true
    // Calls stub function - will fail until op:string-less-than is implemented
    let _ = stub_opstring_less_than();
}
