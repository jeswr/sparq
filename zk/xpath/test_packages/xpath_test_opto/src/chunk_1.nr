//! Test chunk 1 for op:to
//! Contains 50 tests

use dep::xpath::stub_opto;

#[test]
fn k_rangeexpr_21() {
    //  A test whose essence is: `deep-equal((0, -1, -2, -3, -4, -5), reverse(-5 to 0))
    // XPath: deep-equal((0, -1, -2, -3, -4, -5), reverse(-5 to 0))
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_22() {
    //  A test whose essence is: `count(reverse(-5 to -2)) eq 4`.
    // XPath: count(reverse(-5 to -2)) eq 4
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_23() {
    //  A test whose essence is: `count(reverse(-5 to -0)) eq 6`.
    // XPath: count(reverse(-5 to -0)) eq 6
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_24() {
    //  A test whose essence is: `count(reverse(1 to 4)) eq 4`.
    // XPath: count(reverse(1 to 4)) eq 4
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_25() {
    //  A test whose essence is: `empty(1 to 0)`.
    // XPath: empty(1 to 0)
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_26() {
    //  A test whose essence is: `empty(0 to -5)`.
    // XPath: empty(0 to -5)
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_27() {
    //  A test whose essence is: `empty(-4 to -5)`.
    // XPath: empty(-4 to -5)
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_28() {
    //  A test whose essence is: `empty(reverse(1 to 0))`.
    // XPath: empty(reverse(1 to 0))
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_29() {
    //  A test whose essence is: `empty(reverse(0 to -5))`.
    // XPath: empty(reverse(0 to -5))
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_30() {
    //  A test whose essence is: `empty(reverse(-4 to -5))`.
    // XPath: empty(reverse(-4 to -5))
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_31() {
    //  A test whose essence is: `(remove((2.e0, 4), 1) treat as xs:integer to 4)`.
    // XPath: (remove((2.e0, 4), 1) treat as xs:integer to 4)
    // Expected: 4
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_32() {
    //  A test whose essence is: `(4 to remove((2e0, 4), 1) treat as xs:integer)`.
    // XPath: (4 to remove((2e0, 4), 1) treat as xs:integer)
    // Expected: 4
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k2_rangeexpr_3() {
    //  Combine with a for expression.
    // XPath: for $i in 1 to 3 return $i
    // Expected: 1 2 3
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k2_rangeexpr_4() {
    //  Expressions that are tricky for local rewrites.
    // XPath: 1 to <value>5</value>, 5 to <value>5</value>, <value>1</valu
    // Expected: 1 2 3 4 5 5 1 2 3 4 5 1 2 3 4 5 5 6 1001
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_401() {
    // singleton
    // XPath: 3 to 3
    // Expected: 3
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_401a() {
    // singleton
    // XPath: count(3 to 3)
    // Expected: 1
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_401b() {
    // singleton
    // XPath: (3 to 3)[1]
    // Expected: 3
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_401c() {
    // singleton
    // XPath: 3 = 3 to 3
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_401d() {
    // singleton
    // XPath: 4 > 3 to 3
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_402() {
    // negative singleton
    // XPath: -33 to -33
    // Expected: -33
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_402a() {
    // negative singleton
    // XPath: count(-33 to -33)
    // Expected: 1
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_402b() {
    // negative singleton
    // XPath: (-33 to -33)[1]
    // Expected: -33
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_402c() {
    // negative singleton
    // XPath: -33 = -33 to -33
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_402d() {
    // negative singleton
    // XPath: -33 to -33 > -34
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_403() {
    // zero singleton
    // XPath: 0 to 0
    // Expected: 0
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_403a() {
    // zero singleton
    // XPath: count(0 to 0)
    // Expected: 1
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_403b() {
    // zero singleton
    // XPath: (0 to 0)[1]
    // Expected: 0
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_403c() {
    // zero singleton
    // XPath: 1 = 0 to 0
    // Expected: false
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_403d() {
    // zero singleton
    // XPath: -1 > 0 to 0
    // Expected: false
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_404a() {
    // empty range
    // XPath: count(10 to 0)
    // Expected: 0
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_404c() {
    // empty range
    // XPath: 10 = 10 to 0
    // Expected: false
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_404d() {
    // empty range
    // XPath: 10 > 10 to 0
    // Expected: false
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_405a() {
    // empty range
    // XPath: count(0 to -10)
    // Expected: 0
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_406a() {
    // multiple range
    // XPath: count(1 to 10)
    // Expected: 10
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_406b() {
    // multiple range
    // XPath: (1 to 10)[3]
    // Expected: 3
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_406c() {
    // multiple range
    // XPath: 6 = 1 to 10
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_406d() {
    // multiple range
    // XPath: 6 > 1 to 10
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_406e() {
    // multiple range
    // XPath: 1 to 10 > 6
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_407a() {
    // multiple range, negative
    // XPath: count(-200000 to -199997)
    // Expected: 4
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_407b() {
    // multiple range, negative
    // XPath: (-200000 to -199997)[3]
    // Expected: -199998
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_407c() {
    // multiple range, negative
    // XPath: 12345 > -200000 to -199997
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_407d() {
    // multiple range, negative
    // XPath: -200000 to -199997 > 54321
    // Expected: false
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_408a() {
    // multiple range, large numbers
    // XPath: count(100000000000 to 100000000003)
    // Expected: 4
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_408b() {
    // multiple range, large numbers
    // XPath: (100000000000 to 100000000003)[3]
    // Expected: 100000000002
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_408c() {
    // multiple range, large numbers
    // XPath: 100000000002 = 100000000000 to 100001000003
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_408d() {
    // multiple range, large numbers
    // XPath: 100001000002 < 100000000000 to 100001000003
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_409a() {
    // multiple range, very large numbers
    // XPath: count(1000000000000000000000 to 1000000000000000000003)
    // Expected: 4
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_409b() {
    // multiple range, very large numbers
    // XPath: (1000000000000000000000 to 1000000000000000000003)[2]
    // Expected: 1000000000000000000001
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_409c() {
    // multiple range, very large numbers
    // XPath: 1000000000000000020001 = 1000000000000000000000 to 100000000
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_409d() {
    // multiple range, very large numbers
    // XPath: 1000000000000000020001 < 1000000000000000000000 to 100000000
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}
