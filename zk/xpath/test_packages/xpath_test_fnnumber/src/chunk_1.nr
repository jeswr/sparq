//! Test chunk 1 for fn:number
//! Contains 4 tests

use dep::xpath::stub_fnnumber;

#[test]
fn k_nodenumberfunc_14() {
    //  A test whose essence is: `string(number('1')) eq '1'`.
    // XPath: string(number(\"1\")) eq \"1\"
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_15() {
    //  A test whose essence is: `string(number(xs:gYear('2005'))) eq 'NaN'`.
    // XPath: string(number(xs:gYear(\"2005\"))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_16() {
    //  fn:number() inside a predicate.
    // XPath: deep-equal((1, 2, 3)[number()], (1, 2, 3))
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_17() {
    //  fn:number() inside a predicate(#2).
    // XPath: (1)[number()] eq 1
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}
