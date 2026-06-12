//! Test chunk 0 for fn:default-collation
//! Contains 4 tests

use dep::xpath::stub_fndefault_collation;

#[test]
fn fn_default_collation_2() {
    //  Evaluation of an fn:default-collation function that retrieves the default colla
    // XPath: fn:default-collation()
    // Expected: http://www.w3.org/2005/xpath-functions/c
    // Calls stub function - will fail until fn:default-collation is implemented
    let _ = stub_fndefault_collation();
}

#[test]
fn k_contextdefaultcollationfunc_3() {
    //  A test whose essence is: `default-collation() eq 'http://www.w3.org/2005/xpath-
    // XPath: default-collation() eq \"http://www.w3.org/2005/xpath-functi
    // Expected: true
    // Calls stub function - will fail until fn:default-collation is implemented
    let _ = stub_fndefault_collation();
}

#[test]
fn cbcl_default_collation_001() {
    //  test evalaution of fn:default-collation Author: Tim Mills
    // XPath: fn:count(fn:default-collation())
    // Expected: 1
    // Calls stub function - will fail until fn:default-collation is implemented
    let _ = stub_fndefault_collation();
}

#[test]
fn cbcl_default_collation_002() {
    //  test evalaution of fn:boolean on fn:default-collation Author: Tim Mills
    // XPath: fn:boolean(fn:default-collation())
    // Expected: true
    // Calls stub function - will fail until fn:default-collation is implemented
    let _ = stub_fndefault_collation();
}
