//! Test chunk 1 for fn:count
//! Contains 50 tests

use dep::xpath::stub_fncount;

#[test]
fn k_seqcountfunc_14() {
    //  A test whose essence is: `count((1, 2, 3, 'four')) eq 4`.
    // XPath: count((1, 2, 3, \"four\")) eq 4
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_15() {
    //  A test whose essence is: `count(1 to 3) eq 3`.
    // XPath: count(1 to 3) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_16() {
    //  A test whose essence is: `count(reverse((1, 2, 3))) eq 3`.
    // XPath: count(reverse((1, 2, 3))) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_17() {
    //  A test whose essence is: `count(reverse((1, 2, 3))) eq 3`.
    // XPath: count(reverse((1, 2, 3))) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_18() {
    //  A test whose essence is: `not(count((1, 2, current-time(), 4))) eq false()`.
    // XPath: not(count((1, 2, current-time(), 4))) eq false()
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count001() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//employee[@name='John Doe 4']) = 1
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count002() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//employee[@name='John Doe 4']) < 2
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count003() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//employee[@name='John Doe 4']) > 0
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count004() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//employee[@name='John Doe 4']/@name) > 0.5
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count005() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//*[@name='John Doe 4']) lt 1.5
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count006() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//*[@name='John Doe 4']) eq 0
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count007() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//*[@name='John Doe 498']) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count008() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//*[@name='John Doe 4']) lt 1000000000000
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count009() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//*[@name='John Doe 4']) gt -5
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count010() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//*[@name='John Doe 4']) eq 0.3
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count011() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//node()) gt 40
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count012() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: count(//node()) ne -1
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count013() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: 0 = count(//node())
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count014() {
    //  count() applied to nodes (with optimization opportunities)
    // XPath: 40 gt count(//node())
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn count015() {
    //  count() applied to a for expression
    // XPath: count(for $s in (\"red\", \"blue\", \"green\") return string
    // Expected: 12
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_001() {
    //  test fn:count on a range
    // XPath: fn:count(1 to 10000000)
    // Expected: 10000000
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_002() {
    //  test fn:count on a mix of values and expressions
    // XPath: declare function local:generate($arg as xs:integer?) { if ($
    // Expected: 10000004
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_003() {
    //  Test fn:count on a range
    // XPath: let $x := year-from-date(current-date()) return count( 1 to
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_005() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_006() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_007() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_008() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_009() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_010() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_011() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_012() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_013() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_014() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_015() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_016() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_017() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_018() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_019() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_020() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_021() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_022() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_023() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_024() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_025() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_026() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_027() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_028() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_029() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: false
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_030() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn cbcl_count_031() {
    //  test fn:count
    // XPath: declare function local:primes($n as xs:integer) { if ($n lt
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}
