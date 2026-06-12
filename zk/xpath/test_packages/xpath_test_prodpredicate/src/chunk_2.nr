//! Test chunk 2 for prod:Predicate
//! Contains 12 tests

use dep::xpath::stub_prodpredicate;

#[test]
fn cbcl_first_in_sequence_009() {
    //  test fn:first-in-sequence on a for-each expression Author: Tim Mills
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: 3
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_first_in_sequence_010() {
    //  test fn:boolean on fn:first-in-sequence
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_first_in_sequence_011() {
    //  test fn:boolean on fn:first-in-sequence
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: false
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_first_in_sequence_012() {
    //  test fn:first-in-sequence on a for-each expression
    // XPath: declare function local:generate($arg as xs:boolean) as xs:st
    // Expected: 3
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_filter_001() {
    //  Tests various filter optimizations
    // XPath: let $x := exists((1 to 10)[. mod 2 = 0]) return (1 to 100)[p
    // Expected: 6 12 18 24 30 36 42 48 54 60 66 72 78 84
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filter_limits_004() {
    // Test numeric predicates at integer limits
    // XPath: for $i in (-1, 0, 1, 2147483647, 2147483648, 2147483649, 429
    // Expected: 1
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filter_limits_005() {
    // Test numeric predicates at integer limits
    // XPath: for $i in (-1, 0, 1, 2147483647, 2147483648, 2147483649, 429
    // Expected: 1
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filter_limits_006() {
    // Test numeric predicates at integer limits
    // XPath: for $i in (-1, 0, 1, 2147483647, 2147483648, 2147483649, 429
    // Expected: 1
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filter_limits_007() {
    // Test numeric predicates at integer limits
    // XPath: for $i in (-1, 0, 1, 2147483647, 2147483648, 2147483649, 429
    // Expected: 1
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filter_limits_008() {
    // Test numeric predicates at integer limits
    // XPath: (-1, 0, 1, 2147483647, 2147483648, 2147483649, 4294967295, 4
    // Expected: 1
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filter_limits_009() {
    // Test numeric predicates at integer limits
    // XPath: (-1, 0, 1, 2147483647, 2147483648, 2147483649, 4294967295, 4
    // Expected: 1
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filter_limits_010() {
    // Test numeric predicates at integer limits
    // XPath: count((-1, 0, 1, 2147483647, 2147483648, 2147483649, 4294967
    // Expected: 15
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}
