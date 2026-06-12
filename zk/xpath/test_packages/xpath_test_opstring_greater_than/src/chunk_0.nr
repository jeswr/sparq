//! Test chunk 0 for op:string-greater-than
//! Contains 9 tests

use dep::xpath::stub_opstring_greater_than;

#[test]
fn k_stringgt_1() {
    //  A test whose essence is: `'abc' gt 'a'`.
    // XPath: 'abc' gt 'a'
    // Expected: true
    // Calls stub function - will fail until op:string-greater-than is implemented
    let _ = stub_opstring_greater_than();
}

#[test]
fn k_stringgt_2() {
    //  A test whose essence is: `not('abc' gt 'abc')`.
    // XPath: not('abc' gt 'abc')
    // Expected: true
    // Calls stub function - will fail until op:string-greater-than is implemented
    let _ = stub_opstring_greater_than();
}

#[test]
fn k_stringgt_3() {
    //  A test whose essence is: `'abc' ge 'a'`.
    // XPath: 'abc' ge 'a'
    // Expected: true
    // Calls stub function - will fail until op:string-greater-than is implemented
    let _ = stub_opstring_greater_than();
}

#[test]
fn k_stringgt_4() {
    //  A test whose essence is: `not('a' ge 'abc')`.
    // XPath: not('a' ge 'abc')
    // Expected: true
    // Calls stub function - will fail until op:string-greater-than is implemented
    let _ = stub_opstring_greater_than();
}

#[test]
fn k_stringgt_5() {
    //  A test whose essence is: `'abc' ge 'abc'`.
    // XPath: 'abc' ge 'abc'
    // Expected: true
    // Calls stub function - will fail until op:string-greater-than is implemented
    let _ = stub_opstring_greater_than();
}

#[test]
fn cbcl_string_greater_than_001() {
    //  test string comparison
    // XPath: not(string(current-time()) gt \"now\")
    // Expected: true
    // Calls stub function - will fail until op:string-greater-than is implemented
    let _ = stub_opstring_greater_than();
}

#[test]
fn cbcl_string_greater_than_002() {
    //  test string comparison
    // XPath: not(string(current-time()) le \"now\")
    // Expected: false
    // Calls stub function - will fail until op:string-greater-than is implemented
    let _ = stub_opstring_greater_than();
}

#[test]
fn cbcl_string_greater_than_003() {
    //  test string comparison
    // XPath: not(xs:untypedAtomic(current-time()) gt xs:untypedAtomic(\"n
    // Expected: true
    // Calls stub function - will fail until op:string-greater-than is implemented
    let _ = stub_opstring_greater_than();
}

#[test]
fn cbcl_string_greater_than_004() {
    //  test string comparison
    // XPath: not(xs:untypedAtomic(current-time()) le xs:untypedAtomic(\"n
    // Expected: false
    // Calls stub function - will fail until op:string-greater-than is implemented
    let _ = stub_opstring_greater_than();
}
